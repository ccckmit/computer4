pub mod complex;
pub mod polynomial;
pub mod rational;
pub mod roots;

pub use complex::Complex;
pub use polynomial::{horner, Polynomial};
pub use rational::{parse_rational, Rational};
pub use roots::{
    bisection, deflation, find_all_roots, horner as horner_eval, horner_derivative, newton, secant,
};