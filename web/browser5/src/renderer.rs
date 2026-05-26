use eframe::egui::{self, Color32, Frame, Margin, RichText};
use js4::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::css::{self, CssRule, Style};
use crate::js::JsRuntime;
use crate::{Node, NodeType};

pub struct Renderer<'a> {
    pub rules: &'a [CssRule],
    pub js: &'a mut JsRuntime,
    pub clicked_link: &'a mut Option<String>,
    pub button_triggered: &'a mut Option<String>,
    pub current_url: &'a str,
    pub image_cache: &'a mut HashMap<String, egui::TextureHandle>,
}

impl<'a> Renderer<'a> {
    pub fn render_node(&mut self, ui: &mut egui::Ui, node: &Node, parent_style: &Style) {
        match &node.node_type {
            NodeType::Text(text) => {
                let t = text.trim();
                if !t.is_empty() {
                    let mut rt = RichText::new(t);
                    if let Some(ref c) = parent_style.color {
                        if let Some(col) = css::parse_color(c) {
                            rt = rt.color(col);
                        }
                    }
                    if let Some(s) = parent_style.font_size {
                        rt = rt.size(s);
                    }
                    if let Some(ref w) = parent_style.font_weight {
                        if w == "bold" || w == "700" || w == "800" || w == "900" {
                            rt = rt.strong();
                        }
                    }
                    ui.label(rt);
                }
            }
            NodeType::Element(tag) => {
                let style = css::compute_style(node, self.rules);
                self.render_element(ui, node, tag, &style, parent_style);
            }
        }
    }

    fn render_element(
        &mut self,
        ui: &mut egui::Ui,
        node: &Node,
        tag: &str,
        style: &Style,
        parent_style: &Style,
    ) {
        let invisible_tags = ["head", "script", "style", "title", "meta", "link", "noscript"];
        if invisible_tags.contains(&tag) {
            return;
        }

        let display = style.display.as_deref().unwrap_or(
            if matches!(tag, "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "ul" | "ol" | "li" | "header" | "footer" | "nav" | "section" | "article" | "main" | "aside" | "blockquote" | "hr" | "form" | "table" | "tr" | "td" | "th" | "pre" | "canvas") {
                "block"
            } else {
                "inline"
            },
        );

        let bg_color = style.background_color.as_ref()
            .and_then(|c| css::parse_color(c));

        if display == "block" || tag == "html" || tag == "body" || tag == "img" || tag == "canvas" {
            self.render_block(ui, node, tag, style, bg_color);
        } else {
            self.render_inline(ui, node, tag, style, parent_style);
        }
    }

