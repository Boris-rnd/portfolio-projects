use crate::what::cli::{Args, OutputMode, describe_mode};
use regex::Regex;
use serde_json::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct FileType {
    pub mime: String,
    pub description: String,
    pub is_text: bool,
}

pub async fn build_prompt_body(
    args: &Args,
    full_path: &Path,
    metadata: &fs::Metadata,
    repo_context: Option<String>,
    mode: OutputMode,
    is_directory: bool,
) -> Result<String, Box<dyn Error>> {
    if is_directory {
        let mut body = format!(
            "[PATH] {}\n[TYPE] directory\n[MODE] {}\n",
            full_path.display(),
            describe_mode(mode)
        );
        if let Some(question) = &args.ask {
            body.push_str(&format!("[QUESTION] {}\n", question));
        }
        if let Some(repo) = repo_context {
            body.push_str(&format!("[REPO] {}\n", repo));
        }
        body.push_str(&format!(
            "[DIRECTORY] {}\n",
            build_directory_context(full_path)?
        ));
        return Ok(body);
    }

    let file_type = detect_file_type(full_path)?;
    let mut body = format!(
        "[PATH] {}\n[SIZE] {} bytes\n[PERMISSIONS] {}\n[MODIFIED] {}\n[SYMLINK] {}\n[MIME] {}\n[DESCRIPTION] {}\n[MODE] {}\n",
        full_path.display(),
        metadata.len(),
        describe_permissions(metadata)?,
        describe_modified_time(metadata)?,
        describe_symlink(full_path)?,
        file_type.mime,
        file_type.description,
        describe_mode(mode)
    );

    if let Some(question) = &args.ask {
        body.push_str(&format!("[QUESTION] {}\n", question));
    }

    if let Some(repo) = repo_context {
        body.push_str(&format!("[REPO] {}\n", repo));
    }

    let (content_preview, truncated, secret_warning, binary_context) = if file_type.is_text {
        let file_content = tokio::fs::read_to_string(full_path).await;
        match file_content {
            Ok(content) => {
                let (sanitized, redacted) = sanitize_content(&content);
                let preview = preview_text_content(&sanitized, args.max_lines);
                let preview_with_context = format!("[CONTENT] {}\n", preview);
                (
                    preview_with_context,
                    content.lines().count() > args.max_lines,
                    if redacted {
                        Some("Sensitive-looking content was detected and redacted before sending to the model.".to_string())
                    } else {
                        None
                    },
                    None,
                )
            }
            Err(_) => (
                "[CONTENT] (unable to read file as text; binary or encoding issue)\n".to_string(),
                false,
                None,
                None,
            ),
        }
    } else {
        let context = build_binary_context(full_path, &file_type)?;
        (
            "[CONTENT] (binary or non-text content omitted; metadata only)\n".to_string(),
            false,
            None,
            Some(context),
        )
    };

    if let Some(secret_warning) = secret_warning {
        body.push_str(&format!("[SECURITY] {}\n", secret_warning));
    }

    if let Some(binary_context) = binary_context {
        body.push_str(&format!("[BINARY] {}\n", binary_context));
    }

    body.push_str(&content_preview);
    if truncated {
        body.push_str("[NOTE] Content was truncated to a preview for token safety.\n");
    }

    Ok(body)
}

pub fn collect_repo_context(path: &Path) -> Result<Option<String>, Box<dyn Error>> {
    let cwd = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let output = Command::new("git")
        .arg("-C")
        .arg(&cwd)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();

    let git_dir = match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stdout != "true" {
                return Ok(None);
            }
            let root = Command::new("git")
                .arg("-C")
                .arg(&cwd)
                .args(["rev-parse", "--show-toplevel"])
                .output()?;
            if !root.status.success() {
                return Ok(None);
            }
            Some(String::from_utf8_lossy(&root.stdout).trim().to_string())
        }
        _ => return Ok(None),
    };

    let root = git_dir.unwrap();
    let branch = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["branch", "--show-current"])
        .output()?;
    let branch_name = if branch.status.success() {
        String::from_utf8_lossy(&branch.stdout).trim().to_string()
    } else {
        "unknown".to_string()
    };

    let relative_path = path
        .strip_prefix(&root)
        .unwrap_or(path)
        .display()
        .to_string();
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["ls-files", "--error-unmatch", &relative_path])
        .output();
    let tracked_state = match tracked {
        Ok(out) if out.status.success() => "yes",
        _ => "no",
    };

    let status = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--short", "--", &relative_path])
        .output()?;
    let modified_state =
        if status.status.success() && !String::from_utf8_lossy(&status.stdout).trim().is_empty() {
            "yes"
        } else {
            "no"
        };

    let log = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["log", "--oneline", "-10"])
        .output()?;
    let recent_commits = if log.status.success() {
        let output = String::from_utf8_lossy(&log.stdout);
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        "(unavailable)".to_string()
    };

    let relevant_files = list_relevant_files(&root)?;

    Ok(Some(format!(
        "git repo: yes\nrepo root: {}\nbranch: {}\ntracked: {}\nmodified: {}\nrecent commits:\n{}\nrelevant files:\n{}",
        root, branch_name, tracked_state, modified_state, recent_commits, relevant_files
    )))
}

