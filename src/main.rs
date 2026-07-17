use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};

const DEFAULT_CONFIG: &str = "\
# redactd rules, applied top to bottom.
rules:
  - replace: \"[NAME]\"
    match:
      - Alice Example
      - alice@example.com
";

fn main() {
    if std::env::args().nth(1).as_deref() == Some("version") {
        print!("{}", version_info());
        return;
    }

    let config = load_config("redactd");
    let clipboard = read_clipboard();

    let redacted = redact_text(&clipboard, &config.rules);
    if redacted != clipboard {
        overwrite_clipboard(&redacted);
    }
}

fn version_info() -> String {
    format!(
        "Name:      redactd\n\
         Version:   {}\n\
         Revision:  {}\n\
         Reference: {}\n\
         Rustc:     {}\n\
         Built At:  {}\n\
         OS:        {}\n\
         Arch:      {}\n",
        env!("REDACTD_VERSION"),
        env!("REDACTD_REVISION"),
        env!("REDACTD_REFERENCE"),
        env!("REDACTD_RUSTC"),
        env!("REDACTD_BUILT"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    )
}

fn read_clipboard() -> String {
    let output = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .expect("wl-paste not found");

    // wl-paste exits nonzero when the clipboard is empty or non-text
    if !output.status.success() {
        exit(0);
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn overwrite_clipboard(text: &str) {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .expect("failed to start wl-copy");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .expect("failed to write to wl-copy");

    child.wait().expect("wl-copy failed");
}

#[derive(Deserialize)]
struct Rule {
    replace: String,

    #[serde(rename = "match")]
    match_: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    rules: Vec<Rule>,
}

fn config_path(app: &str) -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap()).join(".config"));

    base.join(app).join("config.yaml")
}

fn redact_text(input: &str, rules: &[Rule]) -> String {
    let mut out = input.to_string();

    for rule in rules {
        for m in &rule.match_ {
            out = out.replace(m, &rule.replace);
        }
    }

    out
}

fn load_config(app: &str) -> Config {
    let path = config_path(app);

    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(_) => {
            fs::create_dir_all(path.parent().unwrap()).expect("failed to create config directory");
            fs::write(&path, DEFAULT_CONFIG).expect("failed to write default config");
            eprintln!(
                "created starter config at {}, edit it and rerun",
                path.display()
            );
            exit(1);
        }
    };

    match serde_yaml_ng::from_str(&data) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("invalid config {}: {}", path.display(), e);
            exit(1);
        }
    }
}
