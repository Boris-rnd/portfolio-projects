fn main() {
    let z = Complex::new(10., 2.0f32.sqrt());
    let z2 = j*2.+10.;
    
    let m1 = Matrix {
        rows: vec![
            vec![j*0.+1.,j*0.+2.],
            vec![j*0.+4.,j*0.+5.],
            vec![j*0.+6.,j*0.+7.],
        ],
    };
    let m2 = Matrix {
        rows: vec![
            vec![j*0.+9.,j*0.+10., j*0.+12.],
            vec![j*0.+4.,j*0.+5., j*0.+13.],
        ],
    };
    println!("{} \n\n{}\n\n", &m1,&m2);
    println!("{}", m2*m1);
}

#[derive(Copy, Clone)]
struct Complex {
    pub a: f32,
    pub b: f32,
}
impl Complex {
    pub const fn new(a: f32, b: f32) -> Self {
        Self { a, b }
    }
    pub fn conjugate(&self) -> Self {
        Self::new(self.a, -self.b)
    }
    pub fn module(&self) -> f32 {
        (self.a*self.a+self.b*self.b).sqrt()
    }
    pub fn theta(&self) -> f32 {
        (self.b/self.a).atan()
    }
    pub fn assume_real(&self) -> Option<f32> {
        if self.b!=0. {None}
        else {Some(self.a)}
    }
}
impl std::fmt::Debug for Complex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        if self.a != 0. && self.b != 0. {
            f.write_fmt(format_args!("{}+{}i", self.a, self.b))?
        } else if self.a != 0. {
            f.write_fmt(format_args!("{}", self.a))?
        } else if self.b != 0. {
            f.write_fmt(format_args!("{}i", self.b))?
        } else {
            f.write_fmt(format_args!("0"))?
        }
        Ok(())
    }
}
impl std::ops::Mul<Complex> for Complex {
    type Output = Complex;

    fn mul(self, rhs: Complex) -> Self::Output {
        let a=self.a;let b=self.b;let c=rhs.a;let d=rhs.b;
        // (a+ib)(c+id)
        // ac+aid+ibc+ibid
        // ac-bd + i(ad+bc)
        Self::new(a*c-b*d, a*d+b*c)
    }
}
impl std::ops::Mul<f32> for Complex {
    type Output = Complex;

    fn mul(self, rhs: f32) -> Self::Output {
        let a=self.a;let b=self.b;let c=rhs;
        Self::new(a*c, b*c)
    }
}
impl std::ops::Add<f32> for Complex {
    type Output = Complex;

    fn add(self, rhs: f32) -> Self::Output {
        Self::new(self.a+rhs, self.b)
    }
}

const j: Complex = Complex::new(0., 1.);

#[derive(Clone)]
struct Matrix {
    pub rows: Vec<Vec<Complex>>
}
impl Matrix {
    pub fn with_dimensions(rows: usize, columns: usize) -> Self {
        Self {rows: vec![vec![j*0.; columns]; rows]}
    }
    pub fn dims(&self) -> (usize, usize) {
        (self.rows.len(), self.rows[0].len())
    }
}
impl std::fmt::Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for r in &self.rows {
            f.write_fmt(format_args!("{:?}\n", r))?
        }
        Ok(())
    }
}

impl std::ops::Mul<Matrix> for Matrix {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Self::Output {
        assert!(rhs.dims().0 == self.dims().1 && rhs.dims().1 == self.dims().0);
        let mut new = Matrix::with_dimensions(self.dims().0, self.dims().1);
        for (y, _) in self.rows.iter().enumerate() {
            for x in 0..self.rows[0].len() {
                for i in 0..rhs.rows[y].len() {
                    println!("{},{} = {:?}*{:?}", x,y,self.rows[y][i], rhs.rows[x][i]);
                    new.rows[y][x] = rhs.rows[x][y] * self.rows[y][i]
                }
            }
        }
        new
    }
    
}
