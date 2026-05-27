pub mod data;
pub mod model;
pub mod tensor;
pub mod train;
pub mod vocab;

pub use data::{DataGenerators, TextDataset};
pub use model::{Config, ModernLanguageModel};
pub use tensor::Tensor;
pub use train::{Finetuner, FinetuneConfig, Trainer, TrainConfig};
pub use vocab::Vocab;

pub fn compile(text: &str, vocab: &Vocab) -> Vec<usize> {
    vocab.encode(text)
}

pub fn generate(model: &ModernLanguageModel, vocab: &Vocab, text: &str, max_new_tokens: usize) -> String {
    let indices = vocab.encode(text);
    let generated = model.generate(&indices, max_new_tokens);
    vocab.decode(&generated)
}

pub fn train_pretrain(
    pretrain_text: &str,
    finetune_text: &str,
    config: Option<TrainConfig>,
) -> (Vocab, ModernLanguageModel) {
    let mut chars: Vec<char> = (pretrain_text.to_string() + finetune_text).chars().collect();
    chars.sort();
    chars.dedup();

    let vocab = Vocab::new(&chars);
    let model_config = Config {
        vocab_size: vocab.vocab_size,
        ..Default::default()
    };
    let model = ModernLanguageModel::new(&model_config);

    let dataset = TextDataset::new(pretrain_text, &vocab, 64);
    let train_config = config.unwrap_or_default();
    let mut trainer = Trainer::new(model, vocab.clone(), dataset, train_config);
    trainer.train();

    (trainer.vocab, trainer.model)
}

pub fn finetune(
    model: ModernLanguageModel,
    vocab: &Vocab,
    finetune_text: &str,
    config: Option<FinetuneConfig>,
) -> ModernLanguageModel {
    let dataset = TextDataset::new(finetune_text, vocab, 64);
    let finetune_config = config.unwrap_or_default();
    let mut finetuner = Finetuner::new(model, vocab.clone(), dataset, finetune_config);
    finetuner.finetune();
    finetuner.model
}