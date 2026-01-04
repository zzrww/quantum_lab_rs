use image::{ImageBuffer, Rgb};
use plotters::prelude::*;
use crate::state::QuantumState;
use crate::potentials::Potential;
use crate::constants::N;
use anyhow::Result;


/// [原有函数] 绘制波函数高清帧
pub fn plot_frame(path: &str, _title: &str, state: &QuantumState, pot: &dyn Potential) -> anyhow::Result<()> {
    let mut img = ImageBuffer::new(N as u32, N as u32);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let ix = x as usize;
        let iy = y as usize;
        if ix >= N || iy >= N { continue; }

        let density = state.density[[ix, iy]];
        let v = pot.get(ix, iy);

        let (r, g, b) = if v > 1.0 {
            (150, 50, 50)
        } else {
            let brightness = (density * 500.0).powf(0.6).min(1.0);
            let val = (brightness * 255.0) as u8;
            (0, val, val)
        };

        *pixel = Rgb([r, g, b]);
    }
    img.save(path)?;
    Ok(())
}

/// [新增函数] 绘制透射率对比曲线 (Sim vs Theory)
pub fn plot_transmission_curve(
    path: &str,
    data_sim: &[(f64, f64)],    // (Energy, T_sim)
    data_theory: &[(f64, f64)]  // (Energy, T_theory)
) -> anyhow::Result<()> {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // 自动寻找坐标轴范围
    let x_min = 0.5;
    let x_max = 2.0;
    let y_min = 0.0;
    let y_max = 1.05; // 稍微留点头部空间

    let mut chart = ChartBuilder::on(&root)
        .caption("Transmission Coefficient: Sim vs Theory", ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh()
        .x_desc("Incident Energy (E)")
        .y_desc("Transmission Probability (T)")
        .draw()?;

    // 1. 绘制理论值 (红色实线)
    chart.draw_series(LineSeries::new(
        data_theory.iter().copied(),
        &RED.mix(0.8),
    ))?
    .label("QM Theory")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    // 2. 绘制模拟值 (蓝色点 + 线)
    chart.draw_series(LineSeries::new(
        data_sim.iter().copied(),
        &BLUE,
    ))?
    .label("Simulation")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    // 绘制散点（让数据点更明显）
    chart.draw_series(
        data_sim.iter().map(|(x, y)| Circle::new((*x, *y), 3, BLUE.filled()))
    )?;

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    println!("Chart saved to {}", path);
    Ok(())
}
/// [新增] 绘制双缝干涉条纹 (Intensity vs Y)
pub fn plot_interference_pattern(
    path: &str,
    intensity_data: &[(f64, f64)], // (y_position, probability)
) -> anyhow::Result<()> {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // 寻找最大强度以便归一化Y轴
    let max_prob = intensity_data.iter().map(|v| v.1).fold(0.0/0.0, f64::max);
    let max_y_pos = intensity_data.last().unwrap().0;

    let mut chart = ChartBuilder::on(&root)
        .caption("Double Slit Interference Pattern", ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..max_y_pos, 0.0..max_prob * 1.1)?;

    chart.configure_mesh()
        .x_desc("Screen Position (y)")
        .y_desc("Intensity |psi|^2")
        .draw()?;

    chart.draw_series(LineSeries::new(
        intensity_data.iter().copied(),
        &BLUE,
    ))?
    .label("Simulation")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    println!("Interference plot saved to {}", path);
    Ok(())
}

/// [新增] 绘制谐振子振幅 vs 能量
pub fn plot_amplitude_energy(
    path: &str,
    data: &[(f64, f64)], // (Energy, Max_Amplitude)
) -> anyhow::Result<()> {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    let x_min = data.first().unwrap().0;
    let x_max = data.last().unwrap().0;
    let y_max = data.iter().map(|v| v.1).fold(0.0/0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Harmonic Oscillator: Amplitude vs Energy", ("sans-serif", 40).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(x_min..x_max, 0.0..y_max * 1.1)?;

    chart.configure_mesh()
        .x_desc("Energy (E)")
        .y_desc("Max Amplitude (A)")
        .draw()?;

    // 绘制模拟数据点
    chart.draw_series(
        data.iter().map(|(x, y)| Circle::new((*x, *y), 3, BLUE.filled()))
    )?
    .label("Simulation")
    .legend(|(x, y)| Circle::new((x + 10, y), 3, BLUE.filled()));

    // 绘制理论曲线 A = sqrt(2E/k)
    // 假设 k=0.01 (我们在 main.rs 里硬编码的)
    let k = 0.01;
    // 我们需要重新生成一条光滑曲线用于对比
    let theory_line: Vec<(f64, f64)> = (0..100).map(|i| {
        let e = x_min + (x_max - x_min) * (i as f64 / 99.0);
        let a = (2.0 * e / k).sqrt(); 
        (e, a)
    }).collect();

    chart.draw_series(LineSeries::new(
        theory_line,
        &RED.mix(0.8),
    ))?
    .label("Theory (Classical)")
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.configure_series_labels().draw()?;
    println!("Amplitude plot saved to {}", path);
    Ok(())
}