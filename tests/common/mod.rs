#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{Mutex, MutexGuard, OnceLock},
};

use dee_config_gen::{
    ResolveOptions, read_job, resolve_job,
    spec::SourceTag,
    spec::{EncodeMode, FilterOverrides, JobSpec},
};
use serde::Deserialize;

pub const ATMOS_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.atmos_ec3_v1.json";
pub const AC4_IMS_ATMOS_XSD_CONTRACT_PATH: &str =
    "tests/fixtures/xsd/contract.ac4_ims_atmos_v1.json";
pub const AC4_IMS_PCM_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.ac4_ims_pcm_v1.json";
pub const PCM_DDP_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.pcm_ddp_v1.json";
pub const THD_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.thd_v1.json";
pub const THD_WAV_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.thd_wav_v1.json";
pub const THD_WAV_LIST_XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.thd_wav_list_v1.json";
pub const THD_ATMOS_WAV_XSD_CONTRACT_PATH: &str =
    "tests/fixtures/xsd/contract.thd_atmos_wav_v1.json";
pub const THD_ATMOS_WAV_LIST_XSD_CONTRACT_PATH: &str =
    "tests/fixtures/xsd/contract.thd_atmos_wav_list_v1.json";

#[derive(Debug, Clone, Deserialize)]
pub struct XsdContract {
    pub meta: ContractMeta,
    pub elements: Vec<ContractElement>,
    pub attributes: Vec<ContractAttribute>,
    pub simple_types: Vec<ContractSimpleType>,
    pub paths: Vec<ContractPath>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractMeta {
    pub template_id: String,
    pub dee_version: String,
    pub exported_at: String,
    pub source_files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractElement {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub min_occurs: String,
    pub max_occurs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractAttribute {
    pub name: String,
    pub owner_path: String,
    pub required: bool,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractSimpleType {
    pub name: String,
    pub base: Option<String>,
    pub enumerations: Vec<String>,
    pub min_inclusive: Option<String>,
    pub max_inclusive: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractPath {
    pub path: String,
    pub element: String,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
}

pub fn load_xsd_contract_at(path: &str) -> XsdContract {
    let content = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&content).unwrap_or_else(|e| panic!("failed to parse {path} as JSON: {e}"))
}

pub fn load_xsd_contract() -> XsdContract {
    load_xsd_contract_at(ATMOS_XSD_CONTRACT_PATH)
}

pub fn find_filter_param_path<'a>(contract: &'a XsdContract, key: &str) -> Option<&'a str> {
    find_filter_param_path_with_prefix(
        contract,
        key,
        "/job_config/filter/audio/encode_to_atmos_ddp/",
    )
}

pub fn find_filter_param_path_with_prefix<'a>(
    contract: &'a XsdContract,
    key: &str,
    prefix: &str,
) -> Option<&'a str> {
    contract
        .paths
        .iter()
        .find(|entry| entry.element == key && entry.path.contains(prefix))
        .map(|entry| entry.path.as_str())
        .or_else(|| {
            contract
                .paths
                .iter()
                .find(|entry| entry.element == key)
                .map(|entry| entry.path.as_str())
        })
}

