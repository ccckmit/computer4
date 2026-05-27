use crate::data::TextDataset;
use crate::model::ModernLanguageModel;
use crate::vocab::Vocab;

pub struct Trainer {
    pub model: ModernLanguageModel,
    pub vocab: Vocab,
    pub dataset: TextDataset,
    pub config: TrainConfig,
}

pub struct TrainConfig {
    pub batch_size: usize,
    pub seq_len: usize,
    pub max_iters: usize,
    pub learning_rate: f32,
    pub clip_grad_norm: f32,
}

impl Default for TrainConfig {
    fn default() -> Self {
        TrainConfig {
            batch_size: 32,
            seq_len: 64,
            max_iters: 500,
            learning_rate: 5e-4,
            clip_grad_norm: 1.0,
        }
    }
}

pub struct Finetuner {
    pub model: ModernLanguageModel,
    pub vocab: Vocab,
    pub dataset: TextDataset,
    pub config: FinetuneConfig,
}

pub struct FinetuneConfig {
    pub batch_size: usize,
    pub seq_len: usize,
    pub max_iters: usize,
    pub learning_rate: f32,
}

impl Default for FinetuneConfig {
    fn default() -> Self {
        FinetuneConfig {
            batch_size: 32,
            seq_len: 64,
            max_iters: 300,
            learning_rate: 1e-4,
        }
    }
}

impl Trainer {
    pub fn new(
        model: ModernLanguageModel,
        vocab: Vocab,
        dataset: TextDataset,
        config: TrainConfig,
    ) -> Self {
        Trainer {
            model,
            vocab,
            dataset,
            config,
        }
    }

    pub fn train(&mut self) {
        for iter in 0..self.config.max_iters {
            let (xb, yb) = self.dataset.get_batch(self.config.batch_size);

            let x_indices: Vec<usize> = xb.data().iter().map(|&x| x as usize).collect();
            let y_indices: Vec<usize> = yb.data().iter().map(|&y| y as usize).collect();

            self.model.zero_grad();
            let (_, loss_opt) = self.model.forward(&x_indices, Some(&y_indices));

            let loss_val = if let Some(loss_tensor) = loss_opt {
                loss_tensor.backward();
                loss_tensor.scalar_val()
            } else {
                0.0
            };

            if iter % 100 == 0 || iter == self.config.max_iters - 1 {
                println!("Step {:4} | Loss: {:.4}", iter, loss_val);
            }
        }
    }
}

impl Finetuner {
    pub fn new(
        model: ModernLanguageModel,
        vocab: Vocab,
        dataset: TextDataset,
        config: FinetuneConfig,
    ) -> Self {
        Finetuner {
            model,
            vocab,
            dataset,
            config,
        }
    }

    pub fn finetune(&mut self) {
        for iter in 0..self.config.max_iters {
            let (xb, yb) = self.dataset.get_batch(self.config.batch_size);

            let x_indices: Vec<usize> = xb.data().iter().map(|&x| x as usize).collect();
            let y_indices: Vec<usize> = yb.data().iter().map(|&y| y as usize).collect();

            self.model.zero_grad();
            let (_, loss_opt) = self.model.forward(&x_indices, Some(&y_indices));

            let loss_val = if let Some(loss_tensor) = loss_opt {
                loss_tensor.backward();
                loss_tensor.scalar_val()
            } else {
                0.0
            };

            if iter % 100 == 0 || iter == self.config.max_iters - 1 {
                println!("Finetune Step {:4} | Loss: {:.4}", iter, loss_val);
            }
        }
    }

    pub fn test(&self, prompt: &str) -> String {
        let indices = self.vocab.encode(prompt);
        let generated = self.model.generate(&indices, 100);
        self.vocab.decode(&generated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_train_config_default() {
        let config = TrainConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_iters, 500);
    }

    #[test]
    fn test_finetune_config_default() {
        let config = FinetuneConfig::default();
        assert_eq!(config.batch_size, 32);
        assert_eq!(config.max_iters, 300);
    }
}