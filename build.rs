use std::process::Command;

fn main() {
    for key in [
        "CI_COMMIT_TAG",
        "CI_COMMIT_SHORT_SHA",
        "CI_COMMIT_REF_NAME",
        "CI_PIPELINE_CREATED_AT",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    let version = env_or("CI_COMMIT_TAG", "dev");
    let revision = env_or("CI_COMMIT_SHORT_SHA", "HEAD");
    let reference = env_or("CI_COMMIT_REF_NAME", "HEAD");
    let built = env_or("CI_PIPELINE_CREATED_AT", "now");

    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=REDACTD_VERSION={version}");
    println!("cargo:rustc-env=REDACTD_REVISION={revision}");
    println!("cargo:rustc-env=REDACTD_REFERENCE={reference}");
    println!("cargo:rustc-env=REDACTD_BUILT={built}");
    println!("cargo:rustc-env=REDACTD_RUSTC={}", rustc_version.trim());
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
