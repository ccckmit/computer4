use objdump_lib::ElfFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <elf-file> <symbol-name>", args[0]);
        std::process::exit(1);
    }

    let data = std::fs::read(&args[1])?;
    let elf = ElfFile::parse(&data)?;
    let symbol_name = &args[2];

    if let Some(sym) = elf.get_symbol_by_name(symbol_name) {
        println!("Symbol '{}' found:", symbol_name);
        println!("  Address: {:#018x}", sym.header.st_value);
        println!("  Size: {}", sym.header.st_size);
        println!("  Type: {:?}", sym.header.typ());
        println!("  Binding: {:?}", sym.header.binding());
        println!("  Section index: {}", sym.header.st_shndx);
    } else {
        println!("Symbol '{}' not found", symbol_name);
        println!("Available symbols:");
        for sym in &elf.symbols {
            if !sym.name.is_empty() {
                println!("  {}", sym.name);
            }
        }
    }

    Ok(())
}