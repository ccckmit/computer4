use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Instant;

const ESCAPE_BYTE: u8 = 255;
const MIN_MATCH_LEN: usize = 3;
const MAX_WINDOW_SIZE: usize = 255; 
const MAX_MATCH_LEN: usize = 255;   

// ==========================================
// 1. LZ77 核心演算法 (與先前邏輯相同)
// ==========================================

fn find_longest_match(data: &[u8], current_pos: usize) -> (usize, usize) {
    let mut best_dist = 0;
    let mut best_len = 0;
    let window_start = current_pos.saturating_sub(MAX_WINDOW_SIZE);

    for start_idx in window_start..current_pos {
        let mut match_len = 0;
        while current_pos + match_len < data.len()
            && match_len < MAX_MATCH_LEN
            && data[start_idx + match_len] == data[current_pos + match_len]
        {
            match_len += 1;
        }

        if match_len > best_len {
            best_len = match_len;
            best_dist = current_pos - start_idx;
        }
    }
    (best_dist, best_len)
}

pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        let (dist, len) = find_longest_match(input, i);
        if len >= MIN_MATCH_LEN {
            out.push(ESCAPE_BYTE);
            out.push(dist as u8);
            out.push(len as u8);
            i += len;
        } else {
            out.push(input[i]);
            if input[i] == ESCAPE_BYTE {
                out.push(0);
            }
            i += 1;
        }
    }
    out
}

pub fn decompress(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == ESCAPE_BYTE {
            if i + 1 >= input.len() { return Err("資料損毀：缺少跳脫標記"); }
            if input[i + 1] == 0 {
                out.push(ESCAPE_BYTE);
                i += 2;
            } else {
                if i + 2 >= input.len() { return Err("資料損毀：缺少長度資訊"); }
                let dist = input[i + 1] as usize;
                let len = input[i + 2] as usize;

                if dist == 0 || dist > out.len() { return Err("資料損毀：無效的距離"); }
                for _ in 0..len {
                    let val = out[out.len() - dist];
                    out.push(val);
                }
                i += 3;
            }
        } else {
            out.push(input[i]);
            i += 1;
        }
    }
    Ok(out)
}

// ==========================================
// 2. 簡易打包器 (Archiver)
// 負責將目錄結構轉成平坦的位元組陣列，以便壓縮
// 格式: [檔名長度(u16)][檔名字串][檔案大小(u64)][檔案內容] ...
// ==========================================

fn pack_files(path: &Path, base_dir: &Path, buffer: &mut Vec<u8>) -> io::Result<()> {
    if path.is_file() {
        // 取得相對路徑
        let rel_path = match path.strip_prefix(base_dir) {
            Ok(p) => p,
            Err(_) => Path::new(path.file_name().unwrap_or_default()),
        };
        
        let path_str = rel_path.to_string_lossy();
        let path_bytes = path_str.as_bytes();

        // 寫入: 檔名長度 (2 bytes) + 檔名
        buffer.write_all(&(path_bytes.len() as u16).to_le_bytes())?;
        buffer.write_all(path_bytes)?;

        // 寫入: 檔案大小 (8 bytes) + 內容
        let mut file = File::open(path)?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        buffer.write_all(&(content.len() as u64).to_le_bytes())?;
        buffer.write_all(&content)?;
        
        println!("打包檔案: {}", path_str);

    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            pack_files(&entry?.path(), base_dir, buffer)?;
        }
    }
    Ok(())
}

