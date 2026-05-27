use chart4::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match mode {
        "line" => demo_line()?,
        "scatter" => demo_scatter()?,
        "bar" => demo_bar()?,
        "pie" => demo_pie()?,
        "histogram" => demo_histogram()?,
        "multi" => demo_multi()?,
        "subplot" => demo_subplot()?,
        "serve" => demo_serve()?,
        "all" => {
            demo_line()?;
            demo_scatter()?;
            demo_bar()?;
            demo_pie()?;
            demo_histogram()?;
            demo_multi()?;
            demo_subplot()?;
        }
        _ => eprintln!("用法: cargo run [line|scatter|bar|pie|histogram|multi|subplot|serve|all]"),
    }
    Ok(())
}

fn demo_line() -> Result<(), Box<dyn std::error::Error>> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y1: Vec<f64> = x.iter().map(|&x| x.sin()).collect();
    let y2: Vec<f64> = x.iter().map(|&x| x.cos()).collect();

    let mut fig = Figure::new();
    fig.add_trace(Trace::line(x.clone(), y1).name("sin(x)").line_color("red"));
    fig.add_trace(Trace::line(x.clone(), y2).name("cos(x)").line_color("blue").line_dash("dash"));
    fig.title("Sine & Cosine 曲線").width(900).height(500).showlegend(true);
    fig.save("demo_line.html")?;
    println!("✓ demo_line.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_scatter() -> Result<(), Box<dyn std::error::Error>> {
    let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&x| x * 0.5 + (x * 0.3).sin() * 5.0 + rand_noise()).collect();

    let mut fig = Figure::new();
    fig.add_trace(Trace::scatter(x, y).name("random data").marker_color("green").marker_size(8.0));
    fig.title("Scatter 散點圖").width(800).height(500);
    fig.save("demo_scatter.html")?;
    println!("✓ demo_scatter.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_bar() -> Result<(), Box<dyn std::error::Error>> {
    let values = vec![30.0, 45.0, 25.0, 50.0, 35.0];

    let mut fig = Figure::new();
    fig.add_trace(
        Trace::bar((0..5).map(|i| i as f64).collect(), values)
            .name("水果銷量")
            .marker_color("#2196F3")
    );
    fig.title("水果銷售統計").width(700).height(500);
    fig.xaxis(Axis { title: Some(title("水果")), ..Default::default() });
    fig.yaxis(Axis { title: Some(title("銷量 (箱)")), ..Default::default() });
    fig.save("demo_bar.html")?;
    println!("✓ demo_bar.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_pie() -> Result<(), Box<dyn std::error::Error>> {
    let mut fig = Figure::new();
    fig.add_trace(
        Trace::pie(vec!["Python", "Rust", "JavaScript", "C++", "Go"], vec![35.0, 28.0, 20.0, 10.0, 7.0])
            .name("程式語言")
            .hole(0.4) // 甜甜圈效果
    );
    fig.title("最喜歡的程式語言").width(700).height(500);
    fig.save("demo_pie.html")?;
    println!("✓ demo_pie.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_histogram() -> Result<(), Box<dyn std::error::Error>> {
    use rand_distr::{Distribution, Normal};
    let normal = Normal::new(50.0, 15.0).unwrap();
    let mut rng = rand::thread_rng();
    let data: Vec<f64> = (0..1000).map(|_| normal.sample(&mut rng)).collect();

    let mut fig = Figure::new();
    fig.add_trace(Trace::histogram(data).name("身高分佈").marker_color("#FF5722"));
    fig.title("常態分佈直方圖").width(800).height(500).barmode("overlay");
    fig.xaxis(Axis { title: Some(title("身高 (cm)")), ..Default::default() });
    fig.yaxis(Axis { title: Some(title("人數")), ..Default::default() });
    fig.save("demo_histogram.html")?;
    println!("✓ demo_histogram.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_multi() -> Result<(), Box<dyn std::error::Error>> {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y1: Vec<f64> = x.iter().map(|&x| x.sin() * 2.0).collect();
    let y2: Vec<f64> = x.iter().map(|&x| x.cos() * 2.0).collect();
    let bar_x = vec!["Q1", "Q2", "Q3", "Q4"];
    let bar_y = vec![12.0, 19.0, 15.0, 22.0];

    let mut fig = Figure::new();
    fig.add_trace(Trace::line_scatter(x.clone(), y1).name("sin*2"));
    fig.add_trace(Trace::line_scatter(x, y2).name("cos*2"));
    fig.add_trace(Trace::line_str(bar_x, bar_y).name("季營收").line_color("green"));
    fig.title("混合圖表").width(900).height(500).showlegend(true);
    fig.save("demo_multi.html")?;
    println!("✓ demo_multi.html 已產生");
    fig.show()?;
    Ok(())
}

/// 雙 Y 軸子圖
fn demo_subplot() -> Result<(), Box<dyn std::error::Error>> {
    let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
    let y1: Vec<f64> = x.iter().map(|&x| (x * 0.5).sin() * 100.0 + 100.0).collect();
    let y2: Vec<f64> = x.iter().map(|&x| (x * 0.3).cos() * 0.5 + 0.5).collect();

    let mut fig = Figure::new();
    fig.add_trace(Trace::line(x.clone(), y1).name("溫度 (°C)").line_color("red"));
    fig.add_trace(
        Trace::line(x, y2).name("濕度 (%)").line_color("blue").line_dash("dot")
            .yaxis("y2"),
    );
    // 先建好 layout（含雙 Y 軸），再設標題
    fig.layout(Layout {
        title: Some(title("溫度與濕度 — 雙 Y 軸")),
        yaxis: Some(Axis { title: Some(title("溫度 (°C)")), ..Default::default() }),
        yaxis2: Some(Axis {
            title: Some(title("濕度 (%)")),
            overlaying: Some("y".to_string()),
            side: Some("right".to_string()),
            ..Default::default()
        }),
        width: Some(900),
        height: Some(500),
        showlegend: Some(true),
        ..Default::default()
    });
    fig.save("demo_subplot.html")?;
    println!("✓ demo_subplot.html 已產生");
    fig.show()?;
    Ok(())
}

fn demo_serve() -> Result<(), Box<dyn std::error::Error>> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|&x| (x * 2.0).sin()).collect();

    let mut fig = Figure::new();
    fig.add_trace(Trace::line(x, y).name("sin(2x)"));
    fig.title("互動式圖表 — Server 模式").width(900).height(500);
    println!("啟動 chart4 server (Ctrl+C 結束)...");
    fig.serve(8080)?;
    Ok(())
}

fn rand_noise() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(-2.0..2.0)
}
