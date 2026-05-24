use boa_engine::{Context, Source};
use eframe::egui;
use scraper::{Html, Node};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;


fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 載入我們放在 src/font.ttf 的中文字型
    fonts.font_data.insert(
        "my_cjk_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../font/font.ttf")),
    );

    // 設定比例字型 (一般文字) 優先使用我們的中文字型
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_cjk_font".to_owned());

    // 設定等寬字型 (程式碼區塊) 優先使用我們的中文字型
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_cjk_font".to_owned());

    // 將設定套用到 egui
    ctx.set_fonts(fonts);
}

// ------------------------------------------------------------------
// 1. 我們自定義的 DOM 樹結構 (簡化版)
// ------------------------------------------------------------------
#[derive(Clone, Debug)]
enum DomNode {
    Text(String),
    Element {
        tag: String,
        attrs: HashMap<String, String>,
        children: Vec<DomNode>,
    },
}

// ------------------------------------------------------------------
// 2. 背景處理結果 (包含載入的 DOM 與 JS 執行結果)
// ------------------------------------------------------------------
struct PageData {
    dom: DomNode,
    js_output: String,
}

enum FetchResult {
    Success(PageData),
    Error(String),
}

// ------------------------------------------------------------------
// 3. 瀏覽器主體
// ------------------------------------------------------------------
struct Browser4 {
    url_input: String,
    current_url: String,
    page_data: Option<PageData>,
    is_loading: bool,
    rx: Option<Receiver<FetchResult>>,
    history: Vec<String>,
    history_index: usize,
}

impl Browser4 {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 註：若有中文字體亂碼，請參考上一次的教學加入 setup_custom_fonts(&_cc.egui_ctx);
        setup_custom_fonts(&cc.egui_ctx);
        Self {
            url_input: "https://example.com".to_string(),
            current_url: "https://example.com".to_string(),
            page_data: None,
            is_loading: false,
            rx: None,
            history: vec!["https://example.com".to_string()],
            history_index: 0,
        }
    }

    fn load_url(&mut self, url: String, ctx: egui::Context) {
        let resolved_url = self.resolve_url(&url);
        self.url_input = resolved_url.clone();
        self.current_url = resolved_url.clone();
        self.is_loading = true;
        self.page_data = None;

        self.history.truncate(self.history_index + 1);
        self.history.push(resolved_url.clone());
        self.history_index = self.history.len() - 1;

        let current_dir = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        let is_http = resolved_url.starts_with("http://") || resolved_url.starts_with("https://");

        thread::spawn(move || {
            let html_text = if is_http {
                match reqwest::blocking::get(&resolved_url) {
                    Ok(resp) => resp.text().unwrap_or_default(),
                    Err(e) => {
                        let _ = tx.send(FetchResult::Error(format!("網路錯誤: {}", e)));
                        ctx.request_repaint();
                        return;
                    }
                }
            } else {
                let file_path = if resolved_url.starts_with("file://") {
                    resolved_url.trim_start_matches("file://").to_string()
                } else if resolved_url.starts_with('/') || resolved_url.starts_with("./") || resolved_url.starts_with("../") {
                    resolved_url.to_string()
                } else {
                    format!("{}/{}", current_dir, resolved_url)
                };

                match fs::read_to_string(&file_path) {
                    Ok(content) => content,
                    Err(e) => {
                        let _ = tx.send(FetchResult::Error(format!("檔案讀取錯誤: {} ({})", file_path, e)));
                        ctx.request_repaint();
                        return;
                    }
                }
            };

            // 解析 HTML
            let document = Html::parse_document(&html_text);
            let root_ref = document.tree.root();
            
            // 轉換成自定義的 DOM
            let custom_dom = build_custom_dom(root_ref).unwrap_or(DomNode::Text("Empty".into()));

            // 擷取並執行 JavaScript
            let mut js_output = String::new();
            let mut js_context = Context::default();
            
            let scripts = extract_scripts(&custom_dom);
            for script in scripts {
                match js_context.eval(Source::from_bytes(script.as_bytes())) {
                    Ok(val) => {
                        js_output.push_str(&format!("JS Return: {:?}\n", val));
                    }
                    Err(e) => {
                        js_output.push_str(&format!("JS Error: {}\n", e));
                    }
                }
            }
            if js_output.is_empty() {
                js_output = "No JavaScript output.".to_string();
            }

            let _ = tx.send(FetchResult::Success(PageData {
                dom: custom_dom,
                js_output,
            }));
            ctx.request_repaint();
        });
    }

    fn resolve_url(&self, url: &str) -> String {
        let url = url.trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            return url.to_string();
        }
        if url.starts_with("file://") {
            return url.to_string();
        }
        if url.starts_with('/') || url.starts_with("./") || url.starts_with("../") {
            return url.to_string();
        }
        url.to_string()
    }
}

