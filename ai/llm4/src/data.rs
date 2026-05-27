use crate::tensor::Tensor;
use crate::vocab::Vocab;
use rand::prelude::SliceRandom;
use rand::Rng;

pub struct TextDataset {
    pub data: Vec<usize>,
    pub seq_len: usize,
}

impl TextDataset {
    pub fn new(text: &str, vocab: &Vocab, seq_len: usize) -> Self {
        let indices: Vec<usize> = vocab.encode(text);
        TextDataset { data: indices, seq_len }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        vocab: &Vocab,
        seq_len: usize,
    ) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::new(&content, vocab, seq_len))
    }

    pub fn get_batch(&self, batch_size: usize) -> (Tensor, Tensor) {
        let mut rng = rand::thread_rng();

        let num_samples = self.data.len().saturating_sub(self.seq_len);
        if num_samples == 0 {
            return (
                Tensor::zeros(&[batch_size, self.seq_len]),
                Tensor::zeros(&[batch_size, self.seq_len]),
            );
        }

        let mut x_data = Vec::with_capacity(batch_size * self.seq_len);
        let mut y_data = Vec::with_capacity(batch_size * self.seq_len);

        for _ in 0..batch_size {
            let idx = rng.gen_range(0..num_samples);
            for i in 0..self.seq_len {
                x_data.push(self.data[idx + i] as f32);
                y_data.push(self.data[idx + i + 1] as f32);
            }
        }

        (
            Tensor::new(x_data, vec![batch_size, self.seq_len], false),
            Tensor::new(y_data, vec![batch_size, self.seq_len], false),
        )
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

pub struct DataGenerators;

impl DataGenerators {
    pub fn generate_rule_data() -> (String, String) {
        let num_map: Vec<(&str, &str)> = vec![
            ("1", "一"), ("2", "二"), ("3", "三"), ("4", "四"), ("5", "五"),
            ("6", "六"), ("7", "七"), ("8", "八"), ("9", "九"), ("10", "十"),
        ];
        let days = ["一", "二", "三", "四", "五", "六", "日"];
        let animals = ["鯨魚", "大象", "老虎", "狗", "貓", "老鼠", "螞蟻"];

        let mut pretrain_facts = Vec::new();
        let mut finetune_qa = Vec::new();

        for a in 1..6 {
            for b in 1..6 {
                let c = a + b;
                let na = num_map[a - 1].1;
                let nb = num_map[b - 1].1;
                let nc = num_map[c - 1].1;
                pretrain_facts.push(format!("{}加{}等於{}。", na, nb, nc));
                finetune_qa.push(format!("<Q>{}加{}等於多少？<A>{}", na, nb, nc));
            }
        }

        for i in 0..days.len() {
            let tomorrow = days[(i + 1) % 7];
            let yesterday = days[(i + 7 - 1) % 7];
            let today = days[i];
            pretrain_facts.push(format!("星期{}的明天是星期{}。", today, tomorrow));
            pretrain_facts.push(format!("星期{}的昨天是星期{}。", today, yesterday));
            finetune_qa.push(format!("<Q>星期{}的明天是星期幾？<A>星期{}", today, tomorrow));
            finetune_qa.push(format!("<Q>星期{}的昨天是星期幾？<A>星期{}", today, yesterday));
        }

        for i in 0..animals.len() {
            for j in (i + 1)..animals.len() {
                let big = animals[i];
                let small = animals[j];
                pretrain_facts.push(format!("{}比{}大。", big, small));
                pretrain_facts.push(format!("{}比{}小。", small, big));
                finetune_qa.push(format!("<Q>{}和{}誰比較大？<A>{}", big, small, big));
                finetune_qa.push(format!("<Q>{}和{}誰比較小？<A>{}", small, big, small));
            }
        }

        let pretrain_text = Self::expand_to_target(&pretrain_facts, 200000);
        let finetune_text = Self::expand_to_target(&finetune_qa, 50000);

        (pretrain_text, finetune_text)
    }

    pub fn generate_wuxia_data() -> (String, String) {
        let characters: Vec<(&str, &str, &str, &str)> = vec![
            ("郭靖", "桃花島", "降龍十八掌", "射鵰神弓"),
            ("張無忌", "光明頂", "九陽神功", "屠龍刀"),
            ("楊過", "絕情谷", "黯然銷魂掌", "玄鐵重劍"),
            ("令狐沖", "華山", "獨孤九劍", "青銅劍"),
            ("段譽", "大理", "六脈神劍", "玉骨扇"),
        ];

        let mut pretrain_facts = Vec::new();
        let mut finetune_qa = Vec::new();

        for (name, loc, skill, weapon) in &characters {
            pretrain_facts.push(format!("{}在{}苦練{}。", name, loc, skill));
            pretrain_facts.push(format!("{}的專屬武器是{}。", name, weapon));
            pretrain_facts.push(format!("如果要尋找{}，必須前往{}。", name, loc));

            finetune_qa.push(format!("<Q>{}在哪裡練武？<A>{}", name, loc));
            finetune_qa.push(format!("<Q>{}的武功是什麼？<A>{}", name, skill));
            finetune_qa.push(format!("<Q>{}使用什麼武器？<A>{}", name, weapon));
            finetune_qa.push(format!("<Q>去哪裡可以找到{}？<A>{}", name, loc));
        }

        let pretrain_text = Self::expand_to_target(&pretrain_facts, 200000);
        let finetune_text = Self::expand_to_target(&finetune_qa, 50000);

        (pretrain_text, finetune_text)
    }

    pub fn generate_robot_data() -> (String, String) {
        let intent_map: Vec<(&str, &str, &str)> = vec![
            ("太暗了", "打開", "燈"),
            ("太亮了", "關閉", "燈"),
            ("好熱", "打開", "冷氣"),
            ("好冷", "關閉", "冷氣"),
            ("地板好髒", "啟動", "掃地機器人"),
            ("想看新聞", "打開", "電視"),
            ("有點吵", "關閉", "電視"),
        ];
        let locations = ["客廳", "臥室", "廚房"];

        let mut pretrain_facts = Vec::new();
        let mut finetune_qa = Vec::new();

        for loc in &locations {
            for (trigger, act, device) in &intent_map {
                let sys_cmd = format!("系統指令：{}{}{}", act, loc, device);
                pretrain_facts.push(format!(
                    "當主人在{}說「{}」時，就是要執行{}。",
                    loc, trigger, sys_cmd
                ));
                pretrain_facts.push(format!(
                    "人工智慧收到「把{}的{}{}」的命令時，對應的系統指令。",
                    loc, device, act
                ));
                finetune_qa.push(format!("<Q>我在{}，{}。<A>{}", loc, trigger, sys_cmd));
                finetune_qa.push(format!("<Q>請幫我{}{}的{}。<A>{}", act, loc, device, sys_cmd));
            }
            pretrain_facts.push(format!("{}的環境感測器目前顯示溫度正常。", loc));
            finetune_qa.push(format!("<Q>{}目前的溫度如何？<A>溫度正常", loc));
        }

        let pretrain_text = Self::expand_to_target(&pretrain_facts, 200000);
        let finetune_text = Self::expand_to_target(&finetune_qa, 50000);

        (pretrain_text, finetune_text)
    }

    fn expand_to_target(sentences: &[String], target_len: usize) -> String {
        let mut rng = rand::thread_rng();
        let mut result = String::new();
        while result.len() < target_len {
            let mut sample = sentences.to_vec();
            sample.shuffle(&mut rng);
            for s in &sample {
                result.push_str(s);
                if result.len() >= target_len {
                    break;
                }
            }
        }
        result.truncate(target_len);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_basic() {
        let text = "hello world hello";
        let chars: Vec<char> = text.chars().collect();
        let vocab = Vocab::new(&chars);
        let dataset = TextDataset::new(text, &vocab, 4);

        assert!(dataset.len() > 0);
    }

    #[test]
    fn test_generators() {
        let (pretrain, finetune) = DataGenerators::generate_rule_data();
        assert!(pretrain.len() >= 1000);
        assert!(finetune.len() >= 1000);
    }
}