pub use cuneus::egui::SliderClamping;
pub use bytemuck::{Pod, Zeroable};
pub use cuneus::compute::*;
pub use cuneus::prelude::*;
pub use cuneus::winit::keyboard::Key;
pub use cuneus::winit::keyboard::KeyCode;
pub use cuneus::{Core, RenderKit, ShaderApp, ShaderManager, UniformProvider} ;

pub fn create_compute_shader<T: bytemuck::Pod>(core: &Core, config: ComputeConfiguration, params: T, path: &str) -> ComputeShader {
    // Using the macro expansion to not have to recompile everytime changing the shader
    let mut config = config;
    let caller_file = file!();
    let caller_dir = match caller_file.rfind('/') {
        Some(pos) => &caller_file[..pos],
        None => match caller_file.rfind('\\') {
            Some(pos) => &caller_file[..pos],
            None => "",
        },
    };
    let hot_reload_path = if caller_dir.is_empty() {
        format!("../shaders/{}.wgsl", path)
    } else {
        format!("{}/../shaders/{}.wgsl", caller_dir, path)
    };
    config.hot_reload_path = Some(std::path::PathBuf::from(hot_reload_path.clone()));
    #[cfg(debug_assertions)]
    let compute_shader = ComputeShader::from_builder(core, &std::fs::read_to_string(&hot_reload_path).unwrap(), config);
    #[cfg(not(debug_assertions))]
    let compute_shader = ComputeShader::from_builder(core, &std::fs::read_to_string(&hot_reload_path).unwrap(), config);
    // let compute_shader = ComputeShader::from_builder(core, include_str!(hot_reload_path), config);
    compute_shader.set_custom_params(params, &core.queue);

    compute_shader
}
