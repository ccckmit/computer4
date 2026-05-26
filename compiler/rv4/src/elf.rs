use crate::memory::Memory;

const ET_REL: u16 = 1;
const ET_EXEC: u16 = 2;
const EM_RISCV: u16 = 243;
const PT_LOAD: u32 = 1;
const SHT_RELA: u32 = 4;
const SHT_SYMTAB: u32 = 2;
const SHF_ALLOC: u64 = 2;
const SHF_EXECINSTR: u64 = 4;

const R_RISCV_NONE: u32 = 0;
const R_RISCV_32: u32 = 1;
const R_RISCV_64: u32 = 2;
const R_RISCV_JAL: u32 = 11;
const R_RISCV_BRANCH: u32 = 16;
const R_RISCV_CALL: u32 = 18;
const R_RISCV_CALL_PLT: u32 = 19;
const R_RISCV_HI20: u32 = 7;
const R_RISCV_LO12_I: u32 = 8;
const R_RISCV_LO12_S: u32 = 9;
const R_RISCV_PCREL_HI20: u32 = 23;
const R_RISCV_PCREL_LO12_I: u32 = 24;
const R_RISCV_PCREL_LO12_S: u32 = 25;

fn r16(data: &[u8], off: usize) -> u16 {
    data[off] as u16 | (data[off + 1] as u16) << 8
}
fn r32(data: &[u8], off: usize) -> u32 {
    data[off] as u32 | (data[off + 1] as u32) << 8 | (data[off + 2] as u32) << 16 | (data[off + 3] as u32) << 24
}
fn r64(data: &[u8], off: usize) -> u64 {
    let lo = r32(data, off) as u64;
    let hi = r32(data, off + 4) as u64;
    lo | hi << 32
}

struct Section {
    sh_addr: u64,
    sh_offset: u64,
    sh_size: u64,
    sh_type: u32,
    sh_flags: u64,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: u64,
}

struct Symbol {
    st_value: u64,
    st_shndx: u16,
    name: String,
}

pub struct LoadedElf {
    pub entry: u64,
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub vaddr: u64,
    pub data: Vec<u8>,
    pub mem_size: u64,
    pub _flags: u32,
}

pub fn load(data: &[u8]) -> Result<LoadedElf, String> {
    if data.len() < 16 || data[0] != 0x7f || data[1] != b'E' || data[2] != b'L' || data[3] != b'F' {
        return Err("Not a valid ELF file".to_string());
    }
    let class = data[4];
    if data[5] != 1 {
        return Err("Only little-endian ELF supported".to_string());
    }
    match class {
        1 => load_elf32(data),
        2 => load_elf64(data),
        _ => Err(format!("Unsupported ELF class: {}", class)),
    }
}

fn get_sections_64(data: &[u8], e_shoff: u64, e_shnum: u16, e_shentsize: u16, e_shstrndx: u16) -> Vec<Section> {
    let _shstrtab_sec = if e_shstrndx < e_shnum {
        let soff = e_shoff + e_shstrndx as u64 * e_shentsize as u64;
        Some((r64(data, soff as usize + 24), r64(data, soff as usize + 32))) // offset, size
    } else { None };

    let mut secs = Vec::new();
    for i in 0..e_shnum {
        let off = (e_shoff + i as u64 * e_shentsize as u64) as usize;
        let _sh_name = r32(data, off);
        let sh_type = r32(data, off + 4);
        let sh_flags = r64(data, off + 8);
        let sh_addr = r64(data, off + 16);
        let sh_offset = r64(data, off + 24);
        let sh_size = r64(data, off + 32);
        let sh_link = r32(data, off + 40);
        let sh_info = r32(data, off + 44);
        let sh_addralign = r64(data, off + 48);
        secs.push(Section { sh_addr, sh_offset, sh_size, sh_type, sh_flags, sh_link, sh_info, sh_addralign });
    }
    secs
}

