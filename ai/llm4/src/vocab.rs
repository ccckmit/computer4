use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vocab {
    pub stoi: HashMap<char, usize>,
    pub itos: Vec<char>,
    pub vocab_size: usize,
}

impl Vocab {
    pub fn new(chars: &[char]) -> Self {
        let vocab_size = chars.len();
        let stoi: HashMap<char, usize> = chars.iter().enumerate().map(|(i, &c)| (c, i)).collect();
        let itos: Vec<char> = chars.to_vec();
        Vocab { stoi, itos, vocab_size }
    }

    pub fn from_text(text: &str) -> Self {
        let mut chars: Vec<char> = text.chars().collect();
        chars.sort();
        chars.dedup();
        Self::new(&chars)
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.chars().map(|c| *self.stoi.get(&c).unwrap_or(&0)).collect()
    }

    pub fn decode(&self, indices: &[usize]) -> String {
        indices
            .iter()
            .filter_map(|&i| self.itos.get(i).copied())
            .collect()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let json = fs::read_to_string(path)?;
        let vocab: Vocab = serde_json::from_str(&json)?;
        Ok(vocab)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocab_basic() {
        let chars = vec!['a', 'b', 'c'];
        let vocab = Vocab::new(&chars);
        assert_eq!(vocab.vocab_size, 3);
        assert_eq!(vocab.encode("abc"), vec![0, 1, 2]);
        assert_eq!(vocab.decode(&[0, 1, 2]), "abc");
    }

    #[test]
    fn test_vocab_from_text() {
        let text = "hello world";
        let vocab = Vocab::from_text(text);
        assert!(vocab.vocab_size >= 8); // h,e,l,l,o,,' ',w,r,d
        assert_eq!(vocab.decode(&vocab.encode(text)), text);
    }
}