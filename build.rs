use std::path::Path;
use std::process::Command;

fn git_hash() -> Option<String> {
    if !Path::new(".git").exists() {
        return None;
    }
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-env-changed=ZMEM_GIT_HASH");

    let mut version = env!("CARGO_PKG_VERSION").to_owned();
    if let Some(hash) = std::env::var("ZMEM_GIT_HASH")
        .ok()
        .or_else(git_hash)
        .filter(|h| !h.is_empty())
    {
        version.push_str(&format!(" ({hash})"));
    }
    println!("cargo:rustc-env=ZMEM_VERSION={version}");
}
