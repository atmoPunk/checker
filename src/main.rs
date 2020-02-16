#![allow(dead_code)]

mod github;
mod lab;
mod program;
mod student;
mod test;
mod variant;

use futures::executor::block_on;
pub use lab::*;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

#[derive(Debug, Clone)]
struct Commit {
    sha: String,
    message: String,
}
fn main() {
    let file_config = OpenOptions::new().read(true).open("config.json").expect("Can't open config");
    let reader = BufReader::new(file_config);
    let mut lab: Lab = serde_json::from_reader(reader).expect("Can't deserialize json");
    for (_, student) in lab.students.iter() {
        if let Err(e) = github::clone_repo(&student.program.owner, &student.program.repo) {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                continue;
            } else {
                panic!(e);
            }
        }
    }
    for (_, student) in lab.students.iter() {
        github::pull_repo(&student.program.owner, &student.program.repo).unwrap()
    }
    let future = lab.check_all();
    let results = block_on(future);
    for (name, s) in results.iter() {
        println!("Student: {}, Result: {:?}", name, s);
        let owner = &lab.students[name].program.owner;
        let repo = &lab.students[name].program.repo;
        let commits_json = github::get_commits(owner, repo).unwrap();
        let last_commit = commits_json.as_array().unwrap().last().unwrap()["sha"].as_str().unwrap().to_owned();
        match &lab.students[name].program.last_commit {
            Some(commit) => { if commit != &last_commit {
                github::write_comment(owner, repo, &last_commit, &format!("{:?}", s)).unwrap();
            }},
            None => {
                github::write_comment(owner, repo, &last_commit, &format!("{:?}", s)).unwrap();
            }
        }
        lab.students.get_mut(name).unwrap().program.last_commit = Some(last_commit);
        lab.students.get_mut(name).unwrap().result = Some(s.to_owned());
    }

    let file_config = OpenOptions::new().write(true).open("config.json").expect("Can't open config");
    let writer = BufWriter::new(file_config);
    serde_json::to_writer_pretty(writer, &lab).unwrap();

    // lab.build_doxygen("Ivanov");
    // let commits_json = github::get_commits("atmoPunk", "checker").unwrap();
    // let mut commits: Vec<Commit> = Vec::new();
    // for commit in commits_json.as_array().unwrap().iter() {
    //     commits.push(Commit {
    //         sha: commit["sha"].as_str().unwrap().to_owned(),
    //         message: commit["commit"]["message"].as_str().unwrap().to_owned(),
    //     });
    // }
    // for commit in commits.iter() {
    //     println!("{:?}", commit);
    // }

    // if let Err(e) = github::clone_repo("atmoPunk", "Kilo") {
    //     eprintln!("{:?}", e);
    // }
    // github::pull_repo("atmoPunk", "Kilo").unwrap();
}