// ------------------------------------------------------------------
// 4. 將 Scraper 的樹轉成我們好處理的自定義 DOM 樹
// ------------------------------------------------------------------
fn build_custom_dom(node: ego_tree::NodeRef<Node>) -> Option<DomNode> {
    match node.value() {
        Node::Text(text) => {
            let t = text.text.to_string();
            if t.trim().is_empty() { return None; } // 忽略空白文本
            Some(DomNode::Text(t))
        }
        Node::Element(el) => {
            let tag = el.name().to_lowercase();
            let mut attrs = HashMap::new();
            for (k, v) in el.attrs() {
                attrs.insert(k.to_string(), v.to_string());
            }
            
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(c) = build_custom_dom(child) {
                    children.push(c);
                }
            }
            Some(DomNode::Element { tag, attrs, children })
        }
        Node::Document => {
            // Document 根節點當作一個 div 容器
            let mut children = Vec::new();
            for child in node.children() {
                if let Some(c) = build_custom_dom(child) {
                    children.push(c);
                }
            }
            Some(DomNode::Element { tag: "div".into(), attrs: HashMap::new(), children })
        }
        _ => None,
    }
}

// 提取所有 <script> 標籤內的文字 (模擬 JS 引擎讀取)
fn extract_scripts(node: &DomNode) -> Vec<String> {
    let mut scripts = Vec::new();
    match node {
        DomNode::Element { tag, children, .. } => {
            if tag == "script" {
                for c in children {
                    if let DomNode::Text(t) = c {
                        scripts.push(t.clone());
                    }
                }
            } else {
                for c in children {
                    scripts.extend(extract_scripts(c));
                }
            }
        }
        _ => {}
    }
    scripts
}

// ------------------------------------------------------------------
// 5. 自製的 Render 引擎：將 DOM 映射到 egui 畫面上
// ------------------------------------------------------------------
fn render_dom(ui: &mut egui::Ui, node: &DomNode, clicked_link: &mut Option<String>) {
    match node {
        DomNode::Text(text) => {
            ui.label(text);
        }
        DomNode::Element { tag, attrs, children } => {
            match tag.as_str() {
                // 不顯示的標籤
                "head" | "script" | "style" | "title" => {}

                // 標題 - 垂直排列
                "h1" => {
                    ui.vertical(|ui| {
                        ui.heading(get_text_only(node));
                    });
                    ui.add_space(8.0);
                }
                "h2" => {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(get_text_only(node)).size(22.0).strong());
                    });
                    ui.add_space(6.0);
                }
                "h3" => {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(get_text_only(node)).size(18.0).strong());
                    });
                    ui.add_space(4.0);
                }
                "h4" | "h5" | "h6" => {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(get_text_only(node)).size(16.0).strong());
                    });
                    ui.add_space(4.0);
                }

                // 超連結
                "a" => {
                    let href = attrs.get("href").cloned().unwrap_or_default();
                    let link_text = get_text_only(node);
                    if ui.link(link_text).clicked() {
                        *clicked_link = Some(href);
                    }
                }

                // 圖片
                "img" => {
                    let alt = attrs.get("alt").cloned().unwrap_or("Image".into());
                    ui.label(egui::RichText::new(format!("[圖片: {}]", alt)).color(egui::Color32::GRAY));
                }

                // 區塊元素 - 垂直排列
                "p" | "div" | "article" | "section" | "header" | "footer" | "nav" | "main" | "aside" | "blockquote" | "figure" | "figcaption" | "hr" => {
                    ui.vertical(|ui| {
                        for c in children { render_dom(ui, c, clicked_link); }
                    });
                    ui.add_space(6.0);
                }

                // 列表
                "ul" | "ol" => {
                    ui.vertical(|ui| {
                        for c in children { render_dom(ui, c, clicked_link); }
                    });
                    ui.add_space(6.0);
                }

                // 列表項目
                "li" => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("•");
                            for c in children { render_dom(ui, c, clicked_link); }
                        });
                    });
                }

                // 預格式化文字
                "pre" => {
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        for c in children {
                            if let DomNode::Text(t) = c {
                                ui.label(egui::RichText::new(t).code());
                            } else {
                                render_dom(ui, c, clicked_link);
                            }
                        }
                        ui.add_space(4.0);
                    });
                }

                // 換行
                "br" => {
                    ui.set_height(16.0);
                }

                // 行內標籤 - 橫向排列
                "b" | "strong" => {
                    let text = get_text_only(node);
                    ui.label(egui::RichText::new(text).strong());
                }
                "i" | "em" => {
                    let text = get_text_only(node);
                    ui.label(egui::RichText::new(text).italics());
                }
                "u" => {
                    let text = get_text_only(node);
                    ui.label(egui::RichText::new(text).underline());
                }
                "s" | "del" => {
                    let text = get_text_only(node);
                    ui.label(egui::RichText::new(text).strikethrough());
                }
                "span" | "small" | "sub" | "sup" | "mark" | "code" => {
                    let text = get_text_only(node);
                    ui.label(egui::RichText::new(text).code());
                }

                // 未知標籤 - 垂直排列子節點
                _ => {
                    ui.vertical(|ui| {
                        for c in children { render_dom(ui, c, clicked_link); }
                    });
                    ui.add_space(4.0);
                }
            }
        }
    }
}

