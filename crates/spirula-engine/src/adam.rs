/// Adam with the published bias correction. Not the trainer's fused per-splat step.
#[derive(Clone, Debug)]
pub struct Adam {
    m: Vec<f32>,
    v: Vec<f32>,
    t: i32,
    beta1: f32,
    beta2: f32,
    eps: f32,
}

impl Adam {
    pub fn new(n: usize) -> Self {
        Self {
            m: vec![0.0; n],
            v: vec![0.0; n],
            t: 0,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
        }
    }

    pub fn step(&mut self, param: &mut [f32], grad: &[f32], lr: f32) {
        debug_assert_eq!(param.len(), self.m.len());
        self.t += 1;
        let b1 = self.beta1;
        let b2 = self.beta2;
        let bc1 = 1.0 - b1.powi(self.t);
        let bc2 = 1.0 - b2.powi(self.t);
        for i in 0..param.len() {
            let g = grad[i];
            if !g.is_finite() {
                continue;
            }
            self.m[i] = b1 * self.m[i] + (1.0 - b1) * g;
            self.v[i] = b2 * self.v[i] + (1.0 - b2) * g * g;
            let m = self.m[i] / bc1;
            let v = self.v[i] / bc2;
            param[i] -= lr * m / (v.sqrt() + self.eps);
        }
    }
}
