use clap::{Parser, Subcommand};
use llm4::{
    compile, generate, DataGenerators, FinetuneConfig, Finetuner, ModernLanguageModel, TextDataset,
    TrainConfig, Trainer, Vocab, Config,
};

#[derive(Parser)]
#[command(name = "llm4")]
#[command(version = "0.1.0")]
#[command(about = "Mini Language Model v2 - A from-scratch transformer in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    GenData {
        #[arg(long, default_value = "rule")]
        domain: String,
    },
    Pretrain {
        #[arg(long, default_value = "pretrain.txt")]
        pretrain_file: String,
        #[arg(long, default_value = "finetune.txt")]
        finetune_file: String,
        #[arg(long, default_value = "500")]
        max_iters: usize,
        #[arg(long, default_value = "5e-4")]
        learning_rate: f32,
    },
    Finetune {
        #[arg(long, default_value = "finetune.txt")]
        finetune_file: String,
        #[arg(long, default_value = "pretrain_model.bin")]
        model_file: String,
        #[arg(long, default_value = "300")]
        max_iters: usize,
        #[arg(long, default_value = "1e-4")]
        learning_rate: f32,
    },
    Generate {
        #[arg(long)]
        prompt: String,
        #[arg(long, default_value = "100")]
        max_tokens: usize,
        #[arg(long, default_value = "model.bin")]
        model_file: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GenData { domain } => {
            let (pretrain_text, finetune_text) = match domain.as_str() {
                "rule" | "math" => DataGenerators::generate_rule_data(),
                "wuxia" => DataGenerators::generate_wuxia_data(),
                "robot" => DataGenerators::generate_robot_data(),
                _ => {
                    eprintln!("Unknown domain: {}. Using 'rule'.", domain);
                    DataGenerators::generate_rule_data()
                }
            };

            std::fs::write("pretrain.txt", &pretrain_text)?;
            std::fs::write("finetune.txt", &finetune_text)?;
            
            println!("Generated data files:");
            println!("  pretrain.txt ({} chars)", pretrain_text.len());
            println!("  finetune.txt ({} chars)", finetune_text.len());
        }

        Commands::Pretrain {
            pretrain_file,
            finetune_file,
            max_iters,
            learning_rate,
        } => {
            let pretrain_text = std::fs::read_to_string(&pretrain_file)?;
            let finetune_text = std::fs::read_to_string(&finetune_file)?;

            let mut chars: Vec<char> = (pretrain_text.clone() + &finetune_text).chars().collect();
            chars.sort();
            chars.dedup();
            
            let vocab = Vocab::new(&chars);
            vocab.save("vocab.json")?;
            println!("Vocab saved: {} chars", vocab.vocab_size);

            let config = Config {
                vocab_size: vocab.vocab_size,
                ..Default::default()
            };
            let model = ModernLanguageModel::new(&config);

            let dataset = TextDataset::new(&pretrain_text, &vocab, 64);
            let train_config = TrainConfig {
                max_iters,
                learning_rate,
                ..Default::default()
            };
            let mut trainer = Trainer::new(model, vocab, dataset, train_config);
            trainer.train();

            println!("Pretraining complete!");
        }

        Commands::Finetune {
            finetune_file,
            model_file: _,
            max_iters,
            learning_rate,
        } => {
            let vocab = Vocab::load("vocab.json")?;
            let finetune_text = std::fs::read_to_string(&finetune_file)?;

            let config = Config {
                vocab_size: vocab.vocab_size,
                ..Default::default()
            };
            let model = ModernLanguageModel::new(&config);

            let dataset = TextDataset::new(&finetune_text, &vocab, 64);
            let finetune_config = FinetuneConfig {
                max_iters,
                learning_rate,
                ..Default::default()
            };
            let mut finetuner = Finetuner::new(model, vocab, dataset, finetune_config);
            finetuner.finetune();

            if let Ok(first_line) = std::fs::read_to_string(&finetune_file) {
                let line = first_line.lines().next().unwrap_or("");
                if line.contains("<A>") {
                    let parts: Vec<&str> = line.split("<A>").collect();
                    if parts.len() >= 2 {
                        let prompt = format!("{}<A>", parts[0]);
                        let expected = parts[1];
                        let result = finetuner.test(&prompt);
                        println!();
                        println!("{}", "=".repeat(50));
                        println!("Test Result:");
                        println!("Prompt: {}", prompt);
                        println!("Expected: {}", expected);
                        println!("Generated: {}", result);
                        println!("{}", "=".repeat(50));
                    }
                }
            }

            println!("Fine-tuning complete!");
        }

        Commands::Generate {
            prompt,
            max_tokens,
            model_file: _,
        } => {
            let vocab = Vocab::load("vocab.json")?;
            let config = Config {
                vocab_size: vocab.vocab_size,
                ..Default::default()
            };
            let model = ModernLanguageModel::new(&config);

            let result = generate(&model, &vocab, &prompt, max_tokens);
            println!("Prompt: {}", prompt);
            println!("Generated: {}", result);
        }
    }

    Ok(())
}