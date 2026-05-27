use serde::Serialize;

const CDN_PLOTLY: &str = "https://cdn.plot.ly/plotly-2.35.2.min.js";

const HTML_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="zh-TW">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<script src="{CDN}"></script>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;background:#fff}
#chart{width:100%;height:100vh}
</style>
</head>
<body>
<div id="chart"></div>
<script>
var data = {DATA};
var layout = {LAYOUT};
var config = {CONFIG};
Plotly.newPlot('chart', data, layout, config);
</script>
</body>
</html>"#;

// ── Figure ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Figure {
    data: Vec<Trace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<Layout>,
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<Config>,
}

impl Figure {
    pub fn new() -> Self {
        Figure { data: Vec::new(), layout: None, config: None }
    }

    pub fn add_trace(&mut self, trace: Trace) -> &mut Self {
        self.data.push(trace);
        self
    }

    pub fn layout(&mut self, layout: Layout) -> &mut Self {
        self.layout = Some(layout);
        self
    }

    pub fn title(&mut self, text: &str) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).title = Some(title(text));
        self
    }

    pub fn width(&mut self, w: usize) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).width = Some(w);
        self
    }

    pub fn height(&mut self, h: usize) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).height = Some(h);
        self
    }

    pub fn autosize(&mut self, v: bool) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).autosize = Some(v);
        self
    }

    pub fn showlegend(&mut self, v: bool) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).showlegend = Some(v);
        self
    }

    pub fn barmode(&mut self, mode: &str) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).barmode = Some(mode.to_string());
        self
    }

    pub fn hovermode(&mut self, mode: &str) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).hovermode = Some(mode.to_string());
        self
    }

    pub fn xaxis(&mut self, axis: Axis) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).xaxis = Some(axis);
        self
    }

    pub fn yaxis(&mut self, axis: Axis) -> &mut Self {
        self.layout.get_or_insert(Layout::default()).yaxis = Some(axis);
        self
    }

    fn render(&self) -> String {
        let data_json = serde_json::to_string_pretty(&self.data).unwrap_or_default();
        let layout_json = serde_json::to_string(&self.layout).unwrap_or_else(|_| "null".into());
        let config_json = serde_json::to_string(&self.config).unwrap_or_else(|_| "null".into());
        HTML_TEMPLATE
            .replace("{CDN}", CDN_PLOTLY)
            .replace("{DATA}", &data_json)
            .replace("{LAYOUT}", &layout_json)
            .replace("{CONFIG}", &config_json)
    }

    pub fn to_html(&self) -> String {
        self.render()
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        std::fs::write(path.as_ref(), self.render())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = std::env::temp_dir().join("chart4.html");
        self.save(&path)?;
        webbrowser::open(path.to_str().unwrap())?;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn serve(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let html = self.render();
        let addr = format!("127.0.0.1:{}", port);
        let listener = std::net::TcpListener::bind(&addr)?;
        println!("chart4 server → http://{}", addr);
        webbrowser::open(&format!("http://{}", addr))?;
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    use std::io::{BufRead, Write};
                    let mut reader = std::io::BufReader::new(&s);
                    let mut line = String::new();
                    let mut path = String::new();
                    while reader.read_line(&mut line).unwrap_or(0) > 0 {
                        if line.starts_with("GET ") {
                            path = line.split_whitespace().nth(1).unwrap_or("").to_string();
                        }
                        if line == "\r\n" || line == "\n" { break; }
                        line.clear();
                    }
                    let (status, body, ctype) = if path == "/data" {
                        let json = serde_json::to_string_pretty(&self.data).unwrap_or_default();
                        (200, json, "application/json")
                    } else if path.starts_with("/") {
                        (200, html.clone(), "text/html; charset=utf-8")
                    } else {
                        (404, "Not Found".into(), "text/plain")
                    };
                    let resp = format!(
                        "HTTP/1.1 {status} OK\r\nContent-Type: {ctype}\r\nContent-Length: {len}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{body}",
                        status = status, ctype = ctype, len = body.len(), body = body
                    );
                    s.write_all(resp.as_bytes()).ok();
                }
                Err(_) => break,
            }
        }
        Ok(())
    }
}

impl Default for Figure {
    fn default() -> Self { Self::new() }
}

// ── Trace ───────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Trace {
    #[serde(rename = "type")]
    trace_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    marker: Option<Marker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<Line>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    values: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nbinsx: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hole: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    colorscale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reversescale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    showscale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    connectgaps: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fillcolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    yaxis: Option<String>,
}

fn num_vec(v: Vec<f64>) -> Vec<serde_json::Value> {
    v.into_iter().map(|n| serde_json::json!(n)).collect()
}

fn str_vec(v: Vec<&str>) -> Vec<serde_json::Value> {
    v.into_iter().map(|s| serde_json::json!(s)).collect()
}

fn new_trace(trace_type: &str) -> Trace {
    Trace {
        trace_type: trace_type.to_string(),
        x: None, y: None, mode: None, name: None,
        marker: None, line: None, text: None,
        labels: None, values: None, nbinsx: None,
        orientation: None, opacity: None, hole: None,
        colorscale: None, reversescale: None, showscale: None,
        connectgaps: None, fill: None, fillcolor: None,
        yaxis: None,
    }
}

impl Trace {
    pub fn line(x: Vec<f64>, y: Vec<f64>) -> Self {
        let mut t = new_trace("scatter");
        t.x = Some(num_vec(x));
        t.y = Some(num_vec(y));
        t.mode = Some("lines".into());
        t
    }

