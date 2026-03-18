use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};

use crate::{render::RenderFormat, resolve::ResolvedJob};

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub format: RenderFormat,
    pub runner_cmd: Option<String>,
    pub runner_args: Vec<String>,
    pub keep_config: bool,
    pub generated_config: Option<PathBuf>,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            format: RenderFormat::Xml,
            runner_cmd: None,
            runner_args: Vec::new(),
            keep_config: false,
            generated_config: None,
        }
    }
}

pub fn run_with_runner(job: &ResolvedJob, config_text: &str, options: &RunOptions) -> Result<i32> {
    let config_path = prepare_config_path(options.generated_config.clone(), options.format)?;
    fs::write(&config_path, config_text).with_context(|| {
        format!(
            "failed to write generated {}: {}",
            options.format.as_str().to_ascii_uppercase(),
            config_path.display()
        )
    })?;

    let runner_cmd = options
        .runner_cmd
        .clone()
        .or_else(|| std::env::var("DEE_RUNNER_CMD").ok())
        .unwrap_or_else(|| "dee".to_string());

    let split = shell_words::split(&runner_cmd)
        .with_context(|| format!("invalid runner command syntax: {runner_cmd}"))?;
    let (binary, default_args) = split
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty runner command"))?;

    let mut command = Command::new(binary);
    let merged_runner_args = merge_runner_args(&options.runner_args, &job.run.runner_args);
    prepare_known_dee_dirs(&merged_runner_args)?;
    command.args(default_args).args(&merged_runner_args);

    if !contains_format_arg(&merged_runner_args, options.format) {
        command.arg(options.format.cli_flag()).arg(&config_path);
    }

    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for (k, v) in &job.run.env {
        command.env(k, v);
    }

    let status = command
        .status()
        .with_context(|| format!("failed to execute runner: {runner_cmd}"))?;

    if !options.keep_config && options.generated_config.is_none() {
        let _ = fs::remove_file(&config_path);
    }

    match status.code() {
        Some(code) => Ok(code),
        None => bail!("runner terminated by signal"),
    }
}

fn merge_runner_args(base: &[String], extra: &[String]) -> Vec<String> {
    let mut merged = Vec::with_capacity(base.len() + extra.len());
    merged.extend(base.iter().cloned());
    merged.extend(extra.iter().cloned());
    merged
}

fn contains_format_arg(args: &[String], format: RenderFormat) -> bool {
    args.iter()
        .any(|a| a == format.cli_flag() || a == format.short_cli_flag())
}

fn prepare_known_dee_dirs(args: &[String]) -> Result<()> {
    let mut idx = 0usize;
    while idx < args.len() {
        let flag = args[idx].as_str();
        if matches!(
            flag,
            "--temp" | "--temp-dir" | "--output" | "--output-file" | "--log-file"
        ) && idx + 1 < args.len()
        {
            let raw = args[idx + 1].as_str();
            if let Some(path) = host_path_candidate(raw) {
                match flag {
                    "--temp" | "--temp-dir" => fs::create_dir_all(path).with_context(|| {
                        format!(
                            "failed to create temp directory for runner: {}",
                            path.display()
                        )
                    })?,
                    "--output" | "--output-file" | "--log-file" => {
                        if let Some(parent) = path.parent()
                            && !parent.as_os_str().is_empty()
                        {
                            fs::create_dir_all(parent).with_context(|| {
                                format!(
                                    "failed to create output/log parent for runner: {}",
                                    parent.display()
                                )
                            })?;
                        }
                    }
                    _ => {}
                }
            }
            idx += 2;
            continue;
        }
        idx += 1;
    }
    Ok(())
}

fn host_path_candidate(value: &str) -> Option<&Path> {
    if !cfg!(windows) && is_windows_drive_path(value) {
        return None;
    }
    let p = Path::new(value);
    Some(p)
}

