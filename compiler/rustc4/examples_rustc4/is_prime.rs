fn is_prime(n: i32) -> i32 {
    if n < 2 {
        return 0;
    }
    let mut i: i32 = 2;
    while i * i <= n {
        if n % i == 0 {
            return 0;
        }
        i = i + 1;
    }
    return 1;
}

fn main() -> i32 {
    print_int(is_prime(17));
    print_int(is_prime(4));
    print_int(is_prime(2));
    return 0;
}
