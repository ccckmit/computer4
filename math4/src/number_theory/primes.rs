pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 {
            return false;
        }
        i += 2;
    }
    true
}

pub fn primes_up_to(n: u64) -> Vec<u64> {
    if n < 2 {
        return vec![];
    }
    let mut sieve = vec![true; (n + 1) as usize];
    sieve[0] = false;
    sieve[1] = false;
    let mut p = 2;
    while p * p <= n {
        if sieve[p as usize] {
            let mut i = p * p;
            while i <= n {
                sieve[i as usize] = false;
                i += p;
            }
        }
        p += 1;
    }
    sieve.iter().enumerate().filter(|(_, &v)| v).map(|(i, _)| i as u64).collect()
}

pub fn prime_factors(mut n: u64) -> Vec<(u64, u64)> {
    let mut factors = Vec::new();
    let mut count = 0;
    while n % 2 == 0 {
        n /= 2;
        count += 1;
    }
    if count > 0 {
        factors.push((2, count));
    }
    let mut p = 3;
    while p * p <= n {
        count = 0;
        while n % p == 0 {
            n /= p;
            count += 1;
        }
        if count > 0 {
            factors.push((p, count));
        }
        p += 2;
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

pub fn gcd(a: u64, b: u64) -> u64 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

pub fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        (a / gcd(a, b)) * b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(!is_prime(1));
        assert!(!is_prime(4));
        assert!(!is_prime(6));
        assert!(!is_prime(9));
        assert!(is_prime(29));
    }

    #[test]
    fn test_primes_up_to() {
        let primes = primes_up_to(10);
        assert_eq!(primes, vec![2, 3, 5, 7]);
    }

    #[test]
    fn test_prime_factors() {
        assert_eq!(prime_factors(12), vec![(2, 2), (3, 1)]);
        assert_eq!(prime_factors(60), vec![(2, 2), (3, 1), (5, 1)]);
        assert_eq!(prime_factors(7), vec![(7, 1)]);
    }

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(12, 8), 4);
        assert_eq!(gcd(17, 13), 1);
        assert_eq!(gcd(100, 25), 25);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(5, 7), 35);
        assert_eq!(lcm(0, 5), 0);
    }
}