fn get_sections_32(data: &[u8], e_shoff: u64, e_shnum: u16, e_shentsize: u16, e_shstrndx: u16) -> Vec<Section> {
    let _shstrtab_sec = if e_shstrndx < e_shnum {
        let soff = e_shoff + e_shstrndx as u64 * e_shentsize as u64;
        Some((r32(data, soff as usize + 16) as u64, r32(data, soff as usize + 20) as u64))
    } else { None };

    let mut secs = Vec::new();
    for i in 0..e_shnum {
        let off = (e_shoff + i as u64 * e_shentsize as u64) as usize;
        let _sh_name = r32(data, off);
        let sh_type = r32(data, off + 4);
        let sh_flags = r32(data, off + 8) as u64;
        let sh_addr = r32(data, off + 12) as u64;
        let sh_offset = r32(data, off + 16) as u64;
        let sh_size = r32(data, off + 20) as u64;
        let sh_link = r32(data, off + 24);
        let sh_info = r32(data, off + 28);
        let sh_addralign = r32(data, off + 32) as u64;
        secs.push(Section { sh_addr, sh_offset, sh_size, sh_type, sh_flags, sh_link, sh_info, sh_addralign });
    }
    secs
}

fn sec_name(data: &[u8], secs: &[Section], idx: usize) -> String {
    if secs.is_empty() { return String::new(); }
    let str_sec = &secs[0]; // assume first section is strtab
    let mut s = Vec::new();
    let mut i = str_sec.sh_offset as usize + idx;
    while i < data.len() && data[i] != 0 {
        s.push(data[i]);
        i += 1;
    }
    String::from_utf8_lossy(&s).to_string()
}

fn load_elf64(data: &[u8]) -> Result<LoadedElf, String> {
    let e_type = r16(data, 16);
    let e_machine = r16(data, 18);
    let e_entry = r64(data, 24);
    let e_phoff = r64(data, 32);
    let e_shoff = r64(data, 40);
    let e_phentsize = r16(data, 54);
    let e_phnum = r16(data, 56);
    let e_shentsize = r16(data, 58);
    let e_shnum = r16(data, 60);
    let e_shstrndx = r16(data, 62);

    if e_machine != EM_RISCV {
        return Err(format!("Not RISC-V ELF (machine={})", e_machine));
    }

    if e_type == ET_EXEC {
        let mut segments = Vec::new();
        for i in 0..e_phnum as u64 {
            let off = (e_phoff + i * e_phentsize as u64) as usize;
            let p_type = r32(data, off);
            let p_flags = r32(data, off + 4);
            let p_offset = r64(data, off + 8) as usize;
            let p_vaddr = r64(data, off + 16);
            let p_filesz = r64(data, off + 32);
            let p_memsz = r64(data, off + 40);
            if p_type == PT_LOAD && p_memsz > 0 {
                let mut seg_data = vec![0u8; p_memsz as usize];
                if p_filesz > 0 {
                    let end = p_offset + p_filesz as usize;
                    if end <= data.len() {
                        seg_data[..p_filesz as usize].copy_from_slice(&data[p_offset..end]);
                    }
                }
                segments.push(Segment { vaddr: p_vaddr, data: seg_data, mem_size: p_memsz, _flags: p_flags });
            }
        }
        Ok(LoadedElf { entry: e_entry, segments })
    } else if e_type == ET_REL {
        load_relocatable_64(data, e_shoff, e_shnum, e_shentsize, e_shstrndx)
    } else {
        Err(format!("Unsupported ELF type: {}", e_type))
    }
}

fn load_elf32(data: &[u8]) -> Result<LoadedElf, String> {
    let e_type = r16(data, 16);
    let e_machine = r16(data, 18);
    let e_entry = r32(data, 24) as u64;
    let e_phoff = r32(data, 28) as u64;
    let e_shoff = r32(data, 32) as u64;
    let e_phentsize = r16(data, 42);
    let e_phnum = r16(data, 44);
    let e_shentsize = r16(data, 46);
    let e_shnum = r16(data, 48);
    let e_shstrndx = r16(data, 50);

    if e_machine != EM_RISCV {
        return Err(format!("Not RISC-V ELF (machine={})", e_machine));
    }

    if e_type == ET_EXEC {
        let mut segments = Vec::new();
        for i in 0..e_phnum as u64 {
            let off = (e_phoff + i * e_phentsize as u64) as usize;
            let p_type = r32(data, off);
            let p_offset = r32(data, off + 4) as usize;
            let p_vaddr = r32(data, off + 8) as u64;
            let p_filesz = r32(data, off + 16) as u64;
            let p_memsz = r32(data, off + 20) as u64;
            let p_flags = r32(data, off + 24);
            if p_type == PT_LOAD && p_memsz > 0 {
                let mut seg_data = vec![0u8; p_memsz as usize];
                if p_filesz > 0 {
                    let end = p_offset + p_filesz as usize;
                    if end <= data.len() {
                        seg_data[..p_filesz as usize].copy_from_slice(&data[p_offset..end]);
                    }
                }
                segments.push(Segment { vaddr: p_vaddr, data: seg_data, mem_size: p_memsz, _flags: p_flags });
            }
        }
        Ok(LoadedElf { entry: e_entry, segments })
    } else if e_type == ET_REL {
        load_relocatable_32(data, e_shoff, e_shnum, e_shentsize, e_shstrndx)
    } else {
        Err(format!("Unsupported ELF type: {}", e_type))
    }
}

