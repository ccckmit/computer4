pub fn mod_pow(base: u64, exp: u64, mod_base: u64) -> u64 {
    if mod_base == 1 {
        return 0;
    }
    let mut result = 1u64;
    let mut base = base % mod_base;
    let mut exp = exp;
    while exp > 0 {
        if exp % 2 == 1 {
            result = result * base % mod_base;
        }
        exp /= 2;
        base = base * base % mod_base;
    }
    result
}

pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        return (a.abs(), a.signum(), 0);
    }
    let (g, x1, y1) = extended_gcd(b, a % b);
    let x = y1;
    let y = x1 - (a / b) * y1;
    (g, x, y)
}

pub fn mod_inv(a: i64, m: i64) -> Option<i64> {
    let (g, x, _) = extended_gcd(a, m);
    if g != 1 {
        None
    } else {
        Some(((x % m) + m) % m)
    }
}

pub fn mod_add(a: i64, b: i64, m: i64) -> i64 {
    ((a % m) + (b % m) + m) % m
}

pub fn mod_sub(a: i64, b: i64, m: i64) -> i64 {
    ((a % m) - (b % m) + m) % m
}

pub fn mod_mul(a: i64, b: i64, m: i64) -> i64 {
    ((a % m) * (b % m)) % m
}

pub fn chinese_remainder(remainders: &[i64], moduli: &[i64]) -> Option<i64> {
    let n = remainders.len();
    if n != moduli.len() {
        return None;
    }
    let mut result = 0i64;
    let mut prod = 1i64;
    for &m in moduli {
        prod *= m;
    }
    for i in 0..n {
        let p = prod / moduli[i];
        if let Some(inv) = mod_inv(p % moduli[i], moduli[i]) {
            result = (result + remainders[i] * p as i64 * inv as i64) % prod;
        } else {
            return None;
        }
    }
    Some((result + prod) % prod)
}

pub fn is_coprime(a: u64, b: u64) -> bool {
    let (g, _, _) = extended_gcd(a as i64, b as i64);
    g == 1
}

pub fn euler_totient(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let factors = super::primes::prime_factors(n);
    let mut result = n as f64;
    for (p, _) in factors {
        result = result * (1.0 - 1.0 / p as f64);
    }
    result as u64
}

pub fn is_primitive_root(g: u64, p: u64) -> bool {
    if !super::primes::is_prime(p) {
        return false;
    }
    let phi = p - 1;
    let factors = super::primes::prime_factors(phi);
    for (q, _) in factors {
        if mod_pow(g, phi / q, p) == 1 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mod_pow() {
        assert_eq!(mod_pow(2, 10, 1000), 24);
        assert_eq!(mod_pow(3, 5, 7), 5);
        assert_eq!(mod_pow(5, 0, 13), 1);
    }

    #[test]
    fn test_extended_gcd() {
        assert_eq!(extended_gcd(35, 15), (5, 1, -2));
        assert_eq!(extended_gcd(12, 8), (4, 1, -1));
    }

    #[test]
    fn test_mod_inv() {
        assert_eq!(mod_inv(3, 11), Some(4));
        assert_eq!(mod_inv(2, 5), Some(3));
        assert_eq!(mod_inv(2, 4), None);
    }

    #[test]
    fn test_chinese_remainder() {
        let remainders = [2, 3, 1];
        let moduli = [3, 4, 5];
        assert_eq!(chinese_remainder(&remainders, &moduli), Some(11));
    }

    #[test]
    fn test_is_coprime() {
        assert!(is_coprime(7, 11));
        assert!(is_coprime(8, 15));
        assert!(!is_coprime(8, 12));
    }

    #[test]
    fn test_euler_totient() {
        assert_eq!(euler_totient(1), 1);
        assert_eq!(euler_totient(7), 6);
        assert_eq!(euler_totient(10), 4);
    }

    #[test]
    fn test_is_primitive_root() {
        assert!(is_primitive_root(3, 7));
        assert!(!is_primitive_root(2, 7));
    }
}