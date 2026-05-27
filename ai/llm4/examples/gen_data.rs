use llm4::DataGenerators;

fn main() {
    println!("=== LLM4 Data Generation Example ===\n");

    println!("Generating rule/math data...");
    let (pretrain_rule, finetune_rule) = DataGenerators::generate_rule_data();
    std::fs::write("pretrain.txt", &pretrain_rule).expect("Failed to write pretrain.txt");
    std::fs::write("finetune.txt", &finetune_rule).expect("Failed to write finetune.txt");
    println!("  pretrain.txt: {} chars", pretrain_rule.len());
    println!("  finetune.txt: {} chars", finetune_rule.len());
    
    println!("\nRule data preview:");
    println!("  pretrain: {}...", &pretrain_rule[..100.min(pretrain_rule.len())]);
    println!("  finetune: {}...", &finetune_rule[..100.min(finetune_rule.len())]);

    println!("\nGenerating wuxia data...");
    let (pretrain_wuxia, finetune_wuxia) = DataGenerators::generate_wuxia_data();
    println!("  pretrain: {} chars, finetune: {} chars", 
             pretrain_wuxia.len(), finetune_wuxia.len());

    println!("\nGenerating robot data...");
    let (pretrain_robot, finetune_robot) = DataGenerators::generate_robot_data();
    println!("  pretrain: {} chars, finetune: {} chars", 
             pretrain_robot.len(), finetune_robot.len());

    println!("\nData generation complete!");
    println!("\nTo train:");
    println!("  1. cargo run --example train_pretrain");
    println!("  2. cargo run --example train_finetune");
}