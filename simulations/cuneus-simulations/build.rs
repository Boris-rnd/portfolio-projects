use std::{ffi::OsString, path::PathBuf};

fn main() {
    // Safety: no multithreading in sight !
    unsafe { std::env::set_var("OUT_DIR", "./shaders/compiled") }
    for f in std::fs::read_dir("shaders/libs").unwrap() {
        let fname = f.unwrap().path();
        if let Err(e) = compile_shader(fname.clone()) {
            println!("cargo:warning=Failed to compile {}: {e}", f.unwrap().file_name().display());
        }
    }
    for f in std::fs::read_dir("shaders").unwrap() {
        let fname = f.unwrap().file_name();
        if let Err(e) = compile_shader(fname.clone()) {
            println!("cargo:warning=Failed to compile {fname:?}: {e}");
        }
    }
    // use std::env;
    // let out_dir = env::var("OUT_DIR").unwrap();
    // std::fs::write("a.txt", out_dir.clone()).unwrap();
    // dbg!(out_dir);
}
fn compile_shader(path: PathBuf) -> Result<(), String>{
    let name = path.file_name().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let ext = name.trim().split(".").last().ok_or("Invalid extension")?;
    if ext != "wgsl" && ext != "wesl" {
        println!("cargo:warning=Skipping {name}");
        return Ok(());
    }
    println!("cargo:warning=Compiling {name}");
    let name = name.trim_end().strip_suffix(&format!(".{ext}")).ok_or("Invalid name")?;

    std::fs::write(format!("./shaders/compiled/{name}.wgsl"), load_shader_with_includes(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    wesl::Wesl::new("shaders/compiled")
        .build_artifact(&format!("package::{name}").parse().expect("Invalid artifact"), name)
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn load_shader_with_includes(path: &PathBuf) -> Result<String, String> {
    let mut content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    while let Some(start_idx) = content.find("include!(\"") {
        let rest = &content[start_idx + 10..];
        let end_idx = rest.find("\")").unwrap();
        let include_path = &rest[..end_idx];

        let include_content = std::fs::read_to_string(include_path).map_err(|e| e.to_string())?;

        // Remplace la ligne par le contenu du fichier
        let target = format!("include!(\"{}\")", include_path);
        content = content.replace(&target, &include_content);
    }
    Ok(content)
}
