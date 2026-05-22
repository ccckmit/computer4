use crate::frame::YuvFrame;

/// 简单的整数双线性运动补偿（只针对 Y 分量）。
/// `ref_frame` 为参考帧，`dst_y` 为当前帧的 Y 分量缓冲区（长度 = w*h）。
/// `mb_x`、`mb_y` 为宏块左上角的宏块坐标（以 16 像素为步长）；
/// `mv_x`、`mv_y` 为运动向量（单位像素），`full_pel` 表示是否全像素（步幅 2）。
pub fn motion_compensate(
    ref_frame: &YuvFrame,
    dst_y: &mut [u8],
    mb_x: usize,
    mb_y: usize,
    mv_x: i16,
    mv_y: i16,
    full_pel: bool,
) {
    let step = if full_pel { 2 } else { 1 };
    let src_x0 = (mb_x as isize * 16) + (mv_x as isize) * step as isize;
    let src_y0 = (mb_y as isize * 16) + (mv_y as isize) * step as isize;

    for dy in 0..16 {
        for dx in 0..16 {
            let sx = (src_x0 + dx as isize).max(0).min((ref_frame.width - 1) as isize) as usize;
            let sy = (src_y0 + dy as isize).max(0).min((ref_frame.height - 1) as isize) as usize;
            let dst_idx = (mb_y * 16 + dy) * ref_frame.width + (mb_x * 16 + dx);
            dst_y[dst_idx] = ref_frame.y[sy * ref_frame.width + sx];
        }
    }
}
