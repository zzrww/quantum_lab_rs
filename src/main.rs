use clap::{Parser, ValueEnum};
use rayon::prelude::*; // [新增]
use std::fs;
use std::io::Write;
use indicatif::{ProgressBar, ProgressStyle};

use quantum_lab_rs::{
    constants::*, 
    state::QuantumState, 
    solver, 
    vis, 
    observer::Observer, 
    potentials::{self, Potential, double_slit, oscillator, step}
};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, value_enum, default_value_t = Scenario::DoubleSlit)]
    scenario: Scenario,

    #[arg(long, default_value_t = 1.0)]
    energy: f64,

    /// 开启分析/扫描模式
    #[arg(long)]
    sweep: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Scenario {
    DoubleSlit,
    Oscillator,
    Step,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    fs::create_dir_all("output")?;

    // --- 分支：如果是扫描模式，根据场景进入不同的分析流程 ---
    if args.sweep {
        match args.scenario {
            Scenario::Step => run_step_sweep()?,
            Scenario::DoubleSlit => run_double_slit_analysis()?,
            Scenario::Oscillator => run_oscillator_sweep()?,
        }
        return Ok(());
    }

    // --- 分支：单次可视化运行 (原有逻辑) ---
    run_single_simulation(args.scenario, args.energy)?;
    Ok(())
}

// ==========================================
// 1. 原有的单次运行逻辑 (封装成函数)
// ==========================================
fn run_single_simulation(scenario: Scenario, energy: f64) -> anyhow::Result<()> {
    let potential: Box<dyn Potential> = match scenario {
        Scenario::DoubleSlit => {
            let barrier_idx = N / 2; 
            Box::new(double_slit::DoubleSlit::new(barrier_idx, 8.0, 16.0, 20.0))
        },
        Scenario::Oscillator => {
            Box::new(oscillator::HarmonicOscillator::new(0.01))
        },
        Scenario::Step => {
            let step_idx = N / 2;
            Box::new(step::StepPotential::new(step_idx, 0.8))
        }
    };

    println!("Running Single Simulation: {} (E={})", potential.name(), energy);
    
    let mut state = QuantumState::new();
    let k = (2.0 * energy).sqrt();
    
    // 针对不同场景设置初始位置
    let (start_x, start_y) = match scenario {
        Scenario::Oscillator => (DEFAULT_X0 + 30.0, DEFAULT_Y0), // 偏离中心以产生振荡
        _ => (20.0, DEFAULT_Y0), // 从左侧发射
    };

    state.init_gaussian(k, 0.0, start_x, start_y, DEFAULT_SIGMA);
    
    let mut observer = Observer::new("output/observables.csv")?;
    let bar = ProgressBar::new(TOTAL_STEPS as u64);
    
    for t in 0..TOTAL_STEPS {
        solver::step_leapfrog(&mut state, potential.as_ref());

        if t % 5 == 0 {
            state.update_density();
            observer.measure(t, DT, &state)?;
        }

        if t % PLOT_INTERVAL == 0 {
            let path = format!("output/frame_{:05}.png", t / PLOT_INTERVAL);
            let title = format!("{} (t={:.2})", potential.name(), t as f64 * DT);
            if let Err(_) = vis::plot_frame(&path, &title, &state, potential.as_ref()) {
                // ignore error
            }
        }
        bar.inc(1);
    }
    bar.finish();
    observer.plot_trajectory("output/trajectory.png")?;
    println!("Single simulation done.");
    Ok(())
}

