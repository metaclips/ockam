use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RUNNER_TYPE_CONVERSION_REGEXP: Regex =
        Regex::new("<!-- (?P<run_type>.+) end -->").unwrap();

    static ref COMMAND_SCRAPPER_REGEXP: Regex =
        Regex::new("<!-- (?P<run_type>.+) start (?P<ockam_enroll_data>.*)-->(?:.|\n)*<!--(?P<commands>(?:.|\n)+)-->(?:.|\n)*### .+ end").unwrap();
}

#[derive(Debug, PartialEq)]
pub enum RunType {
    Unknown,
    Bats,
}

impl FromStr for RunType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bats" => Ok(Self::Bats),
            _ => Ok(Self::Unknown),
        }
    }
}

impl Default for RunType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(PartialEq, Debug)]
pub struct CommandScope {
    pub commands: String,
    pub run_type: RunType,
    pub ockam_enroll_data: String,
    pub url: String,
}

pub fn convert_blocks(mut markdown: String, url: String) -> Vec<CommandScope> {
    // Regexp conversion gets complicated when we have multiple blocks e.g.
    // <!-- bats start -->
    // <!-- some commands -->
    // <!-- bats end -->
    // <!-- bash start -->
    // <!-- some commands 2 -->
    // <!-- bash end -->
    //
    // Using regex `<\!-- (?P<start_run_type>.+) start (.*)-->(?:.|\n)*<!--(?P<commands>(?:.|\n)+)-->(?:.|\n)*<\!-- (?P<end_run_type>.+) end -->`
    // returns `start_run_type` as `bats` and `end_run_type` as bash.
    // We provide a hack by first checking if there's a `<!-- bash/bats end -->` string
    // and then convert the command to a `### bash end` text so that it becomes
    //
    // <!-- bats start -->
    // <!-- some commands -->
    // ### bats end
    // <!-- bash start -->
    // <!-- some commands 2 -->
    // <!-- bash end -->
    //
    // and we can parse the string with `<\!-- (?P<start_run_type>.+) start (.*)-->(?:.|\n)*<!--(?P<commands>(?:.|\n)+)-->(?:.|\n)*### (?P<end_run_type>.+) end`

    let mut commands = vec![];

    while RUNNER_TYPE_CONVERSION_REGEXP.is_match(markdown.as_ref()) {
        markdown = RUNNER_TYPE_CONVERSION_REGEXP
            .replace(&markdown, "### $run_type end")
            .to_string();

        for cap in COMMAND_SCRAPPER_REGEXP.captures_iter(&markdown) {
            let mut command = CommandScope {
                ockam_enroll_data: cap
                    .name("ockam_enroll_data")
                    .unwrap()
                    .as_str()
                    .trim()
                    .to_string(),
                run_type: RunType::from_str(cap.name("run_type").unwrap().as_str().trim()).unwrap(),
                commands: cap.name("commands").unwrap().as_str().trim().to_string(),
                url: url.clone(),
            };

            command.commands = command.commands.trim().to_string();

            commands.push(command);
        }

        markdown = COMMAND_SCRAPPER_REGEXP.replace(&markdown, "").to_string();
    }

    commands
}

#[test]
fn test_convert_block() {
    let commands = convert_blocks(
        String::from(
            r#"
<!-- bats start -->
<!--
some commands
-->
<!-- bats end -->
<!-- bash start -->
<!--
some commands 2
-->
<!-- bash end -->
    "#,
        ),
        "test_url".into(),
    );

    assert!(commands.len() == 2);
    assert_eq!(
        commands,
        vec![
            CommandScope {
                commands: "some commands".into(),
                ockam_enroll_data: "".into(),
                run_type: RunType::Bats,
                url: "test_url".into()
            },
            CommandScope {
                commands: "some commands 2".into(),
                ockam_enroll_data: "".into(),
                run_type: RunType::Unknown,
                url: "test_url".into()
            }
        ]
    )
}

#[test]
fn test_convert_block_empty() {
    let commands = convert_blocks(String::from(r#"empty"#), "url".into());
    assert!(commands.is_empty())
}

#[test]
fn test_with_markdown_content() {
    let commands = convert_blocks(
        String::from(
            r#"
# Ockam Artifacts
This repositories includes infrastructures that

<!-- bats start -->
<!--
some commands
-->
<!-- bats end -->
<!-- bash start OCKAM_HOME -->
<!--
some commands 2
-->
<!-- bash end -->
    "#,
        ),
        "test_url".into(),
    );

    assert!(commands.len() == 2);
    assert_eq!(
        commands,
        vec![
            CommandScope {
                commands: "some commands".into(),
                ockam_enroll_data: "".into(),
                run_type: RunType::Bats,
                url: "test_url".into()
            },
            CommandScope {
                commands: "some commands 2".into(),
                ockam_enroll_data: "OCKAM_HOME".into(),
                run_type: RunType::Unknown,
                url: "test_url".into()
            }
        ]
    )
}
