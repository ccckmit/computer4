use crate::adder::Adder4;
use crate::signal::*;
use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

static BOX_T: &str = "\
╔═══════════════════════════════════════════════════╗";
static BOX_B: &str = "\
╚═══════════════════════════════════════════════════╝";

fn bit_char(l: Level) -> char {
    match l {
        Level::H => '1',
        Level::L => '0',
        Level::X => 'X',
        Level::Z => 'Z',
    }
}

fn bits_str(bits: &[Level], active: Option<usize>) -> String {
    bits.iter()
        .enumerate()
        .map(|(i, &l)| {
            let c = bit_char(l);
            if Some(i) == active {
                format!("\x1b[1;33m{}\x1b[0m", c)
            } else {
                format!("\x1b[{}m{}\x1b[0m", if l == Level::H { 32 } else { 2 }, c)
            }
        })
        .collect()
}

#[allow(dead_code)]
fn arrow_str(from: Level, to: Level) -> String {
    let active = from == Level::H || to == Level::H;
    let c = if active { "\x1b[36m→\x1b[0m" } else { "→" };
    c.to_string()
}

fn fa_block_label(i: usize, active: bool) -> String {
    let label = format!("FA{}", i);
    if active {
        format!("\x1b[1;33m{:^4}\x1b[0m", label)
    } else {
        format!("{:^4}", label)
    }
}

fn line(parts: &[&str]) -> String {
    let mut s = String::from("║ ");
    s.push_str(&parts.join(""));
    let pad = 45usize.saturating_sub(parts.iter().map(|p| visible_len(p)).sum::<usize>());
    if pad > 0 {
        for _ in 0..pad {
            s.push(' ');
        }
    }
    s.push_str(" ║");
    s
}

fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for &b in s.as_bytes() {
        if in_escape {
            if b == b'm' {
                in_escape = false;
            }
            continue;
        }
        if b == 0x1b {
            in_escape = true;
            continue;
        }
        if b & 0xC0 != 0x80 {
            len += 1;
        }
    }
    len
}

fn header() -> String {
    let mut s = String::new();
    s.push_str(BOX_T);
    s.push('\n');
    s.push_str(&line(&["    4-bit Ripple-Carry Adder Viz"]));
    s.push('\n');
    s.push_str("╠═══════════════════════════════════════════════════╣\n");
    s
}

fn footer(status: &str) -> String {
    let mut s = String::new();
    s.push_str("╠═══════════════════════════════════════════════════╣\n");
    s.push_str(&line(&[&format!("  {}", status)]));
    s.push('\n');
    s.push_str(BOX_B);
    s
}

struct FaState {
    sum: Level,
    cout: Level,
}

fn compute_fa(a: Level, b: Level, cin: Level) -> (Level, Level) {
    let xor_ab = a.xor(b);
    let s = xor_ab.xor(cin);
    let cout = a.and(b).or(xor_ab.and(cin));
    (s, cout)
}

