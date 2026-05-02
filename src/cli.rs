use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

use dee_config_gen::RenderFormat;

#[derive(Debug, Parser)]
#[command(name = "dee-config-gen")]
#[command(about = "Generate and validate DEE XML/JSON job configs")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Generate DEE XML/JSON config from YAML/JSON job input.
    Generate {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = RenderFormat::Xml)]
        format: RenderFormat,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
        #[arg(long, default_value_t = false)]
        allow_fixed_override: bool,
        #[arg(long, default_value_t = 'Y')]
        win_drive: char,
    },
    /// Validate YAML/JSON job input and target DEE output format.
    Validate {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = RenderFormat::Xml)]
        format: RenderFormat,
        #[arg(long)]
        template: Option<String>,
        #[arg(long, default_value_t = false)]
        allow_fixed_override: bool,
    },
    /// Generate DEE XML/JSON config and invoke an external runner command.
    Run {
        #[arg(short = 'i', long)]
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = RenderFormat::Xml)]
        format: RenderFormat,
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
        keep_config: bool,
        #[arg(long)]
        generated_config: Option<PathBuf>,
    },
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, error::ErrorKind};

    use super::Cli;

    #[test]
    fn exposes_top_level_version_flag() {
        let err = Cli::command()
            .try_get_matches_from(["dee-config-gen", "--version"])
            .expect_err("--version should render version and exit");

        assert_eq!(err.kind(), ErrorKind::DisplayVersion);
        assert!(err.to_string().contains(env!("CARGO_PKG_VERSION")));
    }
}
