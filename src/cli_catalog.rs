use std::path::PathBuf;

use anyhow::Result;
use dee_config_gen::spec::{ModeAvailability, ParamRule, SourceTag, template_metadata};
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct TemplateCatalogEntry {
    pub template_id: &'static str,
    pub description: &'static str,
    pub examples: &'static [&'static str],
    pub canonical_example: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct ExampleAsset {
    pub name: &'static str,
    pub content: &'static str,
}

const TEMPLATE_CATALOG: &[TemplateCatalogEntry] = &[
    TemplateCatalogEntry {
        template_id: "ac4_ims_atmos_v1",
        description: "AC-4 IMS from atmos_mezz-family immersive inputs to .ac4 or .mp4",
        examples: &[
            "ac4_ims_atmos_single.ac4.yaml",
            "ac4_ims_atmos_single.mp4.yaml",
            "ac4_ims_atmos_multi3.ac4.yaml",
        ],
        canonical_example: "ac4_ims_atmos_single.ac4.yaml",
    },
    TemplateCatalogEntry {
        template_id: "ac4_ims_pcm_v1",
        description: "AC-4 IMS from PCM WAV or WAV-list inputs to .ac4 or .mp4",
        examples: &["ac4_ims_pcm_single.ac4.yaml", "ac4_ims_pcm_single.mp4.yaml"],
        canonical_example: "ac4_ims_pcm_single.ac4.yaml",
    },
    TemplateCatalogEntry {
        template_id: "atmos_ec3_v1",
        description: "Atmos Dolby Digital Plus configs for streaming or Blu-ray workflows",
        examples: &[
            "atmos_ec3_single.streaming.yaml",
            "atmos_ec3_single.bluray.yaml",
            "atmos_ec3_album.music.json",
        ],
        canonical_example: "atmos_ec3_single.streaming.yaml",
    },
    TemplateCatalogEntry {
        template_id: "pcm_ddp_v1",
        description: "PCM to Dolby Digital or Dolby Digital Plus configs",
        examples: &[
            "pcm_ddp_single.dd.yaml",
            "pcm_ddp_single.ddp.yaml",
            "pcm_ddp_single.ddp71.yaml",
            "pcm_ddp_single.bluray.yaml",
        ],
        canonical_example: "pcm_ddp_single.ddp.yaml",
    },
    TemplateCatalogEntry {
        template_id: "thd_v1",
        description: "TrueHD MLP from atmos_mezz input",
        examples: &["thd_single.mlp.yaml"],
        canonical_example: "thd_single.mlp.yaml",
    },
    TemplateCatalogEntry {
        template_id: "thd_wav_v1",
        description: "TrueHD MLP from a single WAV input",
        examples: &["thd_wav_single.mlp.yaml"],
        canonical_example: "thd_wav_single.mlp.yaml",
    },
    TemplateCatalogEntry {
        template_id: "thd_wav_list_v1",
        description: "TrueHD MLP from ordered mono WAV stems",
        examples: &["thd_wav_list_single.mlp.yaml"],
        canonical_example: "thd_wav_list_single.mlp.yaml",
    },
    TemplateCatalogEntry {
        template_id: "thd_atmos_wav_v1",
        description: "TrueHD MLP from mixed atmos_mezz and WAV inputs",
        examples: &["thd_atmos_wav_single.mlp.yaml"],
        canonical_example: "thd_atmos_wav_single.mlp.yaml",
    },
    TemplateCatalogEntry {
        template_id: "thd_atmos_wav_list_v1",
        description: "TrueHD MLP from mixed atmos_mezz and ordered WAV stems",
        examples: &["thd_atmos_wav_list_single.mlp.yaml"],
        canonical_example: "thd_atmos_wav_list_single.mlp.yaml",
    },
];

