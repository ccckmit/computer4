# math4rs

Rust math library. API mirrors `math4js`. Version 0.1.0, edition 2021.

## Build & Test

```sh
cargo build
cargo test              # all tests
cargo test stats        # single module
cargo test --lib        # unit tests only (in-module #[cfg(test)])
```

## Architecture

```
src/
├── lib.rs              # library root, re-exports all submodules
├── algebra/            # polynomial, complex, rational, roots
├── calculus/           # derivative, integral, multivariable, sequence, taylor, optimize
├── statistics/         # descriptive stats, distributions, random, interval, hypothesis, theorem, info
├── ndarray/            # NumPy-style arrays using ndarray crate
├── plot/               # R-style plotting using plotters
├── linear_algebra/     # matrix/vector operations
├── geometry/           # point, vector, line, plane, circle, sphere, polygon, distance, transform
└── calculus/           # (see above)
tests/                  # integration tests (currently empty)
_doc/                   # version specs (v0.1-v0.7.md)
```

## Conventions

- All math functions take `&[f64]` slices, return `f64` or `Vec<f64>`
- NaN returned for invalid input (empty slice, divide by zero)
- Tests use `#[cfg(test)]` in-module (not in `tests/` directory)
- `#![allow(dead_code, unused)]` in lib.rs suppresses warnings for planned stubs
- API naming matches R/JavaScript conventions (e.g., `dnorm`, `pnorm`, `qnorm`, `rnorm`)
- Linear algebra Matrix/Vector are separate from ndarray types (Matrix is `Vec<Vec<f64>>`, NdArray is `ndarray::ArrayD<f64>`)
- Polynomial coefficients are in ascending order: `[a0, a1, a2]` means `a0 + a1*x + a2*x^2`

## Dependencies

- `statrs` - statistical distributions
- `rand` - random number generation
- `plotters` + `image` - plotting
- `ndarray` - NumPy-style arrays

## Key Differences from math4js

- Use `::` instead of `.` for module access (Rust vs JS)
- No default exports; use `use math4rs::{function}` explicitly
- Plotting requires opening a device first: `png("/tmp/out.png", w, h)` before plotting, `dev_off()` after
- Distributions use `lower_tail: bool` parameter (not `lower_tail: true` defaults in R)
- `dnorm(x, mean, sd)` returns PDF; `pnorm(q, mean, sd, true)` returns CDF

## Version Status

- v0.1 - statistics (COMPLETED)
- v0.2 - plot (COMPLETED)
- v0.3 - ndarray (COMPLETED)
- v0.4 - linear_algebra (COMPLETED)
- v0.5 - algebra (COMPLETED)
- v0.6 - calculus (COMPLETED)
- v0.7 - geometry (COMPLETED) — 190 tests
- v0.8 - optimization (COMPLETED) — covered by calculus/optimize.rs
- v0.9 - number_theory (COMPLETED) — 207 tests total