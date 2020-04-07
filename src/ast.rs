use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_script(dir: PathBuf, result: String) -> Result<(), ()> {
    let res = Command::new("python")
        .args(&["src/mark_includes.py", dir.to_str().unwrap(), &result[..]])
        .output();
    if res.is_err() {
        return Err(());
    }

    let res = res.unwrap();
    if !res.stderr.is_empty() {
        return Err(());
    }

    Ok(())
}

pub fn add_to_index(hashes: PathBuf, not_up: i32, not_down: i32) -> Result<(), ()> {
    let output = Command::new("python")
        .args(&[
            "src/index.py",
            hashes.to_str().unwrap(),
            &not_down.to_string()[..],
            &not_up.to_string()[..],
        ])
        .output();
    if output.is_err() {
        return Err(());
    }

    let output = output.unwrap();

    if !output.status.success() {
        return Err(());
    }

    // TODO: parse outbut, build HashMap<Name, number of collisions>

    Ok(())
}
