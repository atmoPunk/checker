#![allow(dead_code)]

extern crate async_std;
extern crate futures;
extern crate serde;
extern crate serde_json;

mod lab;
mod runner;
mod test;

use futures::executor::block_on;
pub use lab::*;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let file_config = File::open("config.json").expect("Can't open config");
    let reader = BufReader::new(file_config);
    let lab: Lab = serde_json::from_reader(reader).expect("Can't deserialize json");
    let future = lab.check_all();
    let results = block_on(future);
    for (name, s) in results.iter() {
        println!("Student: {}, Result: {:?}", name, s);
    }
}
