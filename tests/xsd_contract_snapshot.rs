mod common;

use std::{fs, process::Command};

use serde_json::Value;
use tempfile::TempDir;

use common::load_xsd_contract;

#[test]
#[ignore = "runs python script to regenerate contract snapshot"]
fn xsd_contract_snapshot_is_up_to_date() {
    let current = load_xsd_contract();
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
            "--output",
            output_path
                .to_str()
                .expect("temp output path should be valid utf-8"),
        ])
        .status()
        .expect("failed to run python3; install python3 to regenerate xsd contract snapshot");

    assert!(
        status.success(),
        "xsd extraction script failed; rerun manually and inspect errors"
    );

    let expected_text = fs::read_to_string("tests/fixtures/xsd/contract.atmos_ec3_v1.json")
        .expect("read committed xsd contract");
    let generated_text = fs::read_to_string(&output_path).expect("read generated xsd contract");

    let expected: Value = serde_json::from_str(&expected_text).expect("parse committed contract");
    let generated: Value = serde_json::from_str(&generated_text).expect("parse generated contract");

    assert_eq!(
        generated, expected,
        "xsd contract snapshot is stale. regenerate with:\npython3 scripts/extract_xsd_contract.py --template-id {} --dee-version {} --exported-at {} --raw-dir tests/fixtures/xsd/raw --output tests/fixtures/xsd/contract.atmos_ec3_v1.json",
        current.meta.template_id, current.meta.dee_version, current.meta.exported_at,
    );
}
