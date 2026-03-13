mod common;

use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

use common::THD_ATMOS_WAV_LIST_XSD_CONTRACT_PATH;

#[test]
#[ignore = "runs python script to regenerate thd_atmos_wav_list contract snapshot"]
fn thd_atmos_wav_list_xsd_contract_snapshot_is_up_to_date() {
    let temp = TempDir::new().expect("tempdir");
    let output_path = temp.path().join("contract.thd_atmos_wav_list_v1.json");

    let status = Command::new("python3")
        .args([
            "scripts/extract_xsd_contract.py",
            "--template-id",
            "thd_atmos_wav_list_v1",
            "--dee-version",
            "5.2.1",
            "--exported-at",
            "2026-03-13T00:00:00+08:00",
            "--raw-dir",
            "tests/fixtures/xsd/raw",
            "--output",
            output_path.to_str().expect("utf-8"),
            "--filter-path-prefix",
            "/job_config/input/audio/atmos_mezz",
            "--filter-path-prefix",
            "/job_config/input/audio/wav_list",
            "--filter-path-prefix",
            "/job_config/filter/audio/encode_to_dthd",
            "--filter-path-prefix",
            "/job_config/output/mlp",
        ])
        .status()
        .expect("failed to run python3 for thd_atmos_wav_list contract snapshot");
    assert!(
        status.success(),
        "thd_atmos_wav_list xsd extraction script failed"
    );

    let expected_text =
        fs::read_to_string(THD_ATMOS_WAV_LIST_XSD_CONTRACT_PATH).expect("read committed contract");
    let generated_text = fs::read_to_string(&output_path).expect("read generated contract");

    let expected: Value = serde_json::from_str(&expected_text).expect("parse committed contract");
    let generated: Value = serde_json::from_str(&generated_text).expect("parse generated contract");

    assert_eq!(
        expected, generated,
        "thd_atmos_wav_list xsd contract snapshot is stale"
    );
}
