use std::{fmt::Display, vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigInt {
    pub nums: Vec<u128>,
}
impl BigInt {
    pub fn new(n: u128) -> Self {
        BigInt { nums: vec![n] }
    }
    pub fn to_uint(&self) -> num_bigint::BigUint {
        let mut result = Vec::with_capacity(self.nums.len() * 4);
        for &n in &self.nums {
            // Split each u128 into four u32s in little-endian order
            result.push((n & 0xFFFFFFFF) as u32);
            result.push(((n >> 32) & 0xFFFFFFFF) as u32);
            result.push(((n >> 64) & 0xFFFFFFFF) as u32);
            result.push(((n >> 96) & 0xFFFFFFFF) as u32);
        }

        // Remove leading zeros
        while result.len() > 1 && *result.last().unwrap() == 0 {
            result.pop();
        }

        num_bigint::BigUint::new(result)
    }

    pub fn from_uint(num: num_bigint::BigUint) -> Self {
        let digits = num.to_u32_digits();
        let mut result = Vec::with_capacity(digits.len().div_ceil(4));

        for chunk in digits.chunks(4) {
            let mut value = 0u128;
            for (i, &digit) in chunk.iter().enumerate() {
                value |= (digit as u128) << (32 * i);
            }
            if value != 0 || result.is_empty() {
                result.push(value);
            }
        }
        Self { nums: result }
    }
    // See fast exponentiation
    pub fn pow(&self, x: u32) -> Self {
        let mut result = Self::one();
        let mut base = self.clone();
        let mut exp = x;

        while exp > 0 {
            if exp & 1 == 1 {
                result = result.mul_no_copy(&base);
            }
            base = base.mul_no_copy(&base);
            exp >>= 1;
        }

        result
    }
    pub fn mul_no_copy(&self, rhs: &Self) -> Self {
        let out = big_mul(&self.nums, &rhs.nums);
        Self { nums: out }
    }
    pub fn zero() -> Self {
        BigInt { nums: vec![0] }
    }
    pub fn one() -> Self {
        BigInt { nums: vec![1] }
    }
}

pub fn big_mul(a: &[u128], b: &[u128]) -> Vec<u128> {
    if a.len().max(b.len()) > 32 {
        // panic!();
        return karatsuba_mul(a, b);
    }


    let mut result = vec![0u128; a.len() + b.len()];
    
    for (i, &a_val) in a.iter().enumerate() {
        let mut carry = 0u128;
        for (j, &b_val) in b.iter().enumerate() {
            let pos = i + j;
            
            // Multiply current digits and add carry
            let (prod_low, prod_high) = a_val.carrying_mul(b_val, 0);
            let (sum1, c1) = result[pos].carrying_add(prod_low, false);
            let (sum2, c2) = sum1.carrying_add(carry, false);
            result[pos] = sum2;
            
            // Calculate new carry including all sources
            carry = prod_high + (c1 as u128) + (c2 as u128);
            
            // Handle carry propagation
            let mut carry_pos = pos + 1;
            while carry > 0 && carry_pos < result.len() {
                let (new_val, new_carry) = result[carry_pos].carrying_add(carry, false);
                result[carry_pos] = new_val;
                carry = new_carry as u128;
                carry_pos += 1;
            }
        }
    }

    // Remove trailing zeros
    while result.len() > 1 && *result.last().unwrap() == 0 {
        result.pop();
    }


    result
}

fn shift_left_mult_128(a: &[u128], shift: usize) -> Vec<u128> {
    let mut result = vec![0; shift];
    result.extend_from_slice(a);
    result
}

// pub fn karatsuba_mul(a: &[u128], b: &[u128]) -> Vec<u128> {
//     let n = a.len().max(b.len());
//     if n == 1 {
//         panic!();
//         return big_mul(a, b);
//     }

//     let (a_low, a_high) = a.split_at(n / 2);
//     let (b_low, b_high) = if n / 2 > b.len() {
//         (&b[..], &[0u128][..])
//     } else {
//         b.split_at(n / 2)
//     };

//     let z0 = big_mul(&a_low, &b_low);
//     let z1 = big_mul(&big_sub(&a_low, &a_high), &big_sub(&b_low, &b_high));
//     let z2 = big_mul(&a_high, &b_high);
    
    
//     // a*b=z0*2^n + (z0+z2-z1)*2^(n/2) + z2
//     // But we compute z1 as addition instead of sub, so
//     // a*b=z0*2^n + (z1-z0-z2)*2^(n/2) + z2
//     let fst = shift_left_mult_128(&z0, n);
//     let mid = shift_left_mult_128(&big_sub(&big_add(&z0, &z2), &z1), n / 2);

//     big_add(&big_add(&fst, &mid), &z2)
// }
pub fn karatsuba_mul(a: &[u128], b: &[u128]) -> Vec<u128> {
    let n = a.len().max(b.len());
    
    // Base case - use normal multiplication for small inputs
    if n <= 32 {
        return big_mul(a, b);
    }

    // Split at n/2
    let m = n / 2;

    // Pad inputs to even length
    let mut a_pad = vec![0u128; n];
    let mut b_pad = vec![0u128; n]; 
    a_pad[..a.len()].copy_from_slice(a);
    b_pad[..b.len()].copy_from_slice(b);

    let (a_low, a_high) = a_pad.split_at(m);
    let (b_low, b_high) = b_pad.split_at(m);

    // z0 = a_low * b_low
    let z0 = big_mul(a_low, b_low);

    // z2 = a_high * b_high
    let z2 = big_mul(a_high, b_high);
    
    // z1 = (a_low + a_high)(b_low + b_high) - z0 - z2
    let a_sum = big_add(a_low, a_high);
    let b_sum = big_add(b_low, b_high);
    let z1_temp = big_mul(&a_sum, &b_sum);
    let z0_z2 = big_add(&z0, &z2);
    let z1 = big_sub(&z1_temp, &z0_z2);

    // result = z2*2^(2m) + z1*2^m + z0
    let z2_shift = shift_left_mult_128(&z2, 2 * m);
    let z1_shift = shift_left_mult_128(&z1, m);
    big_add(&big_add(&z0, &z1_shift), &z2_shift)
}

impl std::str::FromStr for BigInt {
    type Err = num_bigint::ParseBigIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let big_uint = num_bigint::BigUint::from_str(s)?;
        Ok(BigInt::from_uint(big_uint))
    }
}

