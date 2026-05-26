console.log('Testing try/catch/throw...');

try {
    console.log('Inside try block');
    throw 'An error occurred!';
    console.log('This should not print');
} catch (err) {
    console.log('Caught:', err);
}

try {
    let x = 10;
    if (x > 5) {
        throw 'x is too large';
    }
} catch (e) {
    console.log('Second catch:', e);
}

console.log('After all try/catch blocks');