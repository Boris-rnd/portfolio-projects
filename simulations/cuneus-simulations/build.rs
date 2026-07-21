fn main() {
    // Safety: no multithreading in sight !
    unsafe { std::env::set_var("OUT_DIR", "./shaders/compiled") }
    wesl::Wesl::new("shaders")
        .build_artifact(&"package::particle-basin".parse().unwrap(), "particle-basin");

    std::fs::write("./shaders/compiled/particle-basin.wgsl", load_shader_with_includes("./shaders/compiled/particle-basin.wgsl")).unwrap();
    // use std::env;
    // let out_dir = env::var("OUT_DIR").unwrap();
    // std::fs::write("a.txt", out_dir.clone()).unwrap();
    // dbg!(out_dir);
}

fn load_shader_with_includes(path: &str) -> String {
    let mut content = std::fs::read_to_string(path).unwrap();

    while let Some(start_idx) = content.find("include!(\"") {
        let rest = &content[start_idx + 10..];
        let end_idx = rest.find("\")").unwrap();
        let include_path = &rest[..end_idx];

        let include_content = std::fs::read_to_string(include_path).unwrap();

        // Remplace la ligne par le contenu du fichier
        let target = format!("include!(\"{}\")", include_path);
        content = content.replace(&target, &include_content);
    }
    content
}
