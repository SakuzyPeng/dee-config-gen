use crate::spec::{EncodeMode, Profile};

use super::filter::Ac4V1Filter;

pub fn defaults(_profile: Profile, encode_mode: EncodeMode) -> Ac4V1Filter {
    match encode_mode {
        EncodeMode::Ac4 => Ac4V1Filter,
        other => unreachable!("ac4_v1 does not support {}", other.as_str()),
    }
}
