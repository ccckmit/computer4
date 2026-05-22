use crate::plot::device::{get_device, png, svg, dev_off};
use crate::plot::options::{parse_color, BoxPlotOptions, HistOptions, PlotOptions, PlotType};
use plotters::chart::ChartBuilder;
use plotters::drawing::IntoDrawingArea;
use plotters::element::{Circle, Rectangle};
use plotters::style::Color;
use plotters::style::RGBColor;
use std::cmp::Ordering;
use std::collections::HashMap;

fn range(data: &[f64]) -> (f64, f64) {
    if data.is_empty() { return (0.0, 1.0); }
    let mn = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let p = if mx == mn { 0.5 } else { (mx - mn) * 0.05 };
    (mn - p, mx + p)
}

fn seg(x1: f64, y1: f64, x2: f64, y2: f64, sty: impl Into<plotters::style::ShapeStyle>) -> Rectangle<(f64, f64)> {
    Rectangle::new([(x1, y1), (x2, y2)], sty)
}

macro_rules! draw_seg {
    ($area:expr, $x1:expr, $y1:expr, $x2:expr, $y2:expr, $sty:expr) => {
        if ($y2 - $y1).abs() < 1e-10 { $area.draw(&Rectangle::new([($x1, $y1), ($x2, $y1 + 1e-10)], $sty)); }
        else if ($x2 - $x1).abs() < 1e-10 { $area.draw(&Rectangle::new([($x1, $y1), ($x1 + 1e-10, $y2)], $sty)); }
        else { $area.draw(&Rectangle::new([($x1, $y1), ($x2, $y2)], $sty)); }
    };
}

