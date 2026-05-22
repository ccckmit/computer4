pub fn factorial(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    let mut result = 1u64;
    for i in 2..=n {
        result *= i;
    }
    result
}

pub fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let mut a = 0u64;
    let mut b = 1u64;
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    let k = k.min(n - k);
    let mut result = 1u64;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

pub fn fibonacci_sequence(n: usize) -> Vec<u64> {
    (0..n).map(|i| fibonacci(i as u64)).collect()
}

pub fn is_fibonacci(n: u64) -> bool {
    if n > u64::MAX / 5 {
        return false;
    }
    let sq = n * n;
    if sq > u64::MAX / 5 {
        return false;
    }
    let a = 5 * sq + 4;
    let b = (5 * sq).saturating_sub(4);
    is_perfect_square(a) || is_perfect_square(b)
}

fn is_perfect_square(n: u64) -> bool {
    if n < 2 {
        return true;
    }
    let mut x = (n as f64).sqrt() as u64;
    while x > 0 && x * x < n {
        x += 1;
    }
    while x > 0 && x * x > n {
        x -= 1;
    }
    x > 0 && x * x == n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(10), 55);
        assert_eq!(fibonacci(20), 6765);
    }

    #[test]
    fn test_binomial() {
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(10, 3), 120);
        assert_eq!(binomial(6, 0), 1);
        assert_eq!(binomial(6, 6), 1);
        assert_eq!(binomial(5, 10), 0);
    }

    #[test]
    fn test_fibonacci_sequence() {
        assert_eq!(fibonacci_sequence(7), vec![0, 1, 1, 2, 3, 5, 8]);
    }

    #[test]
    fn test_is_fibonacci() {
        assert!(is_fibonacci(0));
        assert!(is_fibonacci(1));
        assert!(is_fibonacci(5));
        assert!(is_fibonacci(55));
        assert!(!is_fibonacci(4));
        assert!(!is_fibonacci(6));
    }
}