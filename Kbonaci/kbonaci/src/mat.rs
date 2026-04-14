use std::{
    mem::MaybeUninit,
    ops::{Add, Mul},
};

#[derive(Clone)]
pub struct DiagMatrix<T, const LEN: usize> {
    pub values: [T; LEN],
}
impl<T, const LEN: usize> DiagMatrix<T, LEN> {
    pub fn new(values: [T; LEN]) -> Self {
        Self { values }
    }
}
impl<T: Default + Copy, const LEN: usize> DiagMatrix<T, LEN> {
    pub fn to_mat(&self) -> Matrix<T, LEN, LEN> {
        let mut out = Matrix::new([[T::default(); LEN]; LEN]);
        for i in 0..LEN {
            out.values[i][i] = self.values[i];
        }
        out
    }
}

#[derive(Clone)]
pub struct Matrix<T, const ROW: usize, const COL: usize> {
    pub values: [[T; COL]; ROW],
}

impl<T, const ROW: usize, const COL: usize> Matrix<T, ROW, COL> {
    pub const fn new(values: [[T; COL]; ROW]) -> Self {
        Self { values }
    }
    pub const fn rows(&self) -> usize {
        ROW
    }
    pub const fn cols(&self) -> usize {
        COL
    }
}

fn eigen_decomposition(m: [[f32; 2]; 2]) -> ([[f32; 2]; 2], [f32; 2]) {
    // Polynôme caractéristique : λ² - tr λ + det = 0
    let trace = m[0][0] + m[1][1];
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];

    // Valeurs propres : λ = (tr ± √(tr² - 4det)) / 2
    let delta = (trace * trace - 4.0 * det).sqrt();
    let lambda1 = (trace + delta) / 2.0;
    let lambda2 = (trace - delta) / 2.0;

    // Trouver vecteurs propres
    fn eigenvector(m: [[f32; 2]; 2], lambda: f32) -> [f32; 2] {
        // Résoudre (M - λI)v = 0
        let a = m[0][0] - lambda;
        let b = m[0][1];
        let c = m[1][0];
        let d = m[1][1] - lambda;

        // Choisir une équation non triviale
        if b.abs() > 1e-8 {
            [1.0, -a / b]
        } else if c.abs() > 1e-8 {
            [-d / c, 1.0]
        } else {
            [1.0, 0.0] // cas dégénéré
        }
    }

    let v1 = eigenvector(m, lambda1);
    let v2 = eigenvector(m, lambda2);

    // Matrice P (vecteurs propres en colonnes)
    let p = [[v1[0], v2[0]], [v1[1], v2[1]]];

    // Matrice D (diagonale avec valeurs propres)
    let d = [lambda1, lambda2];

    (p, d)
}

impl<T: Default + Copy + Mul<Output = T> + Add<Output = T>, const ROW: usize>
    faer::traits::num_traits::Pow<usize> for Matrix<T, ROW, ROW>
{
    type Output = Self;

    default fn pow(mut self, rhs: usize) -> Self::Output {
        // TODO Fast exp
        let start = self.clone();
        for i in 0..rhs {
            self = &self * &start;
        }
        self
    }
}
impl faer::traits::num_traits::Pow<f32> for Matrix<f32, 2, 2> {
    type Output = Self;

    fn pow(mut self, rhs: f32) -> Self::Output {
        let (p, d) = eigen_decomposition(self.values);
        let p = Matrix::new(p);
        let diag = DiagMatrix::new(d);
        let d_pow = diag.pow(rhs).to_mat();
        &(&p * &d_pow) * &p.inverse().unwrap()
    }
}

impl<RHS: Copy, T: Copy + faer::traits::num_traits::Pow<RHS, Output = T>, const LEN: usize>
    faer::traits::num_traits::Pow<RHS> for DiagMatrix<T, LEN>
{
    type Output = Self;

    fn pow(mut self, rhs: RHS) -> Self::Output {
        for i in 0..LEN {
            self.values[i] = self.values[i].pow(rhs);
        }
        self
    }
}

