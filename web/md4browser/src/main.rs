use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::mpsc::{self, Receiver, Sender};
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

// 接收背景抓取結果的結構
enum FetchResult {
    Success(String),
    Error(String),
}

struct MdBrowser {
    url_input: String,      // 網址列顯示的文字
    current_url: String,    // 當前正在瀏覽的真實網址或路徑
    content: String,        // 下載下來的 Markdown 內容
    is_loading: bool,       // 是否正在載入中（顯示旋轉動畫）
    rx: Option<Receiver<FetchResult>>, // 接收背景執行緒資料的通道
    md_cache: CommonMarkCache,         // Markdown 渲染快取
}

impl MdBrowser {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // === 加入這一行來載入中文字型 ===
        setup_custom_fonts(&cc.egui_ctx);
        let mut app = Self {
            url_input: "README.md".to_string(),
            current_url: "README.md".to_string(),
            content: String::new(),
            is_loading: false,
            rx: None,
            md_cache: CommonMarkCache::default(),
        };
        // 初始啟動時載入本機 README.md
        app.load_url("README.md".to_string(), cc.egui_ctx.clone());
        app
    }

    // 負責發起載入任務 (區分本機或網路)
    fn load_url(&mut self, url: String, ctx: egui::Context) {
        self.url_input = url.clone();
        self.current_url = url.clone();
        self.is_loading = true;
        self.content.clear();

        let (tx, rx): (Sender<FetchResult>, Receiver<FetchResult>) = mpsc::channel();
        self.rx = Some(rx);

        // 啟動背景執行緒去抓資料，避免卡死 UI
        thread::spawn(move || {
            let result = if url.starts_with("http://") || url.starts_with("https://") {
                // 網路請求
                match reqwest::blocking::get(&url) {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            FetchResult::Success(resp.text().unwrap_or_default())
                        } else {
                            FetchResult::Error(format!("HTTP Error: {}", resp.status()))
                        }
                    }
                    Err(e) => FetchResult::Error(format!("Network Error: {}", e)),
                }
            } else {
                // 讀取本機檔案
                match std::fs::read_to_string(&url) {
                    Ok(text) => FetchResult::Success(text),
                    Err(e) => FetchResult::Error(format!("File Error: Cannot read '{}'\n{}", url, e)),
                }
            };

            let _ = tx.send(result);
            ctx.request_repaint(); // 通知介面更新
        });
    }

    // 處理相對路徑 (例如在 GitHub Repo 內點擊其他 md 檔)
    fn resolve_url(&self, target: &str) -> String {
        if target.starts_with("http") {
            return target.to_string();
        }
        if self.current_url.starts_with("http") {
            if let Some(pos) = self.current_url.rfind('/') {
                return format!("{}/{}", &self.current_url[..pos], target);
            }
        }
        target.to_string()
    }
}

impl eframe::App for MdBrowser {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 檢查背景抓取是否完成
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.content = match result {
                    FetchResult::Success(text) => text,
                    FetchResult::Error(err) => format!("# 錯誤\n\n```text\n{}\n```", err),
                };
                self.is_loading = false;
                self.rx = None;
            }
        }

        // 頂部導覽列 (網址列與按鈕)
        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("🏠 首頁").clicked() {
                    self.load_url("README.md".to_string(), ctx.clone());
                }

                // 網址列輸入框
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .hint_text("輸入網址 (http...) 或本機路徑 (README.md)")
                        .desired_width(ui.available_width() - 60.0),
                );

                // 按下 Enter 鍵或點擊 Go 按鈕
                if ui.button("Go 🚀").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.load_url(self.url_input.clone(), ctx.clone());
                }
            });
            ui.add_space(4.0);
        });

        // 主要內容區域 (Markdown 渲染)
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        CommonMarkViewer::new("md_viewer").show(
                            ui,
                            &mut self.md_cache,
                            &self.content,
                        );
                    });
            }
        });

        // 神奇魔法：攔截 egui 預設的外部連結開啟行為，改成在我們的瀏覽器內載入
        let mut clicked_link = None;
        ctx.output_mut(|o| {
            if let Some(open_url) = o.open_url.take() {
                clicked_link = Some(open_url.url);
            }
        });

        if let Some(link) = clicked_link {
            let resolved_url = self.resolve_url(&link);
            self.load_url(resolved_url, ctx.clone());
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 800.0]) // 模擬手機直立畫面尺寸
            .with_title("md4browser"),
        ..Default::default()
    };

    eframe::run_native(
        "md4browser",
        options,
        // 這裡移除了 Ok() ，直接回傳 Box::new(...) 即可
        Box::new(|cc| Box::new(MdBrowser::new(cc))),
    )
}