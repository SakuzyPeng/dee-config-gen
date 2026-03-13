use crate::config::{EncodeMode, Profile};

use super::filter::ThdAtmosWavV1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> ThdAtmosWavV1Filter {
    crate::template::thd_wav_v1::defaults::defaults(profile, encode_mode)
}
