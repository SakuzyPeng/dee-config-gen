use crate::{
    spec::{EncodeMode, Profile},
    template::ac4_ims_shared,
};

use super::filter::Ac4ImsAtmosV1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> Ac4ImsAtmosV1Filter {
    ac4_ims_shared::defaults(profile, encode_mode)
}
