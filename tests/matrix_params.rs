mod common;

use dee_config_gen::{
    config::{JobFile, Profile},
    schema::{Constraint, FixedValue, ModeAvailability, ParamRule},
    template::atmos_ec3_v1::{constraints::CONSTRAINTS, params},
};

use common::{
    base_job_file, find_filter_param_path, load_xsd_contract, resolve_with_defaults,
    set_encode_mode,
};

#[derive(Debug, Clone)]
enum CandidateValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

#[test]
#[ignore = "matrix test: schema + xsd driven"]
fn matrix_params_from_xsd_and_schema() {
    let contract = load_xsd_contract();

    for schema in params::PARAM_SCHEMAS {
        let xsd_path = find_filter_param_path(&contract, schema.key).unwrap_or_else(|| {
            panic!(
                "case_id=path_presence:{} xsd_path=<missing> param_key={} missing in xsd contract",
                schema.key, schema.key
            )
        });

        for mode in params::VALID_ENCODE_MODES {
            let case_id = format!("valid:{}:{}", schema.key, mode);
            let forbidden = forbidden_message(schema.key, mode);
            let allowed_by_mode = mode_allowed(schema.mode_availability, mode);
            let legal = legal_value_for_param(schema.key, schema.rule, mode);

            let mut job = base_job_file();
            set_encode_mode(&mut job, mode);
            set_override(&mut job, schema.key, legal.clone());

            let result = resolve_with_defaults(job);

            if let Some(message) = forbidden {
                let err = result.expect_err("forbidden case should fail").to_string();
                assert!(
                    err.contains(message),
                    "case_id={} xsd_path={} param_key={} expected forbidden message '{}' got '{}'",
                    case_id,
                    xsd_path,
                    schema.key,
                    message,
                    err,
                );
                continue;
            }

            if !allowed_by_mode {
                let err = result
                    .expect_err("mode availability mismatch should fail")
                    .to_string();
                assert!(
                    err.contains("not available for encode_mode"),
                    "case_id={} xsd_path={} param_key={} expected mode_availability error got '{}'",
                    case_id,
                    xsd_path,
                    schema.key,
                    err,
                );
                continue;
            }

            if let Err(err) = result {
                panic!(
                    "case_id={} xsd_path={} param_key={} expected success got '{}'",
                    case_id, xsd_path, schema.key, err
                );
            }

            if let Some(invalid) = invalid_value_for_rule(schema.rule, mode) {
                let case_id = format!("invalid:{}:{}", schema.key, mode);
                let mut job = base_job_file();
                set_encode_mode(&mut job, mode);
                set_override(&mut job, schema.key, invalid);
                let err = resolve_with_defaults(job)
                    .expect_err("invalid value should fail")
                    .to_string();

                assert!(
                    err.contains(schema.key),
                    "case_id={} xsd_path={} param_key={} expected key in error, got '{}'",
                    case_id,
                    xsd_path,
                    schema.key,
                    err,
                );
            }
        }
    }

    for constraint in CONSTRAINTS {
        if let Constraint::Required {
            param,
            when_mode,
            value,
        } = constraint
        {
            let xsd_path = find_filter_param_path(&contract, param).unwrap_or("<missing>");
            let Some(alternative) = required_alternative_value(param, *value, when_mode) else {
                continue;
            };

            let mut job = base_job_file();
            set_encode_mode(&mut job, when_mode);
            set_override(&mut job, param, alternative);

            let err = resolve_with_defaults(job)
                .expect_err("required constraint mismatch should fail")
                .to_string();
            let expected = format!("{} mode requires {}=", when_mode, param);
            assert!(
                err.contains(&expected),
                "case_id=required_mismatch:{}:{} xsd_path={} param_key={} expected '{}' in error, got '{}'",
                param,
                when_mode,
                xsd_path,
                param,
                expected,
                err,
            );
        }
    }

    for constraint in CONSTRAINTS {
        if let Constraint::FixedValues {
            when_profile,
            fields,
            ..
        } = constraint
        {
            if *when_profile != "music" || fields.is_empty() {
                continue;
            }
            let (field, expected) = fields[0];
            let xsd_path = find_filter_param_path(&contract, field).unwrap_or("<missing>");

            let mut job = base_job_file();
            job.profile = Profile::Music;
            set_override(&mut job, field, conflicting_value_for_fixed(expected));

            let err = resolve_with_defaults(job)
                .expect_err("fixed value conflict should fail")
                .to_string();
            assert!(
                err.contains("profile=music locks fixed fields"),
                "case_id=fixed_value_conflict:{} xsd_path={} param_key={} expected profile lock error got '{}'",
                field,
                xsd_path,
                field,
                err,
            );
        }
    }
}