impl<const SIZE: usize> Matrix<f32, SIZE, SIZE>
where
    [(); (SIZE > 1) as usize]: Sized,
{
    pub fn determinant(&self) -> f32 {
        if SIZE==2 {
            return self.values[0][0] * self.values[1][1] - self.values[0][1] * self.values[1][0];
        }
        let mut det = 1.0;
        let mut matrix = self.values;

        // Gaussian elimination to compute determinant
        for i in 0..SIZE {
            // Find maximum in this column
            let mut max_row = i;
            for k in i + 1..SIZE {
                if matrix[k][i].abs() > matrix[max_row][i].abs() {
                    max_row = k;
                }
            }

            // If diagonal element is zero, determinant is zero
            if matrix[max_row][i] == 0.0 {
                return 0.0;
            }

            // Swap maximum row with current row
            if max_row != i {
                for k in 0..SIZE {
                    let temp = matrix[i][k];
                    matrix[i][k] = matrix[max_row][k];
                    matrix[max_row][k] = temp;
                }
                det = -det; // determinant changes sign
            }

            // Reduce rows below
            for k in i + 1..SIZE {
                let factor = matrix[k][i] / matrix[i][i];
                for j in i..SIZE {
                    matrix[k][j] -= factor * matrix[i][j];
                }
            }
            det *= matrix[i][i];
        }

        det
    }
    pub fn inverse(&self) -> Option<Self>  where [(); SIZE * 2]: Sized {
        let det = self.determinant();
        if det.abs() < 1e-8 {
            return None;
        }
        let inv_det = 1.0 / det;
        if SIZE == 2 {
            // Cheat case for 2x2 matrix
            let mut out = Self::default();
            out.values[0][0] = self.values[1][1] * inv_det;
            out.values[0][1] = -self.values[0][1] * inv_det;
            out.values[1][0] = -self.values[1][0] * inv_det;
            out.values[1][1] = self.values[0][0] * inv_det;
            return Some(out);
        }

        // Create augmented matrix [A | I]
        let mut aug = [[0.0; SIZE * 2]; SIZE];
        for i in 0..SIZE {
            for j in 0..SIZE {
                aug[i][j] = self.values[i][j];
            }
            aug[i][i + SIZE] = 1.0;
        }

        // Perform Gaussian elimination
        for i in 0..SIZE {
            // Find maximum in this column
            let mut max_row = i;
            for k in i + 1..SIZE {
                if aug[k][i].abs() > aug[max_row][i].abs() {
                    max_row = k;
                }
            }

            // If diagonal element is zero, matrix is singular
            if aug[max_row][i] == 0.0 {
                return None;
            }

            // Swap maximum row with current row
            if max_row != i {
                for k in 0..SIZE * 2 {
                    let temp = aug[i][k];
                    aug[i][k] = aug[max_row][k];
                    aug[max_row][k] = temp;
                }
            }

            // Normalize pivot row
            let pivot = aug[i][i];
            for k in 0..SIZE * 2 {
                aug[i][k] /= pivot;
            }

            // Eliminate other rows
            for k in 0..SIZE {
                if k != i {
                    let factor = aug[k][i];
                    for j in 0..SIZE * 2 {
                        aug[k][j] -= factor * aug[i][j];
                    }
                }
            }
        }

        // Extract inverse matrix from augmented matrix
        let mut inv = [[0.0; SIZE]; SIZE];
        for i in 0..SIZE {
            for j in 0..SIZE {
                inv[i][j] = aug[i][j + SIZE];
            }
        }

        Some(Self::new(inv))
    }

    // pub fn cofactor(&self, row: usize, col: usize) -> f32 {
    //     let mut minor = vec![vec![0.0; SIZE - 1]; SIZE - 1];
    //     let mut m_row = 0;

    //     // Build minor matrix iteratively
    //     for i in 0..SIZE {
    //         if i == row {
    //             continue;
    //         }
    //         let mut m_col = 0;
    //         for j in 0..SIZE {
    //             if j == col {
    //                 continue;
    //             }
    //             minor[m_row][m_col] = self.values[i][j];
    //             m_col += 1;
    //         }
    //         m_row += 1;
    //     }

    //     // Convert minor to fixed-size array
    //     let mut minor_array = [[0.0; SIZE - 1]; SIZE - 1];
    //     for i in 0..SIZE - 1 {
    //         minor_array[i].copy_from_slice(&minor[i]);
    //     }

    //     let sign = if (row + col) % 2 == 0 { 1.0 } else { -1.0 };
    //     sign * Matrix::<f32, { SIZE - 1 }, { SIZE - 1 }>::new(minor_array).determinant()
    // }
}

impl<const ROW: usize, const COL: usize> Mul<f32> for Matrix<f32, ROW, COL> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut out = self.clone();
        for i in 0..ROW {
            for j in 0..COL {
                out.values[i][j] *= rhs;
            }
        }
        out
    }
}

