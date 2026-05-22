/// 8×8 整数逆离散余弦变换（这里使用最直观的浮点实现，仍然只依赖 std）。
/// `input` 为 64 个量化后系数（行主序），`output` 为 64 个像素（0..255）。
pub fn idct_8x8(input: &[i16; 64], output: &mut [u8; 64]) {
    const C: f64 = std::f64::consts::PI / 16.0;
    for y in 0..8 {
        for x in 0..8 {
            let mut sum = 0.0;
            for v in 0..8 {
                for u in 0..8 {
                    let cu = if u == 0 { (1.0 / 2.0_f64).sqrt() } else { 1.0 };
                    let cv = if v == 0 { (1.0 / 2.0_f64).sqrt() } else { 1.0 };
                    let coeff = input[v * 8 + u] as f64;
                    sum += cu * cv * coeff
                        * ((2.0 * x as f64 + 1.0) * u as f64 * C).cos()
                        * ((2.0 * y as f64 + 1.0) * v as f64 * C).cos();
                }
            }
            let val = (sum / 4.0).round() as i32 + 128;
            output[y * 8 + x] = val.clamp(0, 255) as u8;
        }
    }
}
