pub mod primes;
pub mod combinatorics;
pub mod modular;

pub use primes::{is_prime, primes_up_to, prime_factors, lcm, gcd};
pub use combinatorics::{factorial, fibonacci, binomial, fibonacci_sequence, is_fibonacci};
pub use modular::{mod_pow, mod_inv, extended_gcd, mod_add, mod_sub, mod_mul, chinese_remainder, is_coprime, euler_totient, is_primitive_root};