impl<T: PartialEq, const ROW: usize, const COL: usize> PartialEq for Matrix<T, ROW, COL> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
use std::fmt::Debug;
impl<T: Debug, const ROW: usize, const COL: usize> Debug for Matrix<T, ROW, COL> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Matrix")
            .field("values", &self.values)
            .finish()
    }
}

impl<T: Default + Copy, const ROW: usize, const COL: usize> Default for Matrix<T, ROW, COL> {
    fn default() -> Self {
        Self::new([[T::default(); COL]; ROW])
    }
}
// TODO: not depend on default+copy using uninit
impl<T: Default + Copy + Add<Output = T>, const ROW: usize, const COL: usize> std::ops::Add
    for Matrix<T, ROW, COL>
{
    type Output = Self;

    default fn add(self, rhs: Self) -> Self::Output {
        naive_add(&self, &rhs)
    }
}
impl<T: Default + Copy + Add<Output = T>, const M: usize, const N: usize>
    std::ops::Add<&Matrix<T, M, N>> for &Matrix<T, M, N>
{
    type Output = Matrix<T, M, N>;

    fn add(self, rhs: &Matrix<T, M, N>) -> Self::Output {
        naive_add(self, rhs)
    }
}

impl<T: Default + Copy + Add<Output = T>> std::ops::Add for Matrix<T, 4, 4> {
    fn add(self, rhs: Self) -> Self::Output {
        let mut out = Self::default();
        for i in 0..4 {
            for j in 0..4 {
                out.values[i][j] = self.values[i][j] + rhs.values[i][j];
            }
        }
        out
    }
}
pub fn naive_add<T: Default + Copy + Add<Output = T>, const M: usize, const N: usize>(
    a: &Matrix<T, M, N>,
    b: &Matrix<T, M, N>,
) -> Matrix<T, M, N> {
    let mut out = Matrix::<T, M, N>::default();
    for i in 0..M {
        for j in 0..N {
            out.values[i][j] = out.values[i][j] + b.values[i][j];
        }
    }
    out
}

pub fn naive_mul<
    T: Default + Copy + Mul<Output = T> + Add<Output = T>,
    const M: usize,
    const N: usize,
    const P: usize,
>(
    a: &Matrix<T, M, N>,
    b: &Matrix<T, N, P>,
) -> Matrix<T, M, P> {
    let mut out = Matrix::<T, M, P>::default();
    for i in 0..out.rows() {
        for j in 0..out.cols() {
            for k in 0..N {
                out.values[i][j] = out.values[i][j] + a.values[i][k] * b.values[k][j];
            }
        }
    }
    out
}

impl<
    T: Default + Copy + Mul<Output = T> + Add<Output = T>,
    const M: usize,
    const N: usize,
    const P: usize,
> std::ops::Mul<&Matrix<T, N, P>> for &Matrix<T, M, N>
{
    type Output = Matrix<T, M, P>;

    fn mul(self, rhs: &Matrix<T, N, P>) -> Self::Output {
        naive_mul(self, rhs)
    }
}

pub struct DMatrix<T> {
    pub values: Vec<Vec<T>>,
}

impl<T> DMatrix<T> {
    pub const fn new(values: Vec<Vec<T>>) -> Self {
        Self { values }
    }
    pub const fn rows(&self) -> usize {
        self.values.len()
    }
    pub fn cols(&self) -> usize {
        self.values[0].len()
    }
}
impl<T: Default + Copy> DMatrix<T> {
    pub fn default(rows: usize, cols: usize) -> Self {
        Self::new(vec![vec![T::default(); cols]; rows])
    }
}
// TODO: not depend on default+copy using uninit
impl<T: Default + Copy + Add<Output = T>> std::ops::Add for DMatrix<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.rows(), rhs.rows());
        assert_eq!(self.cols(), rhs.cols());
        let mut out = Self::default(self.rows(), self.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.values[i][j] = self.values[i][j] + rhs.values[i][j];
            }
        }
        out
    }
}

impl<T: Default + Copy + Add<Output = T>> std::ops::Add<&DMatrix<T>> for &DMatrix<T> {
    type Output = DMatrix<T>;

    fn add(self, rhs: &DMatrix<T>) -> Self::Output {
        assert_eq!(self.rows(), rhs.rows());
        assert_eq!(self.cols(), rhs.cols());
        let mut out = DMatrix::<T>::default(self.rows(), self.cols());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.values[i][j] = self.values[i][j] + rhs.values[i][j];
            }
        }
        out
    }
}
