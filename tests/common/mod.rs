#![allow(dead_code)]

use std::{fs, path::Path};

use dee_config_gen::{
    ResolveOptions,
    config::{EncodeMode, FilterOverrides, JobFile},
    load_job_file, resolve_job,
};
use serde::Deserialize;

pub const XSD_CONTRACT_PATH: &str = "tests/fixtures/xsd/contract.atmos_ec3_v1.json";

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

pub fn load_xsd_contract() -> XsdContract {
    let content = fs::read_to_string(XSD_CONTRACT_PATH)
        .unwrap_or_else(|e| panic!("failed to read {XSD_CONTRACT_PATH}: {e}"));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse {XSD_CONTRACT_PATH} as JSON: {e}"))
}

pub fn find_filter_param_path<'a>(contract: &'a XsdContract, key: &str) -> Option<&'a str> {
    contract
        .paths
        .iter()
        .find(|entry| {
            entry.element == key
                && entry
                    .path
                    .contains("/job_config/filter/audio/encode_to_atmos_ddp/")
        })
        .map(|entry| entry.path.as_str())
        .or_else(|| {
            contract
                .paths
                .iter()
                .find(|entry| entry.element == key)
                .map(|entry| entry.path.as_str())
        })
}

pub fn base_job_file() -> JobFile {
    let mut job =
        load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml")).expect("load example");
    job.filter = FilterOverrides::default();
    job
}

pub fn set_encode_mode(job: &mut JobFile, mode: &str) {
    job.encode_mode = match mode {
        "streaming" => EncodeMode::Streaming,
        "bluray" => EncodeMode::Bluray,
        "ddp71" => EncodeMode::Ddp71,
        other => panic!("unsupported encode mode in test: {other}"),
    };
}

pub fn resolve_with_defaults(job: JobFile) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
}
