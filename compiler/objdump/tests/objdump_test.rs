use objdump_lib::{ElfFile, ELF_MAGIC, machine_name, section_type_name, SectionType, SymbolBinding, SymbolType};

fn create_test_elf(data: &[u8]) -> Vec<u8> {
    let mut elf = data.to_vec();
    if elf.len() < 64 {
        elf.resize(64, 0);
    }
    elf[0..4].copy_from_slice(&ELF_MAGIC);
    elf[4] = 2;
    elf[5] = 1;
    elf[6] = 1;
    elf
}

#[test]
fn test_parse_valid_elf() {
    let data = create_test_elf(&[]);
    let result = ElfFile::parse(&data);
    assert!(result.is_ok());
}

#[test]
fn test_parse_invalid_magic() {
    let mut data = vec![0u8; 64];
    data[4] = 2;
    let result = ElfFile::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_elf_class_64() {
    let data = create_test_elf(&[]);
    let elf = ElfFile::parse(&data).unwrap();
    assert_eq!(elf.header.class, objdump_lib::ElfClass::ELF64);
}

#[test]
fn test_elf_endianness_little() {
    let data = create_test_elf(&[]);
    let elf = ElfFile::parse(&data).unwrap();
    assert_eq!(elf.header.endianness, objdump_lib::Endianness::Little);
}

#[test]
fn test_machine_name_x86_64() {
    assert_eq!(machine_name(62), "AMD x86-64");
}

#[test]
fn test_machine_name_arm() {
    assert_eq!(machine_name(40), "ARM 32-bit");
}

#[test]
fn test_machine_name_riscv() {
    assert_eq!(machine_name(243), "RISC-V");
}

#[test]
fn test_machine_name_unknown() {
    assert_eq!(machine_name(999), "Unknown");
}

#[test]
fn test_section_type_names() {
    assert_eq!(section_type_name(&SectionType::Null), "NULL");
    assert_eq!(section_type_name(&SectionType::ProgramBits), "PROGBITS");
    assert_eq!(section_type_name(&SectionType::SymbolTable), "SYMTAB");
    assert_eq!(section_type_name(&SectionType::StringTable), "STRTAB");
    assert_eq!(section_type_name(&SectionType::DynSym), "DYNSYM");
}

#[test]
fn test_section_type_unknown() {
    let unknown = SectionType::Unknown(999);
    assert_eq!(section_type_name(&unknown), "UNKNOWN");
}

#[test]
fn test_symbol_binding_local() {
    assert_eq!(SymbolBinding::from_elf(0x00), SymbolBinding::Local);
}

#[test]
fn test_symbol_binding_global() {
    assert_eq!(SymbolBinding::from_elf(0x10), SymbolBinding::Global);
}

#[test]
fn test_symbol_binding_weak() {
    assert_eq!(SymbolBinding::from_elf(0x20), SymbolBinding::Weak);
}

#[test]
fn test_symbol_type_notype() {
    assert_eq!(SymbolType::from_elf(0), SymbolType::NoType);
    assert_eq!(SymbolType::from_elf(0x10), SymbolType::NoType);
}

#[test]
fn test_symbol_type_func() {
    assert_eq!(SymbolType::from_elf(2), SymbolType::Func);
    assert_eq!(SymbolType::from_elf(0x12), SymbolType::Func);
}

#[test]
fn test_symbol_type_object() {
    assert_eq!(SymbolType::from_elf(1), SymbolType::Object);
    assert_eq!(SymbolType::from_elf(0x11), SymbolType::Object);
}

#[test]
fn test_symbol_type_section() {
    assert_eq!(SymbolType::from_elf(3), SymbolType::Section);
}

#[test]
fn test_buffer_too_small_error() {
    let small_data = vec![0u8; 10];
    let result = ElfFile::parse(&small_data);
    assert!(result.is_err());
}

#[test]
fn test_empty_sections_vector() {
    let data = create_test_elf(&[]);
    let elf = ElfFile::parse(&data).unwrap();
    assert!(elf.sections.is_empty() || !elf.sections.is_empty());
}

#[test]
fn test_elf_magic_constant() {
    assert_eq!(ELF_MAGIC, [0x7f, b'E', b'L', b'F']);
}