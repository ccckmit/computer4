// ============================================================
//  tensor.rs  –  N-dimensional tensor with reverse-mode autograd
//  Following ai4/nn pattern for proper gradient checking
// ============================================================
use std::cell::RefCell;
use std::rc::Rc;

// ── Internal node kept behind Rc<RefCell<…>> ─────────────────
pub(crate) struct TensorInner {
    pub data: Vec<f32>,
    pub grad: Vec<f32>,
    pub shape: Vec<usize>,
    pub requires_grad: bool,
    pub(crate) backward_fn: Option<Box<dyn FnMut()>>,
    pub(crate) prev: Vec<Tensor>,
}

// ── Public handle ─────────────────────────────────────────────
pub struct Tensor {
    inner: Rc<RefCell<TensorInner>>,
}

impl Clone for Tensor {
    fn clone(&self) -> Self {
        Tensor {
            inner: self.inner.clone(),
        }
    }
}

impl Tensor {
    // ── constructors ──────────────────────────────────────────
    pub fn new(data: Vec<f32>, shape: Vec<usize>, requires_grad: bool) -> Self {
        let n = data.len();
        assert_eq!(
            n,
            shape.iter().product::<usize>().max(1),
            "data/shape mismatch: data.len()={}, shape={:?}, product={}",
            n,
            shape,
            shape.iter().product::<usize>().max(1)
        );
        Tensor {
            inner: Rc::new(RefCell::new(TensorInner {
                data,
                grad: vec![0.0; n],
                shape,
                requires_grad,
                backward_fn: None,
                prev: vec![],
            })),
        }
    }

    pub fn scalar(v: f32) -> Self {
        Self::new(vec![v], vec![1], false)
    }

    pub fn zeros(shape: &[usize]) -> Self {
        let n: usize = shape.iter().product();
        Self::new(vec![0.0; n], shape.to_vec(), false)
    }

    pub fn ones(shape: &[usize]) -> Self {
        let n: usize = shape.iter().product();
        Self::new(vec![1.0; n], shape.to_vec(), false)
    }

    pub fn from_slice(data: &[f32], shape: &[usize], requires_grad: bool) -> Self {
        Self::new(data.to_vec(), shape.to_vec(), requires_grad)
    }

    pub fn randn(shape: &[usize], _std: f32) -> Self {
        let n: usize = shape.iter().product();
        let mut v = Vec::with_capacity(n);
        let mut rng = SimpleRng::new(12345);
        for _ in 0..n {
            v.push(rng.randn() as f32);
        }
        Self::new(v, shape.to_vec(), true)
    }

    pub fn uniform(shape: &[usize], low: f32, high: f32) -> Self {
        let n: usize = shape.iter().product();
        let mut v = Vec::with_capacity(n);
        let mut rng = SimpleRng::new(12345);
        for _ in 0..n {
            v.push(rng.next_f32() * (high - low) + low);
        }
        Self::new(v, shape.to_vec(), true)
    }

    // ── accessors ─────────────────────────────────────────────
    pub fn data(&self) -> std::cell::Ref<Vec<f32>> {
        std::cell::Ref::map(self.inner.borrow(), |t| &t.data)
    }

    pub fn grad(&self) -> std::cell::Ref<Vec<f32>> {
        std::cell::Ref::map(self.inner.borrow(), |t| &t.grad)
    }

    pub fn shape(&self) -> Vec<usize> {
        self.inner.borrow().shape.clone()
    }

    pub fn numel(&self) -> usize {
        self.inner.borrow().data.len()
    }

    pub fn requires_grad(&self) -> bool {
        self.inner.borrow().requires_grad
    }

    pub fn set_requires_grad(&self, v: bool) {
        self.inner.borrow_mut().requires_grad = v;
    }

    pub fn zero_grad(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.grad.iter_mut().for_each(|g| *g = 0.0);
    }

    pub fn scalar_val(&self) -> f32 {
        self.inner.borrow().data[0]
    }

    // ── access to inner for low-level operations ──────────────
    pub fn inner_ptr(&self) -> &Rc<RefCell<TensorInner>> {
        &self.inner
    }

    // ── strides helper ────────────────────────────────────────
    fn strides(shape: &[usize]) -> Vec<usize> {
        let n = shape.len();
        if n == 0 {
            return vec![];
        }
        let mut s = vec![1usize; n];
        for i in (0..n.saturating_sub(1)).rev() {
            s[i] = s[i + 1] * shape[i + 1];
        }
        s
    }