    fn render_block(
        &mut self,
        ui: &mut egui::Ui,
        node: &Node,
        tag: &str,
        style: &Style,
        bg_color: Option<Color32>,
    ) {
        if tag == "img" {
            self.render_img(ui, node);
            return;
        }
        if tag == "canvas" {
            self.render_canvas(ui, node);
            return;
        }

        if let Some(id) = node.attrs.get("id") {
            if let Some(el) = self.js.get_cached_element(id) {
                if let Value::Object(m) = el {
                    if let Some(Value::String(text)) = m.borrow().get("innerText").cloned() {
                        if !text.is_empty() {
                            let mut rt = RichText::new(&text);
                            if let Some(ref c) = style.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                            if let Some(s) = style.font_size { rt = rt.size(s); }
                            ui.label(rt);
                            return;
                        }
                    }
                }
            }
        }

        if let Some(var_name) = node.attrs.get("data-js-var") {
            let val = self.js.get_var_string(var_name).unwrap_or_else(|| "undefined".to_string());
            let mut rt = RichText::new(&val);
            if let Some(ref c) = style.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
            if let Some(s) = style.font_size { rt = rt.size(s); }
            ui.label(rt);
            return;
        }

        let margin_top = style.margin_top.unwrap_or(0.0);
        let margin_bottom = style.margin_bottom.unwrap_or(if tag == "p" { 8.0 } else { 0.0 });
        let margin_left = style.margin_left.unwrap_or(0.0);
        let margin_right = style.margin_right.unwrap_or(0.0);
        let padding_top = style.padding_top.unwrap_or(0.0);
        let padding_right = style.padding_right.unwrap_or(0.0);
        let padding_bottom = style.padding_bottom.unwrap_or(0.0);
        let padding_left = style.padding_left.unwrap_or(0.0);

        if margin_top > 0.0 { ui.add_space(margin_top); }

        let has_border = style.border_width.map_or(false, |w| w > 0.0)
            && style.border_style.as_deref() != Some("none");

        let frame = Frame::none();
        let frame = if let Some(bg) = bg_color {
            frame.fill(bg)
        } else {
            frame
        };
        let frame = if has_border {
            if let Some(bc) = style.border_color.as_ref().and_then(|c| css::parse_color(c)) {
                frame.stroke(egui::Stroke::new(style.border_width.unwrap_or(1.0), bc))
            } else {
                frame
            }
        } else {
            frame
        };
        let rounding = style.border_radius.unwrap_or(0.0);
        let frame = if rounding > 0.0 { frame.rounding(egui::Rounding::same(rounding)) } else { frame };

        let inner_margin = Margin {
            left: padding_left.max(margin_left),
            right: padding_right.max(margin_right),
            top: padding_top.max(margin_top),
            bottom: padding_bottom.max(margin_bottom),
        };
        let outer_margin_right = margin_right;
        let outer_margin_bottom = margin_bottom;

        let text_align = style.text_align.as_deref();
        let width = style.width;
        let height = style.height;

        frame
            .inner_margin(inner_margin)
            .show(ui, |ui| {
                let avail = ui.available_width();
                if let Some(w) = width {
                    if w < avail {
                        match text_align {
                            Some("center") => { ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| { self.render_children(ui, node, style); }); }
                            Some("right") => { ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| { self.render_children(ui, node, style); }); }
                            _ => { ui.set_min_width(w); self.render_children(ui, node, style); }
                        }
                    } else {
                        self.render_children(ui, node, style);
                    }
                } else {
                    match text_align {
                        Some("center") => { ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| { self.render_children(ui, node, style); }); }
                        Some("right") => { ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| { self.render_children(ui, node, style); }); }
                        _ => { self.render_children(ui, node, style); }
                    }
                }
            });

        if let Some(h) = height { ui.set_min_height(h); }

        if outer_margin_bottom > 0.0 { ui.add_space(outer_margin_bottom); }
        if outer_margin_right > 0.0 { ui.add_space(outer_margin_right); }

        if tag == "h1" || tag == "h2" || tag == "h3" || tag == "h4" || tag == "p" || tag == "div" || tag == "li" || tag == "hr" {
            ui.add_space(4.0);
        }
    }

    fn render_img(&mut self, ui: &mut egui::Ui, node: &Node) {
        let alt = node.attrs.get("alt").cloned().unwrap_or_default();
        let src = node.attrs.get("src").cloned().unwrap_or_default();
        if src.is_empty() {
            ui.label(RichText::new(if alt.is_empty() { "[Image]" } else { &alt }).color(Color32::GRAY));
            return;
        }

        let file_path = resolve_image_path(&src, self.current_url);
        if let Some(texture) = self.image_cache.get(&file_path) {
            ui.add(egui::Image::new(texture).max_width(ui.available_width()));
        } else {
            match load_image(&file_path) {
                Ok(color_image) => {
                    let texture = ui.ctx().load_texture(
                        &file_path,
                        color_image,
                        egui::TextureOptions::LINEAR,
                    );
                    ui.add(egui::Image::new(&texture).max_width(ui.available_width()));
                    self.image_cache.insert(file_path, texture);
                }
                Err(_) => {
                    let label = if !alt.is_empty() { alt } else { format!("[img: {}]", src) };
                    ui.label(RichText::new(label).color(Color32::GRAY));
                }
            }
        }
    }

    fn render_canvas(&mut self, ui: &mut egui::Ui, node: &Node) {
        let w = node.attrs.get("width")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(300.0)
            .min(ui.available_width());
        let h = node.attrs.get("height")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(150.0);

        let (rect, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 4.0, Color32::from_rgb(30, 30, 30));
        painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, Color32::from_rgb(100, 100, 100)));
        let text = format!("Canvas ({}x{})", w as u32, h as u32);
        let text_pos = rect.center_top() + egui::vec2(0.0, 8.0);
        painter.text(
            text_pos,
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(14.0),
            Color32::GRAY,
        );
        let inner = rect.shrink2(egui::vec2(10.0, 30.0));
        painter.rect_stroke(inner, 2.0, egui::Stroke::new(0.5_f32, Color32::from_rgb(60, 60, 60)));
    }

    fn render_inline(
        &mut self,
        ui: &mut egui::Ui,
        node: &Node,
        tag: &str,
        style: &Style,
        parent_style: &Style,
    ) {
        let merged = Style {
            color: style.color.clone().or_else(|| parent_style.color.clone()),
            font_size: style.font_size.or(parent_style.font_size),
            font_weight: style.font_weight.clone().or_else(|| parent_style.font_weight.clone()),
            ..Style::default()
        };

        match tag {
            "a" => {
                let text = css::get_text_only(node);
                let href = node.attrs.get("href").cloned().unwrap_or_default();
                let mut rt = RichText::new(&text);
                if let Some(ref c) = merged.color.or(Some("#0000ff".to_string())) {
                    if let Some(col) = css::parse_color(c) {
                        rt = rt.color(col);
                    }
                } else {
                    rt = rt.color(Color32::BLUE);
                }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                if ui.link(rt).clicked() {
                    *self.clicked_link = Some(href);
                }
            }
            "button" | "input" => {
                let is_button = tag == "button" || node.attrs.get("type").map_or(false, |t| t == "button" || t == "submit");
                if is_button {
                    let text = css::get_text_only(node);
                    let onclick = node.attrs.get("onclick").cloned().unwrap_or_default();
                    let label = node.attrs.get("value").cloned().unwrap_or(text);
                    let mut rt = RichText::new(&label);
                    if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                    if ui.button(rt).clicked() && !onclick.is_empty() {
                        self.js.eval_expr(&onclick);
                        if let Some(button_id) = node.attrs.get("id").or(node.attrs.get("data-js-id")) {
                            *self.button_triggered = Some(button_id.clone());
                        } else {
                            *self.button_triggered = Some("__button__".to_string());
                        }
                    }
                } else if node.attrs.get("type").map_or(false, |t| t == "text") {
                    let val = node.attrs.get("value").cloned().unwrap_or_default();
                    ui.label(RichText::new(format!("[input: {}]", val)).color(Color32::GRAY));
                } else {
                    ui.label(RichText::new(format!("[{}]", tag)).color(Color32::GRAY));
                }
            }
            "br" => {
                ui.add_space(8.0);
            }
            "hr" => {
                ui.separator();
            }
            "b" | "strong" => {
                let text = css::get_text_only(node);
                let mut rt = RichText::new(&text).strong();
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                ui.label(rt);
            }
            "i" | "em" => {
                let text = css::get_text_only(node);
                let mut rt = RichText::new(&text).italics();
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                ui.label(rt);
            }
            "u" => {
                let text = css::get_text_only(node);
                let mut rt = RichText::new(&text).underline();
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                ui.label(rt);
            }
            "s" | "del" => {
                let text = css::get_text_only(node);
                let mut rt = RichText::new(&text).strikethrough();
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                ui.label(rt);
            }
            "code" | "pre" => {
                let text = css::get_text_only(node);
                let mut rt = RichText::new(&text).code();
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                if tag == "pre" {
                    ui.add_space(4.0);
                    ui.label(rt);
                    ui.add_space(4.0);
                } else {
                    ui.label(rt);
                }
            }
            "span" | "small" | "sub" | "sup" | "mark" | "label" => {
                let js_var = node.attrs.get("data-js-var");
                let text = if let Some(var_name) = js_var {
                    self.js.get_var_string(var_name).unwrap_or_else(|| "undefined".to_string())
                } else {
                    css::get_text_only(node)
                };
                let mut rt = RichText::new(&text);
                if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                if let Some(s) = merged.font_size { rt = rt.size(s); }
                if tag == "mark" { ui.colored_label(Color32::YELLOW, rt); }
                else if tag == "small" { ui.label(rt.small()); }
                else { ui.label(rt); }
            }
            _ => {
                if node.children.is_empty() {
                    let text = css::get_text_only(node);
                    if !text.is_empty() {
                        let mut rt = RichText::new(&text);
                        if let Some(ref c) = merged.color { if let Some(col) = css::parse_color(c) { rt = rt.color(col); } }
                        if let Some(s) = merged.font_size { rt = rt.size(s); }
                        ui.label(rt);
                    }
                } else {
                    self.render_children(ui, node, &merged);
                }
            }
        }
    }

    fn is_inline_node(&self, node: &Node) -> bool {
        match &node.node_type {
            NodeType::Text(_) => true,
            NodeType::Element(tag) => {
                if tag.as_str() == "div" || tag.as_str() == "p" || tag.as_str() == "h1"
                    || tag.as_str() == "h2" || tag.as_str() == "h3"
                    || tag.as_str() == "h4" || tag.as_str() == "h5" || tag.as_str() == "h6"
                    || tag.as_str() == "ul" || tag.as_str() == "ol" || tag.as_str() == "li"
                    || tag.as_str() == "header" || tag.as_str() == "footer"
                    || tag.as_str() == "nav" || tag.as_str() == "section"
                    || tag.as_str() == "article" || tag.as_str() == "main"
                    || tag.as_str() == "aside" || tag.as_str() == "blockquote"
                    || tag.as_str() == "hr" || tag.as_str() == "form"
                    || tag.as_str() == "table" || tag.as_str() == "tr"
                    || tag.as_str() == "td" || tag.as_str() == "th"
                    || tag.as_str() == "pre" || tag.as_str() == "canvas"
                    || tag.as_str() == "html" || tag.as_str() == "body"
                {
                    return false;
                }
                let style = css::compute_style(node, self.rules);
                if style.display.as_deref() == Some("block") { false }
                else { true }
            }
        }
    }

    fn render_children(&mut self, ui: &mut egui::Ui, node: &Node, parent_style: &Style) {
        let mut inline_group: Vec<&Node> = Vec::new();
        for child in &node.children {
            if self.is_inline_node(child) {
                inline_group.push(child);
            } else {
                if !inline_group.is_empty() {
                    ui.horizontal(|ui| {
                        for c in &inline_group { self.render_node(ui, c, parent_style); }
                    });
                    inline_group.clear();
                }
                self.render_node(ui, child, parent_style);
            }
        }
        if !inline_group.is_empty() {
            ui.horizontal(|ui| {
                for c in &inline_group { self.render_node(ui, c, parent_style); }
            });
        }
    }
}

fn resolve_image_path(src: &str, current_url: &str) -> String {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("file://") {
        return src.to_string();
    }
    if src.starts_with('/') {
        return format!("{}/web{}", cwd, src);
    }
    if src.starts_with("./") || src.starts_with("../") {
        let parent = Path::new(&cwd).join("web").join(src);
        return parent.to_string_lossy().to_string();
    }

    let page_dir = Path::new(current_url).parent()
        .and_then(|p| {
            if p.as_os_str().is_empty() { None } else { Some(p.to_string_lossy().to_string()) }
        })
        .unwrap_or_else(|| "web".to_string());

    if page_dir.starts_with('/') || page_dir.starts_with("./") || page_dir.starts_with("../") {
        Path::new(&cwd).join(&page_dir).join(&src).to_string_lossy().to_string()
    } else {
        format!("{}/{}/{}", cwd, page_dir, src)
    }
}

fn load_image(file_path: &str) -> Result<egui::ColorImage, String> {
    let bytes = std::fs::read(file_path).map_err(|e| format!("Cannot read {}: {}", file_path, e))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("Cannot decode {}: {}", file_path, e))?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_vec();
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, &pixels))
}
