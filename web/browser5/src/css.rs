use crate::{Node, NodeType};
use eframe::egui;

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selectors: Vec<String>,
    pub declarations: Vec<(String, String)>,
}

pub fn parse_css(css_text: &str) -> Vec<CssRule> {
    let mut rules = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = css_text.chars().collect();

    while pos < chars.len() {
        skip_whitespace(&chars, &mut pos);
        if pos >= chars.len() { break; }
        if chars[pos] == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
            skip_comment(&chars, &mut pos);
            continue;
        }
        if chars[pos] == '@' {
            skip_at_rule(&chars, &mut pos);
            continue;
        }
        let start = pos;
        let mut depth: i32 = 0;
        while pos < chars.len() {
            match chars[pos] {
                '{' if depth == 0 => break,
                '(' | '[' => depth += 1,
                ')' | ']' => depth = (depth - 1).max(0),
                _ => {}
            }
            pos += 1;
        }
        if pos >= chars.len() || chars[pos] != '{' { pos += 1; continue; }
        let selector_str: String = chars[start..pos].iter().collect();
        pos += 1;
        let decl_start = pos;
        let mut depth = 1;
        while pos < chars.len() && depth > 0 {
            match chars[pos] {
                '{' => depth += 1,
                '}' => { depth -= 1; }
                _ => {}
            }
            pos += 1;
        }
        let block_text: String = if depth == 0 {
            chars[decl_start..pos - 1].iter().collect()
        } else {
            chars[decl_start..].iter().collect()
        };
        let declarations = parse_declaration_block(&block_text);
        let selectors: Vec<String> = selector_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !selectors.is_empty() {
            rules.push(CssRule { selectors, declarations });
        }
    }
    rules
}

fn skip_whitespace(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn skip_comment(chars: &[char], pos: &mut usize) {
    *pos += 2;
    while *pos + 1 < chars.len() {
        if chars[*pos] == '*' && chars[*pos + 1] == '/' {
            *pos += 2;
            return;
        }
        *pos += 1;
    }
    *pos = chars.len();
}

fn skip_at_rule(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos] != '}' {
        if chars[*pos] == '{' {
            let mut depth = 1;
            *pos += 1;
            while *pos < chars.len() && depth > 0 {
                if chars[*pos] == '{' { depth += 1; }
                else if chars[*pos] == '}' { depth -= 1; }
                *pos += 1;
            }
            return;
        }
        *pos += 1;
    }
    if *pos < chars.len() { *pos += 1; }
}

fn parse_declaration_block(text: &str) -> Vec<(String, String)> {
    let mut decls = Vec::new();
    for part in text.split(';') {
        let part = part.trim();
        if part.is_empty() { continue; }
        if let Some(semi) = part.find(':') {
            let prop = part[..semi].trim().to_lowercase();
            let val = part[semi + 1..].trim().to_string();
            if !prop.is_empty() {
                decls.push((prop, val));
            }
        }
    }
    decls
}

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub font_size: Option<f32>,
    pub font_weight: Option<String>,
    pub text_align: Option<String>,
    pub display: Option<String>,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub border_color: Option<String>,
    pub border_width: Option<f32>,
    pub border_style: Option<String>,
    pub border_radius: Option<f32>,
}

fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if val.ends_with("px") {
        val[..val.len() - 2].trim().parse::<f32>().ok()
    } else if val.ends_with("em") {
        val[..val.len() - 2].trim().parse::<f32>().ok().map(|v| v * 16.0)
    } else if val.ends_with("pt") {
        val[..val.len() - 2].trim().parse::<f32>().ok().map(|v| v * 1.333)
    } else {
        val.parse::<f32>().ok()
    }
}