    // ── backward pass ─────────────────────────────────────────
    pub fn backward(&self) {
        let mut topo: Vec<Tensor> = vec![];
        let mut visited: Vec<*const RefCell<TensorInner>> = vec![];

        fn build(node: &Tensor, topo: &mut Vec<Tensor>, visited: &mut Vec<*const RefCell<TensorInner>>) {
            let ptr = Rc::as_ptr(&node.inner);
            if visited.contains(&ptr) {
                return;
            }
            visited.push(ptr);
            let prev = node.inner.borrow().prev.clone();
            for child in &prev {
                build(child, topo, visited);
            }
            topo.push(node.clone());
        }
        build(self, &mut topo, &mut visited);

        // seed gradient
        {
            let mut inner = self.inner.borrow_mut();
            inner.grad.iter_mut().for_each(|g| *g = 1.0);
        }

        // reverse topological order
        for node in topo.iter().rev() {
            let mut f = node.inner.borrow_mut().backward_fn.take();
            if let Some(mut f) = f {
                f();
                node.inner.borrow_mut().backward_fn = Some(f);
            }
        }
    }

    // ── reshape ────────────────────────────────────────────────
    pub fn reshape(&self, shape: Vec<usize>) -> Tensor {
        let data = self.inner.borrow().data.clone();
        let rg = self.requires_grad();
        let out = Tensor::new(data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if this.requires_grad() {
                    let mut inner = this.inner.borrow_mut();
                    for (g, og) in inner.grad.iter_mut().zip(og.iter()) {
                        *g += og;
                    }
                }
            }));
        }
        out
    }

    // ── transpose (swap two axes) ─────────────────────────────
    pub fn transpose(&self, ax1: usize, ax2: usize) -> Tensor {
        let inner = self.inner.borrow();
        let n = inner.shape.len();
        let mut new_shape = inner.shape.clone();
        new_shape.swap(ax1, ax2);
        let strides = Self::strides(&inner.shape);
        let new_strides = Self::strides(&new_shape);

        let mut out_data = vec![0.0f32; inner.data.len()];
        for flat in 0..inner.data.len() {
            let mut rem = flat;
            let mut idx = vec![0usize; n];
            for d in 0..n {
                idx[d] = rem / strides[d];
                rem %= strides[d];
            }
            let mut new_idx = idx.clone();
            new_idx.swap(ax1, ax2);
            let new_flat: usize = new_idx.iter().zip(new_strides.iter()).map(|(i, s)| i * s).sum();
            out_data[new_flat] = inner.data[flat];
        }
        let rg = inner.requires_grad;
        drop(inner);

        let out = Tensor::new(out_data, new_shape.clone(), rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let shape = self.shape();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let strides = Self::strides(&shape);
                let new_strides = Self::strides(&new_shape);
                let mut tg = this.inner.borrow_mut();
                for flat in 0..tg.data.len() {
                    let mut rem = flat;
                    let mut idx = vec![0usize; n];
                    for d in 0..n {
                        idx[d] = rem / strides[d];
                        rem %= strides[d];
                    }
                    let mut new_idx = idx.clone();
                    new_idx.swap(ax1, ax2);
                    let new_flat: usize = new_idx.iter().zip(new_strides.iter()).map(|(i, s)| i * s).sum();
                    tg.grad[flat] += og[new_flat];
                }
            }));
        }
        out
    }

    // ── element-wise add ──────────────────────────────────────
    pub fn add(&self, other: &Tensor) -> Tensor {
        let (a, b) = broadcast_pair(self, other);
        let out_data: Vec<f32> = a.inner.borrow().data.iter().zip(b.inner.borrow().data.iter()).map(|(x, y)| x + y).collect();
        let shape = a.shape();
        let rg = a.requires_grad() || b.requires_grad();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let a2 = a.clone();
            let b2 = b.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![a2.clone(), b2.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if a2.requires_grad() {
                    let mut ai = a2.inner.borrow_mut();
                    for (g, og) in ai.grad.iter_mut().zip(og.iter()) {
                        *g += og;
                    }
                }
                if b2.requires_grad() {
                    let mut bi = b2.inner.borrow_mut();
                    for (g, og) in bi.grad.iter_mut().zip(og.iter()) {
                        *g += og;
                    }
                }
            }));
        }
        out
    }

    pub fn add_scalar(&self, s: f32) -> Tensor {
        let data = self.inner.borrow().data.iter().map(|v| v + s).collect::<Vec<_>>();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if this.requires_grad() {
                    let mut inner = this.inner.borrow_mut();
                    for (g, og) in inner.grad.iter_mut().zip(og.iter()) {
                        *g += og;
                    }
                }
            }));
        }
        out
    }

    pub fn sub(&self, other: &Tensor) -> Tensor {
        self.add(&other.neg())
    }

    pub fn neg(&self) -> Tensor {
        self.mul_scalar(-1.0)
    }

    // ── element-wise mul ──────────────────────────────────────
    pub fn mul(&self, other: &Tensor) -> Tensor {
        let (a, b) = broadcast_pair(self, other);
        let a_data = a.inner.borrow().data.clone();
        let b_data = b.inner.borrow().data.clone();
        let out_data: Vec<f32> = a_data.iter().zip(b_data.iter()).map(|(x, y)| x * y).collect();
        let shape = a.shape();
        let rg = a.requires_grad() || b.requires_grad();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let a2 = a.clone();
            let b2 = b.clone();
            let out_c = out.clone();
            let a_data2 = a_data.clone();
            let b_data2 = b_data.clone();
            out.inner.borrow_mut().prev = vec![a2.clone(), b2.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if a2.requires_grad() {
                    let mut ai = a2.inner.borrow_mut();
                    for ((g, og), bd) in ai.grad.iter_mut().zip(og.iter()).zip(b_data2.iter()) {
                        *g += og * bd;
                    }
                }
                if b2.requires_grad() {
                    let mut bi = b2.inner.borrow_mut();
                    for ((g, og), ad) in bi.grad.iter_mut().zip(og.iter()).zip(a_data2.iter()) {
                        *g += og * ad;
                    }
                }
            }));
        }
        out
    }

    pub fn mul_scalar(&self, s: f32) -> Tensor {
        let data = self.inner.borrow().data.iter().map(|v| v * s).collect::<Vec<_>>();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if this.requires_grad() {
                    let mut inner = this.inner.borrow_mut();
                    for (g, og) in inner.grad.iter_mut().zip(og.iter()) {
                        *g += og * s;
                    }
                }
            }));
        }
        out
    }

    // ── matmul (batched) ──────────────────────────────────────
    pub fn matmul(&self, other: &Tensor) -> Tensor {
        let a_shape = self.shape();
        let b_shape = other.shape();
        let nd = a_shape.len();
        let ndb = b_shape.len();
        assert!(nd >= 2 && ndb >= 2, "matmul needs at least 2D tensors");
        let m = a_shape[nd - 2];
        let k = a_shape[nd - 1];
        let db = b_shape[ndb - 1];
        assert_eq!(k, b_shape[ndb - 2], "matmul inner dim mismatch: {} vs {}", k, b_shape[ndb - 2]);

        let a_batch: Vec<usize> = a_shape[..nd - 2].to_vec();
        let b_batch: Vec<usize> = b_shape[..ndb - 2].to_vec();
        let a_bs: usize = a_batch.iter().product::<usize>().max(1);
        let b_bs: usize = b_batch.iter().product::<usize>().max(1);
        let out_bs = a_bs.max(b_bs);

        let mut out_shape: Vec<usize> = vec![];
        let max_bl = a_batch.len().max(b_batch.len());
        for i in 0..max_bl {
            let ai = *a_batch.get(a_batch.len().wrapping_sub(max_bl - i)).unwrap_or(&1);
            let bi = *b_batch.get(b_batch.len().wrapping_sub(max_bl - i)).unwrap_or(&1);
            out_shape.push(ai.max(bi));
        }
        out_shape.push(m);
        out_shape.push(db);

        let a_data = self.inner.borrow().data.clone();
        let b_data = other.inner.borrow().data.clone();
        let mut out_data = vec![0.0f32; out_bs * m * db];

        for batch in 0..out_bs {
            let ba = batch % a_bs;
            let bb = batch % b_bs;
            for i in 0..m {
                for j in 0..db {
                    let mut sum = 0.0f32;
                    for kk in 0..k {
                        sum += a_data[ba * m * k + i * k + kk] * b_data[bb * k * db + kk * db + j];
                    }
                    out_data[batch * m * db + i * db + j] = sum;
                }
            }
        }

        let rg = self.requires_grad() || other.requires_grad();
        let out = Tensor::new(out_data, out_shape, rg);

        if rg {
            let this = self.clone();
            let other2 = other.clone();
            let out_c = out.clone();
            let a_data2 = a_data.clone();
            let b_data2 = b_data.clone();
            out.inner.borrow_mut().prev = vec![this.clone(), other2.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                if this.requires_grad() {
                    let mut tg = this.inner.borrow_mut();
                    for batch in 0..out_bs {
                        let ba = batch % a_bs;
                        let bb = batch % b_bs;
                        for i in 0..m {
                            for kk in 0..k {
                                let mut g = 0.0f32;
                                for j in 0..db {
                                    g += og[batch * m * db + i * db + j] * b_data2[bb * k * db + kk * db + j];
                                }
                                tg.grad[ba * m * k + i * k + kk] += g;
                            }
                        }
                    }
                }
                if other2.requires_grad() {
                    let mut og2 = other2.inner.borrow_mut();
                    for batch in 0..out_bs {
                        let ba = batch % a_bs;
                        let bb = batch % b_bs;
                        for kk in 0..k {
                            for j in 0..db {
                                let mut g = 0.0f32;
                                for i in 0..m {
                                    g += og[batch * m * db + i * db + j] * a_data2[ba * m * k + i * k + kk];
                                }
                                og2.grad[bb * k * db + kk * db + j] += g;
                            }
                        }
                    }
                }
            }));
        }
        out
    }

    // ── relu ──────────────────────────────────────────────────
    pub fn relu(&self) -> Tensor {
        let data = self.inner.borrow().data.clone();
        let out_data: Vec<f32> = data.iter().map(|&v| v.max(0.0)).collect();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let data_snap = data.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    if data_snap[i] > 0.0 {
                        *g += og;
                    }
                }
            }));
        }
        out
    }

    // ── silu (swish) ─────────────────────────────────────────
    pub fn silu(&self) -> Tensor {
        let data = self.inner.borrow().data.clone();
        let out_data: Vec<f32> = data.iter().map(|&v| {
            let s = 1.0 / (1.0 + (-v).exp());
            v * s
        }).collect();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let data_snap = data.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    let v = data_snap[i];
                    let s = 1.0 / (1.0 + (-v).exp());
                    let ds = s * (1.0 + v * s * (1.0 - s));
                    *g += og * ds;
                }
            }));
        }
        out
    }

    // ── tanh ─────────────────────────────────────────────────
    pub fn tanh(&self) -> Tensor {
        let out_data: Vec<f32> = self.inner.borrow().data.iter().map(|&v| v.tanh()).collect();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out_c_data = out_data.clone();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    let t = out_c_data[i];
                    *g += og * (1.0 - t * t);
                }
            }));
        }
        out
    }

    // ── softmax along axis ────────────────────────────────────
    pub fn softmax(&self, axis: usize) -> Tensor {
        let shape = self.shape();
        let n = shape.len();
        let outer: usize = shape[..axis].iter().product::<usize>().max(1);
        let dim = shape[axis];
        let stride: usize = shape[axis + 1..].iter().product::<usize>().max(1);
        let data = self.inner.borrow().data.clone();
        let mut probs = vec![0.0f32; data.len()];

        for o in 0..outer {
            for s in 0..stride {
                let max = (0..dim).map(|d| data[o * dim * stride + d * stride + s]).fold(f32::NEG_INFINITY, f32::max);
                let sum: f32 = (0..dim).map(|d| (data[o * dim * stride + d * stride + s] - max).exp()).sum();
                for d in 0..dim {
                    let idx = o * dim * stride + d * stride + s;
                    probs[idx] = (data[idx] - max).exp() / (sum + 1e-10);
                }
            }
        }

        let rg = self.requires_grad();
        let probs2 = probs.clone();
        let out = Tensor::new(probs, shape.clone(), rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for o in 0..outer {
                    for s in 0..stride {
                        let mut dot = 0.0f32;
                        for d in 0..dim {
                            let idx = o * dim * stride + d * stride + s;
                            dot += probs2[idx] * og[idx];
                        }
                        for d in 0..dim {
                            let idx = o * dim * stride + d * stride + s;
                            ti.grad[idx] += probs2[idx] * (og[idx] - dot);
                        }
                    }
                }
            }));
        }
        out
    }

    // ── cross-entropy loss (logits [B, V], targets [B]) ─────
    pub fn cross_entropy(&self, targets: &[usize]) -> Tensor {
        let shape = self.shape();
        let batch = shape[0];
        let vocab = *shape.last().unwrap();
        let data = self.inner.borrow().data.clone();
        let mut loss = 0.0f32;

        for b in 0..batch {
            let off = b * vocab;
            let max = data[off..off + vocab].iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let sum_exp: f32 = data[off..off + vocab].iter().map(|&v| (v - max).exp()).sum();
            let t = targets[b];
            loss -= ((data[off + t] - max).exp() / sum_exp + 1e-10).ln();
        }
        loss /= batch as f32;

        let rg = self.requires_grad();
        let out = Tensor::new(vec![loss], vec![1], rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let targets = targets.to_vec();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let _og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for b in 0..batch {
                    let off = b * vocab;
                    let max = ti.data[off..off + vocab].iter().copied().fold(f32::NEG_INFINITY, f32::max);
                    let sum_exp: f32 = ti.data[off..off + vocab].iter().map(|&v| (v - max).exp()).sum();
                    for v in 0..vocab {
                        let prob = (ti.data[off + v] - max).exp() / (sum_exp + 1e-10);
                        let target_flag = if v == targets[b] { 1.0 } else { 0.0 };
                        ti.grad[off + v] += (prob - target_flag) / batch as f32;
                    }
                }
            }));
        }
        out
    }

    // ── sum all ───────────────────────────────────────────────
    pub fn sum_all(&self) -> Tensor {
        let s: f32 = self.inner.borrow().data.iter().sum();
        let rg = self.requires_grad();
        let out = Tensor::new(vec![s], vec![1], rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad[0];
                let mut ti = this.inner.borrow_mut();
                if ti.requires_grad {
                    for g in ti.grad.iter_mut() {
                        *g += og;
                    }
                }
            }));
        }
        out
    }

    pub fn mean_all(&self) -> Tensor {
        let n = self.numel() as f32;
        self.sum_all().mul_scalar(1.0 / n)
    }

    // ── pow (scalar exponent) ─────────────────────────────────
    pub fn pow(&self, p: f32) -> Tensor {
        let shape = self.shape();
        let rg = self.requires_grad();
        let data_snap = self.inner.borrow().data.clone();
        let out = Tensor::new(data_snap.iter().map(|&v| v.powf(p)).collect(), shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let ds = data_snap.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    *g += og * p * ds[i].powf(p - 1.0);
                }
            }));
        }
        out
    }

    // ── rsqrt ────────────────────────────────────────────────
    pub fn rsqrt(&self) -> Tensor {
        let data = self.inner.borrow().data.clone();
        let out_data: Vec<f32> = data.iter().map(|&v| 1.0 / v.sqrt()).collect();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(out_data, shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let data_snap = data.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    let v = data_snap[i];
                    let r = 1.0 / v.sqrt();
                    *g += og * (-0.5 * r / v);
                }
            }));
        }
        out
    }

    // ── rms_norm ──────────────────────────────────────────────
    pub fn rms_norm(&self) -> Tensor {
        let shape = self.shape();
        let d = *shape.last().unwrap();
        let data = self.inner.borrow().data.clone();
        let n = data.len();
        let num_rows = n / d;
        let eps = 1e-5f32;

        let mut rms = vec![0.0f32; num_rows];
        for r in 0..num_rows {
            let m2: f32 = data[r * d..(r + 1) * d].iter().map(|&v| v * v).sum::<f32>() / d as f32;
            rms[r] = (m2 + eps).sqrt();
        }

        let mut out_data = vec![0.0f32; n];
        for i in 0..n {
            let r = i / d;
            out_data[i] = data[i] / rms[r];
        }

        let rg = self.requires_grad();
        let out = Tensor::new(out_data.clone(), shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let rms2 = rms.clone();
            let data2 = data.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for r in 0..num_rows {
                    let inv_std = 1.0 / rms2[r];
                    let mut dot = 0.0f32;
                    for j in 0..d {
                        dot += og[r * d + j] * data2[r * d + j];
                    }
                    for j in 0..d {
                        ti.grad[r * d + j] += og[r * d + j] * inv_std
                            - data2[r * d + j] * inv_std * inv_std * inv_std * dot / d as f32;
                    }
                }
            }));
        }
        out
    }

    // ── exp ───────────────────────────────────────────────────
    pub fn exp(&self) -> Tensor {
        let out_data: Vec<f32> = self.inner.borrow().data.iter().map(|&v| v.exp()).collect();
        let shape = self.shape();
        let rg = self.requires_grad();
        let out = Tensor::new(out_data.clone(), shape, rg);
        if rg {
            let this = self.clone();
            let out_c = out.clone();
            let out_data2 = out_data.clone();
            out.inner.borrow_mut().prev = vec![this.clone()];
            out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
                let og = out_c.inner.borrow().grad.clone();
                let mut ti = this.inner.borrow_mut();
                for (i, (g, og)) in ti.grad.iter_mut().zip(og.iter()).enumerate() {
                    *g += og * out_data2[i];
                }
            }));
        }
        out
    }
}

