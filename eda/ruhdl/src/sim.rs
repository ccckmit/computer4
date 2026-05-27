use crate::signal::{get, set, wire, Level, WireRef};
use std::cell::RefCell;
use std::rc::Rc;

type EvalFn = Rc<RefCell<dyn FnMut()>>;

pub struct Sim {
    comb: Vec<EvalFn>,
    seq: Vec<EvalFn>,
    pub clk: WireRef,
    time: u64,
}

impl Sim {
    pub fn new() -> Self {
        Sim {
            comb: Vec::new(),
            seq: Vec::new(),
            clk: wire("clk"),
            time: 0,
        }
    }

    pub fn add_comb<F: FnMut() + 'static>(&mut self, mut f: F) {
        self.comb.push(Rc::new(RefCell::new(move || f())));
    }

    pub fn add_seq<F: FnMut() + 'static>(&mut self, mut f: F) {
        self.seq.push(Rc::new(RefCell::new(move || f())));
    }

    pub fn eval(&mut self) {
        for _ in 0..10 {
            let mut changed = false;
            for f in &self.comb {
                let orig = get(&self.clk);
                f.borrow_mut()();
                if get(&self.clk) != orig {
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }

    pub fn posedge(&mut self) {
        set(&self.clk, Level::H);
        for f in &self.seq {
            f.borrow_mut()();
        }
        self.eval();
    }

    pub fn negedge(&mut self) {
        set(&self.clk, Level::L);
        for f in &self.seq {
            f.borrow_mut()();
        }
    }

    pub fn tick(&mut self) {
        self.posedge();
        self.negedge();
        self.time += 1;
    }

    pub fn run(&mut self, cycles: u64) {
        for _ in 0..cycles {
            self.tick();
        }
    }

    pub fn reset(&mut self) {
        set(&self.clk, Level::L);
        self.time = 0;
    }

    pub fn time(&self) -> u64 {
        self.time
    }

    pub fn get_clock(&self) -> WireRef {
        self.clk.clone()
    }
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adder::Adder4;
    use crate::gate::And;
    use crate::seq::Counter;
    use crate::signal::{bus, u16_to_bus, bus_to_u16};

    #[test]
    fn test_sim_comb() {
        let mut sim = Sim::new();
        let a = wire("a"); let b = wire("b"); let y = wire("y");
        let mut gate = And::new(a.clone(), b.clone(), y.clone());
        sim.add_comb(move || gate.eval());

        set(&a, Level::H); set(&b, Level::H);
        sim.eval();
        assert_eq!(get(&y), Level::H);
    }

    #[test]
    fn test_sim_counter() {
        let mut sim = Sim::new();
        let q = bus("q", 4);
        let mut ctr = Counter::new(q.clone(), sim.get_clock());
        sim.add_seq(move || ctr.eval());

        sim.run(5);
        assert_eq!(bus_to_u16(&q), 5);
    }

    #[test]
    fn test_sim_adder4() {
        let mut sim = Sim::new();
        let a = bus("a", 4); let b = bus("b", 4);
        let sum = bus("sum", 4); let cin = wire("cin"); let cout = wire("cout");
        let mut adder = Adder4::new(a.clone(), b.clone(), cin.clone(), sum.clone(), cout.clone());
        sim.add_comb(move || adder.eval());

        u16_to_bus(&a, 3); u16_to_bus(&b, 4); set(&cin, Level::L);
        sim.eval();
        assert_eq!(bus_to_u16(&sum), 7);
    }
}
