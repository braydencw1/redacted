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


    let _config_dir = ensure_config_dir("redacted");
    let config = load_config("redacted");

    let redacted = redact_text(&clipboard, &config.rules);
    // println!("redacted {}", redacted);
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
    input.split_whitespace().map(|word| redact_word(word, rules))
    .collect::<Vec<_>>()
    .join(" ")
}

fn redact_word<'a>(
    word: &'a str,
    rules: &'a HashMap<String, RuleGroup>,
) -> &'a str {
    for rule in rules.values() {
        if rule.match_.iter().any(|m| m == word) {
            return &rule.replace;
        }
    }
    return word
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
        .expect("missing ~/.config/redacted/config.toml");

    toml::from_str(&data).expect("invalid config")
}
