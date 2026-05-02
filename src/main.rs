mod cli;
mod cli_catalog;

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::{Cli, CliOutputFormat, Commands, TemplateCommands};
use cli_catalog::{
    default_init_output_path, list_templates_json_string, list_templates_text, select_init_example,
    show_template_json_string, show_template_text,
};
use dee_config_gen::spec::{default_config_path_from_input, write_config_output};
use dee_config_gen::{
    GenerateOptions, ResolveOptions, RunOptions, generate_config, read_job, run_job,
};

fn main() {
    if let Err(err) = real_main() {
        print_error(&err);
        std::process::exit(1);
    }
}

fn print_error(err: &anyhow::Error) {
    eprintln!("error: {err}");
    let causes = err.chain().skip(1).collect::<Vec<_>>();
    if !causes.is_empty() {
        eprintln!();
        eprintln!("caused by:");
        for cause in causes {
            eprintln!("  - {cause}");
        }
    }
}

fn real_main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input,
            output,
            format,
            template,
            dry_run,
            allow_fixed_override,
            win_drive,
        } => {
            let generated = generate_config(
                read_job(&input)?,
                &GenerateOptions {
                    resolve: ResolveOptions {
                        template_override: template,
                        allow_fixed_override,
                        windows_drive: win_drive,
                    },
                    format,
                },
            )?;

            if dry_run {
                println!(
                    "validation passed: template_id={}, profile={:?}, job_mode={:?}, encode_mode={:?}, format={}",
                    generated.resolved.template_id,
                    generated.resolved.profile,
                    generated.resolved.job_mode,
                    generated.resolved.encode_mode,
                    generated.format.as_str()
                );
                return Ok(());
            }

            match output {
                Some(path) => {
                    write_config_output(&path, &generated.rendered, generated.format)?;
                    println!("wrote {}", path.display());
                }
                None => {
                    let mut stdout = std::io::stdout().lock();
                    stdout
                        .write_all(generated.rendered.as_bytes())
                        .with_context(|| {
                            format!(
                                "failed to write {} to stdout",
                                generated.format.as_str().to_ascii_uppercase()
                            )
                        })?;
                }
            }
        }
        Commands::Validate {
            input,
            format,
            template,
            allow_fixed_override,
        } => {
            let generated = generate_config(
                read_job(&input)?,
                &GenerateOptions {
                    resolve: ResolveOptions {
                        template_override: template,
                        allow_fixed_override,
                        windows_drive: 'Y',
                    },
                    format,
                },
            )?;
            println!(
                "valid: template_id={}, profile={:?}, job_mode={:?}, encode_mode={:?}, format={}",
                generated.resolved.template_id,
                generated.resolved.profile,
                generated.resolved.job_mode,
                generated.resolved.encode_mode,
                generated.format.as_str()
            );
        }
        Commands::Run {
            input,
            format,
            template,
            allow_fixed_override,
            win_drive,
            runner_cmd,
            runner_args,
            keep_config,
            generated_config,
        } => {
            let config_output = if keep_config {
                Some(
                    generated_config
                        .unwrap_or_else(|| default_config_path_from_input(&input, format)),
                )
            } else {
                generated_config
            };
            let code = run_job(
                read_job(&input)?,
                &GenerateOptions {
                    resolve: ResolveOptions {
                        template_override: template,
                        allow_fixed_override,
                        windows_drive: win_drive,
                    },
                    format,
                },
                &RunOptions {
                    format,
                    runner_cmd,
                    runner_args,
                    keep_config,
                    generated_config: config_output,
                },
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Templates { command } => match command {
            TemplateCommands::List { format } => match format {
                CliOutputFormat::Text => print!("{}", list_templates_text()?),
                CliOutputFormat::Json => println!("{}", list_templates_json_string()?),
            },
            TemplateCommands::Show {
                template_id,
                format,
            } => match format {
                CliOutputFormat::Text => print!("{}", show_template_text(&template_id)?),
                CliOutputFormat::Json => println!("{}", show_template_json_string(&template_id)?),
            },
        },
        Commands::Init {
            template,
            example,
            output,
            force,
        } => {
            let asset = select_init_example(template.as_deref(), example.as_deref())?;
            let output_path = output.unwrap_or_else(|| default_init_output_path(asset.name));
            write_init_file(&output_path, asset.content, force)?;
            println!("wrote {} from {}", output_path.display(), asset.name);
        }
    }

    Ok(())
}

fn write_init_file(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "refusing to overwrite existing file: {}; pass --force to overwrite",
            path.display()
        );
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory: {}", parent.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("failed to write starter job spec: {}", path.display()))
}