fn list_relevant_files(repo_root: &str) -> Result<String, Box<dyn Error>> {
    let mut files = Vec::new();
    for candidate in [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "README.md",
        "README",
        "src",
    ] {
        let path = Path::new(repo_root).join(candidate);
        if path.exists() {
            files.push(format!("- {}", path.display()));
        }
    }
    Ok(if files.is_empty() {
        "- (none found)".to_string()
    } else {
        files.join("\n")
    })
}

fn detect_file_type(path: &Path) -> Result<FileType, Box<dyn Error>> {
    let mime = run_file_command(path, &["--brief", "--mime-type", "--uncompress"], "mime")?;
    let description = run_file_command(path, &["--brief", "--uncompress"], "description")?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();
    let is_text = mime.starts_with("text/")
        || matches!(
            mime.as_str(),
            "application/json" | "application/xml" | "application/x-yaml" | "application/x-toml"
        )
        || matches!(
            ext.as_str(),
            "json"
                | "xml"
                | "toml"
                | "yaml"
                | "yml"
                | "ini"
                | "cfg"
                | "conf"
                | "csv"
                | "log"
                | "md"
                | "txt"
        );

    Ok(FileType {
        mime: mime.trim().to_string(),
        description: description.trim().to_string(),
        is_text,
    })
}

fn run_file_command(path: &Path, args: &[&str], label: &str) -> Result<String, Box<dyn Error>> {
    let mut command = Command::new("file");
    command.args(args).arg(path);
    match command.output() {
        Ok(output) if output.status.success() => {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        Ok(_) => Ok(match label {
            "mime" => "application/octet-stream".to_string(),
            _ => "unknown file type".to_string(),
        }),
        Err(_) => Ok(match label {
            "mime" => "application/octet-stream".to_string(),
            _ => "unknown file type".to_string(),
        }),
    }
}

fn describe_permissions(metadata: &fs::Metadata) -> Result<String, Box<dyn Error>> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        Ok(format!("0{:o}", metadata.permissions().mode() & 0o7777))
    }

    #[cfg(not(unix))]
    {
        let _ = metadata;
        Ok("unknown".to_string())
    }
}

fn describe_modified_time(metadata: &fs::Metadata) -> Result<String, Box<dyn Error>> {
    let modified = metadata.modified()?;
    let delta_since_now =
        chrono::Utc::now().signed_duration_since(chrono::DateTime::<chrono::Utc>::from(modified));
    let formatted_delta = format!(
        "{} years, {} months, {} days, {} hours, {} minutes, {} seconds",
        delta_since_now.num_days() / 365,
        delta_since_now.num_days() / 30 % 12,
        delta_since_now.num_days() % 30,
        delta_since_now.num_hours() % 24,
        delta_since_now.num_minutes() % 60,
        delta_since_now.num_seconds() % 60
    );
    if delta_since_now.as_seconds_f32() < 0.0 {
        return Ok(format!("in the future of about {formatted_delta}"));
    }
    Ok(format!("{} ago", formatted_delta))
}

fn describe_symlink(path: &Path) -> Result<String, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(path)?;
        Ok(format!("yes -> {}", target.display()))
    } else {
        Ok("no".to_string())
    }
}

