mod common;

use std::{fs, process::Command};

use tempfile::TempDir;

use common::{THD_ATMOS_WAV_XSD_CONTRACT_PATH, load_xsd_contract_at};

#[test]
#[ignore = "runs python script to regenerate thd_atmos_wav contract snapshot"]
fn thd_atmos_wav_xsd_contract_snapshot_is_up_to_date() {
    let temp = TempDir::new().expect("temp dir");
    let output_path = temp.path().join("contract.thd_atmos_wav_v1.json");

    let status = Command::new("python3")
        .args(["scripts/extract_xsd_contract.py"])
        .args(["--template-id", "thd_atmos_wav_v1"])
        .args(["--dee-version", "5.2.1"])
        .args(["--exported-at", "2026-03-13T00:00:00+08:00"])
        .args(["--raw-dir", "tests/fixtures/xsd/raw"])
        .args(["--output", output_path.to_str().expect("utf-8 output path")])
        .args(["--filter-path-prefix", "/job_config/input/audio/atmos_mezz"])
        .args(["--filter-path-prefix", "/job_config/input/audio/wav"])
        .args([
            "--filter-path-prefix",
            "/job_config/filter/audio/encode_to_dthd",
        ])
        .args(["--filter-path-prefix", "/job_config/output/mlp"])
        .status()
        .expect("run extract_xsd_contract.py");

    assert!(status.success(), "xsd extraction should succeed");

    let expected = fs::read_to_string(THD_ATMOS_WAV_XSD_CONTRACT_PATH)
        .expect("read checked-in thd_atmos_wav contract");
    let actual = fs::read_to_string(&output_path).expect("read regenerated thd_atmos_wav contract");
    assert_eq!(actual, expected);

    let regenerated = load_xsd_contract_at(output_path.to_str().expect("utf-8 output path"));
    assert_eq!(regenerated.meta.template_id, "thd_atmos_wav_v1");
}
