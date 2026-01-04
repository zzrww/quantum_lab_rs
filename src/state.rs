use ndarray::Array2;
use crate::constants::*;

pub struct QuantumState {
    pub real: Array2<f64>,
    pub imag: Array2<f64>,
    pub density: Array2<f64>,
}

impl QuantumState {
    pub fn new() -> Self {
        Self {
            real: Array2::zeros((N, N)),
            imag: Array2::zeros((N, N)),
            density: Array2::zeros((N, N)),
        }
    }

    pub fn init_gaussian(&mut self, kx: f64, ky: f64, x0: f64, y0: f64, sigma: f64) {
        let mut norm_sq = 0.0;
        
        // 1. 生成未归一化的波包
        for x in 0..N {
            for y in 0..N {
                let xx = x as f64 * DX;
                let yy = y as f64 * DX;
                
                let dist_sq = (xx - x0).powi(2) + (yy - y0).powi(2);
                let envelope = (-dist_sq / (2.0 * sigma.powi(2))).exp();
                let phase = kx * xx + ky * yy;
                
                self.real[[x, y]] = envelope * phase.cos();
                self.imag[[x, y]] = envelope * phase.sin();
                
                norm_sq += (self.real[[x, y]].powi(2) + self.imag[[x, y]].powi(2)) * DX * DX;
            }
        }

        // 2. 归一化：使得积分概率为 1
        let norm = norm_sq.sqrt();
        self.real.mapv_inplace(|v| v / norm);
        self.imag.mapv_inplace(|v| v / norm);
        self.update_density();
    }

    pub fn update_density(&mut self) {
        // 使用并行迭代加速大数组运算 (rayon)
        use rayon::prelude::*;
        
        // ndarray 的并行操作稍微复杂，这里用简单的串行即可，
        // 或者如果想秀一下 rayon，可以转成 slice 处理
        let r_slice = self.real.as_slice_memory_order().unwrap();
        let i_slice = self.imag.as_slice_memory_order().unwrap();
        let d_slice = self.density.as_slice_memory_order_mut().unwrap();

        d_slice.par_iter_mut().enumerate().for_each(|(idx, val)| {
            *val = r_slice[idx].powi(2) + i_slice[idx].powi(2);
        });
    }
}