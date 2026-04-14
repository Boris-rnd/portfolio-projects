#![allow(unused, dead_code, incomplete_features)]
use std::{cell::OnceCell, str::FromStr};

use astro_float::BigFloat;
use kbonaci::{mat::Matrix, *};
// use dashu_float::DBig;
// use dashu_int::IBig;

static mut CONSTS: OnceCell<[BigFloat; 2]> = OnceCell::new();

fn main() {

    // AX=B
    // X=A^-1B

    let a = Matrix::new([[4.0f32, 2.0, 2.0], [3.0, 1.0, 2.0], [0.; 3]]);
    let b = Matrix::new([[8.0f32], [7.0], [3.0]]);
    let a_inv = a.inverse().unwrap();
    let x = &a_inv * &b;
    println!("A: {:?}\nB: {:?}\nA^-1: {:?}\nX: {:?}", a, b, a_inv, x);
}

fn muls() -> BigInt {
    let mut x = BigInt::from_str("987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234").unwrap();
    for _ in 0..6 {
        x = x.mul_no_copy(&x);
    }
    dbg!(x.nums.len());
    x
}
