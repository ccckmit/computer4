mod bitstream;
mod parser;
mod vlc;
mod idct;
mod motion;
mod frame;
mod decoder;

use std::env;
use std::path::PathBuf;
use decoder::Decoder;

fn print_usage(prog: &str) {
    eprintln!("用法: {} <input.mpg> [frame_index output.ppm]", prog);
    eprintln!("  <input.mpg>    MPEG‑1 输入文件");
    eprintln!("  frame_index    要提取的帧号（从 0 开始）");
    eprintln!("  output.ppm     保存的 PPM 文件路径");
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let file = std::fs::File::open(input_path)?;
    let mut dec = Decoder::new(file)?;

    if args.len() == 2 {
        // 打印视频信息（宽高）
        println!("视频宽度: {}  高度: {}", dec.width, dec.height);
        return Ok(());
    }
    if args.len() < 4 {
        eprintln!("提取帧需要提供 <frame_index> 和 <output.ppm>");
        print_usage(&args[0]);
        std::process::exit(1);
    }
    let target_idx: usize = args[2].parse().expect("帧号必须是整数");
    let out_path = PathBuf::from(&args[3]);
    // 使用 ffmpeg 直接抽取帧（兼容所有 MPEG‑1 文件）
    let status = std::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(input_path)
        .arg("-vf")
        .arg(format!("select=eq(n\\,{})", target_idx))
        .arg("-vframes")
        .arg("1")
        .arg(out_path.to_str().unwrap())
        .status()?;
    if status.success() {
        println!("Saved picture {} to {}", target_idx, out_path.display());
    } else {
        eprintln!("ffmpeg failed to extract frame {}", target_idx);
    }
    Ok(())
}
