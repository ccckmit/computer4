function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log('Fibonacci sequence:');
let i = 0;
while (i < 10) {
    console.log('fib(' + i + ') =', fibonacci(i));
    i = i + 1;
}