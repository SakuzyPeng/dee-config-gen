use std::io::Write;

use anyhow::{Context, Result};
use clap::Parser;
use dee_config_gen::{
    Cli, Commands, ResolveOptions, RunOptions, default_xml_path_from_input, load_job_file,
    render_xml, resolve_job, run_with_runner, write_xml_output,
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
                    "validation passed: template_id={}, profile={:?}, job_mode={:?}, atmos_mode={:?}",
                    resolved.template_id, resolved.profile, resolved.job_mode, resolved.atmos_mode
                );
                return Ok(());
            }

            let xml = render_xml(&resolved);
            match output {
                Some(path) => {
                    write_xml_output(&path, &xml)?;
                    println!("wrote {}", path.display());
                }
                None => {
                    let mut stdout = std::io::stdout().lock();
                    stdout
                        .write_all(xml.as_bytes())
                        .context("failed to write XML to stdout")?;
                }
            }
        }
        Commands::Validate {
            input,
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
            println!(
                "valid: template_id={}, profile={:?}, job_mode={:?}, atmos_mode={:?}",
                resolved.template_id, resolved.profile, resolved.job_mode, resolved.atmos_mode
            );
        }
        Commands::Run {
            input,
            template,
            allow_fixed_override,
            win_drive,
            runner_cmd,
            runner_args,
            keep_xml,
            generated_xml,
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
            let xml = render_xml(&resolved);
            let xml_output = if keep_xml {
                Some(generated_xml.unwrap_or_else(|| default_xml_path_from_input(&input)))
            } else {
                generated_xml
            };
            let code = run_with_runner(
                &resolved,
                &xml,
                &RunOptions {
                    runner_cmd,
                    runner_args,
                    keep_xml,
                    generated_xml: xml_output,
                },
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
    }

    Ok(())
}
