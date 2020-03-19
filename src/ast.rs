use std::path::{Path, PathBuf};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Write};
use std::ffi::OsStr;




pub fn merge_with_incl_markers(dir: &Path) -> Result<PathBuf, String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.to_str().unwrap()));
    }

    let mut lines: Vec<String> = Vec::new();
    let mut marker_count = 0;
    for entry in read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue
        }
        if let None = path.extension() {
            continue
        }
        match path.extension().unwrap().to_str().unwrap() {
            "h" | "c" | "cpp" => {
                let file = File::open(path).unwrap();
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line.unwrap();
                    if line.starts_with("#include <") {
                        lines.push(format!("int __INCLUDE_MARKER_START_{}__\n", marker_count));
                        lines.push(line);
                        lines.push(String::from("\n"));
                        lines.push(format!("int __INCLUDE_MARKER_END_{}__\n", marker_count));
                        marker_count += 1;
                    } else {
                        lines.push(line);
                        lines.push(String::from("\n"));
                    }
                }
                lines.push("\n".to_owned());
            },
            _ => continue,
        }
    }

    let res = PathBuf::from("tmp-res.cpp");

    let mut resfile = File::create(&res).unwrap();
    for line in lines.iter() {
        resfile.write_all(line.as_bytes()).unwrap();
    }

    Ok(res)
}