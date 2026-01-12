use std::io::Write;
use std::process::Stdio;
use std::{collections::HashMap, process::Command};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

fn main() {
    let output = Command::new("wl-paste")
        .output()
        .expect("wl-paste not found");

    let clipboard = String::from_utf8_lossy(&output.stdout).to_string();


    let _config_dir = ensure_config_dir("redactd");
    let config = load_config("redactd");

    let redacted = redact_text(&clipboard, &config.rules);
    overwrite_clipboard(&redacted);
}

fn overwrite_clipboard(text: &str) {
    let mut child = Command::new("wl-copy")
    .stdin(Stdio::piped())
    .spawn()
    .expect("failed to start wl-copy");

    child
    .stdin
    .as_mut()
    .unwrap()
    .write_all(text.as_bytes())
    .expect("failed to write wl-copy")
}


#[derive(Deserialize)]
struct RuleGroup {
    replace: String,

    #[serde(rename = "match")]
    match_: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    rules: HashMap<String, RuleGroup>,
}

fn ensure_config_dir(app: &str) -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = PathBuf::from(std::env::var("HOME").unwrap());
            p.push(".config");
            p
        });
        
        let dir = base.join(app);

        fs::create_dir_all(&dir)
            .expect("failed to create config directory");
        return dir
}

fn redact_text(input: &str, rules: &HashMap<String, RuleGroup>) -> String {
    let mut out = input.to_string();

    for rule in rules.values() {
        for m in &rule.match_ {
            out = out.replace(m, &rule.replace);
        }
    }

    return out
}

fn load_config(app: &str) -> Config {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut p = PathBuf::from(std::env::var("HOME").unwrap());
            p.push(".config");
            p
        });

    let path = base.join(app).join("config.toml");
    let data = fs::read_to_string(&path)
        .expect("missing ~/.config/redactd/config.toml");

    toml::from_str(&data).expect("invalid config")
}
