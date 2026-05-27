// ============================================================
//  grad_check.rs  –  Comprehensive numerical gradient verification
//
//  For every tested operation we:
//    1. Compute analytic grad via .backward()
//    2. Compute numerical grad via central difference  f(x+h) - f(x-h) / 2h
//    3. Report relative error; FAIL if rel_err > RTOL
// ============================================================
use llm4::tensor::{cat, check_op, GradReport, SimpleRng, Tensor};

const H: f32 = 1e-4;
const RTOL: f32 = 1e-3;

fn main() {
    let mut report = GradReport::new();
    let mut rng = SimpleRng::new(7);

    println!("\n══ 1. Element-wise operations ══");

    let data3 = vec![-1.5, 0.3, 2.0, -0.7, 1.1, 0.0];
    let shape3 = [2, 3];

    check_op(&mut report, "relu", &shape3, &data3,
        |x| x.relu().sum_all(), &[0, 1, 2, 4]);

    check_op(&mut report, "tanh", &shape3, &data3,
        |x| x.tanh().sum_all(), &[0, 1, 3, 5]);

    check_op(&mut report, "silu", &shape3, &data3,
        |x| x.silu().sum_all(), &[0, 2, 4]);

    check_op(&mut report, "pow(3)", &shape3, &data3,
        |x| x.pow(3.0).sum_all(), &[0, 2, 4]);

    check_op(&mut report, "mul_scalar(2.5)", &shape3, &data3,
        |x| x.mul_scalar(2.5).sum_all(), &[0, 3]);

    check_op(&mut report, "add_scalar(-1)", &shape3, &data3,
        |x| x.add_scalar(-1.0).sum_all(), &[1, 4]);

    check_op(&mut report, "neg", &shape3, &data3,
        |x| x.neg().sum_all(), &[0, 5]);

    check_op(&mut report, "exp", &shape3, &data3,
        |x| x.exp().sum_all(), &[0, 2, 4]);

    println!("\n══ 2. Reduction & softmax ══");

    let data4 = vec![1.0, -0.5, 0.3, 2.0, 0.1, -1.2];
    let shape4 = [2, 3];

    check_op(&mut report, "sum_all", &shape4, &data4,
        |x| x.sum_all(), &[0, 2, 5]);

    check_op(&mut report, "mean_all", &shape4, &data4,
        |x| x.mean_all(), &[1, 3]);

    check_op(&mut report, "softmax(axis=1) weighted", &shape4, &data4,
        |x| {
            let weights = Tensor::from_slice(&[1.0f32, 2.0, 3.0, 0.5, 1.5, 2.5], &[2, 3], false);
            x.softmax(1).mul(&weights).sum_all()
        }, &[0, 1, 2, 3, 4, 5]);

    let data_attn: Vec<f32> = (0..24).map(|i| i as f32 * 0.1 - 1.2).collect();
    let attn_weights = Tensor::from_slice(
        &(0..24).map(|i| i as f32 * 0.05).collect::<Vec<_>>(), &[2, 2, 2, 3], false);
    check_op(&mut report, "softmax 4D [2,2,2,3] axis=3", &[2, 2, 2, 3], &data_attn,
        move |x| x.softmax(3).mul(&attn_weights).sum_all(), &[0, 5, 11, 23]);

    println!("\n══ 3. matmul ══");

    let a_data: Vec<f32> = (0..6).map(|i| (i as f32 - 2.5) * 0.5).collect();
    let b_data: Vec<f32> = vec![0.1, -0.2, 0.3, 0.4, -0.5, 0.6, 0.7, -0.8, 0.9, 1.0, -1.1, 1.2];
    let b_t = Tensor::from_slice(&b_data, &[3, 4], false);

    check_op(&mut report, "matmul [2,3] x [3,4] grad-A", &[2, 3], &a_data,
        |x| x.matmul(&b_t).sum_all(), &[0, 2, 5]);

    let a_t = Tensor::from_slice(&a_data, &[2, 3], false);
    check_op(&mut report, "matmul [2,3] x [3,4] grad-B", &[3, 4], &b_data,
        |x| a_t.matmul(x).sum_all(), &[0, 4, 11]);

    let ba: Vec<f32> = (0..12).map(|i| i as f32 * 0.1).collect();
    let bb: Vec<f32> = (0..12).map(|i| (11 - i) as f32 * 0.1).collect();
    let bb_t = Tensor::from_slice(&bb, &[2, 3, 2], false);
    check_op(&mut report, "batched matmul [2,2,3] grad", &[2, 2, 3], &ba,
        |x| x.matmul(&bb_t).sum_all(), &[0, 5, 11]);

    println!("\n══ 4. transpose ══");

    let td: Vec<f32> = (0..24).map(|i| i as f32 * 0.5 - 6.0).collect();
    check_op(&mut report, "transpose [2,3,4] (0,1)", &[2, 3, 4], &td,
        |x| x.transpose(0, 1).sum_all(), &[0, 7, 23]);
    check_op(&mut report, "transpose [2,3,4] (1,2)", &[2, 3, 4], &td,
        |x| x.transpose(1, 2).sum_all(), &[1, 8, 22]);
    check_op(&mut report, "transpose [2,3,4] (0,2)", &[2, 3, 4], &td,
        |x| x.transpose(0, 2).sum_all(), &[3, 15]);

    println!("\n══ 5. reshape ══");

    check_op(&mut report, "reshape [2,3]->[6,1]", &[2, 3], &data3,
        |x| x.reshape(vec![6, 1]).sum_all(), &[0, 4]);

    println!("\n══ 6. cat ══");

    let c1: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let c2: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
    let t2 = Tensor::from_slice(&c2, &[2, 2], false);
    check_op(&mut report, "cat axis=0, grad of t1", &[2, 2], &c1,
        |x| cat(&[x.clone(), t2.clone()], 0).sum_all(), &[0, 3]);

    let t1f = Tensor::from_slice(&c1, &[2, 2], false);
    check_op(&mut report, "cat axis=1, grad of t2", &[2, 2], &c2,
        |x| cat(&[t1f.clone(), x.clone()], 1).sum_all(), &[0, 3]);

    println!("\n══ 7. rms_norm ══");

    let rn_data: Vec<f32> = (0..12).map(|i| (i as f32 - 5.5) * 0.7).collect();
    check_op(&mut report, "rms_norm [3,4]", &[3, 4], &rn_data,
        |x| x.rms_norm().sum_all(), &[0, 3, 7, 11]);

    check_op(&mut report, "rms_norm [2,2,3]", &[2, 2, 3], &rn_data[..12].to_vec(),
        |x| x.rms_norm().sum_all(), &[0, 5, 11]);

    println!("\n══ 8. cross_entropy ══");

    let ce_data: Vec<f32> = vec![1.0, 2.0, 0.5, -1.0, 0.5, 3.0];
    check_op(&mut report, "cross_entropy [2,3] targets=[1,2]", &[2, 3], &ce_data,
        |x| x.cross_entropy(&[1, 2]), &[0, 1, 2, 3, 4, 5]);

    println!("\n══ 9. Linear layer ══");

    let lin_w: Vec<f32> = (0..12).map(|i| (i as f32 - 5.5) * 0.08).collect();
    let x_data: Vec<f32> = (0..8).map(|i| (i as f32 - 3.5) * 0.3).collect();
    let w_t = Tensor::from_slice(&lin_w, &[3, 4], false);

    check_op(&mut report, "Linear forward grad-x [2,4]", &[2, 4], &x_data,
        move |x| {
            let wt = w_t.transpose(0, 1);
            x.matmul(&wt).sum_all()
        }, &[0, 3, 7]);

    let x_t = Tensor::from_slice(&x_data, &[2, 4], false);
    check_op(&mut report, "Linear forward grad-W [3,4]", &[3, 4], &lin_w,
        move |w| {
            let wt = w.transpose(0, 1);
            x_t.matmul(&wt).sum_all()
        }, &[0, 5, 11]);

    println!("\n══ 10. Chained computation ══");

    let chain_data: Vec<f32> = vec![0.5, -1.0, 2.0, 0.1];
    check_op(&mut report, "chain: relu->pow(2)->mean", &[2, 2], &chain_data,
        |x| x.relu().pow(2.0).mean_all(), &[0, 1, 2, 3]);

    check_op(&mut report, "chain: tanh->mul_scalar->sum", &[2, 2], &chain_data,
        |x| x.tanh().mul_scalar(3.0).sum_all(), &[0, 2]);

    let mat_data: Vec<f32> = vec![1.0, -0.5, 0.3, -0.8];
    let other_t = Tensor::from_slice(&[0.2f32, -0.3, 0.5, 0.1], &[2, 2], false);
    check_op(&mut report, "chain: matmul->relu->sum", &[2, 2], &mat_data,
        |x| x.matmul(&other_t).relu().sum_all(), &[0, 1, 2, 3]);

    println!("\n══ 11. Attention sub-components ══");

    let b = 1usize;
    let nh = 2usize;
    let t = 3usize;
    let hd = 4usize;
    let q_data: Vec<f32> = (0..b * nh * t * hd).map(|i| (i as f32 - 12.0) * 0.1).collect();
    let k_data: Vec<f32> = (0..b * nh * t * hd).map(|i| (12.0 - i as f32) * 0.1).collect();
    let v_data: Vec<f32> = (0..b * nh * t * hd).map(|i| i as f32 * 0.05).collect();
    let scale = 1.0 / (hd as f32).sqrt();

    let k_t = Tensor::from_slice(&k_data, &[b, nh, t, hd], false);
    let v_t = Tensor::from_slice(&v_data, &[b, nh, t, hd], false);
    check_op(&mut report, "attn Q grad", &[b, nh, t, hd], &q_data,
        move |q| {
            let kt = k_t.transpose(2, 3);
            q.matmul(&kt).mul_scalar(scale).softmax(3).matmul(&v_t).sum_all()
        }, &[0, 5, 11, 23]);

    let q_t2 = Tensor::from_slice(&q_data, &[b, nh, t, hd], false);
    let v_t2 = Tensor::from_slice(&v_data, &[b, nh, t, hd], false);
    check_op(&mut report, "attn K grad", &[b, nh, t, hd], &k_data,
        move |k| {
            let kt = k.transpose(2, 3);
            q_t2.matmul(&kt).mul_scalar(scale).softmax(3).matmul(&v_t2).sum_all()
        }, &[0, 5, 23]);

    let q_t3 = Tensor::from_slice(&q_data, &[b, nh, t, hd], false);
    let k_t3 = Tensor::from_slice(&k_data, &[b, nh, t, hd], false);
    check_op(&mut report, "attn V grad", &[b, nh, t, hd], &v_data,
        move |v| {
            let kt = k_t3.transpose(2, 3);
            q_t3.matmul(&kt).mul_scalar(scale).softmax(3).matmul(v).sum_all()
        }, &[0, 8, 23]);

    println!("\n══ 12. broadcast operations ══");

    let big: Vec<f32> = (0..6).map(|i| i as f32 * 0.5).collect();
    let small = Tensor::from_slice(&[1.0f32, -0.5, 0.3], &[3], false);
    check_op(&mut report, "broadcast add [2,3]+[3]", &[2, 3], &big,
        move |x| x.add(&small).sum_all(), &[0, 2, 5]);

    let big2: Vec<f32> = (0..6).map(|i| (i as f32 + 1.0) * 0.3).collect();
    let small2 = Tensor::from_slice(&[0.5f32, -1.0], &[2, 1], false);
    check_op(&mut report, "broadcast mul [2,3]*[2,1]", &[2, 3], &big2,
        move |x| x.mul(&small2).sum_all(), &[0, 3, 5]);

    println!("\n══ 13. rsqrt & combined ══");

    let rsqrt_data: Vec<f32> = (0..6).map(|i| i as f32 * 0.5 + 0.5).collect();
    check_op(&mut report, "rsqrt", &[2, 3], &rsqrt_data,
        |x| x.rsqrt().sum_all(), &[0, 2, 5]);

    check_op(&mut report, "chain: rsqrt->mul_scalar", &[2, 3], &rsqrt_data,
        |x| x.rsqrt().mul_scalar(2.0).sum_all(), &[0, 3]);

    report.summary();
}