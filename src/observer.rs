use std::fs::File;
use std::io::{Write, BufWriter};
use plotters::prelude::*; // 引入绘图库
use crate::state::QuantumState;
use crate::constants::{N, DX};

pub struct Observer {
    writer: BufWriter<File>,
    // 新增：用于在内存中存储历史数据 (time, exp_x)
    history: Vec<(f64, f64)>, 
}

impl Observer {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        // 写入 CSV 表头
        writeln!(writer, "step,time,norm,exp_x,exp_y")?;
        
        Ok(Self { 
            writer, 
            history: Vec::new() // 初始化为空向量
        })
    }

    pub fn measure(&mut self, step: usize, dt: f64, state: &QuantumState) -> anyhow::Result<()> {
        let mut norm = 0.0;
        let mut exp_x = 0.0;
        let mut exp_y = 0.0;

        // 计算期望值
        for x in 0..N {
            for y in 0..N {
                let prob = state.density[[x, y]] * DX * DX;
                norm += prob;
                exp_x += (x as f64 * DX) * prob;
                exp_y += (y as f64 * DX) * prob;
            }
        }

        let time = step as f64 * dt;
        
        // 1. 写入 CSV
        writeln!(self.writer, "{},{},{:.6},{:.4},{:.4}", step, time, norm, exp_x, exp_y)?;
        
        // 2. 记录到内存中以便稍后画图
        self.history.push((time, exp_x));
        
        Ok(())
    }

    /// 新增：在模拟结束后调用此函数绘制轨迹图
    pub fn plot_trajectory(&self, path: &str) -> anyhow::Result<()> {
        if self.history.is_empty() {
            return Ok(());
        }

        let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
        root.fill(&WHITE)?;

        // 自动计算坐标轴范围
        let t_max = self.history.last().unwrap().0;
        // 找出 X 位置的最小值和最大值，为了美观上下各留 5% 的边距
        let x_min_val = self.history.iter().map(|v| v.1).fold(f64::INFINITY, f64::min);
        let x_max_val = self.history.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
        let margin = (x_max_val - x_min_val) * 0.1;
        
        // 即使波动很小，也确保有一个最小的显示范围
        let y_min = x_min_val - margin.max(1.0); 
        let y_max = x_max_val + margin.max(1.0);

        let mut chart = ChartBuilder::on(&root)
            .caption("Quantum Packet Trajectory (<x> vs t)", ("sans-serif", 40).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0.0..t_max, y_min..y_max)?;

        chart.configure_mesh()
            .x_desc("Time (t)")
            .y_desc("Expected Position <x>")
            .draw()?;

        chart.draw_series(LineSeries::new(
            self.history.iter().copied(),
            &BLUE,
        ))?
        .label("Position <x>")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;

        println!("Trajectory plot saved to {}", path);
        Ok(())
    }
}