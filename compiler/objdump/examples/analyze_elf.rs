use objdump_lib::{ElfFile, machine_name, section_type_name};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <elf-file>", args[0]);
        std::process::exit(1);
    }

    let data = std::fs::read(&args[1])?;
    let elf = ElfFile::parse(&data)?;

    println!("=== ELF File Analysis ===");
    println!();
    println!("Machine: {} ({})", machine_name(elf.header.e_machine), elf.header.e_machine);
    println!("Entry point: {:#018x}", elf.header.e_entry);
    println!();

    println!("=== Sections ===");
    for section in &elf.sections {
        println!(
            "  {:20} {:15} size:{:>8}",
            section.name,
            section_type_name(&section.header.sh_type),
            section.header.sh_size
        );
    }
    println!();

    println!("=== Symbols ===");
    for sym in &elf.symbols {
        if !sym.name.is_empty() {
            println!(
                "  {:30} value:{:#018x} size:{:>8}",
                sym.name,
                sym.header.st_value,
                sym.header.st_size
            );
        }
    }

    Ok(())
}