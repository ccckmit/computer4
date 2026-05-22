pub mod derivative;
pub mod integral;
pub mod multivariable;
pub mod sequence;
pub mod taylor;
pub mod optimize;

pub use derivative::{derivative, second_derivative, partial, gradient, jacobian, directional_derivative};
pub use integral::{trapezoid, simpson, romberg, adaptive, monte_carlo, gauss_legendre};
pub use multivariable::{grad, divergence, curl_2d, curl_3d, laplacian, hessian};
pub use sequence::{sequence, series, converge, limit, infinite_series, alternating_series, ConvergeResult, SeriesResult};
pub use taylor::{taylor, maclaurin, series_sum, power_series_coeffs};
pub use optimize::{golden_section, gradient_descent, newton_method, conjugate_gradient, momentum_gradient_descent, OptimizeResult, NewtonResult};