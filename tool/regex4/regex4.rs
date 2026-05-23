/// Regex4 正規表達式引擎
pub struct Regex4 {
    pattern: Vec<char>,
}

impl Regex4 {
    /// 建立一個新的 Regex4 實例
    pub fn new(pattern: &str) -> Self {
        Regex4 {
            // 將字串轉為字元陣列，完美支援 UTF-8 (Unicode)
            pattern: pattern.chars().collect(),
        }
    }

    /// 檢查目標字串是否符合正規表達式
    pub fn is_match(&self, text: &str) -> bool {
        let text_chars: Vec<char> = text.chars().collect();
        Self::match_text(&self.pattern, &text_chars)
    }

    /// 尋找文字中是否存在匹配的起點
    fn match_text(pattern: &[char], text: &[char]) -> bool {
        // 如果開頭有 '^'，表示只能從字串的最開頭匹配
        if pattern.first() == Some(&'^') {
            return Self::match_here(&pattern[1..], text);
        }

        // 若無 '^'，則嘗試從字串的每一個位置開始匹配（包含空字串的結尾）
        let mut i = 0;
        loop {
            if Self::match_here(pattern, &text[i..]) {
                return true;
            }
            if i == text.len() {
                break;
            }
            i += 1;
        }
        false
    }

    /// 核心遞迴匹配邏輯
    fn match_here(pattern: &[char], text: &[char]) -> bool {
        // 模式已經耗盡，代表全部匹配成功
        if pattern.is_empty() {
            return true;
        }

        // 處理結尾符號 '$'
        if pattern[0] == '$' && pattern.len() == 1 {
            return text.is_empty();
        }

        // 預先查看下一個字元是否為量詞 (*, +, ?)
        if pattern.len() >= 2 {
            let p_char = pattern[0];
            let quantifier = pattern[1];
            let next_pattern = &pattern[2..];

            match quantifier {
                '*' => return Self::match_star(p_char, next_pattern, text),
                '+' => return Self::match_plus(p_char, next_pattern, text),
                '?' => return Self::match_question(p_char, next_pattern, text),
                _ => {} // 如果不是量詞，就繼續往下當作一般字元處理
            }
        }

        // 匹配單一字元 (包含 '.' 萬用字元)
        if !text.is_empty() && (pattern[0] == '.' || pattern[0] == text[0]) {
            return Self::match_here(&pattern[1..], &text[1..]);
        }

        false
    }

    /// 處理 '*' (零次或多次，貪婪模式)
    fn match_star(c: char, pattern: &[char], text: &[char]) -> bool {
        let mut count = 0;
        // 找出可以匹配的最長連續字元數
        while count < text.len() && (text[count] == c || c == '.') {
            count += 1;
        }

        // 貪婪模式：從最長匹配開始回溯，直到 0 次
        while count > 0 {
            if Self::match_here(pattern, &text[count..]) {
                return true;
            }
            count -= 1;
        }

        // 嘗試 0 次
        Self::match_here(pattern, text)
    }

    /// 處理 '+' (一次或多次)
    fn match_plus(c: char, pattern: &[char], text: &[char]) -> bool {
        // 必須至少匹配一次
        if !text.is_empty() && (text[0] == c || c == '.') {
            // 剩下的就跟 '*' 的行為一樣
            return Self::match_star(c, pattern, &text[1..]);
        }
        false
    }

    /// 處理 '?' (零次或一次)
    fn match_question(c: char, pattern: &[char], text: &[char]) -> bool {
        // 優先嘗試匹配 1 次 (貪婪)
        if !text.is_empty() && (text[0] == c || c == '.') {
            if Self::match_here(pattern, &text[1..]) {
                return true;
            }
        }
        // 退回嘗試匹配 0 次
        Self::match_here(pattern, text)
    }
}

fn main() {
    // === 測試案例 ===

    // 1. 基礎文字與 '.'
    assert!(Regex4::new("abc").is_match("xyzabcdef"));
    assert!(Regex4::new("a.c").is_match("xya-cz"));
    assert!(!Regex4::new("a.c").is_match("xyacz"));

    // 2. 開頭與結尾 '^', '$'
    assert!(Regex4::new("^hello").is_match("hello world"));
    assert!(!Regex4::new("^hello").is_match("say hello"));
    assert!(Regex4::new("world$").is_match("hello world"));
    assert!(!Regex4::new("world$").is_match("world class"));
    assert!(Regex4::new("^exact$").is_match("exact"));

    // 3. 星號 '*' (零次或多次)
    assert!(Regex4::new("ab*c").is_match("ac"));       // 0 次
    assert!(Regex4::new("ab*c").is_match("abc"));      // 1 次
    assert!(Regex4::new("ab*c").is_match("abbbc"));    // 多次
    assert!(Regex4::new(".*").is_match("anything"));   // 任意字元任意次數

    // 4. 加號 '+' (一次或多次)
    assert!(!Regex4::new("ab+c").is_match("ac"));      // 0 次 (失敗)
    assert!(Regex4::new("ab+c").is_match("abc"));      // 1 次
    assert!(Regex4::new("ab+c").is_match("abbbc"));    // 多次

    // 5. 問號 '?' (零次或一次)
    assert!(Regex4::new("ab?c").is_match("ac"));       // 0 次
    assert!(Regex4::new("ab?c").is_match("abc"));      // 1 次
    assert!(!Regex4::new("ab?c").is_match("abbc"));    // 2 次 (失敗)

    // 6. Unicode 支援 (因為底層使用 Rust 的 char)
    assert!(Regex4::new("中.文").is_match("測試中O文字串"));
    assert!(Regex4::new("哈+").is_match("哈哈哈哈"));

    println!("所有 Regex4 測試均通過！");
}