pub fn compute_style(node: &Node, rules: &[CssRule]) -> Style {
    let mut style = Style::default();

    for rule in rules {
        let matches = rule.selectors.iter().any(|sel| matches_simple_selector(node, sel));
        if !matches { continue; }
        for (prop, val) in &rule.declarations {
            apply_property(&mut style, prop, val);
        }
    }

    if let Some(inline) = node.attrs.get("style") {
        for part in inline.split(';') {
            let part = part.trim();
            if part.is_empty() { continue; }
            if let Some(semi) = part.find(':') {
                let prop = part[..semi].trim().to_lowercase();
                let val = part[semi + 1..].trim().to_string();
                apply_property(&mut style, &prop, &val);
            }
        }
    }

    style
}

fn apply_property(style: &mut Style, prop: &str, val: &str) {
    match prop {
        "color" => style.color = Some(val.to_string()),
        "background-color" | "background" => style.background_color = Some(val.to_string()),
        "font-size" => { if let Some(v) = parse_length(val) { style.font_size = Some(v); } }
        "font-weight" => style.font_weight = Some(val.to_string()),
        "text-align" => style.text_align = Some(val.to_string()),
        "display" => style.display = Some(val.to_string()),
        "margin" | "margin-top" | "margin-right" | "margin-bottom" | "margin-left"
            => apply_margin(style, prop, val),
        "padding" | "padding-top" | "padding-right" | "padding-bottom" | "padding-left"
            => apply_padding(style, prop, val),
        "width" => style.width = parse_length(val),
        "height" => style.height = parse_length(val),
        "border" => apply_border(style, val),
        "border-color" => style.border_color = Some(val.to_string()),
        "border-width" => { style.border_width = parse_length(val); }
        "border-style" => style.border_style = Some(val.to_string()),
        "border-radius" => { style.border_radius = parse_length(val); }
        _ => {}
    }
}

fn apply_margin(style: &mut Style, prop: &str, val: &str) {
    if prop == "margin" {
        let parts: Vec<f32> = val.split_ascii_whitespace().filter_map(|s| parse_length(s)).collect();
        if parts.len() == 1 {
            style.margin_top = Some(parts[0]); style.margin_right = Some(parts[0]);
            style.margin_bottom = Some(parts[0]); style.margin_left = Some(parts[0]);
        } else if parts.len() == 2 {
            style.margin_top = Some(parts[0]); style.margin_bottom = Some(parts[0]);
            style.margin_right = Some(parts[1]); style.margin_left = Some(parts[1]);
        } else if parts.len() == 4 {
            style.margin_top = Some(parts[0]); style.margin_right = Some(parts[1]);
            style.margin_bottom = Some(parts[2]); style.margin_left = Some(parts[3]);
        }
    } else {
        let v = parse_length(val);
        match prop {
            "margin-top" => style.margin_top = v,
            "margin-right" => style.margin_right = v,
            "margin-bottom" => style.margin_bottom = v,
            "margin-left" => style.margin_left = v,
            _ => {}
        }
    }
}

fn apply_padding(style: &mut Style, prop: &str, val: &str) {
    if prop == "padding" {
        let parts: Vec<f32> = val.split_ascii_whitespace().filter_map(|s| parse_length(s)).collect();
        if parts.len() == 1 {
            style.padding_top = Some(parts[0]); style.padding_right = Some(parts[0]);
            style.padding_bottom = Some(parts[0]); style.padding_left = Some(parts[0]);
        } else if parts.len() == 2 {
            style.padding_top = Some(parts[0]); style.padding_bottom = Some(parts[0]);
            style.padding_right = Some(parts[1]); style.padding_left = Some(parts[1]);
        } else if parts.len() == 4 {
            style.padding_top = Some(parts[0]); style.padding_right = Some(parts[1]);
            style.padding_bottom = Some(parts[2]); style.padding_left = Some(parts[3]);
        }
    } else {
        let v = parse_length(val);
        match prop {
            "padding-top" => style.padding_top = v,
            "padding-right" => style.padding_right = v,
            "padding-bottom" => style.padding_bottom = v,
            "padding-left" => style.padding_left = v,
            _ => {}
        }
    }
}

