pub mod memory;
pub mod elf;
pub mod vm;

use memory::Memory;
use elf::LoadedElf;

pub fn run_elf(data: &[u8]) -> Result<i32, String> {
    let loaded = elf::load(data)?;
    let mut mem = Memory::new();
    let entry = elf::apply_to_memory(&loaded, &mut mem)?;

    eprintln!("rv4: entry={:#x}", entry);
    let mut vm = vm::Vm::new();
    vm.set_pc(entry);
    vm.set_rv32(detect_rv32(&loaded));

    vm.run(&mut mem)?;
    Ok(vm.exit_code())
}

fn detect_rv32(loaded: &LoadedElf) -> bool {
    for seg in &loaded.segments {
        for i in (0..seg.data.len()).step_by(4).take(100) {
            if i + 4 <= seg.data.len() {
                let inst = u32::from_le_bytes([
                    seg.data[i], seg.data[i + 1], seg.data[i + 2], seg.data[i + 3]
                ]);
                if inst & 0x7f == 0x1b || inst & 0x7f == 0x3b {
                    return false;
                }
            }
        }
    }
    true
}