// ── broadcast helper ─────────────────────────────────────────
fn broadcast_pair(a: &Tensor, b: &Tensor) -> (Tensor, Tensor) {
    let as_ = a.shape();
    let bs = b.shape();
    if as_ == bs {
        return (a.clone(), b.clone());
    }
    let max_len = as_.len().max(bs.len());
    let mut as2 = vec![1usize; max_len - as_.len()];
    as2.extend_from_slice(&as_);
    let mut bs2 = vec![1usize; max_len - bs.len()];
    bs2.extend_from_slice(&bs);
    let out_shape: Vec<usize> = as2.iter().zip(bs2.iter()).map(|(a, b)| (*a).max(*b)).collect();
    (tile_to(a, &as2, &out_shape), tile_to(b, &bs2, &out_shape))
}

fn tile_to(t: &Tensor, src_shape: &[usize], out_shape: &[usize]) -> Tensor {
    if src_shape == out_shape {
        return t.clone();
    }
    let n = out_shape.len();
    let out_strides = Tensor::strides(out_shape);
    let src_strides = Tensor::strides(src_shape);
    let out_size: usize = out_shape.iter().product();
    let mut out_data = vec![0.0f32; out_size];
    for i in 0..out_size {
        let mut rem = i;
        let mut src_flat = 0usize;
        for d in 0..n {
            let idx = rem / out_strides[d];
            rem %= out_strides[d];
            let src_idx = idx % src_shape[d];
            src_flat += src_idx * src_strides[d];
        }
        out_data[i] = t.inner.borrow().data[src_flat];
    }
    let rg = t.requires_grad();
    let out = Tensor::new(out_data, out_shape.to_vec(), rg);
    if rg {
        let t2 = t.clone();
        let out_c = out.clone();
        let ss = src_shape.to_vec();
        let os = out_shape.to_vec();
        out.inner.borrow_mut().prev = vec![t2.clone()];
        out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
            let og = out_c.inner.borrow().grad.clone();
            let out_strides = Tensor::strides(&os);
            let src_strides = Tensor::strides(&ss);
            let n2 = os.len();
            let mut ti = t2.inner.borrow_mut();
            for i in 0..og.len() {
                let mut rem = i;
                let mut src_flat = 0usize;
                for d in 0..n2 {
                    let idx = rem / out_strides[d];
                    rem %= out_strides[d];
                    let src_idx = idx % ss[d];
                    src_flat += src_idx * src_strides[d];
                }
                ti.grad[src_flat] += og[i];
            }
        }));
    }
    out
}

