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
    Constraint::Forbidden {
        param: "encoding_backend",
        when_mode: &["streaming"],
        message: "encoding_backend/encoder_mode are mode extensions and cannot be set for streaming mode",
    },
    Constraint::Forbidden {
        param: "encoder_mode",
        when_mode: &["streaming"],
        message: "encoding_backend/encoder_mode are mode extensions and cannot be set for streaming mode",
    },
    Constraint::Forbidden {
        param: "encoding_backend",
        when_mode: &["ddp71"],
        message: "encoding_backend is unsupported for ddp71 mode",
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
