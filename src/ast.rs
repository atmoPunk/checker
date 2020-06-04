use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn gen_hashes(dir: PathBuf, result: String) -> Result<(), ()> {
    let res = Command::new("python")
        .args(&["src/gen_hashes.py", dir.to_str().unwrap(), &result[..]])
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

pub fn compare(hash1: String, hash2: String) -> std::io::Result<f32> {
    let res = Command::new("python")
        .args(&["src/compare.py", &hash1[..], &hash2[..]])
        .output();
    if res.is_err() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "something is wrong"));
    }

    let res = res.unwrap();
    if !res.stderr.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "something is wrong"));
    }
    let value = String::from_utf8(res.stdout).unwrap().parse::<f32>().unwrap();
    Ok(value)
}

pub fn add_to_index(
    hashes: PathBuf,
    not_up: i32,
    not_down: i32,
) -> Result<HashMap<String, Vec<String>>, ()> {
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

    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    let output = String::from_utf8(output.stdout).unwrap();
    let mut current_hash = "";

    for line in output.lines() {
        if line.starts_with("Hash ") {
            current_hash = &line[5..37];
            continue;
        } else {
            let entry = result.entry(current_hash.to_owned()).or_insert(vec![]);
            entry.push(line.to_owned());
        }
    }

    // TODO: maybe change to HashMap<Student name, number of collisions with that student>
    // Currently: Hashmap<Hash, List of students with that hash map>

    Ok(result)
}

pub fn simple_diff(lhs: PathBuf, rhs: PathBuf) -> i32 {
    let output = Command::new("diff")
        .args(&[
            lhs.to_str().unwrap(),
            rhs.to_str().unwrap(),
            "-x",
            "*.out",
            "-x",
            ".git",
            "-N",
            "-i", // ignore case
            "-n",
            "-w", // ignore all white space
            "-B", // ignore blank lines
        ])
        .output()
        .unwrap();

    let mut counter = 0;
    let mut skip_lines = 0;
    let output = String::from_utf8(output.stderr).unwrap();
    for line in output.lines() {
        if skip_lines > 0 {
            skip_lines -= 1;
            continue;
        }
        if line.starts_with("diff -x") {
            continue;
        }
        let words = line.split_ascii_whitespace();
        let diff_lines = words.skip(1).next().unwrap().parse::<i32>().unwrap();
        counter += diff_lines;
        skip_lines = diff_lines;
    }
    counter
}