pub fn requires_dolby_xsd_path(sources: &[SourceTag]) -> bool {
    sources.contains(&SourceTag::DolbyOfficial)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceTier {
    Official,
    RuntimeVerifiedHidden,
    FolkloreUnverified,
}

pub const RUNTIME_VERIFIED_HIDDEN_PARAMS: &[&str] = &["encoding_backend", "encoder_mode"];
pub const FOLKLORE_UNVERIFIED_PARAMS: &[&str] = &[];

pub fn evidence_tier_for_param(key: &str, sources: &[SourceTag]) -> EvidenceTier {
    if requires_dolby_xsd_path(sources) {
        EvidenceTier::Official
    } else if RUNTIME_VERIFIED_HIDDEN_PARAMS.contains(&key) {
        EvidenceTier::RuntimeVerifiedHidden
    } else {
        EvidenceTier::FolkloreUnverified
    }
}

pub fn contract_path_label(contract: &XsdContract, key: &str, sources: &[SourceTag]) -> String {
    match evidence_tier_for_param(key, sources) {
        EvidenceTier::Official => find_filter_param_path(contract, key)
            .unwrap_or_else(|| panic!("missing xsd path for official param_key={key}"))
            .to_string(),
        EvidenceTier::RuntimeVerifiedHidden => find_filter_param_path(contract, key)
            .unwrap_or("<runtime_verified_hidden>")
            .to_string(),
        EvidenceTier::FolkloreUnverified => find_filter_param_path(contract, key)
            .unwrap_or("<folklore_unverified>")
            .to_string(),
    }
}

use tempfile::TempDir;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn require_command(name: &str) {
    let status = Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .unwrap_or_else(|err| panic!("failed to probe command '{name}': {err}"));
    assert!(
        status.success(),
        "required command '{name}' is not available"
    );
}

pub fn create_temp_layout(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
}

pub fn write_text(path: &Path, content: &str) {
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

pub fn run_dee(xml_path: &Path, log_path: &Path) -> Output {
    Command::new("gtimeout")
        .arg("120")
        .arg("dee")
        .args(["--xml", xml_path.to_str().expect("utf-8 xml path")])
        .args(["--log-file", log_path.to_str().expect("utf-8 log path")])
        .arg("--stdout")
        .output()
        .unwrap_or_else(|err| panic!("failed to run dee for {}: {err}", xml_path.display()))
}

pub fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

pub fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn assert_failure(output: &Output, context: &str, expected_fragment: &str) {
    assert!(
        !output.status.success(),
        "{context} should fail, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains(expected_fragment) || stderr.contains(expected_fragment),
        "{context} should mention '{expected_fragment}', stdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
}

pub fn assert_output_exists(path: &Path, context: &str) {
    assert!(path.exists(), "{context} should produce {}", path.display());
    let metadata =
        fs::metadata(path).unwrap_or_else(|err| panic!("failed to stat {}: {err}", path.display()));
    assert!(
        metadata.len() > 0,
        "{context} should produce non-empty output"
    );
}

pub fn runtime_suite_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("lock runtime suite")
}

pub fn local_testfile_path(relative_path: &str) -> PathBuf {
    repo_root().join(relative_path)
}

pub fn has_local_testfiles(relative_paths: &[&str]) -> bool {
    relative_paths
        .iter()
        .all(|path| local_testfile_path(path).exists())
}

pub fn skip_missing_local_testfiles(context: &str, relative_paths: &[&str]) -> bool {
    let missing = relative_paths
        .iter()
        .map(|path| local_testfile_path(path))
        .filter(|path| !path.exists())
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return false;
    }

    eprintln!(
        "SKIP {context}: local runtime fixtures are missing. Expected: {}",
        missing
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    true
}

pub fn runtime_pcm_mono_stems(channel_count: usize) -> (TempDir, String, Vec<String>) {
    require_command("ffmpeg");

    let temp = TempDir::new().expect("mono stem temp dir");
    let stem_dir = temp.path().join("stems");
    fs::create_dir_all(&stem_dir).expect("create stem dir");

    for channel_idx in 0..channel_count {
        let out = stem_dir.join(format!("stem_{channel_idx:02}.wav"));
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-loglevel",
                "error",
                "-nostats",
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=48000:cl=mono",
                "-t",
                "1",
                "-c:a",
                "pcm_s16le",
            ])
            .arg(&out)
            .status()
            .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
        assert!(
            status.success(),
            "ffmpeg stem generation failed for channel {channel_idx}"
        );
    }

    let file_names = (0..channel_count)
        .map(|idx| format!("stem_{idx:02}.wav"))
        .collect::<Vec<_>>();
    (temp, stem_dir.display().to_string(), file_names)
}