// ── cat along axis ────────────────────────────────────────────
pub fn cat(tensors: &[Tensor], axis: usize) -> Tensor {
    let n = tensors[0].shape().len();
    let mut out_shape = tensors[0].shape();
    for t in &tensors[1..] {
        out_shape[axis] += t.shape()[axis];
    }
    let out_strides = Tensor::strides(&out_shape);
    let out_size: usize = out_shape.iter().product();
    let mut out_data = vec![0.0f32; out_size];
    let rg = tensors.iter().any(|t| t.requires_grad());

    let mut dim_offset = 0usize;
    for t in tensors {
        let t_shape = t.shape();
        let in_strides = Tensor::strides(&t_shape);
        let t_data = t.inner.borrow().data.clone();
        for f in 0..t_data.len() {
            let mut rem = f;
            let mut idx = vec![0usize; n];
            for d in 0..n {
                idx[d] = rem / in_strides[d];
                rem %= in_strides[d];
            }
            idx[axis] += dim_offset;
            let of: usize = idx.iter().zip(out_strides.iter()).map(|(i, s)| i * s).sum();
            out_data[of] = t_data[f];
        }
        dim_offset += t_shape[axis];
    }

    let out = Tensor::new(out_data, out_shape.clone(), rg);
    if rg {
        let tensors2: Vec<Tensor> = tensors.to_vec();
        let out_c = out.clone();
        out.inner.borrow_mut().prev = tensors.to_vec();
        out.inner.borrow_mut().backward_fn = Some(Box::new(move || {
            let og = out_c.inner.borrow().grad.clone();
            let mut d_off = 0usize;
            for t in &tensors2 {
                if t.requires_grad() {
                    let t_shape = t.shape();
                    let in_strides = Tensor::strides(&t_shape);
                    let mut ti = t.inner.borrow_mut();
                    for f in 0..ti.data.len() {
                        let mut rem = f;
                        let mut idx = vec![0usize; n];
                        for d in 0..n {
                            idx[d] = rem / in_strides[d];
                            rem %= in_strides[d];
                        }
                        idx[axis] += d_off;
                        let of: usize = idx.iter().zip(out_strides.iter()).map(|(i, s)| i * s).sum();
                        ti.grad[f] += og[of];
                    }
                    d_off += t_shape[axis];
                } else {
                    d_off += t.shape()[axis];
                }
            }
        }));
    }
    out
}

