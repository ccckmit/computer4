use clap::{Parser, ValueHint};
use objdump_lib::{section_type_name, ElfFile};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "objdump",
    about = "Display information from ELF object files",
    author = "Computer4 Compiler Team",
    version = "0.1.0"
)]
struct Args {
    #[arg(long = "header", help = "Display ELF header")]
    elf_header: bool,

    #[arg(short = 't', long = "sym", help = "Display symbol table")]
    symbols: bool,

    #[arg(short = 's', long = "section", help = "Display section contents")]
    sections: bool,

    #[arg(short = 'x', long = "all", help = "Display all information")]
    all: bool,

    #[arg(short = 'H', long = "program-header", help = "Display program headers")]
    program_headers: bool,

    #[arg(short = 'd', long = "disassemble", help = "Display disassembled contents")]
    disassemble: bool,

    #[arg(
        short = 'p',
        long = "string-dump",
        help = "Display string contents of a section"
    )]
    string_dump: Option<String>,

    #[arg(
        help = "Input ELF file",
        required = true,
        value_hint = ValueHint::FilePath
    )]
    input: PathBuf,
}

fn print_elf_header(elf: &ElfFile) {
    let h = &elf.header;
    println!("ELF Header:");
    println!("  Magic: {:02x?}", h.magic);
    println!("  Class: {:?}", match h.class {
        objdump_lib::ElfClass::ELF32 => "ELF32",
        objdump_lib::ElfClass::ELF64 => "ELF64",
    });
    println!("  Data: {}", match h.endianness {
        objdump_lib::Endianness::Little => "2's complement, little endian",
        objdump_lib::Endianness::Big => "2's complement, big endian",
    });
    println!("  Version: {}", h.version);
    println!("  OS/ABI: {}", h.os_abi);
    println!("  ABI Version: {}", h.abi_version);
    println!("  Type: {:#x}", h.e_type);
    println!("  Machine: {} ({})", objdump_lib::machine_name(h.e_machine), h.e_machine);
    println!("  Entry point address: {:#x}", h.e_entry);
    println!("  Start of program headers: {:#x}", h.e_phoff);
    println!("  Start of section headers: {:#x}", h.e_shoff);
    println!("  Flags: {:#x}", h.e_flags);
    println!("  Size of this header: {}", h.e_ehsize);
    println!("  Size of program headers: {}", h.e_phentsize);
    println!("  Number of program headers: {}", h.e_phnum);
    println!("  Size of section headers: {}", h.e_shentsize);
    println!("  Number of section headers: {}", h.e_shnum);
    println!("  Section header string table index: {}", h.e_shstrndx);
}

fn print_sections(elf: &ElfFile) {
    println!("Sections:");
    println!("Idx Name          Type            Address     Offset       Size      Entsize  Flags");
    println!("---------------------------------------------------------------------------------------------------");

    for (i, section) in elf.sections.iter().enumerate() {
        let flags_str = format!("{:#x}", section.header.sh_flags);
        println!(
            "{:3} {:<13} {:<15} {:#018x} {:#018x} {:<8} {:<10} {}",
            i,
            section.name,
            section_type_name(&section.header.sh_type),
            section.header.sh_addr,
            section.header.sh_offset,
            section.header.sh_size,
            if section.header.sh_entsize > 0 {
                format!("{:#x}", section.header.sh_entsize)
            } else {
                "0".to_string()
            },
            flags_str
        );
    }
}

fn print_symbols(elf: &ElfFile) {
    println!("Symbol table (.symtab):");
    println!("Num:    Value  Size Type    Bind    Other   Name");

    for (i, sym) in elf.symbols.iter().enumerate() {
        let bind = format!("{:?}", sym.header.binding());
        let typ = format!("{:?}", sym.header.typ());
        println!(
            "{:4}: {:#018x} {:<5} {:<8} {:<7} {:<6} {}",
            i,
            sym.header.st_value,
            sym.header.st_size,
            typ,
            bind,
            sym.header.st_other,
            sym.name
        );
    }
}

fn print_program_headers(elf: &ElfFile) {
    println!("Program Headers:");
    println!("Type           Offset   VirtAddr   PhysAddr   FileSize   MemSize    Flags  Align");

    for ph in &elf.program_headers {
        let flags_str = match ph.p_flags {
            0 => "",
            1 => "R",
            2 => "W",
            3 => "RW",
            4 => "X",
            5 => "RX",
            6 => "RWX",
            _ => "?",
        };
        println!(
            "{:<14} {:#010x} {:#010x} {:#010x} {:#010x} {:#010x} {:<6} {}",
            format!("{:#x}", ph.p_type),
            ph.p_offset,
            ph.p_vaddr,
            ph.p_paddr,
            ph.p_filesz,
            ph.p_memsz,
            flags_str,
            ph.p_align
        );
    }
}

#[allow(dead_code)]
fn print_section_content(elf: &ElfFile, section_name: &str) {
    if let Some(section) = elf.get_section_by_name(section_name) {
        println!("Contents of section {}:", section_name);
        if section.data.is_empty() {
            println!("  (empty)");
            return;
        }

        for (i, chunk) in section.data.chunks(16).enumerate() {
            let addr = i * 16;
            let hex_part: Vec<String> = chunk.iter()
                .map(|&b| format!("{:02x}", b))
                .collect();
            let hex_line = hex_part.join(" ");
            let ascii: String = chunk.iter()
                .map(|&b| if b >= 0x20 && b < 0x7f { b as char } else { '.' })
                .collect();

            println!("  {:04x} {:<47} {}", addr, hex_line, ascii);
        }
    } else {
        eprintln!("Section '{}' not found", section_name);
    }
}

fn print_strings(elf: &ElfFile, section_name: &str) {
    if let Some(section) = elf.get_section_by_name(section_name) {
        println!("String dump of section '{}':", section_name);
        let mut current_string = Vec::new();
        for &byte in &section.data {
            if byte == 0 {
                if !current_string.is_empty() {
                    let s = String::from_utf8_lossy(&current_string);
                    if s.chars().all(|c| c >= 0x20 as char && c < 0x7f as char) {
                        println!("  {}", s);
                    }
                    current_string.clear();
                }
            } else {
                current_string.push(byte);
            }
        }
    } else {
        eprintln!("Section '{}' not found", section_name);
    }
}

fn print_all(elf: &ElfFile) {
    print_elf_header(elf);
    println!();
    print_program_headers(elf);
    println!();
    print_sections(elf);
    println!();
    print_symbols(elf);
}

fn main() {
    let args = Args::parse();

    let data = match fs::read(&args.input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", args.input.display(), e);
            std::process::exit(1);
        }
    };

    let elf = match ElfFile::parse(&data) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error parsing ELF file: {}", e);
            std::process::exit(1);
        }
    };

    if args.all {
        print_all(&elf);
    } else if args.elf_header {
        print_elf_header(&elf);
    } else if args.symbols {
        print_symbols(&elf);
    } else if args.sections {
        print_sections(&elf);
    } else if args.program_headers {
        print_program_headers(&elf);
    } else if let Some(ref section_name) = args.string_dump {
        print_strings(&elf, section_name);
    } else {
        print_elf_header(&elf);
    }
}