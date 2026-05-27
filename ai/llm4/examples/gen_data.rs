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
    let pretrain_preview: String = pretrain_rule.chars().take(30).collect();
    let finetune_preview: String = finetune_rule.chars().take(30).collect();
    println!("  pretrain: {}...", pretrain_preview);
    println!("  finetune: {}...", finetune_preview);

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
    println!("  1. ./run.sh pretrain");
    println!("  2. ./run.sh finetune");
}