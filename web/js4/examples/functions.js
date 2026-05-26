function factorial(n) {
    if (n == 0) {
        return 1;
    }
    return n * factorial(n - 1);
}

let sum = 0;
let i = 1;
while (i <= 10) {
    sum = sum + i;
    i = i + 1;
}
console.log('Sum of 1..10:', sum);
console.log('Factorial of 5:', factorial(5));

let nums = [3, 1, 4, 1, 5, 9, 2, 6];
console.log('Array:', nums);