fn unpack_files(data: &[u8], output_dir: &Path) -> Result<(), &'static str> {
    let mut cursor = 0;

    while cursor < data.len() {
        // 讀取檔名長度
        if cursor + 2 > data.len() { return Err("解包失敗：檔案格式錯誤"); }
        let path_len = u16::from_le_bytes([data[cursor], data[cursor+1]]) as usize;
        cursor += 2;

        if path_len == 0 { break; } // 安全保護，以防尾端補零

        // 讀取檔名
        if cursor + path_len > data.len() { return Err("解包失敗：檔名讀取錯誤"); }
        let path_str = String::from_utf8_lossy(&data[cursor..cursor+path_len]);
        cursor += path_len;

        // 讀取檔案長度
        if cursor + 8 > data.len() { return Err("解包失敗：檔案大小讀取錯誤"); }
        let mut len_bytes = [0u8; 8];
        len_bytes.copy_from_slice(&data[cursor..cursor+8]);
        let content_len = u64::from_le_bytes(len_bytes) as usize;
        cursor += 8;

        // 讀取檔案內容
        if cursor + content_len > data.len() { return Err("解包失敗：檔案內容長度不符"); }
        let content = &data[cursor..cursor+content_len];
        cursor += content_len;

        // 將檔案寫入磁碟 (建立所需的目錄結構)
        let target_path = output_dir.join(path_str.as_ref());
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|_| "無法建立目錄")?;
        }
        
        fs::write(&target_path, content).map_err(|_| "無法寫入檔案")?;
        println!("解壓縮出: {}", target_path.display());
    }

    Ok(())
}

// ==========================================
// 3. 命令列主程式
// ==========================================

fn print_usage() {
    println!("自製 LZ77 壓縮工具 (純 Rust 實作)");
    println!("用法:");
    println!("  壓縮:   cargo run -- c <輸入檔案或資料夾> [輸出檔案.lz]");
    println!("  解壓縮: cargo run -- d <輸入檔案.lz> <輸出資料夾>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        return;
    }

    let command = args[1].as_str();
    let input_path_str = &args[2];
    
    match command {
        "c" => {
            let input_path = Path::new(input_path_str);
            if !input_path.exists() {
                eprintln!("錯誤：找不到檔案或目錄 {}", input_path_str);
                return;
            }

            // 預設輸出檔名
            let output_path_str = if args.len() > 3 {
                args[3].clone()
            } else {
                format!("{}.mylz", input_path.file_name().unwrap().to_string_lossy())
            };

            println!("開始打包...");
            let mut archive_buffer = Vec::new();
            
            // 決定 base_dir (如果是資料夾，就以該資料夾的上層為基準，保留資料夾名稱)
            let base_dir = input_path.parent().unwrap_or(Path::new(""));
            
            if let Err(e) = pack_files(input_path, base_dir, &mut archive_buffer) {
                eprintln!("打包失敗: {}", e);
                return;
            }

            println!("打包完成，大小: {} bytes", archive_buffer.len());
            println!("開始 LZ 壓縮 (可能需要一些時間)...");

            let start_time = Instant::now();
            let compressed = compress(&archive_buffer);
            let duration = start_time.elapsed();

            if let Err(e) = fs::write(&output_path_str, &compressed) {
                eprintln!("寫入檔案失敗: {}", e);
                return;
            }

            println!("✅ 壓縮成功！寫入檔案: {}", output_path_str);
            println!("壓縮後大小: {} bytes", compressed.len());
            println!("壓縮率: {:.2}%", (compressed.len() as f64 / archive_buffer.len() as f64) * 100.0);
            println!("耗時: {:.2?}", duration);
        }

        "d" => {
            if args.len() < 4 {
                eprintln!("錯誤：解壓縮需要指定輸出資料夾");
                print_usage();
                return;
            }

            let output_dir_str = &args[3];
            let output_dir = Path::new(output_dir_str);

            println!("讀取壓縮檔...");
            let compressed = match fs::read(input_path_str) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("無法讀取檔案: {}", e);
                    return;
                }
            };

            println!("開始 LZ 解壓縮...");
            let start_time = Instant::now();
            let decompressed = match decompress(&compressed) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("解壓縮失敗: {}", e);
                    return;
                }
            };
            let duration = start_time.elapsed();

            println!("解壓縮完成，還原資料大小: {} bytes. 耗時: {:.2?}", decompressed.len(), duration);
            println!("開始解包至目錄: {}", output_dir_str);

            if !output_dir.exists() {
                fs::create_dir_all(output_dir).unwrap();
            }

            if let Err(e) = unpack_files(&decompressed, output_dir) {
                eprintln!("檔案解包失敗: {}", e);
            } else {
                println!("✅ 所有檔案解壓縮與還原完畢！");
            }
        }

        _ => {
            eprintln!("錯誤：未知的指令 '{}'", command);
            print_usage();
        }
    }
}