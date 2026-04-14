#![feature(vec_push_within_capacity)]
//#![feature(bigint_helper_methods)]
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(unused, dead_code, incomplete_features)]

pub mod big_int;
pub mod big_fl;
pub mod mat;
use std::cell::OnceCell;

use astro_float::{BigFloat, Consts, RoundingMode};
pub use big_int::*;
// use dashu_float::DBig;
// use dashu_int::UBig;
use num_bigint::BigUint;

pub fn kbonacci(n: usize, k: u32) -> usize {
    if n == 0 {
        return 0;
    }
    if n <= k as usize {
        return 1;
    }

    let mut seq = vec![0; n + 1];
    seq[0] = 0;
    for i in 1..=k as usize {
        seq[i] = 1;
    }

    for i in (k as usize + 1)..=n {
        seq[i] = seq[i - 1] * 2 - seq[i - k as usize - 1];
    }

    seq[n]
}

pub fn big_kbonacci(n: usize, k: usize) -> BigUint {
    if n == 0 {
        return BigUint::from(0u8);
    }
    let k_big = BigUint::from(k as u8);
    if n <= k {
        return BigUint::from(1u8);
    }

    let mut seq = vec![BigUint::from(0u8); n + 1];
    seq[0] = BigUint::from(0u8);
    let mut i = 1;
    while i <= k {
        seq[i] = BigUint::from(1u8);
        i += 1;
    }

    let mut i = k + 1;
    while i <= n {
        seq[i] = (&seq[i - 1] << 1) - &seq[i - k - 1];
        i += 1;
    }

    seq[n].clone()
}

static mut CONSTS: OnceCell<Box<[BigFloat]>> = OnceCell::new();

pub fn _explicit_fib(n: usize) -> BigFloat {
    #[allow(static_mut_refs)]
    let phi = &unsafe {
        CONSTS.get_or_init(|| {
            let sqrt5 = astro_float::BigFloat::from(5).sqrt(1000, astro_float::RoundingMode::None);
            let phi = (astro_float::BigFloat::from(1).add_full_prec(&sqrt5)); //.div(&astro_float::BigFloat::from(2), 1000, RoundingMode::None);

            // let anti_phi = (DBig::from(1) - &sqrt5) / DBig::from(2);
            Box::new([sqrt5, phi])
        })
    }[1];
    #[allow(static_mut_refs)]
    let sqrt5 = &unsafe { CONSTS.get().unwrap() }[0];
    // #[allow(static_mut_refs)]
    // let anti_phi = &unsafe { CONSTS.get().unwrap() }[2];

    let phi_pow = phi.powi(n, 1000, RoundingMode::None);
    let top = &phi_pow;

    (top.div(
        &sqrt5.mul_full_prec(&astro_float::BigFloat::from(2).powi(n, 1000, RoundingMode::None)),
        1000,
        RoundingMode::None,
    ))
}
pub fn big_explicit_fib(n: BigFloat, p: usize) -> BigFloat {
    #[allow(static_mut_refs)]
    let phi = &unsafe {
        CONSTS.get_or_init(|| {
            let sqrt5 = astro_float::BigFloat::from(5).sqrt(p, astro_float::RoundingMode::None);
            let phi = (astro_float::BigFloat::from(1).add_full_prec(&sqrt5)); //.div(&astro_float::BigFloat::from(2), 1000, RoundingMode::None);

            // let anti_phi = (DBig::from(1) - &sqrt5) / DBig::from(2);
            Box::new([sqrt5, phi])
        })
    }[1];
    #[allow(static_mut_refs)]
    let sqrt5 = &unsafe { CONSTS.get().unwrap() }[0];
    // #[allow(static_mut_refs)]
    // let anti_phi = &unsafe { CONSTS.get().unwrap() }[2];

    let phi_pow = phi.pow(&n, p, RoundingMode::None, &mut Consts::new().unwrap());
    let top = &phi_pow;

    (top.div(
        &sqrt5.mul_full_prec(&astro_float::BigFloat::from(2).pow(
            &n,
            p,
            RoundingMode::None,
            &mut Consts::new().unwrap(),
        )),
        p,
        RoundingMode::None,
    ))
}

/// Serves as reference
pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 2) + fibonacci(n - 1)
    }
}