fn apply_border(style: &mut Style, val: &str) {
    for part in val.split_ascii_whitespace() {
        if let Some(v) = parse_length(part) {
            style.border_width = Some(v);
        } else if matches!(part, "solid" | "dashed" | "dotted" | "none") {
            style.border_style = Some(part.to_string());
        } else {
            style.border_color = Some(part.to_string());
        }
    }
}

fn matches_simple_selector(node: &Node, selector: &str) -> bool {
    let selector = selector.trim();
    if selector.is_empty() { return false; }

    let name = match &node.node_type {
        NodeType::Text(_) => return false,
        NodeType::Element(n) => n,
    };

    let mut i = 0;
    let chars: Vec<char> = selector.chars().collect();

    let has_tag_prefix = i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '*' || chars[i] == '_');
    let _has_special_prefix = i < chars.len() && (chars[i] == '#' || chars[i] == '.');

    if has_tag_prefix {
        if chars[i] == '*' {
            i += 1;
        } else {
            let mut tag = String::new();
            while i < chars.len() && chars[i].is_ascii_alphanumeric() && chars[i] != '#' && chars[i] != '.' {
                tag.push(chars[i]);
                i += 1;
            }
            if *name != tag { return false; }
        }
    }

    while i < chars.len() {
        match chars[i] {
            '#' => {
                i += 1;
                let mut id = String::new();
                while i < chars.len() && chars[i] != '.' && chars[i] != '#' {
                    id.push(chars[i]);
                    i += 1;
                }
                if !node.attrs.get("id").map_or(false, |v| v == &id) {
                    return false;
                }
            }
            '.' => {
                i += 1;
                let mut cls = String::new();
                while i < chars.len() && chars[i] != '.' && chars[i] != '#' {
                    cls.push(chars[i]);
                    i += 1;
                }
                if !node.attrs.get("class").map_or(false, |v| {
                    v.split_ascii_whitespace().any(|c| c == cls)
                }) {
                    return false;
                }
            }
            _ => { i += 1; }
        }
    }

    true
}

pub fn parse_color(color_str: &str) -> Option<egui::Color32> {
    let s = color_str.trim().to_lowercase();
    if s.starts_with('#') {
        let hex = &s[1..];
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(egui::Color32::from_rgb(r, g, b));
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some(egui::Color32::from_rgb(r, g, b));
        }
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().ok()?;
            let g = parts[1].trim().parse::<u8>().ok()?;
            let b = parts[2].trim().parse::<u8>().ok()?;
            return Some(egui::Color32::from_rgb(r, g, b));
        }
    }
    match s.as_str() {
        "red" => Some(egui::Color32::RED),
        "green" => Some(egui::Color32::GREEN),
        "blue" => Some(egui::Color32::BLUE),
        "black" => Some(egui::Color32::BLACK),
        "white" => Some(egui::Color32::WHITE),
        "gray" | "grey" => Some(egui::Color32::GRAY),
        "yellow" => Some(egui::Color32::YELLOW),
        "orange" => Some(egui::Color32::from_rgb(255, 165, 0)),
        "purple" => Some(egui::Color32::from_rgb(128, 0, 128)),
        "pink" => Some(egui::Color32::from_rgb(255, 192, 203)),
        "brown" => Some(egui::Color32::from_rgb(165, 42, 42)),
        "cyan" => Some(egui::Color32::from_rgb(0, 255, 255)),
        "magenta" => Some(egui::Color32::from_rgb(255, 0, 255)),
        "transparent" => Some(egui::Color32::TRANSPARENT),
        "lightgray" | "lightgrey" => Some(egui::Color32::LIGHT_GRAY),
        "darkgray" | "darkgrey" => Some(egui::Color32::DARK_GRAY),
        _ => None,
    }
}

pub fn get_text_only(node: &Node) -> String {
    let mut text = String::new();
    match &node.node_type {
        NodeType::Text(t) => text.push_str(t),
        NodeType::Element(_) => {
            for c in &node.children {
                text.push_str(&get_text_only(c));
            }
        }
    }
    text
}
