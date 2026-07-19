use clap::{ArgAction, Parser, ValueHint};

#[derive(Clone, Copy, Debug)]
pub enum OutputMode {
    Default,
    Short,
    Explain,
    Security,
    Dev,
    HowToRun,
    Json,
}

#[derive(Parser)]
#[command(name = "what", about = "Analyze files quickly with Gemini AI")]
pub struct Args {
    /// Path to the file or directory to analyze
    #[arg(value_hint = ValueHint::FilePath)]
    pub file: std::path::PathBuf,

    /// Ask a specific question about the file
    #[arg(long)]
    pub ask: Option<String>,

    /// Maximum number of lines to include in the prompt
    #[arg(long, default_value_t = 400)]
    pub max_lines: usize,

    /// Give a one- to two-sentence summary
    #[arg(long, action = ArgAction::SetTrue)]
    pub short: bool,

    /// Give a more detailed explanation
    #[arg(long, action = ArgAction::SetTrue)]
    pub explain: bool,

    /// Focus on security risks, permissions, secrets, and suspicious behavior
    #[arg(long, action = ArgAction::SetTrue)]
    pub security: bool,

    /// Focus on development context such as imports, structure, and public API
    #[arg(long, action = ArgAction::SetTrue)]
    pub dev: bool,

    /// Explain how the file should be run or used
    #[arg(long, action = ArgAction::SetTrue)]
    pub how_to_run: bool,

    /// Ask for machine-readable JSON output
    #[arg(long, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Show the generated prompt and metadata before sending it
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    #[arg(short, long, action = ArgAction::SetTrue)]
    pub show_system_prompt: bool,

    /// Print the context and exit without contacting Gemini
    #[arg(long, action = ArgAction::SetTrue)]
    pub show_context: bool,

    /// Print the context and exit without contacting Gemini
    #[arg(long, action = ArgAction::SetTrue)]
    pub dry_run: bool,
}

pub fn resolve_output_mode(args: &Args) -> Result<OutputMode, Box<dyn std::error::Error>> {
    let mut selected = Vec::new();
    if args.short {
        selected.push(OutputMode::Short);
    }
    if args.explain {
        selected.push(OutputMode::Explain);
    }
    if args.security {
        selected.push(OutputMode::Security);
    }
    if args.dev {
        selected.push(OutputMode::Dev);
    }
    if args.how_to_run {
        selected.push(OutputMode::HowToRun);
    }
    if args.json {
        selected.push(OutputMode::Json);
    }

    match selected.as_slice() {
        [] => Ok(OutputMode::Default),
        [mode] => Ok(*mode),
        _ => Err("Please choose only one output mode".into()),
    }
}

pub fn describe_mode(mode: OutputMode) -> &'static str {
    match mode {
        OutputMode::Default => "default",
        OutputMode::Short => "short",
        OutputMode::Explain => "explain",
        OutputMode::Security => "security",
        OutputMode::Dev => "dev",
        OutputMode::HowToRun => "how-to-run",
        OutputMode::Json => "json",
    }
}

pub fn build_system_prompt(mode: OutputMode, is_directory: bool, question: Option<&str>) -> String {
    let mut prompt = String::from(
        "Provide a concise, technical summary in 2-3 sentences. Do not make up content.",
    );
    if is_directory {
        prompt.push_str(" For directories, describe the kind of project or folder this appears to be, highlight important files, and suggest the next 1-3 useful commands.");
    } else if question.is_some() {
        prompt.push_str(" Answer the user's question about the provided file or directory.");
    }
    prompt.push_str(match mode {
        OutputMode::Default => {
            if is_directory {
                " You are a senior software engineer reviewing a directory for quick understanding. Mention the likely project type and the most important files."
            } else {
                " You are a senior software engineer reviewing a file for quick understanding. Mention any notable security concerns only when appropriate."
            }
        }
        OutputMode::Short => {
            " Provide a short answer in 1-2 sentences. Highlight the most important detail."
        }
        OutputMode::Explain => {
            " Provide a more detailed explanation of what this item is, how it works, and why it matters. Keep it technical but concise."
        }
        OutputMode::Security => {
            " Focus on security-relevant findings: secrets, risky commands, suspicious permissions, exposed credentials, unusual behavior, and potential attack surface."
        }
        OutputMode::Dev => {
            " Focus on software development context: imports, structure, public API, dependencies, TODOs, and architectural intent."
        }
        OutputMode::HowToRun => {
            " Explain how the file or directory should be run, used, or interpreted. Mention prerequisites, expected input, and important caveats."
        }
        OutputMode::Json => {
            " Return a compact JSON object with fields: kind, purpose, important_details, and risks. Use plain JSON only, no surrounding prose."
        }
    });
    prompt
}
