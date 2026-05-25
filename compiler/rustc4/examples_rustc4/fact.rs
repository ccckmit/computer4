fn main() -> i32 {
    let n: i32 = 5;
    let mut i: i32 = 1;
    let mut result: i32 = 1;
    while i <= n {
        result = result * i;
        i = i + 1;
    }
    print_int(result);
    return 0;
}
