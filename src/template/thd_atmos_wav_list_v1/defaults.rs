use crate::config::{EncodeMode, Profile};

use super::filter::ThdAtmosWavListV1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> ThdAtmosWavListV1Filter {
    crate::template::thd_wav_list_v1::defaults::defaults(profile, encode_mode)
}
