#[derive(Default, Debug)]
pub struct Matrix2x2 {
    pub inner: [[f32; 2]; 2],
}
impl Matrix2x2 {
    pub fn new(a: f32,b: f32,c: f32,d: f32) -> Self {
        Self {
            inner: [
                [a,b],
                [c,d],
            ],
        }
    }
    pub fn set(&mut self, cell: Cell, new_value: f32) -> f32 {
        let prev = self.inner[cell.row][cell.col];
        self.inner[cell.row][cell.col] = new_value;
        prev
    }
    pub fn mult(&self, rhd: &Self) -> Self {
        let mut new = Self::default();

        for i in 0..self.inner.len() {
            for j in 0..self.inner[0].len() {
                dbg!(Cell::new(i,j), self.inner[i][j]*rhd.inner[j][i]);
                new.set(Cell::new(i,j), self.inner[i][j]*rhd.inner[j][i]);
            }
        }
        new
    }
}
#[derive(Default, Debug)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}
impl Cell {
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
        }
    }
}


pub fn main() {
    let mut m1 = Matrix2x2::new(2.,4.,9.,3.);
    let mut m2 = Matrix2x2::new(7.,1.,4.,5.);
    dbg!(m2.mult(&m1));
}