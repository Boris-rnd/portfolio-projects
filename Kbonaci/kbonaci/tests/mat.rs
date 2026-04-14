use kbonaci::mat::Matrix;

#[test]
fn mat_add() {
    let a = Matrix::new([
        [1,3,5,-2],
        [-2,0,-3,2],
    ]);
    let b = Matrix::new([
        [2,1,-3],
        [3,-1,0],
        [0,-2,2],
        [2,-3,-4],
    ]);
    assert_eq!(&a*&b, Matrix::new([
        [7, -6, 15],
        [0, -2, -8]
    ]));
    let a = Matrix::new([[4.0f32; 4]; 4]);
    assert_eq!(&a*&a, kbonaci::mat::naive_mul(&a, &a));
    let a = Matrix::new([[4.0f32; 4]; 4]);
    assert_eq!(a.clone()+a.clone(), kbonaci::mat::naive_add(&a, &a));
}