const EXAMPLES: &[ExampleAsset] = &[
    ExampleAsset {
        name: "ac4_ims_atmos_multi3.ac4.yaml",
        content: include_str!("../examples/ac4_ims_atmos_multi3.ac4.yaml"),
    },
    ExampleAsset {
        name: "ac4_ims_atmos_single.ac4.yaml",
        content: include_str!("../examples/ac4_ims_atmos_single.ac4.yaml"),
    },
    ExampleAsset {
        name: "ac4_ims_atmos_single.mp4.yaml",
        content: include_str!("../examples/ac4_ims_atmos_single.mp4.yaml"),
    },
    ExampleAsset {
        name: "ac4_ims_pcm_single.ac4.yaml",
        content: include_str!("../examples/ac4_ims_pcm_single.ac4.yaml"),
    },
    ExampleAsset {
        name: "ac4_ims_pcm_single.mp4.yaml",
        content: include_str!("../examples/ac4_ims_pcm_single.mp4.yaml"),
    },
    ExampleAsset {
        name: "atmos_ec3_album.music.json",
        content: include_str!("../examples/atmos_ec3_album.music.json"),
    },
    ExampleAsset {
        name: "atmos_ec3_single.bluray.yaml",
        content: include_str!("../examples/atmos_ec3_single.bluray.yaml"),
    },
    ExampleAsset {
        name: "atmos_ec3_single.streaming.yaml",
        content: include_str!("../examples/atmos_ec3_single.streaming.yaml"),
    },
    ExampleAsset {
        name: "pcm_ddp_single.bluray.yaml",
        content: include_str!("../examples/pcm_ddp_single.bluray.yaml"),
    },
    ExampleAsset {
        name: "pcm_ddp_single.dd.yaml",
        content: include_str!("../examples/pcm_ddp_single.dd.yaml"),
    },
    ExampleAsset {
        name: "pcm_ddp_single.ddp.yaml",
        content: include_str!("../examples/pcm_ddp_single.ddp.yaml"),
    },
    ExampleAsset {
        name: "pcm_ddp_single.ddp71.yaml",
        content: include_str!("../examples/pcm_ddp_single.ddp71.yaml"),
    },
    ExampleAsset {
        name: "thd_atmos_wav_list_single.mlp.yaml",
        content: include_str!("../examples/thd_atmos_wav_list_single.mlp.yaml"),
    },
    ExampleAsset {
        name: "thd_atmos_wav_single.mlp.yaml",
        content: include_str!("../examples/thd_atmos_wav_single.mlp.yaml"),
    },
    ExampleAsset {
        name: "thd_single.mlp.yaml",
        content: include_str!("../examples/thd_single.mlp.yaml"),
    },
    ExampleAsset {
        name: "thd_wav_list_single.mlp.yaml",
        content: include_str!("../examples/thd_wav_list_single.mlp.yaml"),
    },
    ExampleAsset {
        name: "thd_wav_single.mlp.yaml",
        content: include_str!("../examples/thd_wav_single.mlp.yaml"),
    },
];

pub fn list_templates_text() -> Result<String> {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<28}  {:<16}  {:<24}  {:<36}  Description\n",
        "Template", "Profiles", "Encode modes", "Canonical example"
    ));
    out.push_str(&format!(
        "{:-<28}  {:-<16}  {:-<24}  {:-<36}  {:-<60}\n",
        "", "", "", "", ""
    ));
    for item in list_templates_json()? {
        out.push_str(&format!(
            "{:<28}  {:<16}  {:<24}  {:<36}  {}\n",
            item.template_id,
            item.profiles.join(","),
            item.encode_modes.join(","),
            item.canonical_example,
            item.description
        ));
    }
    Ok(out)
}

pub fn list_templates_json_string() -> Result<String> {
    serde_json::to_string_pretty(&list_templates_json()?).map_err(Into::into)
}

pub fn show_template_text(template_id: &str) -> Result<String> {
    let details = show_template_json(template_id)?;
    let mut out = String::new();
    out.push_str(&format!("Template: {}\n", details.template_id));
    out.push_str(&format!("Description: {}\n", details.description));
    out.push_str(&format!("Profiles: {}\n", details.profiles.join(", ")));
    out.push_str(&format!(
        "Encode modes: {}\n",
        details.encode_modes.join(", ")
    ));
    out.push_str(&format!(
        "Canonical example: {}\n",
        details.canonical_example
    ));
    out.push_str("Examples:\n");
    for example in details.examples {
        out.push_str(&format!("  - {example}\n"));
    }
    out.push_str("Parameters:\n");
    if details.parameters.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for parameter in &details.parameters {
            out.push_str(&format!(
                "  - {}: {}; modes={}; sources={}\n",
                parameter.key,
                parameter.rule.text_label(),
                parameter.mode_availability.text_label(),
                parameter.sources.join(",")
            ));
        }
    }
    Ok(out)
}

pub fn show_template_json_string(template_id: &str) -> Result<String> {
    serde_json::to_string_pretty(&show_template_json(template_id)?).map_err(Into::into)
}

pub fn select_init_example(
    template: Option<&str>,
    example: Option<&str>,
) -> Result<&'static ExampleAsset> {
    if let Some(example) = example {
        return find_example(example);
    }

    let template_id = template.unwrap_or("atmos_ec3_v1");
    let entry = find_template(template_id)?;
    find_example(entry.canonical_example)
}

pub fn default_init_output_path(example_name: &str) -> PathBuf {
    if example_name.ends_with(".json") {
        PathBuf::from("job.json")
    } else {
        PathBuf::from("job.yaml")
    }
}

fn list_templates_json() -> Result<Vec<TemplateListItemJson>> {
    TEMPLATE_CATALOG.iter().map(template_list_item).collect()
}

