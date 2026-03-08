use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "dee-config-gen")]
#[command(about = "Generate and validate DEE XML job configs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate XML from YAML/JSON job input.
    Generate {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        allow_fixed_override: bool,
        #[arg(long, default_value_t = 'Y')]
        win_drive: char,
    },
    /// Validate YAML/JSON job input and report issues.
    Validate {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_fixed_override: bool,
    },
    /// Generate XML and invoke an external DEE runner command.
    Run {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_fixed_override: bool,
        #[arg(long, default_value_t = 'Y')]
        win_drive: char,
        #[arg(long)]
        runner_cmd: Option<String>,
        #[arg(long = "runner-arg", action = ArgAction::Append)]
        runner_args: Vec<String>,
        #[arg(long, default_value_t = false)]
        keep_xml: bool,
        #[arg(long)]
        generated_xml: Option<PathBuf>,
    },
}
