use serde_json::Value;

use crate::{
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
    template::ac4_ims_shared,
};

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::Ac4ImsAtmosV1(filter) = &job.filter else {
        panic!("ac4_ims_atmos_v1 received wrong ResolvedFilter variant");
    };

    let input_groups = job
        .input_groups
        .as_ref()
        .expect("ac4_ims_atmos_v1 requires resolved input_groups");
    let atmos_mezz = input_groups
        .atmos_mezz
        .as_ref()
        .expect("ac4_ims_atmos_v1 requires atmos_mezz input group");

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    ac4_ims_shared::json_structure(
        job,
        filter,
        ac4_ims_shared::atmos_input_json_node_with_timecodes(
            storage_tag,
            atmos_mezz,
            &filter.input_timecode_frame_rate,
            &filter.offset,
            &filter.ffoa,
        ),
    )
}
