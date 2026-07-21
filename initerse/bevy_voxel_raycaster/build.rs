fn main() {
    // Safety: no multithreading in sight !
    let shader_dir = "assets/shaders";
    unsafe { std::env::set_var("OUT_DIR", format!("{shader_dir}/compiled")) }
    for file in [] { // "beam", "passthrough", "raytrace", "utils"
        wesl::Wesl::new(shader_dir).build_artifact(&format!("package::{file}").parse().unwrap(), file);

        let path = format!("{}/{file}.wgsl", std::env::var("OUT_DIR").unwrap());
        std::fs::write(path.clone(), load_shader_with_includes(&path)).unwrap();        
    }
}

fn load_shader_with_includes(path: &str) -> String {
    let mut content = std::fs::read_to_string(path).unwrap();

    while let Some(start_idx) = content.find("include!(\"") {
        let rest = &content[start_idx + 10..];
        let end_idx = rest.find("\")").unwrap();
        let include_path = &rest[..end_idx];

        let include_content = std::fs::read_to_string(include_path).unwrap();

        let target = format!("include!(\"{}\")", include_path);
        content = content.replace(&target, &include_content);
    }
    content
}