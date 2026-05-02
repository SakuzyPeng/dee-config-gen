use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};

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
        #[arg(
            short = 'i',
            long,
            value_name = "PATH",
            help = "YAML/JSON job spec to read"
        )]
        input: PathBuf,
        #[arg(
            short = 'o',
            long,
            value_name = "PATH",
            help = "Write generated config to a file instead of stdout"
        )]
        output: Option<PathBuf>,
        #[arg(
            long,
            value_enum,
            default_value_t = RenderFormat::Xml,
            help = "Render target format"
        )]
        format: RenderFormat,
        #[arg(
            long,
            value_name = "TEMPLATE_ID",
            help = "Override template_id from the input spec"
        )]
        template: Option<String>,
        #[arg(
            long,
            default_value_t = false,
            help = "Validate and resolve without writing config output"
        )]
        dry_run: bool,
        #[arg(
            long,
            default_value_t = false,
            help = "Allow overriding fixed profile defaults such as profile=music constraints"
        )]
        allow_fixed_override: bool,
        #[arg(
            long,
            default_value_t = 'Y',
            value_name = "LETTER",
            help = "Windows drive letter used when normalizing generated XML paths"
        )]
        win_drive: char,
    },
    /// Validate YAML/JSON job input and target DEE output format.
    Validate {
        #[arg(
            short = 'i',
            long,
            value_name = "PATH",
            help = "YAML/JSON job spec to validate"
        )]
        input: PathBuf,
        #[arg(
            long,
            value_enum,
            default_value_t = RenderFormat::Xml,
            help = "Target format to validate against"
        )]
        format: RenderFormat,
        #[arg(
            long,
            value_name = "TEMPLATE_ID",
            help = "Override template_id from the input spec"
        )]
        template: Option<String>,
        #[arg(
            long,
            default_value_t = false,
            help = "Allow overriding fixed profile defaults such as profile=music constraints"
        )]
        allow_fixed_override: bool,
    },
    /// Generate DEE XML/JSON config and invoke an external runner command.
    Run {
        #[arg(
            short = 'i',
            long,
            value_name = "PATH",
            help = "YAML/JSON job spec to run"
        )]
        input: PathBuf,
        #[arg(
            long,
            value_enum,
            default_value_t = RenderFormat::Xml,
            help = "Render target format passed to the runner"
        )]
        format: RenderFormat,
        #[arg(
            long,
            value_name = "TEMPLATE_ID",
            help = "Override template_id from the input spec"
        )]
        template: Option<String>,
        #[arg(
            long,
            default_value_t = false,
            help = "Allow overriding fixed profile defaults such as profile=music constraints"
        )]
        allow_fixed_override: bool,
        #[arg(
            long,
            default_value_t = 'Y',
            value_name = "LETTER",
            help = "Windows drive letter used when normalizing generated XML paths"
        )]
        win_drive: char,
        #[arg(
            long,
            value_name = "COMMAND",
            help = "Runner command to execute; overrides DEE_RUNNER_CMD"
        )]
        runner_cmd: Option<String>,
        #[arg(
            long = "runner-arg",
            value_name = "ARG",
            action = ArgAction::Append,
            help = "Additional runner argument; repeat for multiple arguments"
        )]
        runner_args: Vec<String>,
        #[arg(
            long,
            default_value_t = false,
            help = "Keep the generated config next to the input spec"
        )]
        keep_config: bool,
        #[arg(
            long,
            value_name = "PATH",
            help = "Explicit path for the generated runner config"
        )]
        generated_config: Option<PathBuf>,
    },
    /// Discover supported templates and their parameters.
    Templates {
        #[command(subcommand)]
        command: TemplateCommands,
    },
    /// Write a bundled starter job spec to disk.
    Init {
        #[arg(
            long,
            value_name = "TEMPLATE_ID",
            conflicts_with = "example",
            help = "Template whose canonical bundled example should be written"
        )]
        template: Option<String>,
        #[arg(
            long,
            value_name = "EXAMPLE",
            help = "Bundled example file name to write; examples/ prefix is optional"
        )]
        example: Option<String>,
        #[arg(
            short = 'o',
            long,
            value_name = "PATH",
            help = "Output path; defaults to job.yaml or job.json based on the example"
        )]
        output: Option<PathBuf>,
        #[arg(
            long,
            default_value_t = false,
            help = "Overwrite the output path if it already exists"
        )]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommands {
    /// List supported templates.
    List {
        #[arg(long, value_enum, default_value_t = CliOutputFormat::Text, help = "Output format")]
        format: CliOutputFormat,
    },
    /// Show details for one supported template.
    Show {
        #[arg(value_name = "TEMPLATE_ID", help = "Template id to inspect")]
        template_id: String,
        #[arg(long, value_enum, default_value_t = CliOutputFormat::Text, help = "Output format")]
        format: CliOutputFormat,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliOutputFormat {
    Text,
    Json,
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

    #[test]
    fn init_rejects_template_and_example_together() {
        let err = Cli::command()
            .try_get_matches_from([
                "dee-config-gen",
                "init",
                "--template",
                "atmos_ec3_v1",
                "--example",
                "pcm_ddp_single.dd.yaml",
            ])
            .expect_err("--template and --example should conflict");

        assert_eq!(err.kind(), ErrorKind::ArgumentConflict);
    }
}
