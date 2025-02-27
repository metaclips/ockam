use std::io;

use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};

use crate::branding::BrandingCompileEnvVars;
use crate::{docs, OckamCommand};

const LONG_ABOUT: &str = include_str!("./static/long_about.txt");
const PREVIEW_TAG: &str = include_str!("../static/preview_tag.txt");
const AFTER_LONG_HELP: &str = include_str!("./static/after_long_help.txt");

/// Generate shell completion scripts
#[derive(Clone, Debug, Args)]
#[command(
    arg_required_else_help = true,
    long_about = docs::about(LONG_ABOUT),
    before_help = docs::before_help(PREVIEW_TAG),
    after_long_help = docs::after_help(AFTER_LONG_HELP)
)]
pub struct CompletionCommand {
    /// The type of shell
    #[arg(display_order = 900, long, short)]
    shell: Shell,
}

impl CompletionCommand {
    pub fn run(self) -> miette::Result<()> {
        generate(
            self.shell,
            &mut OckamCommand::command(),
            BrandingCompileEnvVars::bin_name(),
            &mut io::stdout(),
        );
        Ok(())
    }

    pub fn name(&self) -> String {
        "completion".to_string()
    }
}