pub fn plot(x: &[f64], y: &[f64], ptype: PlotType, options: PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let params = get_device().ok_or("No device opened")?;
    let pts: Vec<(f64, f64)> = x.iter().zip(y.iter()).map(|(&a, &b)| (a, b)).collect();
    let (xmn, xmx) = range(x);
    let (ymn, ymx) = range(y);
    let col = parse_color(&options.col);
    let ms = options.ms as i32;
    let lw = options.lwd as u32;

    match params.device_type {
        crate::plot::device::DeviceType::PNG => {
            let be = plotters::backend::BitMapBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).caption(&options.main, ("sans-serif", 20))
                .x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(options.xlim.map(|(a,b)| a..b).unwrap_or(xmn..xmx),
                                       options.ylim.map(|(a,b)| a..b).unwrap_or(ymn..ymx))?;
            ch.configure_mesh().x_desc(&options.xlab).y_desc(&options.ylab).draw()?;
            let area = ch.plotting_area();
            match ptype {
                PlotType::Points => {
                    for &(xv, yv) in &pts { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
                }
                PlotType::Line => {
                    for w in pts.windows(2) { draw_seg!(area, w[0].0, w[0].1, w[1].0, w[1].1, col.stroke_width(lw)); }
                }
                PlotType::Both => {
                    for w in pts.windows(2) { draw_seg!(area, w[0].0, w[0].1, w[1].0, w[1].1, col.stroke_width(lw)); }
                    for &(xv, yv) in &pts { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
                }
            }
            root.present()?;
        }
        crate::plot::device::DeviceType::SVG => {
            let be = plotters::backend::SVGBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).caption(&options.main, ("sans-serif", 20))
                .x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(options.xlim.map(|(a,b)| a..b).unwrap_or(xmn..xmx),
                                       options.ylim.map(|(a,b)| a..b).unwrap_or(ymn..ymx))?;
            ch.configure_mesh().x_desc(&options.xlab).y_desc(&options.ylab).draw()?;
            let area = ch.plotting_area();
            match ptype {
                PlotType::Points => {
                    for &(xv, yv) in &pts { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
                }
                PlotType::Line => {
                    for w in pts.windows(2) { draw_seg!(area, w[0].0, w[0].1, w[1].0, w[1].1, col.stroke_width(lw)); }
                }
                PlotType::Both => {
                    for w in pts.windows(2) { draw_seg!(area, w[0].0, w[0].1, w[1].0, w[1].1, col.stroke_width(lw)); }
                    for &(xv, yv) in &pts { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
                }
            }
            root.present()?;
        }
    }
    Ok(())
}

pub fn hist(x: &[f64], options: HistOptions) -> Result<(), Box<dyn std::error::Error>> {
    let params = get_device().ok_or("No device opened")?;
    let mut data = x.to_vec();
    if data.is_empty() { return Ok(()); }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let mn = data[0];
    let bs = if data.len() > 1 { (data[data.len()-1] - mn) / options.breaks as f64 } else { 1.0 };
    let mut cnt: HashMap<usize, usize> = HashMap::new();
    for &v in &data {
        let idx = if bs > 0.0 { ((v - mn) / bs).floor() as usize } else { 0 };
        *cnt.entry(idx).or_insert(0) += 1;
    }
    let (xmn, xmx) = range(&data);
    let ymx = if options.freq { *cnt.values().max().unwrap_or(&1) as f64 }
              else { *cnt.values().max().unwrap_or(&1) as f64 / (data.len() as f64 * bs) };
    let col = parse_color(&options.col);

    match params.device_type {
        crate::plot::device::DeviceType::PNG => {
            let be = plotters::backend::BitMapBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(xmn..xmx, 0.0..ymx * 1.1)?;
            ch.configure_mesh().x_desc("Value").y_desc(if options.freq { "Frequency" } else { "Density" }).draw()?;
            let area = ch.plotting_area();
            for i in 0..options.breaks {
                let c = *cnt.get(&i).unwrap_or(&0) as f64;
                let h = if options.freq { c } else { c / (data.len() as f64 * bs) };
                let xc = if bs > 0.0 { mn + (i as f64 + 0.5) * bs } else { mn };
                area.draw(&Rectangle::new([(xc - bs * 0.4, 0.0), (xc + bs * 0.4, h)], col.filled()));
            }
            root.present()?;
        }
        crate::plot::device::DeviceType::SVG => {
            let be = plotters::backend::SVGBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(xmn..xmx, 0.0..ymx * 1.1)?;
            ch.configure_mesh().x_desc("Value").y_desc(if options.freq { "Frequency" } else { "Density" }).draw()?;
            let area = ch.plotting_area();
            for i in 0..options.breaks {
                let c = *cnt.get(&i).unwrap_or(&0) as f64;
                let h = if options.freq { c } else { c / (data.len() as f64 * bs) };
                let xc = if bs > 0.0 { mn + (i as f64 + 0.5) * bs } else { mn };
                area.draw(&Rectangle::new([(xc - bs * 0.4, 0.0), (xc + bs * 0.4, h)], col.filled()));
            }
            root.present()?;
        }
    }
    Ok(())
}

pub fn boxplot(data: &[&[f64]], options: BoxPlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let params = get_device().ok_or("No device opened")?;
    let mut gd: Vec<Vec<f64>> = data.iter().map(|g| { let mut v = g.to_vec(); v.sort_by(|a,b| a.partial_cmp(b).unwrap_or(Ordering::Equal)); v }).collect();
    let all: Vec<f64> = gd.iter().flat_map(|g| g.iter().cloned()).collect();
    if all.is_empty() { return Ok(()); }
    let n = gd.len();
    let (ymn, ymx) = range(&all);
    let col = parse_color(&options.col);

    match params.device_type {
        crate::plot::device::DeviceType::PNG => {
            let be = plotters::backend::BitMapBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(0.5..(n as f64) + 0.5, ymn..ymx)?;
            ch.configure_mesh().x_desc("Group").y_desc("Value").draw()?;
            let area = ch.plotting_area();
            for (i, g) in gd.iter().enumerate() {
                if g.is_empty() { continue; }
                let nn = g.len();
                let q1 = g[((nn as f64 * 0.25).floor() as usize).min(nn - 1)];
                let med = g[((nn as f64 * 0.5).floor() as usize).min(nn - 1)];
                let q3 = g[((nn as f64 * 0.75).floor() as usize).min(nn - 1)];
                let iqr = q3 - q1;
                let lo = g.iter().filter(|&&v| v >= q1 - 1.5 * iqr).cloned().fold(f64::INFINITY, f64::min);
                let hi = g.iter().filter(|&&v| v <= q3 + 1.5 * iqr).cloned().fold(f64::NEG_INFINITY, f64::max);
                let xc = (i + 1) as f64;
                let bw = 0.5;
                area.draw(&Rectangle::new([(xc - bw / 2.0, q1), (xc + bw / 2.0, q3)], col.filled().stroke_width(2)));
                draw_seg!(area, xc, lo, xc, q1, col.stroke_width(2));
                draw_seg!(area, xc, q3, xc, hi, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, lo, xc + bw / 4.0, lo, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, hi, xc + bw / 4.0, hi, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, med, xc + bw / 4.0, med, RGBColor(0, 0, 0).stroke_width(3));
                if options.showpoints {
                    for &v in g { area.draw(&Circle::new((xc, v), 3, col.filled())); }
                }
            }
            root.present()?;
        }
        crate::plot::device::DeviceType::SVG => {
            let be = plotters::backend::SVGBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).x_label_area_size(40).y_label_area_size(40)
                .build_cartesian_2d(0.5..(n as f64) + 0.5, ymn..ymx)?;
            ch.configure_mesh().x_desc("Group").y_desc("Value").draw()?;
            let area = ch.plotting_area();
            for (i, g) in gd.iter().enumerate() {
                if g.is_empty() { continue; }
                let nn = g.len();
                let q1 = g[((nn as f64 * 0.25).floor() as usize).min(nn - 1)];
                let med = g[((nn as f64 * 0.5).floor() as usize).min(nn - 1)];
                let q3 = g[((nn as f64 * 0.75).floor() as usize).min(nn - 1)];
                let iqr = q3 - q1;
                let lo = g.iter().filter(|&&v| v >= q1 - 1.5 * iqr).cloned().fold(f64::INFINITY, f64::min);
                let hi = g.iter().filter(|&&v| v <= q3 + 1.5 * iqr).cloned().fold(f64::NEG_INFINITY, f64::max);
                let xc = (i + 1) as f64;
                let bw = 0.5;
                area.draw(&Rectangle::new([(xc - bw / 2.0, q1), (xc + bw / 2.0, q3)], col.filled().stroke_width(2)));
                draw_seg!(area, xc, lo, xc, q1, col.stroke_width(2));
                draw_seg!(area, xc, q3, xc, hi, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, lo, xc + bw / 4.0, lo, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, hi, xc + bw / 4.0, hi, col.stroke_width(2));
                draw_seg!(area, xc - bw / 4.0, med, xc + bw / 4.0, med, RGBColor(0, 0, 0).stroke_width(3));
                if options.showpoints {
                    for &v in g { area.draw(&Circle::new((xc, v), 3, col.filled())); }
                }
            }
            root.present()?;
        }
    }
    Ok(())
}

pub fn qqnorm(x: &[f64], options: PlotOptions) -> Result<(), Box<dyn std::error::Error>> {
    let params = get_device().ok_or("No device opened")?;
    let mut sorted = x.to_vec();
    if sorted.is_empty() { return Ok(()); }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let n = sorted.len() as f64;
    let p: Vec<f64> = (0..sorted.len()).map(|i| ((i + 1) as f64 - 0.5) / n).collect();
    let th: Vec<f64> = p.iter().map(|&pi| normal_inv(pi)).collect();
    let mu = sorted.iter().sum::<f64>() / n;
    let vr = sorted.iter().map(|&v| (v - mu).powi(2)).sum::<f64>() / (n - 1.0);
    let ss: Vec<f64> = sorted.iter().map(|&v| (v - mu) / vr.sqrt()).collect();
    let (xmn, xmx) = range(&th);
    let (ymn, ymx) = range(&ss);
    let rm = xmn.min(ymn);
    let r_m = xmx.max(ymx);
    let col = parse_color(&options.col);
    let ms = options.ms as i32;

    match params.device_type {
        crate::plot::device::DeviceType::PNG => {
            let be = plotters::backend::BitMapBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).caption(&options.main, ("sans-serif", 20))
                .x_label_area_size(40).y_label_area_size(40).build_cartesian_2d(xmn..xmx, ymn..ymx)?;
            ch.configure_mesh().x_desc("Theoretical Quantiles").y_desc("Sample Quantiles").draw()?;
            let area = ch.plotting_area();
            for (&xv, &yv) in th.iter().zip(ss.iter()) { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
            draw_seg!(area, rm, rm, r_m, r_m, RGBColor(244, 67, 54).stroke_width(2));
            root.present()?;
        }
        crate::plot::device::DeviceType::SVG => {
            let be = plotters::backend::SVGBackend::new(&params.filename, (params.width, params.height));
            let root = be.into_drawing_area();
            root.fill(&RGBColor(255, 255, 255))?;
            let mut ch = ChartBuilder::on(&root).margin(20).caption(&options.main, ("sans-serif", 20))
                .x_label_area_size(40).y_label_area_size(40).build_cartesian_2d(xmn..xmx, ymn..ymx)?;
            ch.configure_mesh().x_desc("Theoretical Quantiles").y_desc("Sample Quantiles").draw()?;
            let area = ch.plotting_area();
            for (&xv, &yv) in th.iter().zip(ss.iter()) { area.draw(&Circle::new((xv, yv), ms, col.filled())); }
            draw_seg!(area, rm, rm, r_m, r_m, RGBColor(244, 67, 54).stroke_width(2));
            root.present()?;
        }
    }
    Ok(())
}

fn normal_inv(p: f64) -> f64 {
    const A1: f64 = -3.969683028665376e1; const A2: f64 = 2.209460984245205e2;
    const A3: f64 = -2.759285104469687e2; const A4: f64 = 1.38357751867269e2;
    const A5: f64 = -3.066479806614716e1; const A6: f64 = 2.506628277459239;
    const B1: f64 = -5.447609879822406e1; const B2: f64 = 1.615858368580409e2;
    const B3: f64 = -1.556989798598866e2; const B4: f64 = 6.680131188771972e1;
    const B5: f64 = -1.328068155288572e1;
    const C1: f64 = -7.784894002430293e-3; const C2: f64 = -3.223964580411365e-1;
    const C3: f64 = -2.400758277161838; const C4: f64 = -2.549732539343734;
    const C5: f64 = 4.374664141464968; const C6: f64 = 2.938163982698783;
    const D1: f64 = 7.784695709041462e-3; const D2: f64 = 3.224671290700398e-1;
    const D3: f64 = 2.445134137142996; const D4: f64 = 3.754408661907416;
    const P_LO: f64 = 0.02425; const P_HI: f64 = 0.97575;
    if p < P_LO {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C1 * q + C2) * q + C3) * q + C4) * q + C5) * q + C6) / (((D1 * q + D2) * q + D3) * q + D4) + 5e5
    } else if p <= P_HI {
        let q = p - 0.5; let r = q * q;
        ((((((A1 * r + A2) * r + A3) * r + A4) * r + A5) * r + A6) * q) / (((((B1 * r + B2) * r + B3) * r + B4) * r + B5) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C1 * q + C2) * q + C3) * q + C4) * q + C5) * q + C6) / (((D1 * q + D2) * q + D3) * q + D4) + 5e5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test] fn test_plot_points() {
        let _ = fs::remove_file("/tmp/test_pp.png");
        png("/tmp/test_pp.png", 800, 600).unwrap();
        plot(&[1.0,2.0,3.0,4.0,5.0], &[1.0,4.0,9.0,16.0,25.0], PlotType::Points, PlotOptions::default().col("#F44336")).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_pp.png").is_ok());
    }
    #[test] fn test_plot_line() {
        let _ = fs::remove_file("/tmp/test_pl.png");
        png("/tmp/test_pl.png", 800, 600).unwrap();
        plot(&[1.0,2.0,3.0,4.0,5.0], &[1.0,2.0,3.0,4.0,5.0], PlotType::Line, PlotOptions::default()).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_pl.png").is_ok());
    }
    #[test] fn test_hist() {
        let _ = fs::remove_file("/tmp/test_h.png");
        png("/tmp/test_h.png", 800, 600).unwrap();
        hist(&[1.0,2.0,3.0,4.0,5.0,2.0,3.0,4.0,3.0,4.0], HistOptions::default().breaks(5)).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_h.png").is_ok());
    }
    #[test] fn test_boxplot() {
        let _ = fs::remove_file("/tmp/test_bp.png");
        png("/tmp/test_bp.png", 800, 600).unwrap();
        boxplot(&[&[1.0,2.0,3.0,4.0,5.0],&[2.0,4.0,6.0,8.0,10.0]], BoxPlotOptions::default()).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_bp.png").is_ok());
    }
    #[test] fn test_qqnorm() {
        let _ = fs::remove_file("/tmp/test_qq.png");
        png("/tmp/test_qq.png", 800, 600).unwrap();
        qqnorm(&[1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0], PlotOptions::default()).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_qq.png").is_ok());
    }
    #[test] fn test_svg() {
        let _ = fs::remove_file("/tmp/test_sv.svg");
        svg("/tmp/test_sv.svg", 800, 600).unwrap();
        plot(&[1.0,2.0,3.0], &[1.0,2.0,3.0], PlotType::Points, PlotOptions::default()).unwrap();
        dev_off().unwrap();
        assert!(fs::metadata("/tmp/test_sv.svg").is_ok());
    }
}