fn build_binary_context(path: &Path, file_type: &FileType) -> Result<String, Box<dyn Error>> {
    let mut details = vec![format!(
        "mime={} description={}",
        file_type.mime, file_type.description
    )];

    if file_type.mime.starts_with("image/") {
        if let Ok(output) = Command::new("identify")
            .args(["-format", "%wx%h"])
            .arg(path)
            .output()
        {
            if output.status.success() {
                details.push(format!(
                    "image dimensions={}",
                    String::from_utf8_lossy(&output.stdout).trim()
                ));
            }
        }
    } else if file_type.mime == "application/pdf" {
        if let Ok(output) = Command::new("pdfinfo").arg(path).output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                details.push(text.lines().take(3).collect::<Vec<_>>().join(" | "));
            }
        }
    } else if file_type.mime == "application/x-sqlite3" || file_type.description.contains("SQLite")
    {
        if let Ok(output) = Command::new("sqlite3")
            .arg(path)
            .arg("SELECT name FROM sqlite_master WHERE type='table';")
            .output()
        {
            if output.status.success() {
                let tables = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .filter(|line| !line.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join(", ");
                details.push(format!("sqlite tables={tables}"));
            }
        }
    } else if file_type.mime == "application/json" || file_type.mime == "application/x-json" {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(value) = serde_json::from_str::<Value>(&content) {
                let keys = match value {
                    Value::Object(map) => map.keys().cloned().collect::<Vec<_>>().join(", "),
                    _ => "<root is not an object>".to_string(),
                };
                details.push(format!("json top-level keys={keys}"));
            }
        }
    }

    Ok(details.join("; "))
}

fn build_directory_context(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file_count = 0usize;
    let mut dir_count = 0usize;
    let mut total_size = 0u64;
    let mut largest_files = Vec::<(PathBuf, u64)>::new();
    let mut recent_files = Vec::<(PathBuf, std::time::SystemTime)>::new();
    let mut type_counts = BTreeMap::<String, usize>::new();

    let mut stack = vec![(path.to_path_buf(), 0usize)];
    while let Some((current, depth)) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        let mut children = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        children.sort_by(|a, b| {
            a.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .cmp(&b.file_name().unwrap_or_default().to_string_lossy())
        });

        for child in children {
            if should_ignore_path(&child) {
                continue;
            }
            let metadata = match fs::symlink_metadata(&child) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if metadata.is_dir() {
                dir_count += 1;
                if depth < 2 {
                    stack.push((child, depth + 1));
                }
            } else if metadata.is_file() {
                file_count += 1;
                total_size += metadata.len();
                let modified = metadata
                    .modified()
                    .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);
                largest_files.push((child.clone(), metadata.len()));
                recent_files.push((child.clone(), modified));

                let kind = classify_file_type(&child);
                *type_counts.entry(kind).or_insert(0usize) += 1;
            }
        }
    }

    largest_files.sort_by(|a, b| b.1.cmp(&a.1));
    recent_files.sort_by(|a, b| b.1.cmp(&a.1));

    let relevant_files = important_files(path)?;
    let tree = render_directory_tree(path, 0, 2)?;
    let readme_excerpt = read_excerpt(path, &["README.md", "README", "readme.md", "readme"])?;
    let manifest_excerpt = read_excerpt(path, &["Cargo.toml", "package.json", "pyproject.toml"])?;

    Ok(format!(
        "summary:\n- files: {}\n- directories: {}\n- total size: {}\n- important files:\n{}\n\ntree (max depth 2):\n{}\n\nfile type counts:\n{}\n\nlargest files:\n{}\n\nrecently modified:\n{}\n\nreadme excerpt:\n{}\n\nmanifest excerpt:\n{}",
        file_count,
        dir_count,
        format_size(total_size),
        relevant_files,
        tree,
        format_type_counts(&type_counts),
        format_top_files(&largest_files),
        format_recent_files(&recent_files),
        readme_excerpt,
        manifest_excerpt
    ))
}

fn important_files(path: &Path) -> Result<String, Box<dyn Error>> {
    let candidates = [
        "Cargo.toml",
        "package.json",
        "pyproject.toml",
        "README.md",
        "README",
        "src",
        "src/main.rs",
        "src/lib.rs",
        "tests",
        "tests/integration.rs",
        "Makefile",
        ".gitignore",
    ];
    let mut found = Vec::new();
    for candidate in candidates {
        let candidate_path = path.join(candidate);
        if candidate_path.exists() {
            found.push(format!("- {}", candidate_path.display()));
        }
    }
    Ok(if found.is_empty() {
        "- (none found)".to_string()
    } else {
        found.join("\n")
    })
}

fn render_directory_tree(
    path: &Path,
    depth: usize,
    max_depth: usize,
) -> Result<String, Box<dyn Error>> {
    let mut lines = vec![".".to_string()];
    render_directory_tree_inner(path, depth, max_depth, "", &mut lines)?;
    Ok(lines.join("\n"))
}

