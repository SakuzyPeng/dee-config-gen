use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::{Constraint, FixedValue, ModeAvailability, OverridePolicy, ParamRule, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

pub struct ValidationContext<'a> {
    pub encode_mode: &'a str,
    pub bitrate_sets: &'a BTreeMap<&'static str, &'static [u16]>,
    pub bitrate_hard_max: u16,
}

pub struct ConstraintContext<'a> {
    pub profile: &'a str,
    pub encode_mode: &'a str,
    pub allow_fixed_override: bool,
}

pub fn validate_mode_availability(
    key: &str,
    mode: ModeAvailability,
    encode_mode: &str,
) -> Result<()> {
    let is_allowed = match mode {
        ModeAvailability::All => true,
        ModeAvailability::Only(values) => values.contains(&encode_mode),
        ModeAvailability::Except(values) => !values.contains(&encode_mode),
    };

    if is_allowed {
        Ok(())
    } else {
        bail!("parameter '{key}' is not available for encode_mode '{encode_mode}'")
    }
}

pub fn validate_value(
    key: &str,
    rule: ParamRule,
    value: ParamValue,
    ctx: &ValidationContext<'_>,
) -> Result<ParamValue> {
    match (rule, value) {
        (ParamRule::Enum(allowed), ParamValue::Str(raw)) => {
            let lower = raw.to_ascii_lowercase();
            let found = allowed
                .iter()
                .find(|candidate| candidate.to_ascii_lowercase() == lower)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "invalid value '{raw}' for {key}; allowed: {}",
                        allowed.join(", ")
                    )
                })?;
            Ok(ParamValue::Str((*found).to_string()))
        }
        (ParamRule::IntRange { min, max }, ParamValue::Int(raw)) => {
            if (min..=max).contains(&raw) {
                Ok(ParamValue::Int(raw))
            } else {
                bail!("invalid value '{raw}' for {key}; expected {min}..{max}")
            }
        }
        (ParamRule::DataRate, ParamValue::Int(raw)) => {
            let value = u16::try_from(raw)
                .map_err(|_| anyhow::anyhow!("invalid value '{raw}' for {key}; expected u16"))?;
            validate_data_rate(value, ctx)?;
            Ok(ParamValue::Int(i64::from(value)))
        }
        (ParamRule::Bool, ParamValue::Bool(raw)) => Ok(ParamValue::Bool(raw)),
        (ParamRule::FreeString, ParamValue::Str(raw)) => Ok(ParamValue::Str(raw)),
        (ParamRule::Enum(_), _) => bail!("parameter {key} is not enum"),
        (ParamRule::IntRange { .. }, _) => bail!("parameter {key} is not integer range"),
        (ParamRule::DataRate, _) => bail!("parameter {key} is not data_rate"),
        (ParamRule::Bool, _) => bail!("parameter {key} is not bool"),
        (ParamRule::FreeString, _) => bail!("parameter {key} is not string"),
    }
}

pub fn evaluate_constraints<F>(
    constraints: &[Constraint],
    ctx: &ConstraintContext<'_>,
    mut read_value: F,
) -> Result<()>
where
    F: FnMut(&str) -> Option<Value>,
{
    for constraint in constraints {
        match constraint {
            Constraint::FixedValues {
                when_profile,
                fields,
                override_policy,
            } => {
                if ctx.profile != *when_profile {
                    continue;
                }
                if *override_policy == OverridePolicy::AllowFixedOverride
                    && ctx.allow_fixed_override
                {
                    continue;
                }

                let mut conflicts = Vec::new();
                for (field, expected) in *fields {
                    let actual = read_value(field);
                    let expect_value = Value::from(*expected);
                    if actual != Some(expect_value) {
                        conflicts.push(*field);
                    }
                }

                if !conflicts.is_empty() {
                    bail!(
                        "profile={} locks fixed fields by default. conflicting fields: {}. Use --allow-fixed-override to bypass.",
                        when_profile,
                        conflicts.join(", ")
                    );
                }
            }
            Constraint::Forbidden {
                param,
                when_mode,
                message,
            } => {
                if when_mode.contains(&ctx.encode_mode) && read_value(param).is_some() {
                    bail!("{message}");
                }
            }
            Constraint::Required {
                param,
                when_mode,
                value,
            } => {
                if ctx.encode_mode != *when_mode {
                    continue;
                }

                let expected = Value::from(*value);
                if read_value(param) != Some(expected) {
                    bail!(
                        "{} mode requires {}={}",
                        when_mode,
                        param,
                        fixed_value_for_error(*value)
                    );
                }
            }
        }
    }

    Ok(())
}

fn validate_data_rate(value: u16, ctx: &ValidationContext<'_>) -> Result<()> {
    if value > ctx.bitrate_hard_max {
        bail!(
            "invalid data_rate '{value}'; hard max is {}",
            ctx.bitrate_hard_max
        );
    }

    let allowed = ctx
        .bitrate_sets
        .get(ctx.encode_mode)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("unsupported encode_mode '{}'", ctx.encode_mode))?;

    if allowed.contains(&value) {
        Ok(())
    } else {
        bail!(
            "invalid data_rate '{value}' for mode '{}'; allowed: {}",
            ctx.encode_mode,
            allowed
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn fixed_value_for_error(value: FixedValue) -> String {
    match value {
        FixedValue::Bool(true) => "true".to_string(),
        FixedValue::Bool(false) => "false".to_string(),
        FixedValue::Int(v) => v.to_string(),
        FixedValue::Str(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        ConstraintContext, ParamValue, ValidationContext, evaluate_constraints, validate_value,
    };
    use crate::schema::{Constraint, FixedValue, ParamRule, Value};

    fn ctx() -> ValidationContext<'static> {
        let mut bitrate_sets = BTreeMap::new();
        bitrate_sets.insert("streaming", &[384, 448, 576][..]);
        bitrate_sets.insert("bluray", &[768, 1280, 1664][..]);
        bitrate_sets.insert("ddp71", &[448, 1024, 1664][..]);

        ValidationContext {
            encode_mode: "bluray",
            bitrate_sets: Box::leak(Box::new(bitrate_sets)),
            bitrate_hard_max: 1664,
        }
    }

    #[test]
    fn canonicalizes_enum_case_insensitive() {
        let out = validate_value(
            "preferred_downmix_mode",
            ParamRule::Enum(&["loro", "ltrt"]),
            ParamValue::Str("LoRo".to_string()),
            &ctx(),
        )
        .unwrap();
        assert_eq!(out, ParamValue::Str("loro".to_string()));
    }

    #[test]
    fn validates_data_rate_hard_max() {
        let err = validate_value(
            "data_rate",
            ParamRule::DataRate,
            ParamValue::Int(1700),
            &ctx(),
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("hard max"));
    }

    #[test]
    fn evaluates_required_constraint() {
        let constraints = [Constraint::Required {
            param: "encoder_mode",
            when_mode: "bluray",
            value: FixedValue::Str("bluray"),
        }];

        let err = evaluate_constraints(
            &constraints,
            &ConstraintContext {
                profile: "standard",
                encode_mode: "bluray",
                allow_fixed_override: false,
            },
            |_| Some(Value::Str("ddp71".to_string())),
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("bluray mode requires encoder_mode=bluray"));
    }
}
