use plotters::style::RGBColor;

#[derive(Debug, Clone)]
pub struct PlotOptions {
    pub ptype: PlotType,
    pub col: String,
    pub ms: f64,
    pub lwd: f64,
    pub name: String,
    pub main: String,
    pub xlab: String,
    pub ylab: String,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
    pub showlegend: bool,
}

impl Default for PlotOptions {
    fn default() -> Self {
        PlotOptions {
            ptype: PlotType::Points,
            col: "#2196F3".to_string(),
            ms: 8.0,
            lwd: 2.0,
            name: String::new(),
            main: String::new(),
            xlab: String::new(),
            ylab: String::new(),
            xlim: None,
            ylim: None,
            showlegend: false,
        }
    }
}

impl PlotOptions {
    pub fn col(mut self, col: &str) -> Self {
        self.col = col.to_string();
        self
    }
    pub fn ptype(mut self, ptype: PlotType) -> Self {
        self.ptype = ptype;
        self
    }
    pub fn ms(mut self, ms: f64) -> Self {
        self.ms = ms;
        self
    }
    pub fn lwd(mut self, lwd: f64) -> Self {
        self.lwd = lwd;
        self
    }
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn main(mut self, main: &str) -> Self {
        self.main = main.to_string();
        self
    }
    pub fn xlab(mut self, xlab: &str) -> Self {
        self.xlab = xlab.to_string();
        self
    }
    pub fn ylab(mut self, ylab: &str) -> Self {
        self.ylab = ylab.to_string();
        self
    }
    pub fn xlim(mut self, xlim: (f64, f64)) -> Self {
        self.xlim = Some(xlim);
        self
    }
    pub fn ylim(mut self, ylim: (f64, f64)) -> Self {
        self.ylim = Some(ylim);
        self
    }
    pub fn showlegend(mut self, showlegend: bool) -> Self {
        self.showlegend = showlegend;
        self
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum PlotType {
    #[default]
    Points,
    Line,
    Both,
}

#[derive(Debug, Clone)]
pub struct HistOptions {
    pub breaks: usize,
    pub col: String,
    pub border: String,
    pub freq: bool,
}

impl Default for HistOptions {
    fn default() -> Self {
        HistOptions {
            breaks: 10,
            col: "#2196F3".to_string(),
            border: "#FFFFFF".to_string(),
            freq: true,
        }
    }
}

impl HistOptions {
    pub fn breaks(mut self, breaks: usize) -> Self {
        self.breaks = breaks;
        self
    }
    pub fn col(mut self, col: &str) -> Self {
        self.col = col.to_string();
        self
    }
    pub fn border(mut self, border: &str) -> Self {
        self.border = border.to_string();
        self
    }
    pub fn freq(mut self, freq: bool) -> Self {
        self.freq = freq;
        self
    }
}

#[derive(Debug, Clone)]
pub struct BoxPlotOptions {
    pub col: String,
    pub names: Vec<String>,
    pub showpoints: bool,
}

impl Default for BoxPlotOptions {
    fn default() -> Self {
        BoxPlotOptions {
            col: "#2196F3".to_string(),
            names: vec![],
            showpoints: false,
        }
    }
}

impl BoxPlotOptions {
    pub fn col(mut self, col: &str) -> Self {
        self.col = col.to_string();
        self
    }
    pub fn names(mut self, names: &[&str]) -> Self {
        self.names = names.iter().map(|s| s.to_string()).collect();
        self
    }
    pub fn showpoints(mut self, showpoints: bool) -> Self {
        self.showpoints = showpoints;
        self
    }
}

const COLOR_MAP: &[(&str, &str)] = &[
    ("red", "#F44336"),
    ("blue", "#2196F3"),
    ("green", "#4CAF50"),
    ("orange", "#FF9800"),
    ("purple", "#9C27B0"),
    ("cyan", "#00BCD4"),
    ("black", "#000000"),
    ("white", "#FFFFFF"),
    ("gray", "#607D8B"),
    ("grey", "#607D8B"),
];

pub fn parse_color(col: &str) -> RGBColor {
    if col.starts_with('#') && col.len() == 7 {
        let r = u8::from_str_radix(&col[1..3], 16).unwrap_or(33);
        let g = u8::from_str_radix(&col[3..5], 16).unwrap_or(150);
        let b = u8::from_str_radix(&col[5..7], 16).unwrap_or(243);
        return RGBColor(r, g, b);
    }
    for (name, hex) in COLOR_MAP {
        if col.eq_ignore_ascii_case(name) {
            return parse_color(hex);
        }
    }
    parse_color("#2196F3")
}