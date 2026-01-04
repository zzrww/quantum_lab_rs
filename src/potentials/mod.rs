pub mod double_slit;
pub mod oscillator;
pub mod step;

/// 势能场特征 (Interface)
/// 任何一种物理场景只需要实现这个接口即可被求解器使用
pub trait Potential: Sync + Send {
    /// 获取网格 (ix, iy) 处的势能值 V(x, y)
    fn get(&self, ix: usize, iy: usize) -> f64;
    
    /// 获取场景名称
    fn name(&self) -> &'static str;
}