fn big_add(a: &[u128], b: &[u128]) -> Vec<u128> {
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len + 1);
    let mut carry = false;

    for i in 0..max_len {
        let a = if i < a.len() { a[i] } else { 0 };
        let b = if i < b.len() { b[i] } else { 0 };
        let (sum, new_carry) = a.carrying_add(b, carry);
        carry = new_carry;
        result.push_within_capacity(sum).unwrap();
    }

    if carry {
        result.push_within_capacity(1).unwrap();
    }
    result
}
fn big_sub(a: &[u128], b: &[u128]) -> Vec<u128> {
    let max_len = a.len().max(b.len());
    let mut result = Vec::with_capacity(max_len);
    let mut borrow = false;

    for i in 0..max_len {
        let a = if i < a.len() { a[i] } else { 0 };
        let b = if i < b.len() { b[i] } else { 0 };
        let (diff, new_borrow) = a.borrowing_sub(b, borrow);
        borrow = new_borrow;
        result.push_within_capacity(diff).unwrap();
    }

    if borrow {
        result.push_within_capacity(1).unwrap();
    }
    result
}


impl std::ops::Add for BigInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO in place modif, no new alloc

        BigInt { nums: big_add(&self.nums, &rhs.nums) }
    }
}
impl std::ops::Mul for BigInt {
    type Output = Self;

    // x= 10a+b
    // y= 10c+d
    // z = 10e+f
    // x*y*z = (10a+b)(10c+d)(10e+f) = 1000ace + 100ad + 100bc + bd
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_no_copy(&rhs)
    }
}

impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.nums.len() != other.nums.len() {
            return self.nums.len().partial_cmp(&other.nums.len());
        }

        for (a, b) in self.nums.iter().rev().zip(other.nums.iter().rev()) {
            if a != b {
                return a.partial_cmp(b);
            }
        }

        Some(std::cmp::Ordering::Equal)
    }
}

impl Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_uint().to_string().fmt(f)
    }
}
