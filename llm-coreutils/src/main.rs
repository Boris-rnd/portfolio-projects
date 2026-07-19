mod what;

use clap::Parser;
use display_error_chain::DisplayErrorChain;
use futures::TryStreamExt;
use gemini_rust::Gemini;
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use crate::what::{
    Args, build_prompt_body, build_system_prompt, collect_repo_context, format_terminal_output,
    resolve_output_mode,
};

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();
    match do_main(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let error_chain = DisplayErrorChain::new(e.as_ref());
            tracing::error!(error.debug = ?e, error.chained = %error_chain, "execution failed");
            ExitCode::FAILURE
        }
    }
}

async fn do_main(args: Args) -> Result<(), Box<dyn Error>> {
    let file_path = args.file.clone();
    let full_file_path = fs::canonicalize(&file_path)?;
    let metadata = fs::metadata(&full_file_path)?;

    let mode = resolve_output_mode(&args)?;
    let repo_context = collect_repo_context(&full_file_path)?;
    let is_directory = metadata.is_dir();

    let prompt_body = build_prompt_body(
        &args,
        &full_file_path,
        &metadata,
        repo_context,
        mode,
        is_directory,
    )
    .await?;

    if args.verbose || args.show_context || args.dry_run {
        println!("--- WHAT prompt ---");
        println!("{}", prompt_body);
        println!("--- end prompt ---");
    }

    if args.show_context || args.dry_run {
        return Ok(());
    }

    let api_key = env::var("GEMINI_API_KEY").map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("GEMINI_API_KEY environment variable not set: {e}"),
        )
    })?;

    let client = Gemini::new(api_key)?;
    let system_prompt = build_system_prompt(mode, is_directory, args.ask.as_deref());
    if args.show_system_prompt {
        println!("--- WHAT system prompt ---");
        println!("{}", system_prompt);
        println!("--- end system prompt ---");
    }

    let mut stream = client
        .generate_content()
        .with_system_prompt(system_prompt)
        .with_user_message(&prompt_body)
        .execute_stream()
        .await?;

    #[cfg(debug_assertions)]
    tracing::debug!(%prompt_body);

    while let Some(chunk) = stream.try_next().await? {
        print!("{}", format_terminal_output(&chunk.text()));
        io::stdout().flush()?;
    }
    println!();

    Ok(())
}

struct Solution {}
impl Solution {
    pub fn complex_number_multiply(num1: String, num2: String) -> String {
        let (a,b) = parse(num1);
        let (c,d) = parse(num2);
        // (a+ib)*(c+id)=ac+aid+ibc+ibid
        // = ac-bd + i(ad+bc)
        let r = a*c-b*d;
        let ri = a*d+b*c;
        return format!("{r}+{ri}i");
    }
}
fn parse(n: String) -> (i32,i32) {
    let mut sp = n.split("+");
    let a: i32 = sp.next().unwrap().parse().unwrap();
    let b: i32 = sp.next().unwrap().strip_suffix("i").unwrap().parse().unwrap();
    (a,b)
}