    pub fn scatter(x: Vec<f64>, y: Vec<f64>) -> Self {
        let mut t = new_trace("scatter");
        t.x = Some(num_vec(x));
        t.y = Some(num_vec(y));
        t.mode = Some("markers".into());
        t
    }

    pub fn line_scatter(x: Vec<f64>, y: Vec<f64>) -> Self {
        let mut t = new_trace("scatter");
        t.x = Some(num_vec(x));
        t.y = Some(num_vec(y));
        t.mode = Some("lines+markers".into());
        t
    }

    pub fn bar(x: Vec<f64>, y: Vec<f64>) -> Self {
        let mut t = new_trace("bar");
        t.x = Some(num_vec(x));
        t.y = Some(num_vec(y));
        t
    }

    pub fn barh(x: Vec<f64>, y: Vec<f64>) -> Self {
        let mut t = new_trace("bar");
        t.y = Some(num_vec(x));
        t.x = Some(num_vec(y));
        t.orientation = Some("h".into());
        t
    }

    pub fn pie(labels: Vec<&str>, values: Vec<f64>) -> Self {
        let mut t = new_trace("pie");
        t.labels = Some(labels.into_iter().map(String::from).collect());
        t.values = Some(values);
        t
    }

    pub fn histogram(x: Vec<f64>) -> Self {
        let mut t = new_trace("histogram");
        t.x = Some(num_vec(x));
        t
    }

    pub fn line_str(x: Vec<&str>, y: Vec<f64>) -> Self {
        let mut t = new_trace("scatter");
        t.x = Some(str_vec(x));
        t.y = Some(num_vec(y));
        t.mode = Some("lines+markers".into());
        t
    }

    // ── consuming builder ──

    pub fn mode(mut self, mode: &str) -> Self { self.mode = Some(mode.to_string()); self }
    pub fn name(mut self, name: &str) -> Self { self.name = Some(name.to_string()); self }
    pub fn opacity(mut self, o: f64) -> Self { self.opacity = Some(o); self }
    pub fn hole(mut self, h: f64) -> Self { self.hole = Some(h); self }
    pub fn orientation(mut self, o: &str) -> Self { self.orientation = Some(o.to_string()); self }
    pub fn fill(mut self, f: &str) -> Self { self.fill = Some(f.to_string()); self }
    pub fn fillcolor(mut self, c: &str) -> Self { self.fillcolor = Some(c.to_string()); self }
    pub fn connectgaps(mut self, v: bool) -> Self { self.connectgaps = Some(v); self }
    pub fn nbinsx(mut self, n: usize) -> Self { self.nbinsx = Some(n); self }
    pub fn text(mut self, t: Vec<&str>) -> Self {
        self.text = Some(t.into_iter().map(String::from).collect()); self
    }
    pub fn yaxis(mut self, axis: &str) -> Self { self.yaxis = Some(axis.to_string()); self }

    pub fn marker(mut self, m: Marker) -> Self { self.marker = Some(m); self }
    pub fn line_style(mut self, l: Line) -> Self { self.line = Some(l); self }

    pub fn marker_color(mut self, color: &str) -> Self {
        self.marker.get_or_insert(Marker::default()).color = Some(color.to_string());
        self
    }
    pub fn marker_size(mut self, size: f64) -> Self {
        self.marker.get_or_insert(Marker::default()).size = Some(size);
        self
    }

    pub fn line_color(mut self, color: &str) -> Self {
        self.line.get_or_insert(Line::default()).color = Some(color.to_string());
        self
    }
    pub fn line_width(mut self, w: f64) -> Self {
        self.line.get_or_insert(Line::default()).width = Some(w);
        self
    }
    pub fn line_dash(mut self, d: &str) -> Self {
        self.line.get_or_insert(Line::default()).dash = Some(d.to_string());
        self
    }
}

// ── Marker ──────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Marker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colorscale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showscale: Option<bool>,
}

// ── Line ────────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Line {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,
}

// ── Layout ──────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Layout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autosize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showlegend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barmode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bargroupgap: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hovermode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_bgcolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plot_bgcolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xaxis: Option<Axis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yaxis: Option<Axis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yaxis2: Option<Axis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<Margin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colorway: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

pub fn title(text: &str) -> Title {
    Title { text: text.to_string(), font: None }
}

#[derive(Serialize)]
pub struct Title {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
}

// ── Axis ────────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Axis {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zeroline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showgrid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gridcolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gridwidth: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dtick: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tickangle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub showline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linecolor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linewidth: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categoryorder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlaying: Option<String>,
}

pub fn xaxis() -> Axis { Axis::default() }
pub fn yaxis() -> Axis { Axis::default() }

// ── Margin ──────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Margin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub b: Option<usize>,
}

// ── Font ────────────────────────────────────────────────────

#[derive(Serialize, Default)]
pub struct Font {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

// ── Config ──────────────────────────────────────────────────

#[derive(Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displaylogo: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_mode_bar: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_zoom: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_plot: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            responsive: Some(true),
            displaylogo: Some(false),
            display_mode_bar: None,
            scroll_zoom: None,
            static_plot: None,
        }
    }
}

// ── 便利工具 ───────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub fn quick_line(x: Vec<f64>, y: Vec<f64>, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut fig = Figure::new();
    fig.add_trace(Trace::line(x, y));
    fig.title(title);
    fig.show()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn quick_pie(labels: Vec<&str>, values: Vec<f64>, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut fig = Figure::new();
    fig.add_trace(Trace::pie(labels, values));
    fig.title(title);
    fig.show()
}
