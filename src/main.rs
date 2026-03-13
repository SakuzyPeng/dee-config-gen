use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use dee_config_gen::{
    Cli, Commands, ResolveOptions, RunOptions, default_config_path_from_input, load_job_file,
    render_config, resolve_job, run_with_runner, write_config_output,
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
            let spec = load_job_file(&input)?;
            let resolved = resolve_job(
                spec,
                &ResolveOptions {
                    template_override: template,
                    allow_fixed_override,
                    windows_drive: win_drive,
                },
            )?;

            if dry_run {
                println!(
                    "validation passed: template_id={}, profile={:?}, job_mode={:?}, encode_mode={:?}, format={}",
                    resolved.template_id,
                    resolved.profile,
                    resolved.job_mode,
                    resolved.encode_mode,
                    format.as_str()
                );
                return Ok(());
            }

            let rendered = render_config(&resolved, format)?;
            match output {
                Some(path) => {
                    write_config_output(&path, &rendered, format)?;
                    println!("wrote {}", path.display());
                }
                None => {
                    let mut stdout = std::io::stdout().lock();
                    stdout.write_all(rendered.as_bytes()).with_context(|| {
                        format!(
                            "failed to write {} to stdout",
                            format.as_str().to_ascii_uppercase()
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
            let spec = load_job_file(&input)?;
            let resolved = resolve_job(
                spec,
                &ResolveOptions {
                    template_override: template,
                    allow_fixed_override,
                    windows_drive: 'Y',
                },
            )?;
            let _ = render_config(&resolved, format)?;
            println!(
                "valid: template_id={}, profile={:?}, job_mode={:?}, encode_mode={:?}, format={}",
                resolved.template_id,
                resolved.profile,
                resolved.job_mode,
                resolved.encode_mode,
                format.as_str()
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
            let spec = load_job_file(&input)?;
            let resolved = resolve_job(
                spec,
                &ResolveOptions {
                    template_override: template,
                    allow_fixed_override,
                    windows_drive: win_drive,
                },
            )?;
            let rendered = render_config(&resolved, format)?;
            let config_output = if keep_config {
                Some(
                    generated_config
                        .unwrap_or_else(|| default_config_path_from_input(&input, format)),
                )
            } else {
                generated_config
            };
            let code = run_with_runner(
                &resolved,
                &rendered,
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