fn load_relocatable_64(data: &[u8], e_shoff: u64, e_shnum: u16, e_shentsize: u16, e_shstrndx: u16) -> Result<LoadedElf, String> {
    let secs = get_sections_64(data, e_shoff, e_shnum, e_shentsize, e_shstrndx);
    let (syms, _strtab_idx) = find_symtab_64(data, &secs).unwrap_or((Vec::new(), 0));

    let base: u64 = 0x10000;
    let mut cur = base;
    let mut sec_addrs = vec![0u64; secs.len()];
    for (i, s) in secs.iter().enumerate() {
        if s.sh_flags & SHF_ALLOC != 0 {
            let align = s.sh_addralign.max(1);
            let a = (cur + align - 1) & !(align - 1);
            sec_addrs[i] = a;
            cur = a + s.sh_size;
        }
    }

    let total = std::cmp::max((cur + 0xfff) & !0xfff, 0x100000);
    let mut seg = vec![0u8; total as usize];

    for (i, s) in secs.iter().enumerate() {
        if s.sh_flags & SHF_ALLOC != 0 && s.sh_size > 0 {
            let a = (sec_addrs[i] - base) as usize;
            let end = s.sh_offset as usize + s.sh_size as usize;
            if end <= data.len() {
                seg[a..a + s.sh_size as usize].copy_from_slice(&data[s.sh_offset as usize..end]);
            }
        }
    }

    for (i, s) in secs.iter().enumerate() {
        if s.sh_type != SHT_RELA { continue; }
        let tgt = s.sh_info as usize;
        if tgt >= secs.len() || secs[tgt].sh_flags & SHF_ALLOC == 0 { continue; }
        let tgt_base = sec_addrs[tgt];
        let n = s.sh_size / 24;

        struct R { off: u64, ty: u32, sym: u32, add: i64 }
        let mut relocs: Vec<R> = Vec::new();
        for j in 0..n {
            let ro = (s.sh_offset + j * 24) as usize;
            let r_off = r64(data, ro);
            let r_info = r64(data, ro + 8);
            let r_add = r64(data, ro + 16) as i64;
            relocs.push(R { off: r_off, ty: (r_info & 0xffffffff) as u32, sym: (r_info >> 32) as u32, add: r_add });
        }

        let sym_val = |sym_idx: u32| -> Option<u64> {
            let sym = syms.get(sym_idx as usize)?;
            let si = sym.st_shndx as usize;
            if si > 0 && si < sec_addrs.len() { Some(sec_addrs[si] + sym.st_value) } else { None }
        };

        for r in &relocs {
            let sv = sym_val(r.sym).unwrap_or(0);
            if r.ty == R_RISCV_PCREL_LO12_I || r.ty == R_RISCV_PCREL_LO12_S {
                let label_addr = sv;
                let auipc_off = label_addr.wrapping_sub(tgt_base);
                let hi_sym = relocs.iter().find(|rr| rr.off == auipc_off && rr.ty == R_RISCV_PCREL_HI20)
                    .map(|rr| rr.sym);
                let auipc_pc = tgt_base + auipc_off;
                if let Some(hs) = hi_sym {
                    let real_sv = sym_val(hs).unwrap_or(0);
                    do_reloc_64(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, real_sv, tgt_base, auipc_pc);
                } else {
                    do_reloc_64(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, sv, tgt_base, auipc_pc);
                }
            } else {
                let pc = tgt_base + r.off;
                do_reloc_64(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, sv, tgt_base, pc);
            }
        }
    }

    let entry = find_entry2(&sec_addrs, &syms);
    Ok(LoadedElf { entry, segments: vec![Segment { vaddr: base, data: seg, mem_size: total, _flags: 5 }] })
}

