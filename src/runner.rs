use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};

use crate::resolve::ResolvedJob;

#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub runner_cmd: Option<String>,
    pub runner_args: Vec<String>,
    pub keep_xml: bool,
    pub generated_xml: Option<PathBuf>,
}

pub fn run_with_runner(job: &ResolvedJob, xml: &str, options: &RunOptions) -> Result<i32> {
    let xml_path = prepare_xml_path(options.generated_xml.clone())?;
    fs::write(&xml_path, xml)
        .with_context(|| format!("failed to write generated XML: {}", xml_path.display()))?;

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

    if !contains_xml_arg(&merged_runner_args) {
        command.arg("--xml").arg(&xml_path);
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

    if !options.keep_xml && options.generated_xml.is_none() {
        let _ = fs::remove_file(&xml_path);
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

fn contains_xml_arg(args: &[String]) -> bool {
    args.iter().any(|a| a == "--xml" || a == "-x")
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
    if is_windows_drive_path(value) {
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

fn prepare_xml_path(path_override: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = path_override {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create generated XML directory: {}",
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
    Ok(std::env::temp_dir().join(format!("dee-config-gen-{timestamp}.xml")))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use crate::{
        runner::{RunOptions, run_with_runner},
        test_support::sample_resolved_job,
    };

    #[test]
    fn executes_runner_and_preserves_generated_file_when_requested() {
        let tmp = TempDir::new().unwrap();
        let xml_path = tmp.path().join("generated.xml");
        let marker = tmp.path().join("runner_marker.txt");

        let cmd = format!(
            "bash -lc 'echo ok > {} && exit 0'",
            marker.to_string_lossy()
        );

        let code = run_with_runner(
            &sample_resolved_job(),
            "<job_config/>",
            &RunOptions {
                runner_cmd: Some(cmd),
                runner_args: vec![],
                keep_xml: true,
                generated_xml: Some(xml_path.clone()),
            },
        )
        .unwrap();

        assert_eq!(code, 0);
        assert!(xml_path.exists());
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
        assert!(!super::contains_xml_arg(&["--output".to_string()]));
        assert!(super::contains_xml_arg(&[
            "--xml".to_string(),
            "y:/job.xml".to_string()
        ]));
        assert!(super::contains_xml_arg(&[
            "-x".to_string(),
            "job.xml".to_string()
        ]));
    }
}
