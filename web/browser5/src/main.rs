use eframe::egui;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use browser5::css::{self, CssRule};
use browser5::html::{self, extract_inline_css, extract_link_css, extract_scripts};
use browser5::js::JsRuntime;
use browser5::renderer::Renderer;
use browser5::Node;

struct PageData {
    dom: Node,
    css_rules: Vec<CssRule>,
    scripts: Vec<String>,
    js_output: String,
}

enum FetchResult {
    Success(PageData),
    Error(String),
}

struct Browser5 {
    url_input: String,
    current_url: String,
    page_data: Option<PageData>,
    js_runtime: JsRuntime,
    is_loading: bool,
    rx: Option<Receiver<FetchResult>>,
    history: Vec<String>,
    history_index: usize,
    image_cache: HashMap<String, egui::TextureHandle>,
    show_console: bool,
}

impl Browser5 {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Browser5 {
            url_input: "index.html".to_string(),
            current_url: "index.html".to_string(),
            page_data: None,
            js_runtime: JsRuntime::new(),
            is_loading: false,
            rx: None,
            history: vec!["index.html".to_string()],
            history_index: 0,
            image_cache: HashMap::new(),
            show_console: false,
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

        thread::spawn(move || {
            let file_path = if resolved_url.starts_with("http://") || resolved_url.starts_with("https://") {
                let _ = tx.send(FetchResult::Error("HTTP not supported; use local files".to_string()));
                ctx.request_repaint();
                return;
            } else if resolved_url.starts_with("file://") {
                resolved_url.trim_start_matches("file://").to_string()
            } else if resolved_url.starts_with('/') {
                resolved_url.to_string()
            } else if resolved_url.starts_with("./") || resolved_url.starts_with("../") {
                let cwd = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                format!("{}/{}", cwd, resolved_url)
            } else {
                format!("{}/web/{}", current_dir, resolved_url)
            };

            let html_text = match fs::read_to_string(&file_path) {
                Ok(content) => content,
                Err(e) => {
                    let _ = tx.send(FetchResult::Error(format!("File read error: {} ({})", file_path, e)));
                    ctx.request_repaint();
                    return;
                }
            };

            let dom = html::parse_html(&html_text);

            let mut css_text = String::new();
            for s in extract_inline_css(&dom) {
                css_text.push_str(&s);
                css_text.push('\n');
            }
            let link_urls = extract_link_css(&dom, "");
            for href in link_urls {
                let css_path = resolve_css_path(&href, &file_path);
                if let Ok(css_content) = fs::read_to_string(&css_path) {
                    css_text.push_str(&css_content);
                    css_text.push('\n');
                }
            }
            let css_rules = css::parse_css(&css_text);

            let scripts = extract_scripts(&dom);

            let _ = tx.send(FetchResult::Success(PageData {
                dom,
                css_rules,
                scripts,
                js_output: String::new(),
            }));
            ctx.request_repaint();
        });
    }

    fn resolve_url(&self, url: &str) -> String {
        let url = url.trim();
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://") {
            return url.to_string();
        }
        if url.starts_with('/') || url.starts_with("./") || url.starts_with("../") {
            return url.to_string();
        }
        url.to_string()
    }

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

impl eframe::App for Browser5 {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                match result {
                    FetchResult::Success(data) => {
                        let mut rt = JsRuntime::new();
                        rt.set_dom(data.dom.clone());
                        for script in &data.scripts {
                            rt.execute(script);
                        }
                        self.js_runtime = rt;
                        self.image_cache.clear();
                        self.page_data = Some(data);
                    }
                    FetchResult::Error(err) => {
                        self.page_data = Some(PageData {
                            dom: Node::new_text(&err),
                            css_rules: vec![],
                            scripts: vec![],
                            js_output: String::new(),
                        });
                    }
                }
                self.is_loading = false;
                self.rx = None;
            }
        }

        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("\u{25C0}").clicked() {
                    self.go_back(ctx.clone());
                }
                if ui.button("\u{25B6}").clicked() {
                    self.go_forward(ctx.clone());
                }
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .hint_text("page.html (in web/)")
                        .desired_width(ui.available_width() - 100.0),
                );
                if ui.button("Go").clicked()
                    || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    self.load_url(self.url_input.clone(), ctx.clone());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.is_loading {
                ui.centered_and_justified(|ui| ui.spinner());
            } else if let Some(page) = &mut self.page_data {
                let mut clicked_link: Option<String> = None;
                let mut button_triggered: Option<String> = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        let mut renderer = Renderer {
                            rules: &page.css_rules,
                            js: &mut self.js_runtime,
                            clicked_link: &mut clicked_link,
                            button_triggered: &mut button_triggered,
                            current_url: &self.current_url,
                            image_cache: &mut self.image_cache,
                        };
                        renderer.render_node(ui, &page.dom, &css::Style::default());
                    });

                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button(if self.show_console { "Console ▲" } else { "Console ▼" }).clicked() {
                            self.show_console = !self.show_console;
                        }
                    });

                    if self.show_console {
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(&self.js_runtime.output).code().color(egui::Color32::LIGHT_GREEN));
                    }
                });

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
                        } else if base_dir.is_empty() {
                            link.to_string()
                        } else {
                            format!("{}/{}", base_dir, link)
                        }
                    };
                    self.load_url(target, ctx.clone());
                }

                if button_triggered.is_some() {
                    let mut js_output = String::new();
                    js_output.push_str(&self.js_runtime.output);
                    if let Some(ref mut page) = self.page_data {
                        page.js_output = self.js_runtime.output.clone();
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a page from web/ (e.g. index.html, counter.html)");
                });
            }
        });
    }
}

fn resolve_css_path(href: &str, html_path: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") || href.starts_with("file://") {
        return href.to_string();
    }

    if href.starts_with('/') {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        return format!("{}/web{}", cwd, href);
    }

    let html_dir = Path::new(html_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if html_dir.is_empty() {
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        format!("{}/web/{}", cwd, href)
    } else {
        Path::new(&html_dir).join(href).to_string_lossy().to_string()
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Browser5 - HTML + CSS + JS Renderer"),
        ..Default::default()
    };
    eframe::run_native("browser5", options, Box::new(|cc| Box::new(Browser5::new(cc))))
}
