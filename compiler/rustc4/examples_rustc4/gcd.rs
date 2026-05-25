fn gcd(a: i32, b: i32) -> i32 {
    let mut x: i32 = a;
    let mut y: i32 = b;
    while y != 0 {
        let t: i32 = y;
        y = x % y;
        x = t;
    }
    return x;
}

fn main() -> i32 {
    let result: i32 = gcd(48, 18);
    print_int(result);
    return 0;
}