fn show_template_json(template_id: &str) -> Result<TemplateDetailsJson> {
    let entry = find_template(template_id)?;
    let metadata = template_metadata(template_id)?;
    let parameters = metadata
        .param_schemas
        .iter()
        .map(|param| TemplateParameterJson {
            key: param.key,
            rule: RuleJson::from(param.rule),
            mode_availability: ModeAvailabilityJson::from(param.mode_availability),
            sources: param
                .sources
                .iter()
                .map(|source| source_label(*source))
                .collect(),
        })
        .collect();

    Ok(TemplateDetailsJson {
        template_id: entry.template_id,
        description: entry.description,
        profiles: metadata.valid_profiles,
        encode_modes: metadata.valid_encode_modes,
        examples: entry.examples,
        canonical_example: entry.canonical_example,
        parameters,
    })
}

fn template_list_item(entry: &TemplateCatalogEntry) -> Result<TemplateListItemJson> {
    let metadata = template_metadata(entry.template_id)?;
    Ok(TemplateListItemJson {
        template_id: entry.template_id,
        description: entry.description,
        profiles: metadata.valid_profiles,
        encode_modes: metadata.valid_encode_modes,
        examples: entry.examples,
        canonical_example: entry.canonical_example,
    })
}

fn find_template(template_id: &str) -> Result<&'static TemplateCatalogEntry> {
    TEMPLATE_CATALOG
        .iter()
        .find(|entry| entry.template_id == template_id)
        .ok_or_else(|| {
            let supported = TEMPLATE_CATALOG
                .iter()
                .map(|entry| entry.template_id)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!("unknown template '{template_id}'; supported templates: {supported}")
        })
}

fn find_example(example_name: &str) -> Result<&'static ExampleAsset> {
    let normalized = example_name
        .strip_prefix("examples/")
        .unwrap_or(example_name);
    EXAMPLES
        .iter()
        .find(|example| example.name == normalized)
        .ok_or_else(|| {
            let supported = EXAMPLES
                .iter()
                .map(|example| example.name)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!("unknown example '{example_name}'; supported examples: {supported}")
        })
}

fn source_label(source: SourceTag) -> &'static str {
    match source {
        SourceTag::DolbyOfficial => "dolby_official",
        SourceTag::DeewObserved => "deew_observed",
        SourceTag::DeezyObserved => "deezy_observed",
    }
}

#[derive(Debug, Serialize)]
struct TemplateListItemJson {
    template_id: &'static str,
    description: &'static str,
    profiles: &'static [&'static str],
    encode_modes: &'static [&'static str],
    examples: &'static [&'static str],
    canonical_example: &'static str,
}

#[derive(Debug, Serialize)]
struct TemplateDetailsJson {
    template_id: &'static str,
    description: &'static str,
    profiles: &'static [&'static str],
    encode_modes: &'static [&'static str],
    examples: &'static [&'static str],
    canonical_example: &'static str,
    parameters: Vec<TemplateParameterJson>,
}

#[derive(Debug, Serialize)]
struct TemplateParameterJson {
    key: &'static str,
    rule: RuleJson,
    mode_availability: ModeAvailabilityJson,
    sources: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RuleJson {
    Enum { values: &'static [&'static str] },
    IntRange { min: i64, max: i64 },
    Bool,
    FreeString,
    DataRate,
}

impl RuleJson {
    fn text_label(&self) -> String {
        match self {
            Self::Enum { values } => format!("enum({})", values.join("|")),
            Self::IntRange { min, max } => format!("int({min}..{max})"),
            Self::Bool => "bool".to_string(),
            Self::FreeString => "string".to_string(),
            Self::DataRate => "data_rate".to_string(),
        }
    }
}

impl From<ParamRule> for RuleJson {
    fn from(rule: ParamRule) -> Self {
        match rule {
            ParamRule::Enum(values) => Self::Enum { values },
            ParamRule::IntRange { min, max } => Self::IntRange { min, max },
            ParamRule::Bool => Self::Bool,
            ParamRule::FreeString => Self::FreeString,
            ParamRule::DataRate => Self::DataRate,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ModeAvailabilityJson {
    All,
    Only { modes: &'static [&'static str] },
    Except { modes: &'static [&'static str] },
}

impl ModeAvailabilityJson {
    fn text_label(&self) -> String {
        match self {
            Self::All => "all".to_string(),
            Self::Only { modes } => format!("only({})", modes.join("|")),
            Self::Except { modes } => format!("except({})", modes.join("|")),
        }
    }
}

impl From<ModeAvailability> for ModeAvailabilityJson {
    fn from(availability: ModeAvailability) -> Self {
        match availability {
            ModeAvailability::All => Self::All,
            ModeAvailability::Only(modes) => Self::Only { modes },
            ModeAvailability::Except(modes) => Self::Except { modes },
        }
    }
}
