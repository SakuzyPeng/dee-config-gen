mod cli;

use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands};
use dee_config_gen::spec::{default_config_path_from_input, write_config_output};
use dee_config_gen::{
    GenerateOptions, ResolveOptions, RunOptions, generate_config, read_job, run_job,
};

fn main() {
    if let Err(err) = real_main() {
        eprintln!("error: {err}");
        std::process::exit(1);
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
    }

    Ok(())
}
