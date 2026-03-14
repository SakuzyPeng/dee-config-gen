use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const VALID_PROFILES: &[&str] = &["standard"];
pub const VALID_ENCODE_MODES: &[&str] = &["ac4"];
pub const PARAM_SCHEMAS: &[crate::spec::ParamSchema] = &[];
pub const BITRATE_HARD_MAX: u16 = 0;

pub fn bitrate_sets() -> &'static BTreeMap<&'static str, &'static [u16]> {
    static BITRATE_SETS: OnceLock<BTreeMap<&'static str, &'static [u16]>> = OnceLock::new();
    BITRATE_SETS.get_or_init(BTreeMap::new)
}
