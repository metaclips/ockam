use std::{
    env, fs,
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
};

mod parser;
mod runner;

fn main() {
    let ockam_enroll_script=env::var("OCKAM_ENROLL_SCRIPT").expect("please set environment variable OCKAM_ENROLL_SCRIPT which indicates the directory of shell script that automates enrollment.");

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut path: String = String::new();

    let mut markdowns_to_run = vec![];

    while let Ok(n_bytes) = stdin.read_line(&mut path) {
        if n_bytes == 0 {
            break;
        }

        path = path.trim().to_string();

        let file_path = match PathBuf::from_str(&path) {
            Ok(e) => e,
            Err(e) => panic!("invalid path specified {path}: {e:?}"),
        };

        if !file_path.exists() || !file_path.is_file() {
            panic!("file {file_path:?} does not exist");
        }

        println!("checking for test in {path}");

        let markdown_content = fs::read_to_string(file_path).unwrap();
        let mut command = parser::convert_blocks(markdown_content, path.clone());
        markdowns_to_run.append(&mut command);
        path.clear();
    }

    println!("====> Found {} tests to run", markdowns_to_run.len());
    runner::run_commands(markdowns_to_run, ockam_enroll_script);
}
