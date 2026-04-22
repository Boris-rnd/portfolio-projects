#![allow(
    unused_parens,
    unused_braces,
    dead_code,
    unused_import_braces,
    unused_imports,
    unused_variables
)]

use easy_wgpu::Rect;

#[cfg(not(feature = "testing"))]
fn main() {
    let mut app = easy_wgpu::ExampleApp {};
    easy_wgpu::run(&mut app);
}

// Sadly I need this because winit doesn't want to be launched from any thread :c
#[cfg(feature = "testing")]
mod test {
    include!("../tests/mod.rs");
}
#[cfg(feature = "testing")]
fn main() {
    test::run_tests()
}
