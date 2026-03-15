use serde_json::Value;

use crate::{
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
    template::ac4_ims_shared,
};

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::Ac4ImsPcmV1(filter) = &job.filter else {
        panic!("ac4_ims_pcm_v1 received wrong ResolvedFilter variant");
    };

    let input_groups = job
        .input_groups
        .as_ref()
        .expect("ac4_ims_pcm_v1 requires resolved input_groups");

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    ac4_ims_shared::json_structure(
        job,
        filter,
        ac4_ims_shared::pcm_input_json_node_with_timecodes(
            storage_tag,
            input_groups,
            &filter.input_timecode_frame_rate,
            &filter.offset,
            &filter.ffoa,
        ),
    )
}