pub fn animate_adder4(a_val: u16, b_val: u16, cin_val: bool, delay_ms: u64) {
    let a_wires = bus("va", 4);
    let b_wires = bus("vb", 4);
    let sum_wires = bus("vsum", 4);
    let cin_wire = wire("vcin");
    let cout_wire = wire("vcout");

    let mut adder = Adder4::new(
        a_wires.clone(),
        b_wires.clone(),
        cin_wire.clone(),
        sum_wires.clone(),
        cout_wire.clone(),
    );

    u16_to_bus(&a_wires, a_val);
    u16_to_bus(&b_wires, b_val);
    set(&cin_wire, Level::from_bool(cin_val));

    let a_bits: Vec<Level> = a_wires.iter().map(get).collect();
    let b_bits: Vec<Level> = b_wires.iter().map(get).collect();
    let cin = get(&cin_wire);

    let mut fa: [FaState; 4] = [
        FaState { sum: Level::X, cout: Level::X },
        FaState { sum: Level::X, cout: Level::X },
        FaState { sum: Level::X, cout: Level::X },
        FaState { sum: Level::X, cout: Level::X },
    ];

    let total_steps = 5;

    for step in 0..=total_steps {
        let active_fa = if step == 0 {
            None
        } else if step <= 4 {
            Some(step - 1)
        } else {
            None
        };

        if let Some(i) = active_fa {
            let ci = if i == 0 {
                cin
            } else {
                fa[i - 1].cout
            };
            let (s, co) = compute_fa(a_bits[i], b_bits[i], ci);
            fa[i].sum = s;
            fa[i].cout = co;
        }

        let mut frame = String::new();
        frame.push_str(&header());

        let a_str = bits_str(&a_bits, None);
        let b_str = bits_str(&b_bits, None);
        frame.push_str(&line(&[
            &format!("  a = {}  ({})", a_str, a_val),
        ]));
        frame.push('\n');
        frame.push_str(&line(&[
            &format!("  b = {}  ({})", b_str, b_val),
        ]));
        frame.push('\n');
        frame.push_str(&line(&[
            &format!(
                "+ cin = {}",
                if cin == Level::H { "\x1b[1;32m1\x1b[0m" } else { "0" }
            ),
        ]));
        frame.push('\n');
        frame.push_str(&line(&["  ─────────────────────────"]));
        frame.push('\n');

        let mut fline = String::from("        ");
        for i in (0..4).rev() {
            fline.push_str(&fa_block_label(i, active_fa == Some(i)));
            if i > 0 {
                fline.push_str("  ");
            }
        }
        frame.push_str(&line(&[&fline]));
        frame.push('\n');

        let mut cin_line = String::from("  cin:  ");
        for i in (0..4).rev() {
            let ci = if i == 0 {
                cin
            } else {
                fa[i - 1].cout
            };
            let ci_char = bit_char(ci);
            cin_line.push(ci_char);
            if i > 0 {
                cin_line.push_str(" ← ");
            }
        }
        frame.push_str(&line(&[&cin_line]));
        frame.push('\n');

        let mut sum_line = String::from("  sum:  ");
        for i in (0..4).rev() {
            let c = bit_char(fa[i].sum);
            let active = active_fa == Some(i);
            if active {
                sum_line.push_str(&format!("\x1b[1;33m{}\x1b[0m", c));
            } else if fa[i].sum == Level::H {
                sum_line.push_str(&format!("\x1b[32m{}\x1b[0m", c));
            } else {
                sum_line.push(c);
            }
            if i > 0 {
                sum_line.push_str("   ");
            }
        }
        frame.push_str(&line(&[&sum_line]));
        frame.push('\n');

        let mut cout_line = String::from("  cout: ");
        for i in (0..4).rev() {
            let co_char = bit_char(fa[i].cout);
            cout_line.push(co_char);
            if i > 0 {
                cout_line.push_str(" → ");
            }
        }
        frame.push_str(&line(&[&cout_line]));
        frame.push('\n');
        frame.push_str(&line(&["  ─────────────────────────"]));
        frame.push('\n');

        let s_val = (0..4).fold(0u16, |acc, i| {
            if fa[i].sum == Level::H {
                acc | (1 << i)
            } else {
                acc
            }
        });
        let co_val = fa[3].cout;

        let mut result_line = format!("  result = ");
        for i in (0..4).rev() {
            let c = bit_char(fa[i].sum);
            if fa[i].sum == Level::H {
                result_line.push_str(&format!("\x1b[32m{}\x1b[0m", c));
            } else {
                result_line.push(c);
            }
        }
        result_line.push_str(&format!(
            " ({})  cout = {}",
            s_val,
            if co_val == Level::H {
                "\x1b[1;32m1\x1b[0m"
            } else {
                "0"
            },
        ));
        frame.push_str(&line(&[&result_line]));
        frame.push('\n');

        let status = if step == 0 {
            "Initial state — press enter to start".to_string()
        } else if step <= 4 {
            let i = step - 1;
            let carry_out = if i == 0 {
                cin
            } else {
                fa[i - 1].cout
            };
            format!(
                "  ▶ FA{}: {} ⊕ {} ⊕ {} = {}  carry-out = {} {}",
                i,
                bit_char(a_bits[i]),
                bit_char(b_bits[i]),
                bit_char(carry_out),
                bit_char(fa[i].sum),
                bit_char(fa[i].cout),
                if i < 3 {
                    format!("→ FA{}", i + 1)
                } else {
                    "done".to_string()
                },
            )
        } else {
            let expected = a_val as u32 + b_val as u32 + cin_val as u32;
            let actual = s_val as u32 + if co_val == Level::H { 16 } else { 0 };
            if actual == expected {
                format!(
                    "  ✓ {} + {} + {} = {} (carry-out: {})",
                    a_val,
                    b_val,
                    cin_val as u8,
                    s_val,
                    co_val,
                )
            } else {
                format!(
                    "  ✗ {} + {} + {} = {} (carry-out: {})  expected {}",
                    a_val,
                    b_val,
                    cin_val as u8,
                    s_val,
                    co_val,
                    expected,
                )
            }
        };
        frame.push_str(&footer(&status));

        print!("\x1b[2J\x1b[H{}", frame);
        io::stdout().flush().unwrap();

        sleep(Duration::from_millis(delay_ms));
    }

    adder.eval();

    let actual_sum = bus_to_u16(&sum_wires);
    let actual_cout = get(&cout_wire);
    let (_, final_co) = compute_fa(a_bits[3], b_bits[3],
        if Level::H == Level::H { Level::H } else { Level::L });
    let _ = final_co;

    let mut frame = String::new();
    frame.push_str(&header());
    let a_str = bits_str(&a_bits, None);
    let b_str = bits_str(&b_bits, None);
    frame.push_str(&line(&[&format!("  a = {}  ({})", a_str, a_val)]));
    frame.push('\n');
    frame.push_str(&line(&[&format!("  b = {}  ({})", b_str, b_val)]));
    frame.push('\n');
    frame.push_str(&line(&[
        &format!("+ cin = {}", if cin == Level::H { "\x1b[1;32m1\x1b[0m" } else { "0" }),
    ]));
    frame.push('\n');
    frame.push_str(&line(&["  ─────────────────────────"]));
    frame.push('\n');

    let mut sum_line = String::from("  sum:  ");
    let sum_bits: Vec<Level> = sum_wires.iter().map(get).collect();
    for i in (0..4).rev() {
        let c = bit_char(sum_bits[i]);
        if sum_bits[i] == Level::H {
            sum_line.push_str(&format!("\x1b[32m{}\x1b[0m", c));
        } else {
            sum_line.push(c);
        }
        if i > 0 {
            sum_line.push_str("   ");
        }
    }
    frame.push_str(&line(&[&sum_line]));
    frame.push('\n');

    let co_char = bit_char(actual_cout);
    frame.push_str(&line(&[&format!(
        "  cout: {}",
        if actual_cout == Level::H {
            "\x1b[1;32m1\x1b[0m"
        } else {
            "0"
        },
    )]));
    frame.push('\n');
    frame.push_str(&line(&["  ─────────────────────────"]));
    frame.push('\n');

    let expected = a_val as u32 + b_val as u32 + cin_val as u32;
    let actual_val = actual_sum as u32 + if actual_cout == Level::H { 16 } else { 0 };
    let ok = actual_val == expected;
    let pass_str = if ok { "✓ PASS" } else { "✗ FAIL" };
    let status = format!(
        "  {}  {} + {} + {} = {} (carry-out: {}, expected: {})",
        pass_str,
        a_val,
        b_val,
        cin_val as u8,
        actual_sum,
        co_char,
        expected,
    );
    frame.push_str(&footer(&status));

    print!("\x1b[2J\x1b[H{}", frame);
    io::stdout().flush().unwrap();
}

pub fn demo_adder4() {
    let cases = [
        (0, 0, false, 1200),
        (3, 1, false, 1200),
        (5, 3, false, 1200),
        (11, 4, false, 1200),
        (15, 1, false, 1200),
    ];

    for (i, &(a, b, cin, delay)) in cases.iter().enumerate() {
        println!("\x1b[2J\x1b[H");
        println!("\n╔═══════════════════════════════════════════════════╗");
        println!("║  Demo Case {} of {}                              ║", i + 1, cases.len());
        println!("╚═══════════════════════════════════════════════════╝\n");
        sleep(Duration::from_millis(800));
        animate_adder4(a, b, cin, delay);
        sleep(Duration::from_millis(1500));
    }
}
