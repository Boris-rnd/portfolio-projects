// use std::{
//     cell::OnceCell,
//     ops::{Add, Mul},
//     str::FromStr,
// };

// use astro_float::BigFloat;
// use kbonaci::*;
// // use num_bigfloat::BigFloat;
// use num_bigint::BigUint;

// // #[global_allocator]
// // static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
}
//     #[allow(static_mut_refs)]
//     // unsafe { CONSTS.set([
//     // // let sqrt5 = BigFloat::from_i8(5).sqrt();
//     // // let phi = (BigFloat::from_u8(1) + sqrt5) / BigFloat::from_i8(2);
//     // BigFloat::from_i8(5).sqrt(),
//     // (BigFloat::from_u8(1) + BigFloat::from_i8(5).sqrt()) / BigFloat::from_i8(2),
//     // ]).unwrap() };
//     divan::main();
// }

// #[divan::bench(types = [
//     BigInt,
//     num_bigint::BigUint,
// ])]
// fn add_big<T: Add>(bencher: divan::Bencher) {
//     bencher
//         .with_inputs(|| {
//             (
//                 BigInt::from_uint(num_bigint::BigUint::new(vec![876543234u32; 1000000])),
//                 BigInt::from_uint(num_bigint::BigUint::new(vec![876543234u32; 1000000])),
//             )
//         })
//         .bench_values(|(a, b)| a + b);
// }
// #[divan::bench(types = [
//     BigInt,
//     num_bigint::BigUint,
// ])]
// fn add_big_num<T: Add>(bencher: divan::Bencher) {
//     bencher
//         .with_inputs(|| {
//             (
//                 num_bigint::BigUint::new(vec![876543234u32; 1000000]),
//                 num_bigint::BigUint::new(vec![876543234u32; 1000000]),
//             )
//         })
//         .bench_values(|(a, b)| a + b);
// }

// #[divan::bench(types = [
//     BigInt,
//     num_bigint::BigUint,
// ])]
// fn mul_big_num<T: Mul + FromStr>(bencher: divan::Bencher) {
//     bencher
//         .with_inputs(|| {
//             (
//                 T::from_str("987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234").ok().unwrap(),
//                 T::from_str("987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234987654321234567890987654321209876543234567898765432345678987654323456789098765432123456789009876543212345678909876543234567890987654321234567890876543234").ok().unwrap(),
//             )
//         })
//         .bench_values(|(a,b)| a * b);
// }

// // #[divan::bench(types = [
// //     BigInt,
// //     num_bigint::BigUint,
// // ])]
// // fn mul<T: Mul>(bencher: divan::Bencher) {
// //     bencher
// //         .with_inputs(|| {
// //             (
// //                 BigInt::from_uint(num_bigint::BigUint::new(vec![876543234u32; 1000000])),
// //                 BigInt::from_uint(num_bigint::BigUint::new(vec![876543234u32; 1000000])),
// //             )
// //         })
// //         .bench_values(|(a,b)| a * b);
// // }
// #[divan::bench()]
// fn pow(bencher: divan::Bencher) {
//     bencher
//         .with_inputs(|| {
//             (
//                 num_bigint::BigUint::new(vec![10; 1]),
//                 num_bigint::BigUint::new(vec![1000; 1]),
//             )
//         })
//         .bench_values(|(a, b)| a.pow(b.to_u32_digits()[0]));
// }

// #[divan::bench()]
// fn my_pow(bencher: divan::Bencher) {
//     bencher
//         .with_inputs(|| {
//             (
//                 BigInt::from_uint(num_bigint::BigUint::new(vec![10; 1])),
//                 BigInt::from_uint(num_bigint::BigUint::new(vec![1000; 1])),
//             )
//         })
//         .bench_values(|(a, b)| a.pow(b.nums[0] as u32));
// }

// #[divan::bench(args = [1, 2, 4, 8, 16, 32, 10000])]
// fn fast_fib(n: usize) -> BigUint {
//     big_kbonacci(n, 2)
// }

// #[divan::bench(args = [10usize.pow(13),10usize.pow(15),10usize.pow(18),10usize.pow(19),10usize.pow(20),10usize.pow(30)] )]
// fn explicit_fib(n: usize) -> astro_float::BigFloat {
//     _explicit_fib(n)
// }

// // static mut CONSTS: OnceCell<[BigFloat; 2]> = OnceCell::new();

// // fn _explicit_fib(n: u64) -> BigFloat {
// //     #[allow(static_mut_refs)]
// //     let phi = unsafe { CONSTS.get().unwrap() }[1];
// //     #[allow(static_mut_refs)]
// //     let sqrt5 = unsafe { CONSTS.get().unwrap() }[0];

// //     let top = phi.pow(&BigFloat::from_u64(n))
// //         - (BigFloat::from_u8(1) - phi.pow(&BigFloat::from_u64(n)));

// //     (top / (sqrt5 * BigFloat::from_u8(2)))
// // }

// // #[divan::bench(args = [num_bigint::BigUint::new(vec![100])])]
// // fn add_num_big(n: usize) -> BigInt {
// //     BigInt::from_uint(num_bigint::BigUint::new(vec![1u32; n]))+BigInt::from_uint(num_bigint::BigUint::new(vec![1u32; n]))
// // }

// #[divan::bench(args = [1, 2, 4, 8, 16, 32])]
// fn fib(n: u64) -> u64 {
//     fibonacci(n)
// }