// 輔助函數：提取節點下的純文字 (用來簡化 a 和 h1 的渲染)
fn get_text_only(node: &DomNode) -> String {
    let mut text = String::new();
    match node {
        DomNode::Text(t) => text.push_str(t),
        DomNode::Element { children, .. } => {
            for c in children {
                text.push_str(&get_text_only(c));
            }
        }
    }
    text
}

// ------------------------------------------------------------------
// 6. 介面更新 (App)
// ------------------------------------------------------------------
impl eframe::App for Browser4 {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 接收背景資料
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    FetchResult::Success(data) => self.page_data = Some(data),
                    FetchResult::Error(err) => {
                        self.page_data = Some(PageData {
                            dom: DomNode::Text(err),
                            js_output: String::new(),
                        });
                    }
                }
                self.is_loading = false;
                self.rx = None;
            }
        }

        // 網址列
        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {
                    self.go_back(ctx.clone());
                }
                if ui.button("▶").clicked() {
                    self.go_forward(ctx.clone());
                }
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .hint_text("輸入網址 http://...")
                        .desired_width(ui.available_width() - 100.0),
                );
                if ui.button("Go").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.load_url(self.url_input.clone(), ctx.clone());
                }
            });
        });

        // 畫面渲染區
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.centered_and_justified(|ui| ui.spinner());
            } else if let Some(page) = &self.page_data {
                let mut clicked_link = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // --- 1. 自製 HTML 渲染區 ---
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.heading("📄 Rendered Page");
                        ui.separator();
                        render_dom(ui, &page.dom, &mut clicked_link);
                    });

                    ui.add_space(10.0);

                    // --- 2. JavaScript 執行結果區 ---
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.heading("⚙️ JS Engine Output");
                        ui.separator();
                        ui.label(egui::RichText::new(&page.js_output).code().color(egui::Color32::LIGHT_GREEN));
                    });
                });

                // 若有超連結被點擊，觸發跳轉
                if let Some(link) = clicked_link {
                    let target = if link.starts_with("http") || link.starts_with("file://") {
                        link.to_string()
                    } else {
                        let base_dir = Path::new(&self.current_url)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if link.starts_with('/') {
                            format!("{}/{}", base_dir.trim_end_matches('/'), link.trim_start_matches('/'))
                        } else {
                            format!("{}/{}", base_dir, link)
                        }
                    };
                    self.load_url(target, ctx.clone());
                }
            } else {
                ui.centered_and_justified(|ui| ui.label("輸入網址以開始瀏覽"));
            }
        });
    }
}

impl Browser4 {
    fn go_back(&mut self, ctx: egui::Context) {
        if self.history_index > 0 {
            self.history_index -= 1;
            let url = self.history[self.history_index].clone();
            self.url_input = url.clone();
            self.load_url(url, ctx);
        }
    }

    fn go_forward(&mut self, ctx: egui::Context) {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            let url = self.history[self.history_index].clone();
            self.url_input = url.clone();
            self.load_url(url, ctx);
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Browser4 - 自製渲染引擎"),
        ..Default::default()
    };
    eframe::run_native("browser4", options, Box::new(|cc| Box::new(Browser4::new(cc))))
}