fn load_relocatable_32(data: &[u8], e_shoff: u64, e_shnum: u16, e_shentsize: u16, e_shstrndx: u16) -> Result<LoadedElf, String> {
    let secs = get_sections_32(data, e_shoff, e_shnum, e_shentsize, e_shstrndx);
    let (syms, _strtab_idx) = find_symtab_32(data, &secs).unwrap_or((Vec::new(), 0));

    let base: u64 = 0x10000;
    let mut cur = base;
    let mut sec_addrs = vec![0u64; secs.len()];
    for (i, s) in secs.iter().enumerate() {
        if s.sh_flags & SHF_ALLOC != 0 {
            let align = s.sh_addralign.max(1);
            let a = (cur + align - 1) & !(align - 1);
            sec_addrs[i] = a;
            cur = a + s.sh_size;
        }
    }



    let total = std::cmp::max((cur + 0xfff) & !0xfff, 0x100000);
    let mut seg = vec![0u8; total as usize];

    for (i, s) in secs.iter().enumerate() {
        if s.sh_flags & SHF_ALLOC != 0 && s.sh_size > 0 {
            let a = (sec_addrs[i] - base) as usize;
            let end = s.sh_offset as usize + s.sh_size as usize;
            if end <= data.len() {
                seg[a..a + s.sh_size as usize].copy_from_slice(&data[s.sh_offset as usize..end]);
            }
        }
    }

    for (i, s) in secs.iter().enumerate() {
        if s.sh_type != SHT_RELA { continue; }
        let tgt = s.sh_info as usize;
        if tgt >= secs.len() || secs[tgt].sh_flags & SHF_ALLOC == 0 { continue; }
        let tgt_base = sec_addrs[tgt];
        let n = s.sh_size / 12;

        struct R32 { off: u64, ty: u32, sym: u32, add: i64 }
        let mut relocs: Vec<R32> = Vec::new();
        for j in 0..n {
            let ro = (s.sh_offset + j * 12) as usize;
            let r_off = r32(data, ro) as u64;
            let r_info = r32(data, ro + 4) as u64;
            relocs.push(R32 { off: r_off, ty: (r_info & 0xff) as u32, sym: (r_info >> 8) as u32, add: r32(data, ro + 8) as i64 });
        }

        let sym_val = |sym_idx: u32| -> Option<u64> {
            let sym = syms.get(sym_idx as usize)?;
            let si = sym.st_shndx as usize;
            if si > 0 && si < sec_addrs.len() { Some(sec_addrs[si] + sym.st_value) } else { None }
        };

        for r in &relocs {
            let sv = sym_val(r.sym).unwrap_or(0);
            if r.ty == R_RISCV_PCREL_LO12_I || r.ty == R_RISCV_PCREL_LO12_S {
                let label_addr = sv;
                let auipc_off = label_addr.wrapping_sub(tgt_base);
                let hi_sym = relocs.iter().find(|rr| rr.off == auipc_off && rr.ty == R_RISCV_PCREL_HI20)
                    .map(|rr| rr.sym);
                let auipc_pc = tgt_base + auipc_off;
                if let Some(hs) = hi_sym {
                    let real_sv = sym_val(hs).unwrap_or(0);
                    do_reloc_32(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, real_sv, tgt_base, auipc_pc);
                } else {
                    do_reloc_32(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, sv, tgt_base, auipc_pc);
                }
            } else {
                let pc = tgt_base + r.off;
                do_reloc_32(&mut seg, (tgt_base - base) as usize, r.off, r.ty, r.add, sv, tgt_base, pc);
            }
        }
    }

    let entry = find_entry2(&sec_addrs, &syms);
    Ok(LoadedElf { entry, segments: vec![Segment { vaddr: base, data: seg, mem_size: total, _flags: 5 }] })
}

