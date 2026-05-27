use ruhdl::cpu::{program_5factorial, CPU};
use ruhdl::sim::Sim;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("=== ruhdl 16-bit CPU — 5! Demo ===");

    let mut sim = Sim::new();
    let clk = sim.get_clock();
    let cpu = Rc::new(RefCell::new(CPU::new(program_5factorial(), clk)));
    let cpu2 = cpu.clone();
    sim.add_seq(move || cpu2.borrow_mut().tick());

    println!("Executing 5! = 5 × 4 × 3 × 2 × 1\n");

    for cycle in 0..50 {
        sim.tick();
        let c = cpu.borrow();
        if c.halted {
            print_state(cycle, &c);
            println!("                  ^ HALT");
            break;
        }
        if cycle < 14 {
            print_state(cycle, &c);
        }
    }

    let result = cpu.borrow().reg(1);
    println!("\nResult: 5! = {}", result);
    assert_eq!(result, 120);
    println!("✓ Correct!");
}

fn print_state(cycle: u64, cpu: &CPU) {
    println!(
        "[{:2}] R0={:3}  R1={:4}  R2={}  halted={}",
        cycle,
        cpu.reg(0),
        cpu.reg(1),
        cpu.reg(2),
        cpu.halted,
    );
}
