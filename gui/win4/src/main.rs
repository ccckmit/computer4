use eframe::egui;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct DirEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    size: u64,
    modified: String,
}

impl DirEntry {
    fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let metadata = fs::metadata(&path).ok()?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        let modified = metadata.modified().ok()
            .map(|t| {
                let datetime: chrono::DateTime<chrono::Local> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());

        Some(Self { name, path, is_dir, size, modified })
    }

    fn format_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.is_dir { return String::new(); }

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

struct FileManagerApp {
    current_path: PathBuf,
    entries: Vec<DirEntry>,
    selected_index: Option<usize>,
    error_message: Option<String>,
    path_input: String,
}

impl FileManagerApp {
    fn new() -> Self {
        let current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let mut app = Self {
            current_path: current_path.clone(),
            entries: Vec::new(),
            selected_index: None,
            error_message: None,
            path_input: current_path.to_string_lossy().to_string(),
        };
        app.load_directory();
        app
    }

    fn load_directory(&mut self) {
        self.entries.clear();
        self.error_message = None;

        let read_dir = match fs::read_dir(&self.current_path) {
            Ok(r) => r,
            Err(e) => {
                self.error_message = Some(format!("Cannot read directory: {}", e));
                return;
            }
        };

        for entry in read_dir.filter_map(|e| e.ok()) {
            if let Some(dir_entry) = DirEntry::from_path(entry.path()) {
                self.entries.push(dir_entry);
            }
        }

        self.entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }

    fn go_to_parent(&mut self) {
        if let Some(parent) = self.current_path.parent() {
            self.current_path = parent.to_path_buf();
            self.path_input = self.current_path.to_string_lossy().to_string();
            self.selected_index = None;
            self.load_directory();
        }
    }

    fn navigate_to(&mut self, index: usize) {
        if index < self.entries.len() && self.entries[index].is_dir {
            self.current_path = self.entries[index].path.clone();
            self.path_input = self.current_path.to_string_lossy().to_string();
            self.selected_index = None;
            self.load_directory();
        }
    }

    fn navigate_to_path(&mut self, path_str: &str) {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            self.current_path = path;
            self.path_input = self.current_path.to_string_lossy().to_string();
            self.selected_index = None;
            self.load_directory();
        } else if path.exists() {
            self.error_message = Some(format!("Not a directory: {}", path_str));
        } else {
            self.error_message = Some(format!("Path not found: {}", path_str));
        }
    }
}

impl eframe::App for FileManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let nav_height = 36.0;
        let status_height = 36.0;
        let spacing = 8.0;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_ui(egui::vec2(ui.available_width(), nav_height), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("◀").clicked() { self.go_to_parent(); }
                    if ui.button("⟳").clicked() { self.load_directory(); }
                    if ui.button("🏠").clicked() {
                        if let Some(home) = dirs::home_dir() {
                            self.navigate_to_path(&home.to_string_lossy());
                        }
                    }
                    let le = ui.add(egui::TextEdit::singleline(&mut self.path_input)
                        .hint_text("Enter path...")
                        .desired_width(ui.available_width() - 20.0));
                    if le.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let path = self.path_input.clone();
                        self.navigate_to_path(&path);
                    }
                });
            });

            ui.add_space(spacing);

            let available_after_nav = ui.available_height() - nav_height - status_height - spacing * 2.0;
            ui.allocate_ui(egui::vec2(ui.available_width(), available_after_nav), |ui| {
                let item_height = 36.0;
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show_rows(ui, item_height, self.entries.len(), |ui, row_range| {
                        for idx in row_range {
                            if idx >= self.entries.len() { continue; }
                            let entry = &self.entries[idx];
                            let is_selected = self.selected_index == Some(idx);

                            let response = ui.selectable_label(
                                is_selected,
                                egui::RichText::new(format!(
                                    "{}  {:<40}  {:>10}  {}",
                                    if entry.is_dir { "📁" } else { "📄" },
                                    entry.name,
                                    entry.format_size(),
                                    entry.modified
                                )),
                            );

                            if response.clicked() { self.selected_index = Some(idx); }
                            if response.double_clicked() && entry.is_dir { self.navigate_to(idx); }
                        }
                    });
            });

            ui.add_space(spacing);

            ui.allocate_ui(egui::vec2(ui.available_width(), status_height), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{} items", self.entries.len()));
                    if let Some(ref err) = self.error_message {
                        ui.add_space(20.0);
                        ui.colored_label(egui::Color32::RED, err);
                    }
                    ui.add_space(20.0);
                    if self.selected_index.map(|i| i < self.entries.len() && self.entries[i].is_dir).unwrap_or(false) {
                        if ui.button("Open").clicked() {
                            if let Some(idx) = self.selected_index { self.navigate_to(idx); }
                        }
                    }
                });
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_title("Win4 File Manager"),
        ..Default::default()
    };

    eframe::run_native(
        "Win4 File Manager",
        options,
        Box::new(|cc| {
            let app = FileManagerApp::new();

            let mut fonts = egui::FontDefinitions::default();
            if let Ok(font_data) = std::fs::read("font/font.ttf") {
                fonts.font_data.insert("chinese".to_owned(), egui::FontData::from_owned(font_data).into());
                fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".to_owned());
                fonts.families.entry(egui::FontFamily::Monospace).or_default().push("chinese".to_owned());
            }
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(app))
        }),
    )
    .expect("Failed to run file manager");
}