// ── tiny seeded PRNG ─────────────────────────────────────────
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 11) as f32 / (1u64 << 53) as f32
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Box-Muller normal
    pub fn randn(&mut self) -> f64 {
        use std::f64::consts::PI;
        let u1 = self.next_f64().max(1e-10);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }

    pub fn next_usize(&mut self, n: usize) -> usize {
        (self.next_u64() as usize) % n
    }
}

// ── Extra public helpers ──────────────────────────────────────
impl Tensor {
    pub fn push_prev(&self, t: Tensor) {
        self.inner.borrow_mut().prev.push(t);
    }

    pub fn set_backward_fn(&self, f: Box<dyn FnMut()>) {
        self.inner.borrow_mut().backward_fn = Some(f);
    }
}

// ── numerical gradient checker ────────────────────────────────
const H: f32 = 1e-3;
const RTOL: f32 = 5e-2;

pub struct GradReport {
    pub passed: usize,
    pub failed: usize,
}

impl GradReport {
    pub fn new() -> Self {
        Self { passed: 0, failed: 0 }
    }

    pub fn check(&mut self, name: &str, analytic: f32, numeric: f32) {
        let denom = analytic.abs().max(numeric.abs()).max(1e-8);
        let rel = (analytic - numeric).abs() / denom;
        if rel < RTOL {
            println!("  ✓ {:40} analytic={:+10.6e}  numeric={:+10.6e}  rel={:.2e}", name, analytic, numeric, rel);
            self.passed += 1;
        } else {
            println!("  ✗ {:40} analytic={:+10.6e}  numeric={:+10.6e}  rel={:.2e}  ← FAIL", name, analytic, numeric, rel);
            self.failed += 1;
        }
    }

