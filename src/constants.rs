pub const N: usize = 600; 
pub const DX: f64 = 100.0 / N as f64; // 自动计算步长，保证总宽度不变

pub const DT: f64 = 0.005; 

pub const TOTAL_STEPS: usize = 40000;

// 绘图间隔
pub const PLOT_INTERVAL: usize = 400;

// 物理位置保持不变 (它们是物理单位，不是网格索引)
pub const DEFAULT_X0: f64 = 20.0;
pub const DEFAULT_Y0: f64 = 50.0; // 始终在 100.0 的中间
pub const DEFAULT_SIGMA: f64 = 6.0;