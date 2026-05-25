use scroll::{LE, Pread, Error as ScrollError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObjdumpError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to read from buffer: {0}")]
    ScrollError(#[from] ScrollError),
    #[error("Invalid ELF magic number")]
    InvalidMagic,
    #[error("Invalid ELF class: {0}")]
    InvalidClass(u8),
    #[error("Invalid ELF data encoding: {0}")]
    InvalidDataEncoding(u8),
    #[error("Buffer too small: needed {needed}, got {got}")]
    BufferTooSmall { needed: usize, got: usize },
    #[error("Invalid section index: {0}")]
    InvalidSectionIndex(usize),
    #[error("Invalid symbol index: {0}")]
    InvalidSymbolIndex(usize),
    #[error("Unsupported ELF version: {0}")]
    UnsupportedVersion(u8),
    #[error("Unknown machine type: {0}")]
    UnknownMachine(u16),
}

pub type Result<T> = std::result::Result<T, ObjdumpError>;

pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElfClass {
    ELF32,
    ELF64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    Null,
    ProgramBits,
    SymbolTable,
    StringTable,
    Rela,
    SymbolTableHash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    Shlib,
    DynSym,
    InitArray,
    FiniArray,
    PreinitArray,
    Group,
    SymTabShndx,
    Num,
    Unknown(u32),
}

impl SectionType {
    pub fn from_elf(typ: u32) -> Self {
        match typ {
            0 => SectionType::Null,
            1 => SectionType::ProgramBits,
            2 => SectionType::SymbolTable,
            3 => SectionType::StringTable,
            4 => SectionType::Rela,
            5 => SectionType::SymbolTableHash,
            6 => SectionType::Dynamic,
            7 => SectionType::Note,
            8 => SectionType::NoBits,
            9 => SectionType::Rel,
            10 => SectionType::Shlib,
            11 => SectionType::DynSym,
            14 => SectionType::InitArray,
            15 => SectionType::FiniArray,
            16 => SectionType::PreinitArray,
            17 => SectionType::Group,
            18 => SectionType::SymTabShndx,
            19 => SectionType::Num,
            other => SectionType::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolBinding {
    Local,
    Global,
    Weak,
    Unknown(u8),
}

impl SymbolBinding {
    pub fn from_elf(info: u8) -> Self {
        match info >> 4 {
            0 => SymbolBinding::Local,
            1 => SymbolBinding::Global,
            2 => SymbolBinding::Weak,
            other => SymbolBinding::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    NoType,
    Object,
    Func,
    Section,
    File,
    Common,
    TLS,
    Unknown(u8),
}

impl SymbolType {
    pub fn from_elf(info: u8) -> Self {
        match info & 0xf {
            0 => SymbolType::NoType,
            1 => SymbolType::Object,
            2 => SymbolType::Func,
            3 => SymbolType::Section,
            4 => SymbolType::File,
            5 => SymbolType::Common,
            6 => SymbolType::TLS,
            other => SymbolType::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionFlags {
    None,
    Write,
    Alloc,
    ExecInstr,
    MaskProc,
    Unknown(u32),
}

impl SectionFlags {
    pub fn from_elf(flags: u32) -> Self {
        match flags {
            0 => SectionFlags::None,
            2 => SectionFlags::Write,
            4 => SectionFlags::Alloc,
            8 => SectionFlags::ExecInstr,
            0xf0 => SectionFlags::MaskProc,
            other => SectionFlags::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElfHeader64 {
    pub magic: [u8; 4],
    pub class: ElfClass,
    pub endianness: Endianness,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionHeader64 {
    pub sh_name: u32,
    pub sh_type: SectionType,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol64 {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

impl Symbol64 {
    pub fn binding(&self) -> SymbolBinding {
        SymbolBinding::from_elf(self.st_info)
    }

    pub fn typ(&self) -> SymbolType {
        SymbolType::from_elf(self.st_info)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramHeader64 {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub header: SectionHeader64,
    pub name: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub header: Symbol64,
    pub name: String,
}

#[derive(Debug)]
pub struct ElfFile {
    pub header: ElfHeader64,
    pub sections: Vec<Section>,
    pub symbols: Vec<Symbol>,
    pub program_headers: Vec<ProgramHeader64>,
    pub raw_data: Vec<u8>,
}

impl ElfFile {
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 64 {
            return Err(ObjdumpError::BufferTooSmall {
                needed: 64,
                got: data.len(),
            });
        }

        let magic = data[0..4].try_into().unwrap();
        if magic != ELF_MAGIC {
            return Err(ObjdumpError::InvalidMagic);
        }

        let class = match data[4] {
            1 => ElfClass::ELF32,
            2 => ElfClass::ELF64,
            other => return Err(ObjdumpError::InvalidClass(other)),
        };

        let endianness = match data[5] {
            1 => Endianness::Little,
            2 => Endianness::Big,
            other => return Err(ObjdumpError::InvalidDataEncoding(other)),
        };

        let version = data[6];
        if version != 1 {
            return Err(ObjdumpError::UnsupportedVersion(version));
        }

        let mut offset: usize = 16;

        let (e_type, e_machine, e_entry, e_phoff, e_shoff, e_flags,
             e_ehsize, e_phentsize, e_phnum, e_shentsize, e_shnum, e_shstrndx) =
            if class == ElfClass::ELF64 {
                let e_type = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_machine = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_entry = data.pread_with::<u64>(offset, LE)?;
                offset += 8;
                let e_phoff = data.pread_with::<u64>(offset, LE)?;
                offset += 8;
                let e_shoff = data.pread_with::<u64>(offset, LE)?;
                offset += 8;
                let e_flags = data.pread_with::<u32>(offset, LE)?;
                offset += 4;
                let e_ehsize = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_phentsize = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_phnum = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_shentsize = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_shnum = data.pread_with::<u16>(offset, LE)?;
                offset += 2;
                let e_shstrndx = data.pread_with::<u16>(offset, LE)?;

                (e_type, e_machine, e_entry, e_phoff, e_shoff, e_flags,
                 e_ehsize, e_phentsize, e_phnum, e_shentsize, e_shnum, e_shstrndx)
            } else {
                return Err(ObjdumpError::InvalidClass(1));
            };

        let header = ElfHeader64 {
            magic,
            class,
            endianness,
            version,
            os_abi: data[7],
            abi_version: data[8],
            e_type,
            e_machine,
            e_entry,
            e_phoff,
            e_shoff,
            e_flags,
            e_ehsize,
            e_phentsize,
            e_phnum,
            e_shentsize,
            e_shnum,
            e_shstrndx,
        };

        let mut sections = Vec::new();
        let mut string_section_data: Option<Vec<u8>> = None;

        for i in 0..header.e_shnum {
            let sh_offset = header.e_shoff as usize + (i as usize * header.e_shentsize as usize);
            if sh_offset + 64 > data.len() {
                continue;
            }

            let sh_name = data.pread_with::<u32>(sh_offset, LE)?;
            let sh_type_val = data.pread_with::<u32>(sh_offset + 4, LE)?;
            let sh_flags = data.pread_with::<u64>(sh_offset + 8, LE)?;
            let sh_addr = data.pread_with::<u64>(sh_offset + 16, LE)?;
            let sh_offset_val = data.pread_with::<u64>(sh_offset + 24, LE)?;
            let sh_size = data.pread_with::<u64>(sh_offset + 32, LE)?;
            let sh_link = data.pread_with::<u32>(sh_offset + 40, LE)?;
            let sh_info = data.pread_with::<u32>(sh_offset + 44, LE)?;
            let sh_addralign = data.pread_with::<u64>(sh_offset + 48, LE)?;
            let sh_entsize = data.pread_with::<u64>(sh_offset + 56, LE)?;

            let section_header = SectionHeader64 {
                sh_name,
                sh_type: SectionType::from_elf(sh_type_val),
                sh_flags,
                sh_addr,
                sh_offset: sh_offset_val,
                sh_size,
                sh_link,
                sh_info,
                sh_addralign,
                sh_entsize,
            };

            let section_data = if sh_size > 0 && (sh_offset_val as usize) + (sh_size as usize) <= data.len() {
                data[(sh_offset_val as usize)..((sh_offset_val as usize) + (sh_size as usize))].to_vec()
            } else {
                Vec::new()
            };

            if i as u16 == header.e_shstrndx {
                string_section_data = Some(section_data.clone());
            }

            sections.push(Section {
                header: section_header,
                name: String::new(),
                data: section_data,
            });
        }

        if let Some(str_data) = string_section_data {
            for section in sections.iter_mut() {
                let name_offset = section.header.sh_name as usize;
                if name_offset < str_data.len() {
                    if let Some(end) = str_data[name_offset..].iter().position(|&b| b == 0) {
                        section.name = String::from_utf8_lossy(&str_data[name_offset..name_offset + end]).to_string();
                    }
                }
            }
        }

        let mut symbols = Vec::new();
        let strtab_section = sections.iter().find(|s| s.name == ".strtab");
        let symtab_section = sections.iter().find(|s| s.name == ".symtab");

        if let (Some(strtab), Some(symtab)) = (strtab_section, symtab_section) {
            let entry_size = if symtab.header.sh_entsize > 0 {
                symtab.header.sh_entsize as usize
            } else {
                24
            };

            for i in (0..symtab.data.len()).step_by(entry_size) {
                if i + entry_size > symtab.data.len() {
                    break;
                }

                let st_name = symtab.data.pread_with::<u32>(i, LE)?;
                let st_info = symtab.data[i + 4];
                let st_other = symtab.data[i + 5];
                let st_shndx = symtab.data.pread_with::<u16>(i + 6, LE)?;
                let st_value = symtab.data.pread_with::<u64>(i + 8, LE)?;
                let st_size = symtab.data.pread_with::<u64>(i + 16, LE)?;

                let sym_header = Symbol64 {
                    st_name,
                    st_info,
                    st_other,
                    st_shndx,
                    st_value,
                    st_size,
                };

                let name = if st_name > 0 && (st_name as usize) < strtab.data.len() {
                    let start = st_name as usize;
                    if let Some(end) = strtab.data[start..].iter().position(|&b| b == 0) {
                        String::from_utf8_lossy(&strtab.data[start..start + end]).to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                symbols.push(Symbol { header: sym_header, name });
            }
        }

        let mut program_headers = Vec::new();
        if header.e_phnum > 0 && header.e_phoff > 0 {
            for i in 0..header.e_phnum {
                let ph_offset = (header.e_phoff as usize) + (i as usize * header.e_phentsize as usize);
                if ph_offset + 56 > data.len() {
                    continue;
                }

                let p_type = data.pread_with::<u32>(ph_offset, LE)?;
                let p_flags = data.pread_with::<u32>(ph_offset + 4, LE)?;
                let p_offset = data.pread_with::<u64>(ph_offset + 8, LE)?;
                let p_vaddr = data.pread_with::<u64>(ph_offset + 16, LE)?;
                let p_paddr = data.pread_with::<u64>(ph_offset + 24, LE)?;
                let p_filesz = data.pread_with::<u64>(ph_offset + 32, LE)?;
                let p_memsz = data.pread_with::<u64>(ph_offset + 40, LE)?;
                let p_align = data.pread_with::<u64>(ph_offset + 48, LE)?;

                program_headers.push(ProgramHeader64 {
                    p_type,
                    p_flags,
                    p_offset,
                    p_vaddr,
                    p_paddr,
                    p_filesz,
                    p_memsz,
                    p_align,
                });
            }
        }

        Ok(ElfFile {
            header,
            sections,
            symbols,
            program_headers,
            raw_data: data.to_vec(),
        })
    }

    pub fn get_section_by_name(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name == name)
    }

    pub fn get_symbol_by_name(&self, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.name == name)
    }
}

pub fn machine_name(machine: u16) -> &'static str {
    match machine {
        0 => "No machine",
        1 => "AT&T WE 32100",
        2 => "Sun SPARC",
        3 => "Intel 386",
        4 => "Motorola 68000",
        5 => "Motorola 88000",
        7 => "Intel 80860",
        8 => "MIPS R3000",
        15 => "HP PA-RISC",
        20 => "PowerPC",
        21 => "PowerPC 64-bit",
        22 => "IBM S/390",
        23 => "IBM SPARC v9-64",
        40 => "ARM 32-bit",
        41 => "AMD x86-64",
        62 => "AMD x86-64",
        83 => "IA-64",
        183 => "ARM 64-bit",
        243 => "RISC-V",
        247 => "Berkeley Packet Filter",
        _ => "Unknown",
    }
}

pub fn section_type_name(typ: &SectionType) -> &'static str {
    match typ {
        SectionType::Null => "NULL",
        SectionType::ProgramBits => "PROGBITS",
        SectionType::SymbolTable => "SYMTAB",
        SectionType::StringTable => "STRTAB",
        SectionType::Rela => "RELA",
        SectionType::SymbolTableHash => "HASH",
        SectionType::Dynamic => "DYNAMIC",
        SectionType::Note => "NOTE",
        SectionType::NoBits => "NOBITS",
        SectionType::Rel => "REL",
        SectionType::Shlib => "SHLIB",
        SectionType::DynSym => "DYNSYM",
        SectionType::InitArray => "INIT_ARRAY",
        SectionType::FiniArray => "FINI_ARRAY",
        SectionType::PreinitArray => "PREINIT_ARRAY",
        SectionType::Group => "GROUP",
        SectionType::SymTabShndx => "SYMTAB_SHNDX",
        SectionType::Num => "NUM",
        SectionType::Unknown(_) => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_minimal_elf() -> Vec<u8> {
        let mut data = vec![0u8; 64];
        data[0..4].copy_from_slice(&ELF_MAGIC);
        data[4] = 2;
        data[5] = 1;
        data[6] = 1;
        data[7] = 0;
        data
    }

    #[test]
    fn test_elf_magic_validation() {
        let valid_elf = create_minimal_elf();
        assert!(ElfFile::parse(&valid_elf).is_ok());

        let invalid_elf = vec![0u8; 64];
        assert!(matches!(ElfFile::parse(&invalid_elf), Err(ObjdumpError::InvalidMagic)));
    }

    #[test]
    fn test_elf_class_detection() {
        let mut data = create_minimal_elf();
        data[4] = 1;
        assert!(matches!(ElfFile::parse(&data), Err(ObjdumpError::InvalidClass(1))));
    }

    #[test]
    fn test_elf_endianness_detection() {
        let mut data = create_minimal_elf();
        data[5] = 3;
        assert!(matches!(ElfFile::parse(&data), Err(ObjdumpError::InvalidDataEncoding(3))));
    }

    #[test]
    fn test_section_type_from_elf() {
        assert_eq!(SectionType::from_elf(0), SectionType::Null);
        assert_eq!(SectionType::from_elf(1), SectionType::ProgramBits);
        assert_eq!(SectionType::from_elf(2), SectionType::SymbolTable);
        assert_eq!(SectionType::from_elf(3), SectionType::StringTable);
        assert_eq!(SectionType::from_elf(11), SectionType::DynSym);
    }

    #[test]
    fn test_symbol_binding() {
        assert_eq!(SymbolBinding::from_elf(0x00), SymbolBinding::Local);
        assert_eq!(SymbolBinding::from_elf(0x10), SymbolBinding::Global);
        assert_eq!(SymbolBinding::from_elf(0x20), SymbolBinding::Weak);
    }

    #[test]
    fn test_symbol_type() {
        assert_eq!(SymbolType::from_elf(0), SymbolType::NoType);
        assert_eq!(SymbolType::from_elf(1), SymbolType::Object);
        assert_eq!(SymbolType::from_elf(2), SymbolType::Func);
        assert_eq!(SymbolType::from_elf(3), SymbolType::Section);
    }

    #[test]
    fn test_machine_name() {
        assert_eq!(machine_name(3), "Intel 386");
        assert_eq!(machine_name(62), "AMD x86-64");
        assert_eq!(machine_name(243), "RISC-V");
    }

    #[test]
    fn test_section_type_name() {
        assert_eq!(section_type_name(&SectionType::Null), "NULL");
        assert_eq!(section_type_name(&SectionType::SymbolTable), "SYMTAB");
        assert_eq!(section_type_name(&SectionType::StringTable), "STRTAB");
    }

    #[test]
    fn test_buffer_too_small() {
        let small_data = vec![0u8; 32];
        assert!(matches!(
            ElfFile::parse(&small_data),
            Err(ObjdumpError::BufferTooSmall { .. })
        ));
    }

    #[test]
    fn test_symbol_accessors() {
        let sym = Symbol64 {
            st_name: 0,
            st_info: 0x22,
            st_other: 0,
            st_shndx: 1,
            st_value: 0x1000,
            st_size: 16,
        };
        assert_eq!(sym.binding(), SymbolBinding::Weak);
        assert_eq!(sym.typ(), SymbolType::Func);
    }
}