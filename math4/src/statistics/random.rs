static mut SEED: u64 = 1;
static mut INITIAL_SEED: u64 = 1;

pub fn set_seed(seed: u64) {
    unsafe {
        SEED = seed;
        INITIAL_SEED = seed;
    }
}

pub fn get_seed() -> u64 {
    unsafe { INITIAL_SEED }
}

pub fn reset_seed() {
    unsafe {
        SEED = INITIAL_SEED;
    }
}

pub fn random() -> f64 {
    unsafe {
        SEED = SEED.wrapping_mul(1664525).wrapping_add(1013904223);
        (SEED & 0xFFFFFFFF) as f64 / 4294967296.0_f64
    }
}

pub fn rand_int(min: i64, max: i64) -> i64 {
    (random() * (max - min + 1) as f64) as i64 + min
}

pub fn mix(a: u64, b: u64) -> u64 {
    ((a ^ b).wrapping_mul(1664525)).wrapping_add(1013904223)
}

pub fn string_to_seeds(s: &str) -> Vec<u64> {
    s.bytes()
        .enumerate()
        .map(|(i, b)| (b as u64 ^ ((i as u64).wrapping_mul(1664525))))
        .collect()
}

pub fn set_seed_string(s: &str) {
    let seeds = string_to_seeds(s);
    let mut s = if !seeds.is_empty() {
        seeds[0]
    } else {
        1
    };
    for seed in seeds.iter().skip(1) {
        s = mix(s, *seed);
    }
    set_seed(s);
}

pub fn random_batch(n: usize) -> Vec<f64> {
    (0..n).map(|_| random()).collect()
}

pub fn rand_int_batch(n: usize, min: i64, max: i64) -> Vec<i64> {
    (0..n).map(|_| rand_int(min, max)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random() {
        set_seed(42);
        let r1 = random();
        let r2 = random();
        assert!(r1 >= 0.0 && r1 < 1.0);
        assert!(r2 >= 0.0 && r2 < 1.0);
        assert!(r1 != r2);
    }

    #[test]
    fn test_rand_int() {
        set_seed(42);
        let v = rand_int(1, 6);
        assert!(v >= 1 && v <= 6);
    }

    #[test]
    fn test_set_seed() {
        set_seed(123);
        assert_eq!(get_seed(), 123);
    }

    #[test]
    fn test_reset_seed() {
        set_seed(100);
        random();
        random();
        reset_seed();
        let after = random();
        set_seed(100);
        let before = random();
        assert_eq!(after, before);
    }
}