use crate::state::QuantumState;
use crate::potentials::Potential;
use crate::constants::*;

pub fn step_leapfrog(state: &mut QuantumState, pot: &dyn Potential) {
    let coeff = 1.0 / (DX * DX);

    // 1. Update Imaginary part (half step)
    // 利用 stencil 运算，这里用 safe indexing，编译器优化后效率很高
    for x in 1..N-1 {
        for y in 1..N-1 {
            let laplacian = (state.real[[x+1, y]] + state.real[[x-1, y]] +
                             state.real[[x, y+1]] + state.real[[x, y-1]] -
                             4.0 * state.real[[x, y]]) * coeff;
            
            let v = pot.get(x, y);
            let h_psi = -0.5 * laplacian + v * state.real[[x, y]];
            
            state.imag[[x, y]] -= DT * h_psi;
        }
    }

    // 2. Update Real part (full step)
    for x in 1..N-1 {
        for y in 1..N-1 {
            let laplacian = (state.imag[[x+1, y]] + state.imag[[x-1, y]] +
                             state.imag[[x, y+1]] + state.imag[[x, y-1]] -
                             4.0 * state.imag[[x, y]]) * coeff;
            
            let v = pot.get(x, y);
            let h_psi = -0.5 * laplacian + v * state.imag[[x, y]];
            
            state.real[[x, y]] += DT * h_psi;
        }
    }
}