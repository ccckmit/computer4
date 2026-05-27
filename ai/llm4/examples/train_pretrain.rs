use llm4::{DataGenerators, ModernLanguageModel, TextDataset, TrainConfig, Trainer, Vocab, Config};

fn main() {
    println!("=== LLM4 Pre-training Example ===\n");

    let (pretrain_text, finetune_text) = DataGenerators::generate_rule_data();
    println!("Generated {} pretrain chars, {} finetune chars", 
             pretrain_text.len(), finetune_text.len());

    let mut chars: Vec<char> = (pretrain_text.clone() + &finetune_text).chars().collect();
    chars.sort();
    chars.dedup();
    
    let vocab = Vocab::new(&chars);
    vocab.save("vocab.json").expect("Failed to save vocab");
    println!("Created vocab with {} characters", vocab.vocab_size);

    let config = Config {
        vocab_size: vocab.vocab_size,
        d_model: 64,
        n_heads: 4,
        n_layers: 2,
        seq_len: 32,
    };
    let model = ModernLanguageModel::new(&config);
    println!("Created model with {} parameters", model.parameters().len());

    let dataset = TextDataset::new(&pretrain_text, &vocab, 32);
    println!("Created dataset with {} tokens\n", dataset.len());

    let train_config = TrainConfig {
        batch_size: 16,
        seq_len: 32,
        max_iters: 50,
        learning_rate: 5e-4,
        clip_grad_norm: 1.0,
    };
    let mut trainer = Trainer::new(model, vocab, dataset, train_config);
    
    println!("Starting training...\n");
    trainer.train();
    
    println!("\nPre-training complete!");
}