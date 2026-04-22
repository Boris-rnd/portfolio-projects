use ultraviolet::*;

fn main() {
    use std::f32::consts::SQRT_2;

    let alpha = Complex::new(1.0/SQRT_2, 0.0);
    let beta = Complex::new(0.0, 0.0); // State |0>
    let psi = CVec2::new(alpha, beta);
    
    let zero = Complex::zero();
    let one = Complex::one();
    
    // Pauli X Gate (NOT)
    let pauli_x_gate = CMat2::new(
        CVec2::new(zero, one), // Col 0
        CVec2::new(one, zero), // Col 1
    );
    
    let psi_after_gate = pauli_x_gate * psi;
    println!("State before: {:?}", psi);
    println!("State after X gate: {:?}", psi_after_gate);
    
    // Hadamard Gate
    let h = 1.0 / SQRT_2;
    let hadamard_gate = CMat2::new(
        CVec2::new(Complex::new(h, h), Complex::new(h, h)),
        CVec2::new(Complex::new(h, h), Complex::new(-h, -h)),
    );
    
    let psi_superposition = hadamard_gate * psi;
    println!("State after H gate: {:?}", psi_superposition);
}


// To support gates:

    // let x_gate = Mat2::new(0.0, 1.0, 1.0, 0.0);
    // let y_gate = Mat2::new(0.0, -1.0, 1.0, 0.0);
    // let z_gate = Mat2::new(1.0, 0.0, 0.0, -1.0);
    // let h_gate = Mat2::new(1.0/SQRT_2, 1.0/SQRT_2, 1.0/SQRT_2, -1.0/SQRT_2);
    // let s_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let t_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let cnot_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let swap_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let toffoli_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_z_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_y_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_x_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_s_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_t_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_cnot_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_swap_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);
    // let controlled_toffoli_gate = Mat2::new(1.0, 0.0, 0.0, 1.0);