use crate::schema::{Constraint, FixedValue, OverridePolicy};

pub const CONSTRAINTS: &[Constraint] = &[
    Constraint::FixedValues {
        when_profile: "music",
        fields: &[
            ("dialogue_intelligence", FixedValue::Bool(false)),
            ("speech_threshold", FixedValue::Int(100)),
            ("line_mode_drc_profile", FixedValue::Str("music_light")),
            ("rf_mode_drc_profile", FixedValue::Str("music_light")),
        ],
        override_policy: OverridePolicy::AllowFixedOverride,
    },
    Constraint::Required {
        param: "encoder_mode",
        when_mode: "bluray",
        value: FixedValue::Str("bluray"),
    },
    Constraint::Required {
        param: "encoder_mode",
        when_mode: "ddp71",
        value: FixedValue::Str("ddp71"),
    },
];
