use super::Potential;
use crate::constants::{N, DX};

pub struct DoubleSlit {
    center_x: usize,
    slit_width: f64,
    separation: f64,
    v_height: f64,
}

impl DoubleSlit {
    pub fn new(center_x: usize, width: f64, sep: f64, v: f64) -> Self {
        Self { center_x, slit_width: width, separation: sep, v_height: v }
    }
}

impl Potential for DoubleSlit {
    fn name(&self) -> &'static str { "Double Slit Interference" }

    fn get(&self, ix: usize, iy: usize) -> f64 {
        // 如果不在墙的 x 位置附近，势能为 0
        if ix < self.center_x || ix > self.center_x + 3 {
            return 0.0;
        }

        let y = iy as f64 * DX;
        let center_y = (N / 2) as f64 * DX;
        let half_sep = self.separation / 2.0;
        let half_w = self.slit_width / 2.0;

        // 判断是否在缝隙内
        let in_upper_slit = (y > center_y + half_sep - half_w) && (y < center_y + half_sep + half_w);
        let in_lower_slit = (y > center_y - half_sep - half_w) && (y < center_y - half_sep + half_w);

        if in_upper_slit || in_lower_slit {
            0.0
        } else {
            self.v_height
        }
    }
}