use ruhdl::adder::Adder4;
use ruhdl::prelude::*;

fn main() {
    println!("=== 4-bit Ripple-Carry Adder (Adder4) ===\n");

    let a = bus("a", 4);
    let b = bus("b", 4);
    let sum = bus("sum", 4);
    let cin = wire("cin");
    let cout = wire("cout");

    let mut adder = Adder4::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone());

    let cases = [
        (0, 0, 0),
        (5, 3, 0),
        (8, 7, 0),
        (15, 1, 0),
        (0, 0, 1),
        (5, 0, 1),
        (15, 0, 1),
    ];

    for &(x, y, c) in &cases {
        u16_to_bus(&a, x);
        u16_to_bus(&b, y);
        set(&cin, Level::from_bool(c != 0));
        adder.eval();

        let s = bus_to_u16(&sum);
        let co = get(&cout);
        let full = x as u32 + y as u32 + c as u32;
        let expected_sum = (full & 0xF) as u16;
        let expected_co = full > 0xF;

        print_case(x, y, c, s, co, expected_sum, expected_co);
    }
}

fn print_case(x: u16, y: u16, cin: u16, s: u16, co: Level, es: u16, ec: bool) {
    let co_ok = (co == Level::H) == ec;
    let sum_ok = s == es;
    let ok = co_ok && sum_ok;

    print!(
        "  {:04b} + {:04b} + {} = {:04b}  carry-out: {}",
        x, y, cin, s, co,
    );
    if x as u32 + y as u32 + cin as u32 > 0xF {
        print!(" (overflow)");
    }
    if ok {
        println!("  ✓");
    } else {
        println!("  ✗ (expected sum={}, co={})", es, ec as u8);
    }
}
