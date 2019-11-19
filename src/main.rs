mod lab;
mod runner;
mod test;

use lab::*;
use runner::*;
use test::*;

use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let test1 = Test::new(
        PathBuf::from("./tests/test1.in"),
        PathBuf::from("./tests/test1.out"),
        1.0,
        1,
    );
    let test2 = Test::new(
        PathBuf::from("./tests/test2.in"),
        PathBuf::from("./tests/test2.out"),
        1.0,
        1,
    );

    let var = Variant::new(vec![test1, test2]);

    let prog = Program::new(
        Lang::Cpp,
        PathBuf::from("./prog/").canonicalize().unwrap(),
        PathBuf::from("./prog/prog.out").canonicalize().unwrap(),
    );
    let stud = Student::new(prog, var, None);
    let mut lab = Lab::new(HashMap::new());
    lab.students.insert(String::from("Ivanov"), stud);

    for (student_name, student_res) in lab.students.iter() {
        println!(
            "Student: {}, Result: {:?}",
            student_name,
            student_res.check()
        );
    }
}
