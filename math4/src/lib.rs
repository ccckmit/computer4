#![allow(dead_code, unused)]

pub mod algebra;
pub mod calculus;
pub mod statistics;
pub mod plot;
pub mod ndarray;
pub mod linear_algebra;
pub mod geometry;
pub mod number_theory;

pub use algebra::{
    Complex, Polynomial, Rational,
    horner, horner_eval,
    bisection, newton, secant, deflation, find_all_roots, horner_derivative,
    parse_rational,
};
pub use calculus::{
    derivative, second_derivative, partial, gradient, jacobian, directional_derivative,
    trapezoid, simpson, romberg, adaptive, monte_carlo, gauss_legendre,
    grad, divergence, curl_2d, curl_3d, laplacian, hessian,
    sequence, series, converge, limit, infinite_series, alternating_series,
    taylor, maclaurin, series_sum, power_series_coeffs,
    golden_section, gradient_descent, newton_method, conjugate_gradient, momentum_gradient_descent,
    sequence::ConvergeResult, sequence::SeriesResult, optimize::OptimizeResult, optimize::NewtonResult,
};
pub use statistics::*;
pub use plot::*;
pub use ndarray::{
    zeros, ones, full, eye, identity, diag,
    arange, linspace, array, from_flat,
    concatenate, vstack, hstack,
    add, sub, mul, div, matmul, dot,
    abs, sqrt, exp, log, sin, cos, tan,
    sum, mean, std, var_, min, max,
    argmin, argmax, prod,
    gt, gte, lt, lte, eq, ne,
    where_ as r#where,
    rand, randn, randint, choice, shuffle,
    Array as NdArray, Matrix as NdMatrix, Vector as NdVector,
};
pub use linear_algebra::{
    Matrix, Vector,
    norm_vector, dot_product, cross_product, normalize,
};
pub use geometry::{
    Point2D, Point3D, Vec3,
    Line2D, Plane, Circle, Arc, Sphere,
    Polygon, triangle_area, quadrilateral_area,
    point_to_point_2d, point_to_point_3d, point_to_line_2d, line_to_line_2d,
    Transform2D, Transform3D,
};
pub use number_theory::{
    is_prime, primes_up_to, prime_factors,
    factorial, fibonacci, binomial, fibonacci_sequence, is_fibonacci,
    mod_pow, mod_inv, extended_gcd, euler_totient, is_primitive_root,
    gcd, lcm,
};