pub fn runtime_pcm_wav_input_51() -> (TempDir, String, Vec<String>) {
    require_command("ffmpeg");

    let temp = TempDir::new().expect("5.1 wav temp dir");
    let input_dir = temp.path().join("inputs");
    fs::create_dir_all(&input_dir).expect("create 5.1 wav dir");

    let out = input_dir.join("input_6ch_runtime.wav");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-nostats",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=48000:cl=5.1",
            "-t",
            "1",
            "-c:a",
            "pcm_s16le",
        ])
        .arg(&out)
        .status()
        .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
    assert!(status.success(), "ffmpeg 5.1 wav generation failed");

    (
        temp,
        input_dir.display().to_string(),
        vec!["input_6ch_runtime.wav".to_string()],
    )
}

pub fn create_mono_wav_stems(channel_count: usize) -> (TempDir, String, Vec<String>) {
    let temp = TempDir::new().expect("create mono stem tempdir");
    let storage = temp.path().join("stems");
    fs::create_dir_all(&storage).expect("create mono stem dir");

    for idx in 0..channel_count {
        let path = storage.join(format!("stem_{idx:02}.wav"));
        write_test_wav(&path, 1, 16);
    }

    let file_names = (0..channel_count)
        .map(|idx| format!("stem_{idx:02}.wav"))
        .collect();
    (temp, storage.display().to_string(), file_names)
}

pub fn create_test_wav_inputs(
    channel_count: usize,
    bits_per_sample: u16,
    file_count: usize,
) -> (TempDir, String, Vec<String>) {
    let temp = TempDir::new().expect("create test wav tempdir");
    let storage = temp.path().join("inputs");
    fs::create_dir_all(&storage).expect("create test wav dir");

    for idx in 0..file_count {
        let path = storage.join(format!("input_{idx:02}.wav"));
        write_test_wav(&path, channel_count as u16, bits_per_sample);
    }

    let file_names = (0..file_count)
        .map(|idx| format!("input_{idx:02}.wav"))
        .collect();
    (temp, storage.display().to_string(), file_names)
}

fn write_test_wav(path: &Path, channels: u16, bits_per_sample: u16) {
    use std::io::Write;

    let sample_rate = 48_000_u32;
    let data_size = u32::from(channels) * u32::from(bits_per_sample / 8) * 8;
    let byte_rate = sample_rate * u32::from(channels) * u32::from(bits_per_sample / 8);
    let block_align = channels * (bits_per_sample / 8);
    let riff_size = 36 + data_size;

    let mut file = fs::File::create(path).expect("create wav");
    file.write_all(b"RIFF").expect("riff");
    file.write_all(&riff_size.to_le_bytes()).expect("riff size");
    file.write_all(b"WAVE").expect("wave");
    file.write_all(b"fmt ").expect("fmt");
    file.write_all(&16_u32.to_le_bytes()).expect("fmt size");
    file.write_all(&1_u16.to_le_bytes()).expect("pcm");
    file.write_all(&channels.to_le_bytes()).expect("channels");
    file.write_all(&sample_rate.to_le_bytes())
        .expect("sample rate");
    file.write_all(&byte_rate.to_le_bytes()).expect("byte rate");
    file.write_all(&block_align.to_le_bytes())
        .expect("block align");
    file.write_all(&bits_per_sample.to_le_bytes())
        .expect("bits per sample");
    file.write_all(b"data").expect("data");
    file.write_all(&data_size.to_le_bytes()).expect("data size");
    file.write_all(&vec![0_u8; data_size as usize])
        .expect("samples");
}

pub fn base_job_file() -> JobSpec {
    let mut job =
        read_job(Path::new("examples/atmos_ec3_single.streaming.yaml")).expect("load example");
    job.filter = FilterOverrides::default();
    job
}

pub fn set_encode_mode(job: &mut JobSpec, mode: &str) {
    job.encode_mode = match mode {
        "streaming" => EncodeMode::Streaming,
        "dd" => EncodeMode::Dd,
        "ddp" => EncodeMode::Ddp,
        "mlp" => EncodeMode::Mlp,
        "bluray" => EncodeMode::Bluray,
        "ddp71" => EncodeMode::Ddp71,
        "ac4" => EncodeMode::Ac4,
        other => panic!("unsupported encode mode in test: {other}"),
    };
}

pub fn resolve_with_defaults(job: JobSpec) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
}