// ==========================================
// 2. 阶跃势扫描 (透射率 vs 能量)
// ==========================================
fn run_step_sweep() -> anyhow::Result<()> {
    println!("=== Step Potential Sweep (T vs E) [Parallel Mode] ===");
    let mut file = fs::File::create("output/transmission.csv")?;
    writeln!(file, "Energy,T_Sim,T_Theory")?;

    let v0 = 0.8;
    let points = 40;
    
    // 1. 预先生成所有要扫描的能量点
    let energies: Vec<f64> = (0..points)
        .map(|i| 0.5 + i as f64 * (1.5 / (points as f64 - 1.0)))
        .collect();

    println!("Computing {} points in parallel... (This may take a moment)", points);

    // 2. [核心修改] 使用 par_iter() 并行计算
    // 这会自动利用你所有的 CPU 核心
    let results: Vec<(f64, f64, f64)> = energies.par_iter().map(|&e| {
        // 每个线程内部拥有独立的 state，互不干扰
        let mut state = QuantumState::new();
        let k = (2.0 * e).sqrt();
        state.init_gaussian(k, 0.0, 20.0, 50.0, 6.0);
        
        let pot = step::StepPotential::new(N/2, v0);

        // 步数优化：对于扫描，跑 10000 步通常够了 (假设 N=200, dt=0.01)
        // 如果是 N=600, dt=0.0025，这里可能需要 40000
        let sim_steps = if N > 300 { 40000 } else { 10000 };

        for _ in 0..sim_steps {
            solver::step_leapfrog(&mut state, &pot);
        }
        state.update_density();

        // 统计透射率
        let mut t_sim = 0.0;
        for x in (N/2)..N {
            for y in 0..N { t_sim += state.density[[x, y]] * DX * DX; }
        }

        let t_theory = if e > v0 {
            let k1 = (2.0 * e).sqrt();
            let k2 = (2.0 * (e - v0)).sqrt();
            4.0 * k1 * k2 / (k1 + k2).powi(2)
        } else { 0.0 };

        // 返回结果元组
        (e, t_sim, t_theory)
    }).collect(); // collect 会自动等待所有线程完成

    // 3. 结果写回文件并绘图
    let mut data_sim = Vec::new();
    let mut data_theory = Vec::new();

    for (e, t_sim, t_theory) in results {
        writeln!(file, "{:.4},{:.4},{:.4}", e, t_sim, t_theory)?;
        data_sim.push((e, t_sim));
        data_theory.push((e, t_theory));
    }
    
    // 此时按能量排序一下，防止并行乱序导致连线混乱
    data_sim.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    data_theory.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    vis::plot_transmission_curve("output/transmission_curve.png", &data_sim, &data_theory)?;
    println!("Parallel sweep done.");
    Ok(())
}
// ==========================================
// 3. 双缝干涉分析 (屏幕强度分布)
//    这里不需要扫描能量，而是分析"屏幕"上的分布
// ==========================================
fn run_double_slit_analysis() -> anyhow::Result<()> {
    println!("=== Double Slit Analysis (Interference Pattern) ===");
    let e: f64 = 1.0; // 固定能量
    let mut state = QuantumState::new();
    let k = (2.0 * e).sqrt();
    state.init_gaussian(k, 0.0, 20.0, 50.0, 6.0);
    
    // 障碍物在中间
    let barrier_idx = N / 2;
    let pot = double_slit::DoubleSlit::new(barrier_idx, 8.0, 16.0, 20.0);

    println!("Simulating propagation...");
    let bar = ProgressBar::new(TOTAL_STEPS as u64);
    for _ in 0..TOTAL_STEPS {
        solver::step_leapfrog(&mut state, &pot);
        bar.inc(1);
    }
    bar.finish();
    state.update_density();

    // 统计屏幕上的强度分布
    // 我们取最右侧的一列 (x = N - 5) 作为屏幕
    let screen_x = N - 10; 
    let mut pattern = Vec::new();
    let mut file = fs::File::create("output/interference_pattern.csv")?;
    writeln!(file, "Y,Intensity")?;

    for y in 0..N {
        let y_pos = y as f64 * DX;
        // 为了减少噪音，我们可以平均几列
        let mut intensity = 0.0;
        for dx in 0..5 {
            intensity += state.density[[screen_x + dx, y]];
        }
        intensity /= 5.0;
        
        writeln!(file, "{},{}", y_pos, intensity)?;
        pattern.push((y_pos, intensity));
    }

    vis::plot_interference_pattern("output/interference_pattern.png", &pattern)?;
    println!("Analysis done. See output/interference_pattern.png");
    Ok(())
}

// ==========================================
// 4. 谐振子扫描 (振幅 vs 能量)
// ==========================================
fn run_oscillator_sweep() -> anyhow::Result<()> {
    println!("=== Oscillator Sweep (Amplitude vs Energy) [Parallel] ===");
    let mut file = fs::File::create("output/oscillator_amplitude.csv")?;
    writeln!(file, "Energy,Max_Amplitude")?;

    let points = 20;
    
    // 生成能量点
    let energies: Vec<f64> = (0..points)
        .map(|i| 0.5 + i as f64 * 0.2)
        .collect();

    println!("Computing {} points in parallel...", points);

    //进度条
    let bar = ProgressBar::new(points as u64);
    bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})").unwrap());

    // 并行计算
    let mut results: Vec<(f64, f64)> = energies.par_iter().map(|&e| {
        let mut state = QuantumState::new();
        let k_energy = (2.0 * e).sqrt();
        
        // 初始位置在中心，给它初速度
        state.init_gaussian(k_energy, 0.0, 50.0, 50.0, 4.0);
        
        let k_spring = 0.01;
        let pot = oscillator::HarmonicOscillator::new(k_spring);

        let mut max_x = 0.0;
        let center = 50.0; 

        for t in 0..TOTAL_STEPS {
            solver::step_leapfrog(&mut state, &pot);
            
            if t % 20 == 0 {
                // 简化版的 update_density，只为了求 <x>
                let mut exp_x = 0.0;
                let mut norm = 0.0;
                // 这里为了性能，可以只计算中心区域，或者全算
                for x in 0..N {
                    for y in 0..N {
                        // 手动计算模方，减少内存读写
                        let r = state.real[[x, y]];
                        let i = state.imag[[x, y]];
                        let p = r*r + i*i;
                        
                        norm += p;
                        exp_x += x as f64 * DX * p;
                    }
                }
                
                if norm > 1e-9 {
                    exp_x /= norm;
                    let dist = (exp_x - center).abs();
                    if dist > max_x { max_x = dist; }
                }
            }
        }
        
        //每算完一个点，进度条加一
        bar.inc(1);
        
        (e, max_x)
    }).collect();

    bar.finish(); // 结束进度条

    // 排序并写入
    results.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut plot_data = Vec::new();
    for (e, max_x) in results {
        writeln!(file, "{:.4},{:.4}", e, max_x)?;
        plot_data.push((e, max_x));
    }

    vis::plot_amplitude_energy("output/oscillator_amplitude.png", &plot_data)?;
    println!("Oscillator sweep done.");
    Ok(())
}