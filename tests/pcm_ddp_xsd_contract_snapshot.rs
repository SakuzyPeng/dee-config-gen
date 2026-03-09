mod common;

use std::{fs, process::Command};

use serde_json::Value;
use tempfile::TempDir;

use common::{PCM_DDP_XSD_CONTRACT_PATH, load_xsd_contract_at};

#[test]
#[ignore = "runs python script to regenerate pcm_ddp contract snapshot"]
fn pcm_ddp_xsd_contract_snapshot_is_up_to_date() {
    let current = load_xsd_contract_at(PCM_DDP_XSD_CONTRACT_PATH);
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
            "/job_config/filter/audio/pcm_to_ddp",
            "--output",
            output_path
                .to_str()
                .expect("temp output path should be valid utf-8"),
        ])
        .status()
        .expect("failed to run python3; install python3 to regenerate xsd contract snapshot");

    assert!(
        status.success(),
        "pcm_ddp xsd extraction script failed; rerun manually and inspect errors"
    );

    let expected_text =
        fs::read_to_string(PCM_DDP_XSD_CONTRACT_PATH).expect("read committed pcm_ddp xsd contract");
    let generated_text = fs::read_to_string(&output_path).expect("read generated pcm_ddp contract");

    let expected: Value = serde_json::from_str(&expected_text).expect("parse committed contract");
    let generated: Value = serde_json::from_str(&generated_text).expect("parse generated contract");

    assert_eq!(
        generated,
        expected,
        "pcm_ddp xsd contract snapshot is stale. regenerate with:\npython3 scripts/extract_xsd_contract.py --template-id {} --dee-version {} --exported-at {} --raw-dir tests/fixtures/xsd/raw --filter-path-prefix /job_config/filter/audio/pcm_to_ddp --output {}",
        current.meta.template_id,
        current.meta.dee_version,
        current.meta.exported_at,
        PCM_DDP_XSD_CONTRACT_PATH,
    );
}
