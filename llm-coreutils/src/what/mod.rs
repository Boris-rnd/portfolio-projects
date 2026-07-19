pub mod analysis;
pub mod cli;
pub mod formatting;

pub use analysis::{build_prompt_body, collect_repo_context};
pub use cli::{Args, build_system_prompt, resolve_output_mode};
pub use formatting::format_terminal_output;
