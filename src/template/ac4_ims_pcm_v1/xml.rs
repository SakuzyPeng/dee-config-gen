use crate::{
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
    template::ac4_ims_shared,
};

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
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

    ac4_ims_shared::xml_structure(
        job,
        filter,
        ac4_ims_shared::pcm_input_node(storage_tag, input_groups),
    )
}
