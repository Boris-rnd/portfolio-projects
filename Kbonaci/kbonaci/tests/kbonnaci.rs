use astro_float::BigFloat;
// use dashu_float::DBig;
// use dashu_int::{UBig};
use kbonaci::*;
// use num_bigfloat::BigFloat;
use num_bigint::BigUint;

#[test]
fn fib() {
    assert_eq!(kbonacci(0, 2), 0);
    assert_eq!(kbonacci(1, 2), 1);
    assert_eq!(kbonacci(2, 2), 1);
    assert_eq!(kbonacci(3, 2), 2);
    assert_eq!(kbonacci(4, 2), 3);
    assert_eq!(
        big_kbonacci(100, 2),
        <BigUint as std::str::FromStr>::from_str("354224848179261915075").unwrap()
    );
    assert_eq!(
        &_explicit_fib(500).to_string(),
        "1.39423224561697880139724382870407283950070256587697307264108962948325571622863290691557658876222521294125000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001434481239612239411488584531443753111266062283567438380014473026890035570870064242374954094687363445e+104"
    );
    // assert_eq!(_explicit_fib(5), BigFloat::from(5));
}
// fn explicit_fib(n: u64) -> u128 {
//     let sqrt5 = BigFloat::from_i8(5).sqrt();
//     let phi = (BigFloat::from_u8(1) + sqrt5) / BigFloat::from_i8(2);

//     println!("{phi}");
//     let top = phi.pow(&BigFloat::from_u64(n))
//         - (BigFloat::from_u8(1) - phi.pow(&BigFloat::from_u64(n)));
//     println!("{top}");

//     (top / (sqrt5 * BigFloat::from_u8(2))).round(0, num_bigfloat::RoundingMode::Up).to_u128().unwrap()
// }
