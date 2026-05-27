use llm4::{generate, ModernLanguageModel, Vocab, Config};

fn main() {
    println!("=== LLM4 Text Generation Example ===\n");

    let vocab = Vocab::load("vocab.json").expect("Failed to load vocab");
    println!("Loaded vocab with {} characters", vocab.vocab_size);

    let config = Config {
        vocab_size: vocab.vocab_size,
        d_model: 64,
        n_heads: 4,
        n_layers: 2,
        seq_len: 32,
    };
    let model = ModernLanguageModel::new(&config);
    println!("Loaded model\n");

    let prompt = "<Q>一加二等於多少？<A>";
    println!("Prompt: {}", prompt);
    
    let result = generate(&model, &vocab, prompt, 50);
    println!("Generated: {}", result);
}