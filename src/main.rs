#![allow(dead_code)]

mod github;
mod lab;
mod program;
mod student;
mod test;
mod variant;

use futures::executor::block_on;
pub use lab::*;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone)]
struct Commit {
    sha: String,
    message: String,
}
fn main() {
    let file_config = File::open("config.json").expect("Can't open config");
    let reader = BufReader::new(file_config);
    let lab: Lab = serde_json::from_reader(reader).expect("Can't deserialize json");
    // let future = lab.check_all();
    // let results = block_on(future);
    // for (name, s) in results.iter() {
    //     println!("Student: {}, Result: {:?}", name, s);
    // }
    // lab.build_doxygen("Ivanov");
    let commits_json = github::get_commits("atmoPunk", "checker").unwrap();
    let mut commits: Vec<Commit> = Vec::new();
    for commit in commits_json.as_array().unwrap().iter() {
        commits.push(Commit {
            sha: commit["sha"].as_str().unwrap().to_owned(),
            message: commit["commit"]["message"].as_str().unwrap().to_owned(),
        });
    }
    for commit in commits.iter() {
        println!("{:?}", commit);
    }

    if let Err(e) = github::clone_repo("atmoPunk", "Kilo") {
        eprintln!("{:?}", e);
    }
    github::pull_repo("atmoPunk", "Kilo").unwrap();
}
