use std::fs::{self, DirBuilder};
use std::process::Command;
use std::path::Path;

pub fn get_commits(owner: &str, repo: &str) -> Result<serde_json::value::Value, std::io::Error> {
    let resp = ureq::get(&format!(
        "https://api.github.com/repos/{}/{}/commits",
        owner, repo
    ))
    .call();

    if resp.ok() {
        //let mut json: Result<serde_json::value::Value, std::io::Error> = resp.into_json();
        resp.into_json()
    } else {
        eprintln!("{}", resp.status_line());
        Err(std::io::Error::new(std::io::ErrorKind::Other, "not OK"))
    }
}

pub fn clone_repo(owner: &str, repo: &str) -> std::io::Result<()> {
    let path = format!("students/{}/{}", owner, repo);
    let mut path = Path::new(&path);
    if let Ok(_) = fs::metadata(&path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Repo already cloned",
        ));
    }
    path = path.parent().unwrap();
    let _ = DirBuilder::new().recursive(true).create(&path)?;
    let clone = Command::new("git")
        .args(&["clone", &format!("https://github.com/{}/{}", owner, repo)])
        .current_dir(&path)
        .output()?;
    println!("{}", String::from_utf8(clone.stdout).unwrap());
    eprintln!("{}", String::from_utf8(clone.stderr).unwrap());
    Ok(())
}

pub fn pull_repo(owner: &str, repo: &str) -> std::io::Result<()> {
    let path = format!("students/{}/{}", owner, repo);
    let path = Path::new(&path);
    fs::metadata(&path)?;
    let pull = Command::new("git")
        .args(&["pull"])
        .current_dir(&path)
        .output()?;
    println!("{}", String::from_utf8(pull.stdout).unwrap());
    eprintln!("{}", String::from_utf8(pull.stderr).unwrap());
    Ok(())
}