fn render_directory_tree_inner(
    path: &Path,
    depth: usize,
    max_depth: usize,
    prefix: &str,
    lines: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let mut entries = fs::read_dir(path)?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        a.file_name()
            .to_string_lossy()
            .cmp(&b.file_name().to_string_lossy())
    });

    let visible_entries = entries
        .into_iter()
        .filter(|entry| !should_ignore_path(&entry.path()))
        .collect::<Vec<_>>();

    for (index, entry) in visible_entries.iter().enumerate() {
        let child_path = entry.path();
        let name = child_path.file_name().unwrap_or_default().to_string_lossy();
        let connector = if index + 1 == visible_entries.len() {
            "└── "
        } else {
            "├── "
        };
        let display_name = if child_path.is_dir() {
            format!("{}/", name)
        } else {
            name.to_string()
        };
        lines.push(format!("{}{}{}", prefix, connector, display_name));

        if child_path.is_dir() && depth < max_depth {
            let child_prefix = if index + 1 == visible_entries.len() {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            render_directory_tree_inner(&child_path, depth + 1, max_depth, &child_prefix, lines)?;
        }
    }

    Ok(())
}

fn should_ignore_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | "dist" | "build" | ".venv" | "__pycache__"
            )
        })
}

fn classify_file_type(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();
    match ext.as_str() {
        "rs" => "Rust source".to_string(),
        "md" => "Markdown".to_string(),
        "toml" => "TOML config".to_string(),
        "json" => "JSON".to_string(),
        "yaml" | "yml" => "YAML".to_string(),
        "py" => "Python".to_string(),
        "sh" | "bash" => "Shell script".to_string(),
        "png" | "jpg" | "jpeg" | "gif" | "svg" => "Image".to_string(),
        _ if path.to_string_lossy().contains("target") => "Build artifact".to_string(),
        _ => "Other".to_string(),
    }
}

fn format_type_counts(type_counts: &BTreeMap<String, usize>) -> String {
    if type_counts.is_empty() {
        "- (none)".to_string()
    } else {
        type_counts
            .iter()
            .map(|(kind, count)| format!("- {kind}: {count}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_top_files(files: &[(PathBuf, u64)]) -> String {
    if files.is_empty() {
        "- (none)".to_string()
    } else {
        files
            .iter()
            .take(5)
            .map(|(path, size)| format!("- {} ({})", path.display(), format_size(*size)))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn format_recent_files(files: &[(PathBuf, std::time::SystemTime)]) -> String {
    if files.is_empty() {
        "- (none)".to_string()
    } else {
        files
            .iter()
            .take(5)
            .map(|(path, _modified)| format!("- {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn read_excerpt(path: &Path, candidates: &[&str]) -> Result<String, Box<dyn Error>> {
    for candidate in candidates {
        let candidate_path = path.join(candidate);
        if candidate_path.exists() {
            let content = fs::read_to_string(&candidate_path)?;
            let lines = content.lines().take(20).collect::<Vec<_>>();
            return Ok(if lines.is_empty() {
                "(empty)".to_string()
            } else {
                lines.join("\n")
            });
        }
    }
    Ok("(not found)".to_string())
}

fn format_size(size: u64) -> String {
    if size >= 1024 * 1024 {
        format!("{:.1} MiB", size as f64 / (1024.0 * 1024.0))
    } else if size >= 1024 {
        format!("{:.1} KiB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    }
}

fn sanitize_content(content: &str) -> (String, bool) {
    let mut sanitized = content.to_string();
    let mut redacted = false;

    let patterns = [
        (
            Regex::new(r#"(?i)\b(api[_-]?key|access[_-]?token|secret|password|token)\b\s*[:=]\s*["']?([^\s"'`]+)"#).unwrap(),
            "[REDACTED]",
        ),
        (
            Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
            "[REDACTED AWS ACCESS KEY]",
        ),
        (
            Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----").unwrap(),
            "[REDACTED PRIVATE KEY]",
        ),
    ];

    for (pattern, replacement) in patterns {
        if pattern.is_match(&sanitized) {
            redacted = true;
            sanitized = pattern.replace_all(&sanitized, replacement).to_string();
        }
    }

    if redacted {
        tracing::info!("Sensitive content detected and redacted in file preview.");
    }

    (sanitized, redacted)
}

fn preview_text_content(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        return text.to_string();
    }

    let marker_count = 2;
    let available = max_lines.saturating_sub(marker_count);
    let head = available / 3;
    let tail = available / 3;
    let middle = available - head - tail;

    let mut preview = Vec::new();
    preview.extend(lines[..head].iter().copied());
    preview.push("... [content truncated by WHAT] ...");
    preview.extend(
        lines[lines.len() / 2 - middle / 2..lines.len() / 2 - middle / 2 + middle]
            .iter()
            .copied(),
    );
    preview.push("... [content truncated by WHAT] ...");
    preview.extend(lines[lines.len() - tail..].iter().copied());

    preview.join("\n")
}