fn find_symtab_64(data: &[u8], secs: &[Section]) -> Option<(Vec<Symbol>, usize)> {
    for (_si, sec) in secs.iter().enumerate() {
        if sec.sh_type == SHT_SYMTAB {
            let n = sec.sh_size / 24;
            let mut syms = Vec::new();
            for j in 0..n {
                let off = (sec.sh_offset + j * 24) as usize;
                let st_name = r32(data, off);
                let st_shndx = r16(data, off + 6);
                let st_value = r64(data, off + 8);
                let name = if (sec.sh_link as usize) < secs.len() {
                    let strsec = &secs[sec.sh_link as usize];
                    rstr(data, strsec.sh_offset as usize + st_name as usize)
                } else { String::new() };
                syms.push(Symbol { st_value, st_shndx, name });
            }
            return Some((syms, sec.sh_link as usize));
        }
    }
    None
}

fn find_symtab_32(data: &[u8], secs: &[Section]) -> Option<(Vec<Symbol>, usize)> {
    for (_si, sec) in secs.iter().enumerate() {
        if sec.sh_type == SHT_SYMTAB {
            let n = sec.sh_size / 16;
            let mut syms = Vec::new();
            for j in 0..n {
                let off = (sec.sh_offset + j * 16) as usize;
                let st_name = r32(data, off);
                let st_value = r32(data, off + 4) as u64;
                let st_shndx = r16(data, off + 14);
                let name = if sec.sh_link > 0 {
                    let strsec = &secs[sec.sh_link as usize];
                    rstr(data, strsec.sh_offset as usize + st_name as usize)
                } else { String::new() };
                syms.push(Symbol { st_value, st_shndx, name });
            }
            return Some((syms, sec.sh_link as usize));
        }
    }
    None
}

fn rstr(data: &[u8], off: usize) -> String {
    let mut v = Vec::new();
    let mut i = off;
    while i < data.len() && data[i] != 0 { v.push(data[i]); i += 1; }
    String::from_utf8_lossy(&v).to_string()
}

fn find_entry2(sec_addrs: &[u64], syms: &[Symbol]) -> u64 {
    for sym in syms {
        if sym.name == "_start" && sym.st_shndx > 0 {
            let si = sym.st_shndx as usize;
            if si < sec_addrs.len() {
                return sec_addrs[si] + sym.st_value;
            }
        }
    }
    0x10000
}

fn do_reloc_64(seg: &mut [u8], base: usize, r_off: u64, r_ty: u32, r_add: i64, sym_val: u64, tgt_base: u64, pc: u64) {
    let addr = base + r_off as usize;
    match r_ty {
        R_RISCV_NONE => {}
        R_RISCV_64 => seg[addr..addr + 8].copy_from_slice(&sym_val.wrapping_add(r_add as u64).to_le_bytes()),
        R_RISCV_32 => seg[addr..addr + 4].copy_from_slice(&(sym_val.wrapping_add(r_add as u64) as u32).to_le_bytes()),
        R_RISCV_HI20 => {
            let v = (sym_val.wrapping_add(r_add as u64) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v & 0xfffff) << 12).to_le_bytes());
        }
        R_RISCV_LO12_I => {
            let v = (sym_val.wrapping_add(r_add as u64) & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x000fffff;
            seg[addr..addr + 4].copy_from_slice(&(insn | v << 20).to_le_bytes());
        }
        R_RISCV_LO12_S => {
            let v = (sym_val.wrapping_add(r_add as u64) & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x01fff07f;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v >> 5) << 25 | (v & 0x1f) << 7).to_le_bytes());
        }
        R_RISCV_PCREL_HI20 => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = ((diff + 0x800) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v & 0xfffff) << 12).to_le_bytes());
        }
        R_RISCV_PCREL_LO12_I => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = (diff & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x000fffff;
            seg[addr..addr + 4].copy_from_slice(&(insn | v << 20).to_le_bytes());
        }
        R_RISCV_PCREL_LO12_S => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = (diff & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x01fff07f;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v >> 5) << 25 | (v & 0x1f) << 7).to_le_bytes());
        }
        R_RISCV_JAL => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            encode_j(seg, addr, diff as i32);
        }
        R_RISCV_BRANCH => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            encode_b(seg, addr, diff as i32);
        }
        R_RISCV_CALL | R_RISCV_CALL_PLT => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let hi = ((diff + 0x800) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (hi & 0xfffff) << 12).to_le_bytes());
            if addr + 4 < seg.len() {
                let lo = (diff & 0xfff) as u32;
                let insn2 = r32(seg, addr + 4) & 0x000fffff;
                seg[addr + 4..addr + 8].copy_from_slice(&(insn2 | lo << 20).to_le_bytes());
            }
        }
        _ => eprintln!("rv4: unhandled relocation {}", r_ty),
    }
}

