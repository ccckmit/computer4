use crate::gate::{And, Or, Xor};
use crate::signal::{wire, WireRef};

#[derive(Debug, Clone)]
pub struct HalfAdder {
    pub a: WireRef,
    pub b: WireRef,
    pub sum: WireRef,
    pub carry: WireRef,
    xor: Xor,
    and: And,
}

impl HalfAdder {
    pub fn new(a: WireRef, b: WireRef, sum: WireRef, carry: WireRef) -> Self {
        HalfAdder {
            xor: Xor::new(a.clone(), b.clone(), sum.clone()),
            and: And::new(a.clone(), b.clone(), carry.clone()),
            a, b, sum, carry,
        }
    }

    pub fn eval(&mut self) {
        self.xor.eval();
        self.and.eval();
    }
}

#[derive(Debug, Clone)]
pub struct FullAdder {
    pub a: WireRef,
    pub b: WireRef,
    pub cin: WireRef,
    pub sum: WireRef,
    pub cout: WireRef,
    xor1: Xor,
    xor2: Xor,
    and1: And,
    and2: And,
    or: Or,
    #[allow(dead_code)]
    s: WireRef,
    #[allow(dead_code)]
    c1: WireRef,
    #[allow(dead_code)]
    c2: WireRef,
}

impl FullAdder {
    pub fn new(a: WireRef, b: WireRef, cin: WireRef, sum: WireRef, cout: WireRef) -> Self {
        let s = wire("fa_s");
        let c1 = wire("fa_c1");
        let c2 = wire("fa_c2");
        FullAdder {
            xor1: Xor::new(a.clone(), b.clone(), s.clone()),
            xor2: Xor::new(s.clone(), cin.clone(), sum.clone()),
            and1: And::new(a.clone(), b.clone(), c1.clone()),
            and2: And::new(s.clone(), cin.clone(), c2.clone()),
            or: Or::new(c1.clone(), c2.clone(), cout.clone()),
            a, b, cin, sum, cout, s, c1, c2,
        }
    }

    pub fn eval(&mut self) {
        self.xor1.eval();
        self.xor2.eval();
        self.and1.eval();
        self.and2.eval();
        self.or.eval();
    }
}

#[derive(Debug, Clone)]
pub struct Adder4 {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub cin: WireRef,
    pub sum: Vec<WireRef>,
    pub cout: WireRef,
    pub fa0: FullAdder,
    pub fa1: FullAdder,
    pub fa2: FullAdder,
    pub fa3: FullAdder,
}

impl Adder4 {
    pub fn new(
        a: Vec<WireRef>, b: Vec<WireRef>, cin: WireRef,
        sum: Vec<WireRef>, cout: WireRef,
    ) -> Self {
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 4);
        assert_eq!(sum.len(), 4);
        let c0 = wire("adder4_c0");
        let c1 = wire("adder4_c1");
        let c2 = wire("adder4_c2");
        Adder4 {
            fa0: FullAdder::new(a[0].clone(), b[0].clone(), cin.clone(), sum[0].clone(), c0.clone()),
            fa1: FullAdder::new(a[1].clone(), b[1].clone(), c0.clone(), sum[1].clone(), c1.clone()),
            fa2: FullAdder::new(a[2].clone(), b[2].clone(), c1.clone(), sum[2].clone(), c2.clone()),
            fa3: FullAdder::new(a[3].clone(), b[3].clone(), c2.clone(), sum[3].clone(), cout.clone()),
            a, b, cin, sum, cout,
        }
    }

    pub fn eval(&mut self) {
        self.fa0.eval();
        self.fa1.eval();
        self.fa2.eval();
        self.fa3.eval();
    }
}

#[derive(Debug, Clone)]
pub struct Adder8 {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub cin: WireRef,
    pub sum: Vec<WireRef>,
    pub cout: WireRef,
    adder_low: Adder4,
    adder_high: Adder4,
    #[allow(dead_code)]
    c4: WireRef,
}

impl Adder8 {
    pub fn new(
        a: Vec<WireRef>, b: Vec<WireRef>, cin: WireRef,
        sum: Vec<WireRef>, cout: WireRef,
    ) -> Self {
        assert_eq!(a.len(), 8);
        assert_eq!(b.len(), 8);
        assert_eq!(sum.len(), 8);
        let c4 = wire("adder8_c4");
        Adder8 {
            adder_low: Adder4::new(
                a[0..4].to_vec(), b[0..4].to_vec(), cin.clone(),
                sum[0..4].to_vec(), c4.clone(),
            ),
            adder_high: Adder4::new(
                a[4..8].to_vec(), b[4..8].to_vec(), c4.clone(),
                sum[4..8].to_vec(), cout.clone(),
            ),
            a, b, cin, sum, cout, c4,
        }
    }

    pub fn eval(&mut self) {
        self.adder_low.eval();
        self.adder_high.eval();
    }
}

