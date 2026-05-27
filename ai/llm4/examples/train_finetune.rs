use llm4::{DataGenerators, ModernLanguageModel, TextDataset, FinetuneConfig, Finetuner, Vocab, Config};

fn main() {
    println!("=== LLM4 Fine-tuning Example ===\n");

    let vocab = Vocab::load("vocab.json").expect("Failed to load vocab");
    println!("Loaded vocab with {} characters", vocab.vocab_size);

    let finetune_text = std::fs::read_to_string("finetune.txt")
        .expect("Failed to read finetune.txt. Run gen_data first!");
    
    let config = Config {
        vocab_size: vocab.vocab_size,
        d_model: 64,
        n_heads: 4,
        n_layers: 2,
        seq_len: 32,
    };
    let model = ModernLanguageModel::new(&config);
    println!("Created model for fine-tuning");

    let dataset = TextDataset::new(&finetune_text, &vocab, 32);
    println!("Created dataset with {} tokens\n", dataset.len());

    let finetune_config = FinetuneConfig {
        batch_size: 16,
        seq_len: 32,
        max_iters: 30,
        learning_rate: 1e-4,
    };
    let mut finetuner = Finetuner::new(model, vocab, dataset, finetune_config);
    
    println!("Starting fine-tuning...\n");
    finetuner.finetune();
    
    println!("\n=== Testing the model ===");
    if let Ok(first_line) = std::fs::read_to_string("finetune.txt") {
        let line = first_line.lines().next().unwrap_or("");
        if line.contains("<A>") {
            let parts: Vec<&str> = line.split("<A>").collect();
            if parts.len() >= 2 {
                let prompt = format!("{}<A>", parts[0]);
                let expected = parts[1];
                let result = finetuner.test(&prompt);
                println!();
                println!("Prompt: {}", prompt);
                println!("Expected: {}", expected);
                println!("Generated: {}", result);
            }
        }
    }
    
    println!("\nFine-tuning complete!");
}