fn mode_allowed(mode_availability: ModeAvailability, mode: &str) -> bool {
    match mode_availability {
        ModeAvailability::All => true,
        ModeAvailability::Only(modes) => modes.contains(&mode),
        ModeAvailability::Except(modes) => !modes.contains(&mode),
    }
}

fn forbidden_message(param: &str, mode: &str) -> Option<&'static str> {
    for constraint in CONSTRAINTS {
        if let Constraint::Forbidden {
            param: forbidden_param,
            when_mode,
            message,
        } = constraint
            && *forbidden_param == param
            && when_mode.contains(&mode)
        {
            return Some(*message);
        }
    }
    None
}

fn required_value(param: &str, mode: &str) -> Option<CandidateValue> {
    for constraint in CONSTRAINTS {
        if let Constraint::Required {
            param: required_param,
            when_mode,
            value,
        } = constraint
            && *required_param == param
            && *when_mode == mode
        {
            return Some(fixed_to_candidate(*value));
        }
    }
    None
}

fn required_alternative_value(
    param: &str,
    expected: FixedValue,
    mode: &str,
) -> Option<CandidateValue> {
    let schema = params::find_schema(param)?;

    match (schema.rule, expected) {
        (ParamRule::Enum(allowed), FixedValue::Str(expected_str)) => {
            let alt = allowed
                .iter()
                .find(|candidate| **candidate != expected_str)
                .copied()?;
            Some(CandidateValue::Str(alt.to_string()))
        }
        (ParamRule::Bool, FixedValue::Bool(v)) => Some(CandidateValue::Bool(!v)),
        (ParamRule::IntRange { min, max }, FixedValue::Int(v)) => {
            let alt = if v == min { min + 1 } else { v - 1 };
            if alt < min || alt > max {
                None
            } else {
                Some(CandidateValue::Int(alt))
            }
        }
        (ParamRule::DataRate, FixedValue::Int(v)) => {
            let allowed = params::bitrate_sets().get(mode).copied()?;
            let alt = allowed
                .iter()
                .find(|candidate| i64::from(**candidate) != v)
                .copied()?;
            Some(CandidateValue::Int(i64::from(alt)))
        }
        _ => None,
    }
}

fn fixed_to_candidate(value: FixedValue) -> CandidateValue {
    match value {
        FixedValue::Bool(v) => CandidateValue::Bool(v),
        FixedValue::Int(v) => CandidateValue::Int(v),
        FixedValue::Str(v) => CandidateValue::Str(v.to_string()),
    }
}

fn conflicting_value_for_fixed(value: FixedValue) -> CandidateValue {
    match value {
        FixedValue::Bool(v) => CandidateValue::Bool(!v),
        FixedValue::Int(v) => CandidateValue::Int(v.saturating_sub(1)),
        FixedValue::Str(v) => {
            let alt = if v == "film_light" {
                "music_light"
            } else {
                "film_light"
            };
            CandidateValue::Str(alt.to_string())
        }
    }
}

fn legal_value_for_param(key: &str, rule: ParamRule, mode: &str) -> CandidateValue {
    if let Some(required) = required_value(key, mode) {
        return required;
    }

    match rule {
        ParamRule::Enum(allowed) => CandidateValue::Str(allowed[0].to_string()),
        ParamRule::IntRange { min, .. } => CandidateValue::Int(min),
        ParamRule::Bool => CandidateValue::Bool(true),
        ParamRule::FreeString => CandidateValue::Str("sample_value".to_string()),
        ParamRule::DataRate => {
            let bitrate = params::bitrate_sets()
                .get(mode)
                .and_then(|values| values.first())
                .copied()
                .unwrap_or(384);
            CandidateValue::Int(i64::from(bitrate))
        }
    }
}

