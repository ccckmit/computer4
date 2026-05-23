const ESCAPE_BYTE: u8 = 255;
const MIN_MATCH_LEN: usize = 3;
const MAX_WINDOW_SIZE: usize = 255; // 用 1 byte 儲存距離，最大 255
const MAX_MATCH_LEN: usize = 255;   // 用 1 byte 儲存長度，最大 255

/// 尋找滑動視窗內的最長匹配
fn find_longest_match(data: &[u8], current_pos: usize) -> (usize, usize) {
    let mut best_dist = 0;
    let mut best_len = 0;

    // 決定視窗的起始位置 (最多往回看 MAX_WINDOW_SIZE)
    let window_start = current_pos.saturating_sub(MAX_WINDOW_SIZE);

    for start_idx in window_start..current_pos {
        let mut match_len = 0;
        
        // 計算當前位置與歷史位置的匹配長度
        while current_pos + match_len < data.len()
            && match_len < MAX_MATCH_LEN
            && data[start_idx + match_len] == data[current_pos + match_len]
        {
            match_len += 1;
        }

        // 更新最長匹配
        if match_len > best_len {
            best_len = match_len;
            best_dist = current_pos - start_idx;
        }
    }

    (best_dist, best_len)
}

/// 壓縮函數
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < input.len() {
        let (dist, len) = find_longest_match(input, i);

        if len >= MIN_MATCH_LEN {
            // 找到足夠長的匹配，寫入壓縮標記：[ESCAPE, 距離, 長度]
            out.push(ESCAPE_BYTE);
            out.push(dist as u8);
            out.push(len as u8);
            i += len; // 跳過已匹配的部分
        } else {
            // 沒有匹配，寫入原始位元組
            out.push(input[i]);
            
            // 如果原始資料剛好是 ESCAPE_BYTE，寫入一個 0 作為跳脫處理
            if input[i] == ESCAPE_BYTE {
                out.push(0);
            }
            i += 1;
        }
    }

    out
}

/// 解壓縮函數
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut out = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i] == ESCAPE_BYTE {
            if i + 1 >= input.len() {
                return Err("壓縮資料損毀：缺少跳脫標記的後續資料");
            }

            if input[i + 1] == 0 {
                // 這是原始資料的 255 (ESCAPE_BYTE)
                out.push(ESCAPE_BYTE);
                i += 2;
            } else {
                // 這是一個壓縮標記：[ESCAPE, 距離, 長度]
                if i + 2 >= input.len() {
                    return Err("壓縮資料損毀：缺少長度資訊");
                }

                let dist = input[i + 1] as usize;
                let len = input[i + 2] as usize;

                if dist == 0 || dist > out.len() {
                    return Err("壓縮資料損毀：無效的距離指標");
                }

                // 從已經解壓縮的資料中複製 (注意：必須一個一個字元推入，因為可能會出現 len > dist 的重疊複製情況)
                for _ in 0..len {
                    let val = out[out.len() - dist];
                    out.push(val);
                }
                i += 3;
            }
        } else {
            // 一般的原始字元
            out.push(input[i]);
            i += 1;
        }
    }

    Ok(out)
}

fn main() {
    // 測試資料：包含大量重複字串
    let original_text = "Hello Rust! Hello Rust! 這是一個測試，這是一個測試，這是一個測試。重複重複重複重複重複。";
    let original_data = original_text.as_bytes();

    println!("--- 自製 LZ77 壓縮演算法 ---");
    println!("原始資料長度: {} bytes", original_data.len());

    // 進行壓縮
    let compressed_data = compress(original_data);
    println!("壓縮後長度: {} bytes", compressed_data.len());
    
    let compression_ratio = compressed_data.len() as f64 / original_data.len() as f64 * 100.0;
    println!("壓縮率: {:.2}%", compression_ratio);

    // 進行解壓縮
    match decompress(&compressed_data) {
        Ok(decompressed_data) => {
            println!("解壓縮後長度: {} bytes", decompressed_data.len());
            
            // 驗證解壓縮後的資料是否與原始資料一致
            if original_data == decompressed_data.as_slice() {
                println!("✅ 成功！解壓縮的資料與原始資料完全相同。");
                let recovered_text = String::from_utf8_lossy(&decompressed_data);
                println!("還原內容: {}", recovered_text);
            } else {
                println!("❌ 錯誤！解壓縮後的資料與原始資料不符。");
            }
        }
        Err(e) => println!("解壓縮失敗: {}", e),
    }
}