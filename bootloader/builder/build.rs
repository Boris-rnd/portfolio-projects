include!("src/lib.rs");

fn main() {
    // println!("cargo:rerun-if-changed=../first_stage");
    // println!("cargo:rerun-if-changed=../second_stage");
    // println!("cargo:rerun-i-changed=../second_stage");
    // exec_cmd_wait(format!("cargo build --release --target ../{}", TARGET).as_str(), "../first_stage").expect("Failed compiling first stage");
    // exec_cmd_wait(format!("cargo build --release --target ../{}", TARGET).as_str(), "../second_stage").expect("Failed compiling second stage");
}