fn invalid_value_for_rule(rule: ParamRule, mode: &str) -> Option<CandidateValue> {
    match rule {
        ParamRule::Enum(_) => Some(CandidateValue::Str("__invalid__".to_string())),
        ParamRule::IntRange { max, .. } => Some(CandidateValue::Int(max + 1)),
        ParamRule::Bool => None,
        ParamRule::FreeString => None,
        ParamRule::DataRate => {
            let allowed = params::bitrate_sets().get(mode).copied()?;
            let mut invalid = 1_u16;
            while allowed.contains(&invalid) && invalid < params::BITRATE_HARD_MAX {
                invalid += 1;
            }
            Some(CandidateValue::Int(i64::from(invalid)))
        }
    }
}

fn set_override(job: &mut JobFile, key: &str, value: CandidateValue) {
    match (key, value) {
        (
            "metering_mode"
            | "timecode_frame_rate"
            | "start"
            | "end"
            | "time_base"
            | "prepend_silence_duration"
            | "append_silence_duration"
            | "line_mode_drc_profile"
            | "rf_mode_drc_profile"
            | "loro_center_mix_level"
            | "loro_surround_mix_level"
            | "ltrt_center_mix_level"
            | "ltrt_surround_mix_level"
            | "preferred_downmix_mode"
            | "surround_trim_5_1"
            | "surround_trim_7_1"
            | "height_trim_5_1"
            | "encoding_backend"
            | "encoder_mode",
            CandidateValue::Str(v),
        ) => match key {
            "metering_mode" => job.filter.metering_mode = Some(v),
            "timecode_frame_rate" => job.filter.timecode_frame_rate = Some(v),
            "start" => job.filter.start = Some(v),
            "end" => job.filter.end = Some(v),
            "time_base" => job.filter.time_base = Some(v),
            "prepend_silence_duration" => job.filter.prepend_silence_duration = Some(v),
            "append_silence_duration" => job.filter.append_silence_duration = Some(v),
            "line_mode_drc_profile" => job.filter.line_mode_drc_profile = Some(v),
            "rf_mode_drc_profile" => job.filter.rf_mode_drc_profile = Some(v),
            "loro_center_mix_level" => job.filter.loro_center_mix_level = Some(v),
            "loro_surround_mix_level" => job.filter.loro_surround_mix_level = Some(v),
            "ltrt_center_mix_level" => job.filter.ltrt_center_mix_level = Some(v),
            "ltrt_surround_mix_level" => job.filter.ltrt_surround_mix_level = Some(v),
            "preferred_downmix_mode" => job.filter.preferred_downmix_mode = Some(v),
            "surround_trim_5_1" => job.filter.surround_trim_5_1 = Some(v),
            "surround_trim_7_1" => job.filter.surround_trim_7_1 = Some(v),
            "height_trim_5_1" => job.filter.height_trim_5_1 = Some(v),
            "encoding_backend" => job.filter.encoding_backend = Some(v),
            "encoder_mode" => job.filter.encoder_mode = Some(v),
            _ => unreachable!(),
        },
        ("dialogue_intelligence", CandidateValue::Bool(v)) => {
            job.filter.dialogue_intelligence = Some(v);
        }
        ("speech_threshold", CandidateValue::Int(v)) => {
            job.filter.speech_threshold = Some(
                u8::try_from(v)
                    .unwrap_or_else(|_| panic!("speech_threshold out of u8 range in test: {v}")),
            );
        }
        ("data_rate", CandidateValue::Int(v)) => {
            job.filter.data_rate = Some(
                u16::try_from(v)
                    .unwrap_or_else(|_| panic!("data_rate out of u16 range in test: {v}")),
            );
        }
        ("custom_dialnorm", CandidateValue::Int(v)) => {
            job.filter.custom_dialnorm = Some(
                i8::try_from(v)
                    .unwrap_or_else(|_| panic!("custom_dialnorm out of i8 range in test: {v}")),
            );
        }
        (other, value) => {
            panic!("unsupported override in matrix test: key={other}, value={value:?}")
        }
    }
}
