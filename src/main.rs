mod lab;
mod runner;
mod test;

use lab::*;
use runner::*;
use test::*;

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let mut test1 = Test::new(
        PathBuf::from("./tests/test1.in"),
        PathBuf::from("./tests/test1.out"),
        1.0,
        1,
    );
    let mut test2 = Test::new(
        PathBuf::from("./tests/test2.in"),
        PathBuf::from("./tests/test2.out"),
        1.0,
        1,
    );

    let mut var = Variant::new(vec![test1, test2]);

    //println!("{:?}", PathBuf::from("./prog/").canonicalize().unwrap().is_dir());

    let mut prog = Program::new(
        Lang::Cpp,
        PathBuf::from("./prog/").canonicalize().unwrap(),
        PathBuf::from("./prog/prog.out").canonicalize().unwrap(),
    );
    let mut stud = Student::new(prog, var, None);
    let mut lab = Lab::new(HashMap::new());
    lab.students.insert(String::from("Ivanov"), stud);

    println!("Start building");
    lab.students["Ivanov"].program.build().unwrap();
    println!("Stop building");
}