    pub fn summary(&self) {
        println!("\n{}", "-".repeat(70));
        println!("  PASSED: {}   FAILED: {}", self.passed, self.failed);
        println!("{}", "-".repeat(70));
        if self.failed > 0 {
            std::process::exit(1);
        }
    }
}

fn analytic_grad(
    make_input: impl Fn(Vec<f32>) -> Tensor,
    data: &[f32],
    make_loss: impl Fn(&Tensor) -> Tensor,
    idx: usize,
) -> f32 {
    let x = make_input(data.to_vec());
    x.zero_grad();
    let loss = make_loss(&x);
    loss.backward();
    let g = x.grad()[idx];
    g
}

fn numeric_grad(
    make_input: impl Fn(Vec<f32>) -> Tensor,
    data: &[f32],
    make_loss: impl Fn(&Tensor) -> Tensor,
    idx: usize,
) -> f32 {
    let mut d_plus = data.to_vec();
    d_plus[idx] += H;
    let mut d_minus = data.to_vec();
    d_minus[idx] -= H;
    let lp = make_loss(&make_input(d_plus)).scalar_val();
    let lm = make_loss(&make_input(d_minus)).scalar_val();
    (lp - lm) / (2.0 * H)
}

pub fn check_op(
    report: &mut GradReport,
    label: &str,
    shape: &[usize],
    data: &[f32],
    make_loss: impl Fn(&Tensor) -> Tensor + Clone,
    check_indices: &[usize],
) {
    for &i in check_indices {
        let d2 = data.to_vec();
        let sh = shape.to_vec();
        let ml = make_loss.clone();
        let ml2 = make_loss.clone();
        let a = analytic_grad(
            |d| Tensor::from_slice(&d, &sh, true),
            &d2,
            |x| ml(x),
            i,
        );
        let n = numeric_grad(
            |d| Tensor::from_slice(&d, &sh, true),
            &d2,
            |x| ml2(x),
            i,
        );
        report.check(&format!("{} [{}]", label, i), a, n);
    }
}

