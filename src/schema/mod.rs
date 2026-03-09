pub mod validate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTag {
    DolbyOfficial,
    DeewObserved,
    DeezyObserved,
}

#[derive(Debug, Clone, Copy)]
pub enum ParamRule {
    Enum(&'static [&'static str]),
    IntRange { min: i64, max: i64 },
    Bool,
    FreeString,
    DataRate,
}

#[derive(Debug, Clone, Copy)]
pub enum ModeAvailability {
    All,
    Only(&'static [&'static str]),
    Except(&'static [&'static str]),
}

#[derive(Debug, Clone, Copy)]
pub struct ParamSchema {
    pub key: &'static str,
    pub rule: ParamRule,
    pub mode_availability: ModeAvailability,
    pub sources: &'static [SourceTag],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedValue {
    Bool(bool),
    Int(i64),
    Str(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverridePolicy {
    None,
    AllowFixedOverride,
}

#[derive(Debug, Clone, Copy)]
pub enum Constraint {
    FixedValues {
        when_profile: &'static str,
        fields: &'static [(&'static str, FixedValue)],
        override_policy: OverridePolicy,
    },
    Forbidden {
        param: &'static str,
        when_mode: &'static [&'static str],
        message: &'static str,
    },
    Required {
        param: &'static str,
        when_mode: &'static str,
        value: FixedValue,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl From<FixedValue> for Value {
    fn from(value: FixedValue) -> Self {
        match value {
            FixedValue::Bool(v) => Self::Bool(v),
            FixedValue::Int(v) => Self::Int(v),
            FixedValue::Str(v) => Self::Str(v.to_string()),
        }
    }
}