#[derive(Debug, Clone)]
pub struct RippleAdder {
    pub a: Vec<WireRef>,
    pub b: Vec<WireRef>,
    pub cin: WireRef,
    pub sum: Vec<WireRef>,
    pub cout: WireRef,
    fas: Vec<FullAdder>,
    #[allow(dead_code)]
    carries: Vec<WireRef>,
}

impl RippleAdder {
    pub fn new(
        a: Vec<WireRef>, b: Vec<WireRef>, cin: WireRef,
        sum: Vec<WireRef>, cout: WireRef,
    ) -> Self {
        let width = a.len();
        assert_eq!(b.len(), width);
        assert_eq!(sum.len(), width);

        let mut carries = Vec::with_capacity(width);
        for i in 0..width {
            carries.push(wire(&format!("ripple_c{}", i)));
        }

        let mut fas = Vec::with_capacity(width);
        for i in 0..width {
            let c_in = if i == 0 { cin.clone() } else { carries[i - 1].clone() };
            let c_out = if i == width - 1 { cout.clone() } else { carries[i].clone() };
            fas.push(FullAdder::new(
                a[i].clone(), b[i].clone(), c_in,
                sum[i].clone(), c_out,
            ));
        }

        RippleAdder { a, b, cin, sum, cout, fas, carries }
    }

    pub fn eval(&mut self) {
        for fa in &mut self.fas {
            fa.eval();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::{bus, u16_to_bus, bus_to_u16, get, set, Level};

    #[test]
    fn test_half_adder() {
        let a = wire("a"); let b = wire("b");
        let s = wire("s"); let c = wire("c");
        let mut ha = HalfAdder::new(a.clone(), b.clone(), s.clone(), c.clone());

        for &av in &[Level::L, Level::H] {
            for &bv in &[Level::L, Level::H] {
                set(&a, av); set(&b, bv); ha.eval();
                let expected_sum = av.xor(bv);
                let expected_carry = av.and(bv);
                assert_eq!(get(&s), expected_sum, "HA sum {:?}+{:?}", av, bv);
                assert_eq!(get(&c), expected_carry, "HA carry {:?}+{:?}", av, bv);
            }
        }
    }

    #[test]
    fn test_full_adder() {
        let a = wire("a"); let b = wire("b"); let cin = wire("cin");
        let s = wire("s"); let cout = wire("cout");
        let mut fa = FullAdder::new(a.clone(), b.clone(), cin.clone(), s.clone(), cout.clone());

        for av in [Level::L, Level::H] {
            for bv in [Level::L, Level::H] {
                for cv in [Level::L, Level::H] {
                    set(&a, av); set(&b, bv); set(&cin, cv);
                    fa.eval();
                    let expected_sum = av.xor(bv).xor(cv);
                    let expected_cout =
                        (av.and(bv)).or(av.xor(bv).and(cv));
                    assert_eq!(get(&s), expected_sum, "FA sum {:?}+{:?}+{:?}", av, bv, cv);
                    assert_eq!(get(&cout), expected_cout, "FA cout {:?}+{:?}+{:?}", av, bv, cv);
                }
            }
        }
    }

    #[test]
    fn test_adder4() {
        let a = bus("a", 4); let b = bus("b", 4);
        let sum = bus("sum", 4); let cin = wire("cin"); let cout = wire("cout");
        let mut adder = Adder4::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone());

        for x in 0u16..16 {
            for y in 0u16..16 {
                u16_to_bus(&a, x); u16_to_bus(&b, y); set(&cin, Level::L);
                adder.eval();
                let result = bus_to_u16(&sum);
                let expected = (x + y) & 0xF;
                assert_eq!(result, expected, "0x{:X} + 0x{:X} = 0x{:X} (expected 0x{:X})", x, y, result, expected);
                let expected_cout = (x + y) > 0xF;
                assert_eq!(get(&cout), Level::from_bool(expected_cout));
            }
        }
    }

    #[test]
    fn test_adder8() {
        let a = bus("a", 8); let b = bus("b", 8);
        let sum = bus("sum", 8); let cin = wire("cin"); let cout = wire("cout");
        let mut adder = Adder8::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone());

        u16_to_bus(&a, 0x34); u16_to_bus(&b, 0x56); set(&cin, Level::L);
        adder.eval();
        assert_eq!(bus_to_u16(&sum), 0x8A);
        assert_eq!(get(&cout), Level::L);

        u16_to_bus(&a, 0xFF); u16_to_bus(&b, 0x01); set(&cin, Level::L);
        adder.eval();
        assert_eq!(bus_to_u16(&sum), 0x00);
        assert_eq!(get(&cout), Level::H);
    }

    #[test]
    fn test_ripple_adder() {
        let a = bus("ra", 8); let b = bus("rb", 8);
        let sum = bus("rsum", 8); let cin = wire("rcin"); let cout = wire("rcout");
        let mut adder = RippleAdder::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone());

        u16_to_bus(&a, 42); u16_to_bus(&b, 27); set(&cin, Level::L);
        adder.eval();
        assert_eq!(bus_to_u16(&sum), 69);
        assert_eq!(get(&cout), Level::L);
    }
}