// ── tests ────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relu_grad() {
        let data = vec![-1.0f32, 0.5, 2.0, -0.5, 1.0, 0.0];
        let shape = vec![2, 3];
        let make_loss = |x: &Tensor| x.relu().sum_all();
        check_op(&mut GradReport::new(), "relu", &shape, &data, make_loss, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_tanh_grad() {
        let data = vec![-0.5f32, 0.3, 1.0, -1.0, 0.8, 0.2];
        let shape = vec![2, 3];
        let make_loss = |x: &Tensor| x.tanh().sum_all();
        check_op(&mut GradReport::new(), "tanh", &shape, &data, make_loss, &[0, 2, 4, 5]);
    }

    #[test]
    fn test_mul_grad() {
        let a_data = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = Tensor::from_slice(&[0.5f32, -1.0, 0.3, 0.2], &[2, 2], false);
        let make_loss = |x: &Tensor| x.mul(&b).sum_all();
        check_op(&mut GradReport::new(), "mul", &[2, 2], &a_data, make_loss, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_add_grad() {
        let a_data = vec![1.0f32, 2.0, 3.0, 4.0];
        let b = Tensor::from_slice(&[0.5f32, -1.0], &[2, 1], false);
        let make_loss = |x: &Tensor| x.add(&b).sum_all();
        check_op(&mut GradReport::new(), "add_broadcast", &[2, 2], &a_data, make_loss, &[0, 2, 3]);
    }

    #[test]
    fn test_matmul_grad() {
        let a_data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = Tensor::from_slice(&[0.1f32, -0.2, 0.3, 0.4, -0.5, 0.6, 0.7, -0.8], &[2, 4], false);
        let make_loss = |x: &Tensor| x.matmul(&b).sum_all();
        check_op(&mut GradReport::new(), "matmul", &[2, 3], &a_data, make_loss, &[0, 2, 5]);
    }

    #[test]
    fn test_softmax_grad() {
        let data = vec![1.0f32, 2.0, 0.5, -1.0, 0.5, 3.0];
        let make_loss = |x: &Tensor| {
            let weights = Tensor::from_slice(&[1.0f32, 2.0, 3.0, 0.5, 1.5, 2.5], &[2, 3], false);
            x.softmax(1).mul(&weights).sum_all()
        };
        check_op(&mut GradReport::new(), "softmax", &[2, 3], &data, make_loss, &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_cross_entropy_grad() {
        let data = vec![1.0f32, 2.0, 0.5, -1.0, 0.5, 3.0];
        let make_loss = |x: &Tensor| x.cross_entropy(&[1, 2]);
        check_op(&mut GradReport::new(), "cross_entropy", &[2, 3], &data, make_loss, &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_pow_grad() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let make_loss = |x: &Tensor| x.pow(2.0).sum_all();
        check_op(&mut GradReport::new(), "pow(2)", &[2, 2], &data, make_loss, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_sum_all_grad() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let make_loss = |x: &Tensor| x.sum_all();
        check_op(&mut GradReport::new(), "sum_all", &[2, 2], &data, make_loss, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_rms_norm_grad() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
        let make_loss = |x: &Tensor| x.rms_norm().sum_all();
        check_op(&mut GradReport::new(), "rms_norm", &[2, 3], &data, make_loss, &[0, 2, 4, 5]);
    }

    #[test]
    fn test_exp_grad() {
        let data = vec![0.0f32, 0.5, 1.0, 1.5];
        let make_loss = |x: &Tensor| x.exp().sum_all();
        check_op(&mut GradReport::new(), "exp", &[2, 2], &data, make_loss, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_neg_grad() {
        let data = vec![1.0f32, -2.0, 3.0, -4.0];
        let make_loss = |x: &Tensor| x.neg().sum_all();
        check_op(&mut GradReport::new(), "neg", &[2, 2], &data, make_loss, &[0, 1, 2, 3]);
    }

    #[test]
    fn test_scalar_ops() {
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let make_loss = |x: &Tensor| x.mul_scalar(3.0).add_scalar(-1.0).sum_all();
        check_op(&mut GradReport::new(), "mul_scalar+add_scalar", &[2, 2], &data, make_loss, &[0, 2]);
    }
}