fn add(x: i32, y: i32) -> i32 {
    return x + y;
}

fn main() -> i32 {
    let a: i32 = 3;
    let b: i32 = 4;
    let c: i32 = add(a, b);
    print_int(c);
    return 0;
}