fn is_windows_drive_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn prepare_config_path(path_override: Option<PathBuf>, format: RenderFormat) -> Result<PathBuf> {
    if let Some(path) = path_override {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create generated config directory: {}",
                    parent.display()
                )
            })?;
        }
        return Ok(path);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_millis();
    Ok(std::env::temp_dir().join(format!("dee-config-gen-{timestamp}.{}", format.as_str())))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::TempDir;

    use crate::{
        render::RenderFormat,
        runner::{RunOptions, run_with_runner},
        test_support::sample_resolved_job,
    };

    fn make_runner_script(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = fs::metadata(path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).unwrap();
        }
    }

    #[cfg(windows)]
    fn success_runner_script(tmp: &TempDir, marker: &Path) -> PathBuf {
        let path = tmp.path().join("success_runner.bat");
        make_runner_script(
            &path,
            &format!(
                "@echo off\r\necho ok> \"{}\"\r\nexit /b 0\r\n",
                marker.display()
            ),
        );
        path
    }

    #[cfg(not(windows))]
    fn success_runner_script(tmp: &TempDir, marker: &Path) -> PathBuf {
        let path = tmp.path().join("success_runner.sh");
        make_runner_script(
            &path,
            &format!(
                "#!/bin/sh\nprintf 'ok\\n' > \"{}\"\nexit 0\n",
                marker.display()
            ),
        );
        path
    }

    #[cfg(windows)]
    fn capture_args_runner_script(tmp: &TempDir, marker: &Path) -> PathBuf {
        let path = tmp.path().join("capture_args.bat");
        make_runner_script(
            &path,
            &format!(
                "@echo off\r\necho %* > \"{}\"\r\nexit /b 0\r\n",
                marker.display()
            ),
        );
        path
    }

    #[cfg(not(windows))]
    fn capture_args_runner_script(tmp: &TempDir, marker: &Path) -> PathBuf {
        let path = tmp.path().join("capture_args.sh");
        make_runner_script(
            &path,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"{}\"\n",
                marker.display()
            ),
        );
        path
    }

    fn script_runner_cmd(path: &Path) -> String {
        #[cfg(windows)]
        {
            path.display().to_string().replace('\\', "/")
        }
        #[cfg(not(windows))]
        {
            path.display().to_string()
        }
    }

    #[test]
    fn executes_runner_and_preserves_generated_file_when_requested() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("generated.json");
        let marker = tmp.path().join("runner_marker.txt");
        let script = success_runner_script(&tmp, &marker);
        let cmd = script_runner_cmd(&script);

        let code = run_with_runner(
            &sample_resolved_job(),
            "{\"job_config\":{}}",
            &RunOptions {
                format: RenderFormat::Json,
                runner_cmd: Some(cmd),
                runner_args: vec![],
                keep_config: true,
                generated_config: Some(config_path.clone()),
            },
        )
        .unwrap();

        assert_eq!(code, 0);
        assert!(config_path.exists());
        assert!(fs::read_to_string(marker).unwrap().contains("ok"));
    }

    #[test]
    fn prepares_known_runner_directories_for_host_paths() {
        let temp_dir = TempDir::new().unwrap();
        let output = temp_dir.path().join("out").join("x.ec3");
        let temp_path = temp_dir.path().join("tmpd");
        let log = temp_dir.path().join("logs").join("run.log");

        let args = vec![
            "--output".to_string(),
            output.display().to_string(),
            "--temp".to_string(),
            temp_path.display().to_string(),
            "--log-file".to_string(),
            log.display().to_string(),
            "--input-audio".to_string(),
            "Y:/testADM.wav".to_string(),
        ];

        super::prepare_known_dee_dirs(&args).unwrap();
        assert!(Path::new(&temp_path).is_dir());
        assert!(output.parent().unwrap().is_dir());
        assert!(log.parent().unwrap().is_dir());
    }

    #[test]
    fn detects_xml_flag_presence() {
        assert!(!super::contains_format_arg(
            &["--output".to_string()],
            RenderFormat::Xml
        ));
        assert!(super::contains_format_arg(
            &["--xml".to_string(), "y:/job.xml".to_string()],
            RenderFormat::Xml
        ));
        assert!(super::contains_format_arg(
            &["-x".to_string(), "job.xml".to_string()],
            RenderFormat::Xml
        ));
    }

    #[test]
    fn detects_json_flag_presence() {
        assert!(!super::contains_format_arg(
            &["--output".to_string()],
            RenderFormat::Json
        ));
        assert!(super::contains_format_arg(
            &["--json".to_string(), "y:/job.json".to_string()],
            RenderFormat::Json
        ));
        assert!(super::contains_format_arg(
            &["-j".to_string(), "job.json".to_string()],
            RenderFormat::Json
        ));
    }

    #[test]
    fn injects_json_flag_for_json_runs() {
        let tmp = TempDir::new().unwrap();
        let marker = tmp.path().join("args.txt");
        let script = capture_args_runner_script(&tmp, &marker);
        let cmd = script_runner_cmd(&script);

        let config_path = tmp.path().join("generated.json");
        let code = run_with_runner(
            &sample_resolved_job(),
            "{\"job_config\":{}}",
            &RunOptions {
                format: RenderFormat::Json,
                runner_cmd: Some(cmd),
                runner_args: vec![],
                keep_config: true,
                generated_config: Some(config_path.clone()),
            },
        )
        .unwrap();

        assert_eq!(code, 0);
        let args = fs::read_to_string(marker).unwrap();
        assert!(args.contains("--json"));
        assert!(args.contains(config_path.to_string_lossy().as_ref()));
        assert!(!args.contains("--xml"));
    }
}
