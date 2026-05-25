use crate::signal::{get, set, Level, WireRef};

#[derive(Debug, Clone)]
pub struct DFF {
    pub d: WireRef,
    pub q: WireRef,
    pub clk: WireRef,
    state: Level,
    prev_clk: Level,
}

impl DFF {
    pub fn new(d: WireRef, q: WireRef, clk: WireRef) -> Self {
        DFF { d, q, clk, state: Level::L, prev_clk: Level::L }
    }

    pub fn eval(&mut self) {
        let clk_val = get(&self.clk);
        if self.prev_clk == Level::L && clk_val == Level::H {
            self.state = get(&self.d);
        }
        self.prev_clk = clk_val;
        if get(&self.q) != self.state {
            set(&self.q, self.state);
        }
    }

    pub fn set_state(&mut self, val: Level) {
        self.state = val;
    }

    pub fn state(&self) -> Level {
        self.state
    }
}

#[derive(Debug, Clone)]
pub struct Register {
    pub d: Vec<WireRef>,
    pub q: Vec<WireRef>,
    pub clk: WireRef,
    pub load: Option<WireRef>,
    state: Vec<Level>,
    prev_clk: Level,
}

impl Register {
    pub fn new(d: Vec<WireRef>, q: Vec<WireRef>, clk: WireRef) -> Self {
        assert_eq!(d.len(), q.len());
        Register {
            state: vec![Level::L; d.len()],
            d, q, clk, load: None, prev_clk: Level::L,
        }
    }

    pub fn with_load(
        d: Vec<WireRef>, q: Vec<WireRef>, clk: WireRef, load: WireRef,
    ) -> Self {
        assert_eq!(d.len(), q.len());
        Register {
            state: vec![Level::L; d.len()],
            d, q, clk, load: Some(load), prev_clk: Level::L,
        }
    }

    pub fn eval(&mut self) {
        let clk_val = get(&self.clk);
        if self.prev_clk == Level::L && clk_val == Level::H {
            let do_load = self.load.as_ref().map_or(true, |l| get(l) == Level::H);
            if do_load {
                for (i, d_wire) in self.d.iter().enumerate() {
                    self.state[i] = get(d_wire);
                }
            }
        }
        self.prev_clk = clk_val;
        for (i, q_wire) in self.q.iter().enumerate() {
            if get(q_wire) != self.state[i] {
                set(q_wire, self.state[i]);
            }
        }
    }

    pub fn set_state(&mut self, vals: &[Level]) {
        for (i, &v) in vals.iter().enumerate() {
            if i < self.state.len() {
                self.state[i] = v;
            }
        }
    }

    pub fn state(&self) -> &[Level] {
        &self.state
    }
}

#[derive(Debug, Clone)]
pub struct Counter {
    pub q: Vec<WireRef>,
    pub clk: WireRef,
    pub en: Option<WireRef>,
    count: u64,
    width: usize,
    prev_clk: Level,
}

impl Counter {
    pub fn new(q: Vec<WireRef>, clk: WireRef) -> Self {
        let width = q.len();
        Counter { q, clk, en: None, count: 0, width, prev_clk: Level::L }
    }

    pub fn with_enable(q: Vec<WireRef>, clk: WireRef, en: WireRef) -> Self {
        let width = q.len();
        Counter { q, clk, en: Some(en), count: 0, width, prev_clk: Level::L }
    }

    pub fn eval(&mut self) {
        let clk_val = get(&self.clk);
        if self.prev_clk == Level::L && clk_val == Level::H {
            let do_count = self.en.as_ref().map_or(true, |e| get(e) == Level::H);
            if do_count {
                self.count = self.count.wrapping_add(1);
            }
        }
        self.prev_clk = clk_val;
        for (i, q_wire) in self.q.iter().enumerate() {
            let bit = if i < self.width {
                if (self.count >> i) & 1 == 1 { Level::H } else { Level::L }
            } else {
                Level::L
            };
            if get(q_wire) != bit {
                set(q_wire, bit);
            }
        }
    }

    pub fn set_count(&mut self, val: u64) {
        self.count = val;
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::{bus, u16_to_bus, bus_to_u16, wire};

    #[test]
    fn test_dff() {
        let d = wire("d"); let q = wire("q"); let clk = wire("clk");
        let mut dff = DFF::new(d.clone(), q.clone(), clk.clone());

        set(&d, Level::H);
        set(&clk, Level::L); dff.eval();
        assert_eq!(get(&q), Level::L);

        set(&clk, Level::H); dff.eval();
        assert_eq!(get(&q), Level::H);

        set(&d, Level::L);
        set(&clk, Level::L); dff.eval();
        assert_eq!(get(&q), Level::H);

        set(&clk, Level::H); dff.eval();
        assert_eq!(get(&q), Level::L);
    }

    #[test]
    fn test_register() {
        let d = bus("rd", 4); let q = bus("rq", 4); let clk = wire("rclk");
        let mut reg = Register::new(d.clone(), q.clone(), clk.clone());

        u16_to_bus(&d, 0xA);
        set(&clk, Level::L); reg.eval();
        assert_eq!(bus_to_u16(&q), 0);

        set(&clk, Level::H); reg.eval();
        assert_eq!(bus_to_u16(&q), 0xA);

        u16_to_bus(&d, 0x5);
        set(&clk, Level::L); reg.eval();
        assert_eq!(bus_to_u16(&q), 0xA);

        set(&clk, Level::H); reg.eval();
        assert_eq!(bus_to_u16(&q), 0x5);
    }

    #[test]
    fn test_register_with_load() {
        let d = bus("rld", 4); let q = bus("rlq", 4);
        let clk = wire("rlclk"); let load = wire("load");
        let mut reg = Register::with_load(d.clone(), q.clone(), clk.clone(), load.clone());

        u16_to_bus(&d, 0xA);
        set(&load, Level::L);
        set(&clk, Level::L); reg.eval();
        set(&clk, Level::H); reg.eval();
        assert_eq!(bus_to_u16(&q), 0);

        set(&load, Level::H);
        set(&clk, Level::L); reg.eval();
        set(&clk, Level::H); reg.eval();
        assert_eq!(bus_to_u16(&q), 0xA);

        u16_to_bus(&d, 0x5);
        set(&load, Level::L);
        set(&clk, Level::L); reg.eval();
        set(&clk, Level::H); reg.eval();
        assert_eq!(bus_to_u16(&q), 0xA);
    }

    #[test]
    fn test_counter() {
        let q = bus("cq", 4); let clk = wire("cclk");
        let mut ctr = Counter::new(q.clone(), clk.clone());

        for expected in [0u64, 1, 2, 3, 4] {
            assert_eq!(ctr.count(), expected);
            assert_eq!(bus_to_u16(&q), expected as u16);
            set(&clk, Level::L); ctr.eval();
            set(&clk, Level::H); ctr.eval();
        }
    }

    #[test]
    fn test_counter_wraparound() {
        let q = bus("cqw", 2); let clk = wire("cwclk");
        let mut ctr = Counter::new(q.clone(), clk.clone());
        ctr.set_count(3);

        ctr.eval();
        assert_eq!(bus_to_u16(&q), 3);
        set(&clk, Level::L); ctr.eval();
        set(&clk, Level::H); ctr.eval();
        assert_eq!(bus_to_u16(&q), 0);
    }
}
