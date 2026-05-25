fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

fn main() -> i32 {
    let n: i32 = 10;
    let result: i32 = fib(n);
    print_int(result);
    return 0;
}
