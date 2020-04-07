#![allow(dead_code)]

mod ast;
mod github;
mod lab;
mod program;
mod student;
mod test;
mod variant;

use futures::executor::block_on;
pub use lab::*;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::{thread, time};

#[derive(Debug, Clone)]
struct Commit {
    sha: String,
    message: String,
}

// TODO: remove unwraps

// TODO: get commit sha while downloading, so we dont have to go to github now
fn update_results(results: HashMap<String, Result<(), LabError>>, lab: &mut Lab) {
    for (name, s) in results.iter() {
        let owner = &lab.students[name].program.owner;
        let repo = &lab.students[name].program.repo;
        let commits_json = github::get_commits(owner, repo).unwrap();
        let last_commit = commits_json.as_array().unwrap().last().unwrap()["sha"]
            .as_str()
            .unwrap()
            .to_owned();
        match &lab.students[name].program.last_commit {
            Some(commit) => {
                if commit != &last_commit {
                    github::write_comment(owner, repo, &last_commit, &format!("{:?}", s)).unwrap();
                }
            }
            None => {
                github::write_comment(owner, repo, &last_commit, &format!("{:?}", s)).unwrap();
            }
        }
        lab.students.get_mut(name).unwrap().program.last_commit = Some(last_commit);
        lab.students.get_mut(name).unwrap().result = Some(s.to_owned());
    }
}

fn main() {
    let res =
        ast::merge_with_incl_markers(std::path::Path::new("/home/atmopunk/Documents/Code/test1"))
            .unwrap();
    return;

    let file_config = OpenOptions::new()
        .read(true)
        .open("config.json")
        .expect("Can't open config");
    let reader = BufReader::new(file_config);
    let mut lab: Lab = serde_json::from_reader(reader).expect("Can't deserialize json");
    loop {
        let update = lab.download_all(); // Download or update student repos
        let students = block_on(update).unwrap(); // TODO: return students that changed, and run tests only for these students

        let future = lab.check_students(students); // Run student programs on tests
        let results = block_on(future);

        update_results(results, &mut lab); // Send results to github comments

        let file_config = OpenOptions::new()
            .write(true)
            .open("config.json")
            .expect("Can't open config");
        let writer = BufWriter::new(file_config);
        serde_json::to_writer_pretty(writer, &lab).unwrap();
        thread::sleep(time::Duration::from_secs(300)); // Sleep for 5 minutes
    }
    // lab.build_doxygen("Ivanov");
}
