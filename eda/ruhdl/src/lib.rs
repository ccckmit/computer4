pub mod signal;
pub mod gate;
pub mod adder;
pub mod seq;
pub mod mux;
pub mod sim;
pub mod cpu;
pub mod viz;

pub mod prelude {
    pub use crate::adder::{Adder4, Adder8, FullAdder, HalfAdder, RippleAdder};
    pub use crate::cpu::{program_5factorial, CPU};
    pub use crate::gate::{And, Nand, Nor, Not, Or, Xor};
    pub use crate::mux::{Decoder2x4, Mux2, Mux4};
    pub use crate::seq::{Counter, DFF, Register};
    pub use crate::signal::{
        bits_to_u64, bus, bus_to_u16, get, get_bus, set, set_bus, u16_to_bus, val_to_bits, wire,
        Level, WireRef,
    };
    pub use crate::sim::Sim;
    pub use crate::viz::{animate_adder4, demo_adder4};
}
