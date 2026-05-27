use crate::signal::{get, set, Level, WireRef};

#[derive(Debug, Clone)]
pub struct Mux2 {
    pub a: WireRef,
    pub b: WireRef,
    pub sel: WireRef,
    pub y: WireRef,
}

impl Mux2 {
    pub fn new(a: WireRef, b: WireRef, sel: WireRef, y: WireRef) -> Self {
        Mux2 { a, b, sel, y }
    }

    pub fn eval(&mut self) {
        let result = match get(&self.sel) {
            Level::L => get(&self.a),
            Level::H => get(&self.b),
            _ => Level::X,
        };
        if get(&self.y) != result {
            set(&self.y, result);
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mux4 {
    pub inputs: Vec<Vec<WireRef>>,
    pub sel: Vec<WireRef>,
    pub y: Vec<WireRef>,
}

impl Mux4 {
    pub fn new(inputs: Vec<Vec<WireRef>>, sel: Vec<WireRef>, y: Vec<WireRef>) -> Self {
        assert_eq!(inputs.len(), 4);
        assert_eq!(sel.len(), 2);
        for input in &inputs {
            assert_eq!(input.len(), y.len());
        }
        Mux4 { inputs, sel, y }
    }

    pub fn eval(&mut self) {
        let sel0 = get(&self.sel[0]);
        let sel1 = get(&self.sel[1]);

        if sel0 == Level::X || sel1 == Level::X || sel0 == Level::Z || sel1 == Level::Z {
            for w in &self.y {
                if get(w) != Level::X {
                    set(w, Level::X);
                }
            }
            return;
        }

        let idx = (if sel1 == Level::H { 2u32 } else { 0 })
                + (if sel0 == Level::H { 1 } else { 0 });

        for (i, y_wire) in self.y.iter().enumerate() {
            let v = get(&self.inputs[idx as usize][i]);
            if get(y_wire) != v {
                set(y_wire, v);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Decoder2x4 {
    pub en: WireRef,
    pub a: Vec<WireRef>,
    pub y: Vec<WireRef>,
}

impl Decoder2x4 {
    pub fn new(en: WireRef, a: Vec<WireRef>, y: Vec<WireRef>) -> Self {
        assert_eq!(a.len(), 2);
        assert_eq!(y.len(), 4);
        Decoder2x4 { en, a, y }
    }

    pub fn eval(&mut self) {
        if get(&self.en) == Level::L {
            for w in &self.y {
                if get(w) != Level::L { set(w, Level::L); }
            }
            return;
        }
        let a0 = get(&self.a[0]);
        let a1 = get(&self.a[1]);
        if a0 == Level::X || a1 == Level::X {
            for w in &self.y {
                if get(w) != Level::X { set(w, Level::X); }
            }
            return;
        }
        let idx = (if a1 == Level::H { 2usize } else { 0 })
                + (if a0 == Level::H { 1 } else { 0 });
        for (i, w) in self.y.iter().enumerate() {
            let v = if i == idx { Level::H } else { Level::L };
            if get(w) != v { set(w, v); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::{bus, wire};

    #[test]
    fn test_mux2() {
        let a = wire("a"); let b = wire("b");
        let sel = wire("sel"); let y = wire("y");
        let mut mux = Mux2::new(a.clone(), b.clone(), sel.clone(), y.clone());

        set(&a, Level::L); set(&b, Level::H);
        set(&sel, Level::L); mux.eval();
        assert_eq!(get(&y), Level::L);

        set(&sel, Level::H); mux.eval();
        assert_eq!(get(&y), Level::H);
    }

    #[test]
    fn test_mux4() {
        let inputs = vec![bus("i0", 4), bus("i1", 4), bus("i2", 4), bus("i3", 4)];
        let sel = bus("sel", 2);
        let y = bus("my", 4);
        let mut mux = Mux4::new(inputs.clone(), sel.clone(), y.clone());

        crate::signal::u16_to_bus(&inputs[0], 0xA);
        crate::signal::u16_to_bus(&inputs[1], 0xB);
        crate::signal::u16_to_bus(&inputs[2], 0xC);
        crate::signal::u16_to_bus(&inputs[3], 0xD);

        set(&sel[0], Level::L); set(&sel[1], Level::L); mux.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 0xA);

        set(&sel[0], Level::H); set(&sel[1], Level::L); mux.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 0xB);

        set(&sel[0], Level::L); set(&sel[1], Level::H); mux.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 0xC);

        set(&sel[0], Level::H); set(&sel[1], Level::H); mux.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 0xD);
    }

    #[test]
    fn test_decoder2x4() {
        let en = wire("en");
        let a = bus("da", 2);
        let y = bus("dy", 4);
        let mut dec = Decoder2x4::new(en.clone(), a.clone(), y.clone());

        set(&en, Level::H);
        set(&a[0], Level::L); set(&a[1], Level::L); dec.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 1);

        set(&a[0], Level::H); set(&a[1], Level::L); dec.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 2);

        set(&a[0], Level::L); set(&a[1], Level::H); dec.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 4);

        set(&a[0], Level::H); set(&a[1], Level::H); dec.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 8);

        set(&en, Level::L); dec.eval();
        assert_eq!(crate::signal::bus_to_u16(&y), 0);
    }
}
