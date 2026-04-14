// pub struct BigFl {
//     pub 
// }

pub struct Myf32 {
    pub inner: u32,
}
impl Myf32 {
    pub const fn new(sign: bool, exponent: u8, fraction: u32) -> Self {
        assert!(fraction < (1<<23));
        let inner = ((sign as u32) << 31) | ((exponent as u32) << 23) | (fraction & ((1<<23)-1));
        Self { inner }
    }
    pub const fn from_i8(value: i8) -> Self {
        let sign = value < 0;
        let exponent = 127;
        let fraction = value.unsigned_abs() as u32;
        Self::new(sign, exponent, fraction)
    }
    pub fn to_i8(&self) -> i8 {
        if self.is_NaN() || self.exponent() != 127 {
            panic!("Cannot convert to i8");
        }
        let frac = self.fraction();
        if frac > i8::MAX as u32 + (self.sign() as u32) {
            panic!("Overflow when converting to i8");
        }
        if self.sign() {
            -(frac as i8)
        } else {
            frac as i8
        }
    }
    #[allow(non_snake_case)]
    pub const fn is_NaN(&self) -> bool {
        self.exponent() == 255 && self.fraction() == 0
    }

    pub const fn sign(&self) -> bool {
        self.inner & (1<<31) != 0
    }
    pub const fn exponent(&self) -> u8 {
        ((self.inner >> 23) & 0xff) as u8
    }
    pub const fn fraction(&self) -> u32 {
        self.inner & ((1<<23)-1)
    }
}

// Format	        Bits for the encoding		Exponent
// bias	Bits
// precision	Number of
// decimal digits
// Sign	Exponent	Significand	Total
// Half (binary16)	1	5	10	16	15	11	~3.3
// Single (binary32)	1	8	23	32	127	24	~7.2
// Double (binary64)	1	11	52	64	1023	53	~15.9
// x86 extended	1	15	64	80	16383	64	~19.2
// Quadruple (binary128)	1	15	112	128	16383	113	~34.0
// Octuple (binary256)	1	19	236	256	262143	237	~71.3

// Final value: (s*b^e)/(b^p-1)

// impl std::ops::Add for Myf32 {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self::Output {

//     }
// }