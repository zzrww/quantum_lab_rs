// src/potentials/step.rs

use super::Potential;

/// 阶跃势垒 (Step Potential)
/// 模拟粒子遇到一个突然升高的电势区域
pub struct StepPotential {
    /// 阶跃发生的 x 坐标网格索引
    step_x: usize,
    /// 势能的高度 (V0)
    v_height: f64,
}

impl StepPotential {
    /// 创建一个新的阶跃势
    /// step_x: 势能突变的位置 (0..N)
    /// v_height: 突变后的势能高度
    pub fn new(step_x: usize, v_height: f64) -> Self {
        Self { step_x, v_height }
    }
}

// 实现 Potential 特征，使其能被求解器使用
impl Potential for StepPotential {
    fn name(&self) -> &'static str {
        "Step Potential Scattering"
    }

    fn get(&self, ix: usize, _iy: usize) -> f64 {
        // 这是一个一维势垒在二维平面的延伸 (像一道长墙)
        // 势能只取决于 x 坐标，与 y 无关
        
        if ix >= self.step_x {
            // 在阶跃点右侧，势能为 v_height
            self.v_height
        } else {
            // 在阶跃点左侧，势能为 0 (自由空间)
            0.0
        }
    }
}