fn do_reloc_32(seg: &mut [u8], base: usize, r_off: u64, r_ty: u32, r_add: i64, sym_val: u64, tgt_base: u64, pc: u64) {
    let addr = base + r_off as usize;
    match r_ty {
        R_RISCV_NONE => {}
        R_RISCV_32 => seg[addr..addr + 4].copy_from_slice(&(sym_val.wrapping_add(r_add as u64) as u32).to_le_bytes()),
        R_RISCV_HI20 => {
            let v = (sym_val.wrapping_add(r_add as u64) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v & 0xfffff) << 12).to_le_bytes());
        }
        R_RISCV_LO12_I => {
            let v = (sym_val.wrapping_add(r_add as u64) & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x000fffff;
            seg[addr..addr + 4].copy_from_slice(&(insn | v << 20).to_le_bytes());
        }
        R_RISCV_LO12_S => {
            let v = (sym_val.wrapping_add(r_add as u64) & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x01fff07f;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v >> 5) << 25 | (v & 0x1f) << 7).to_le_bytes());
        }
        R_RISCV_PCREL_HI20 => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = ((diff + 0x800) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v & 0xfffff) << 12).to_le_bytes());
        }
        R_RISCV_PCREL_LO12_I => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = (diff & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x000fffff;
            seg[addr..addr + 4].copy_from_slice(&(insn | v << 20).to_le_bytes());
        }
        R_RISCV_PCREL_LO12_S => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let v = (diff & 0xfff) as u32;
            let insn = r32(seg, addr) & 0x01fff07f;
            seg[addr..addr + 4].copy_from_slice(&(insn | (v >> 5) << 25 | (v & 0x1f) << 7).to_le_bytes());
        }
        R_RISCV_JAL => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            encode_j(seg, addr, diff as i32);
        }
        R_RISCV_BRANCH => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            encode_b(seg, addr, diff as i32);
        }
        R_RISCV_CALL | R_RISCV_CALL_PLT => {
            let diff = (sym_val.wrapping_add(r_add as u64) as i64).wrapping_sub(pc as i64);
            let hi = ((diff + 0x800) >> 12) as u32;
            let insn = r32(seg, addr) & 0x00000fff;
            seg[addr..addr + 4].copy_from_slice(&(insn | (hi & 0xfffff) << 12).to_le_bytes());
            if addr + 4 < seg.len() {
                let lo = (diff & 0xfff) as u32;
                let insn2 = r32(seg, addr + 4) & 0x000fffff;
                seg[addr + 4..addr + 8].copy_from_slice(&(insn2 | lo << 20).to_le_bytes());
            }
        }
        _ => eprintln!("rv4: unhandled relocation {}", r_ty),
    }
}

fn encode_j(seg: &mut [u8], addr: usize, imm: i32) {
    let v = imm as u32;
    let encoded = ((v >> 20) & 1) << 31
                | ((v >> 1) & 0x3ff) << 21
                | ((v >> 11) & 1) << 20
                | ((v >> 12) & 0xff) << 12;
    let insn = r32(seg, addr) & 0x00000fff;
    seg[addr..addr + 4].copy_from_slice(&(insn | encoded).to_le_bytes());
}

fn encode_b(seg: &mut [u8], addr: usize, imm: i32) {
    let v = imm as u32;
    let encoded = ((v >> 12) & 1) << 31
                | ((v >> 5) & 0x3f) << 25
                | ((v >> 1) & 0xf) << 8
                | ((v >> 11) & 1) << 7;
    let insn = r32(seg, addr) & 0x01fff07f;
    seg[addr..addr + 4].copy_from_slice(&(insn | encoded).to_le_bytes());
}

pub fn apply_to_memory(loaded: &LoadedElf, mem: &mut Memory) -> Result<u64, String> {
    for seg in &loaded.segments {
        mem.write_bytes(seg.vaddr, &seg.data)?;
    }
    Ok(loaded.entry)
}
