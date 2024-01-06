use crate::parser::RunType;

use super::parser::CommandScope;
use std::{
    collections::HashMap,
    fs,
    process::{exit, Command, Stdio},
};

#[derive(Debug)]
enum RunStatus {
    Success,
    Failure,
}

impl Default for RunStatus {
    fn default() -> Self {
        Self::Failure
    }
}

#[derive(Debug, Default)]
struct RunData {
    status: RunStatus,
    url: String,
    run_type: super::parser::RunType,
}

pub fn run_commands(commands: Vec<CommandScope>, enroll_script_path: String) {
    let mut runs = vec![];

    for command in commands {
        match command.run_type {
            RunType::Bats => {
                println!("\n\n===Running bats test for path {}===", command.url);

                let mut run_data = RunData::default();
                if run_bats_test(&command, enroll_script_path.clone()) {
                    run_data.status = RunStatus::Success
                }

                run_data.url = command.url;
                run_data.run_type = command.run_type;

                runs.push(run_data);
            }
            RunType::Unknown => {}
        }
    }

    let mut failed = false;

    for run in runs {
        match run.status {
            RunStatus::Success => println!("✅ {} {:?} test passed", run.url, run.run_type),
            RunStatus::Failure => {
                eprintln!("❌ {} {:?} test failed", run.url, run.run_type);
                failed = true;
            }
        }
    }

    if failed {
        exit(1)
    }
}

fn run_bats_test(command: &CommandScope, enroll_script_path: String) -> bool {
    let enroll_email_address = vec!["docs-test-client@ockam.io", "docs-test-server@ockam.io"];
    let mut ockam_home_dir: HashMap<String, String> = std::env::vars().collect();

    // Run ockam enroll script if needed
    if !command.ockam_enroll_data.is_empty() {
        let directories = command.ockam_enroll_data.split_ascii_whitespace();
        let directories = directories
            .into_iter()
            .map(|a| a.to_string())
            .collect::<Vec<String>>();

        if directories.len() > 2 {
            panic!(
                "Invalid number of ockam enroll that can be supported for URL {}",
                command.url
            );
        }

        for (index, directory) in directories.into_iter().enumerate() {
            let ockam_home_temp_dir = tempfile::tempdir()
                .unwrap()
                .into_path()
                .to_str()
                .unwrap()
                .to_string();

            println!("Temporary OCKAM_HOME for {directory} is {ockam_home_temp_dir:?}");

            ockam_home_dir.insert(directory, ockam_home_temp_dir.clone());

            let mut env: HashMap<String, String> = std::env::vars().collect();
            env.insert("OCKAM_HOME".to_string(), ockam_home_temp_dir);
            env.insert("SCRIPT_DIR".to_string(), enroll_script_path.clone());
            env.insert(
                "EMAIL_ADDRESS".to_string(),
                enroll_email_address[index].to_string(),
            );

            println!("Script dir is {}", enroll_script_path);

            match Command::new("bash")
                .arg(format!("{enroll_script_path}/ockam_enroll.sh"))
                .envs(env)
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
            {
                Ok(e) => {
                    if !e.status.success() {
                        return false;
                    }
                }
                Err(e) => {
                    eprintln!("error running bats test for path {}: {e:?}", command.url);
                    return false;
                }
            }
        }
    }

    // Run bats test
    println!("Enroll was successful");
    let bats_file = tempfile::tempdir().unwrap().into_path();
    let bats_file = bats_file.join("file.bats");

    fs::write(bats_file.clone(), command.commands.clone()).unwrap();

    match Command::new("bats")
        .arg(bats_file.as_os_str())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .envs(ockam_home_dir)
        .output()
    {
        Ok(e) => e.status.success(),
        Err(e) => {
            eprintln!("error running bats test for path {}: {e:?}", command.url);
            return false;
        }
    }
}

#[test]
fn test_bats_ockam_help_pass() {
    let command = CommandScope {
        commands: r#"
# Ockam binary to use
if [[ -z $OCKAM ]]; then
  OCKAM=ockam
fi

if [[ -z $BATS_LIB ]]; then
  BATS_LIB=$(brew --prefix)/lib # macos
fi

setup() {
  load "$BATS_LIB/bats-support/load.bash"
  load "$BATS_LIB/bats-assert/load.bash"
}

@test "run ockam help" {
  run $OCKAM --help
  assert_success
}
"#
        .to_string(),
        ockam_enroll_data: "".to_string(),
        url: "url".to_string(),
        run_type: RunType::Bats,
    };
    assert!(run_bats_test(&command, "../scripts".to_string()) == true);
}

#[test]
fn test_bats_ockam_help_fail() {
    let command = CommandScope {
        commands: r#"
# Ockam binary to use
if [[ -z $OCKAM ]]; then
  OCKAM=ockam
fi

if [[ -z $BATS_LIB ]]; then
  BATS_LIB=$(brew --prefix)/lib # macos
fi

setup() {
  load "$BATS_LIB/bats-support/load.bash"
  load "$BATS_LIB/bats-assert/load.bash"
}

@test "run ockam help" {
  run $OCKAM --helpp
  assert_success
}
"#
        .to_string(),
        ockam_enroll_data: "".to_string(),
        url: "url".to_string(),
        run_type: RunType::Bats,
    };
    assert!(run_bats_test(&command, "../scripts".to_string()) == false);
}
