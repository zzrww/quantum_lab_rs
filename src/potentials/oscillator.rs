use super::Potential;
use crate::constants::{N, DX};

pub struct HarmonicOscillator {
    k: f64, // 弹性系数
}

impl HarmonicOscillator {
    pub fn new(k: f64) -> Self { Self { k } }
}

impl Potential for HarmonicOscillator {
    fn name(&self) -> &'static str { "2D Harmonic Oscillator" }

    fn get(&self, ix: usize, iy: usize) -> f64 {
        let x = ix as f64 * DX;
        let y = iy as f64 * DX;
        let center = (N / 2) as f64 * DX;
        
        let r2 = (x - center).powi(2) + (y - center).powi(2);
        0.5 * self.k * r2
    }
}