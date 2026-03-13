mod common;

use std::{fs, process::Command};

use serde_json::Value;
use tempfile::TempDir;

use common::{THD_WAV_LIST_XSD_CONTRACT_PATH, load_xsd_contract_at};

#[test]
#[ignore = "runs python script to regenerate thd_wav_list contract snapshot"]
fn thd_wav_list_xsd_contract_snapshot_is_up_to_date() {
    let current = load_xsd_contract_at(THD_WAV_LIST_XSD_CONTRACT_PATH);
    let temp_dir = TempDir::new().expect("create temp dir");
    let output_path = temp_dir.path().join("contract.generated.json");

    let status = Command::new("python3")
        .args([
            "scripts/extract_xsd_contract.py",
            "--template-id",
            &current.meta.template_id,
            "--dee-version",
            &current.meta.dee_version,
            "--exported-at",
            &current.meta.exported_at,
            "--raw-dir",
            "tests/fixtures/xsd/raw",
            "--filter-path-prefix",
            "/job_config/input/audio/wav_list",
            "--filter-path-prefix",
            "/job_config/filter/audio/encode_to_dthd",
            "--filter-path-prefix",
            "/job_config/output/mlp",
            "--output",
            output_path
                .to_str()
                .expect("temp output path should be valid utf-8"),
        ])
        .status()
        .expect("failed to run python3 for thd_wav_list contract snapshot");

    assert!(
        status.success(),
        "thd_wav_list xsd extraction script failed"
    );

    let expected_text = fs::read_to_string(THD_WAV_LIST_XSD_CONTRACT_PATH)
        .expect("read committed thd_wav_list contract");
    let generated_text =
        fs::read_to_string(&output_path).expect("read generated thd_wav_list contract");

    let expected: Value =
        serde_json::from_str(&expected_text).expect("parse committed thd_wav_list contract");
    let generated: Value =
        serde_json::from_str(&generated_text).expect("parse generated thd_wav_list contract");

    assert_eq!(
        generated, expected,
        "thd_wav_list xsd contract snapshot is stale"
    );
}
