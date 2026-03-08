use crate::config::{JobMode, ResolvedJob};

pub fn render_xml(job: &ResolvedJob) -> String {
    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    let input_files = render_file_name_list(&job.input.file_names);
    let output_files = render_file_name_list(&job.output.file_names);

    let mut buf = String::with_capacity(4096);
    push_line(&mut buf, 0, "<?xml version=\"1.0\"?>");
    push_line(&mut buf, 0, "<job_config>");

    push_line(&mut buf, 1, "<input>");
    push_line(&mut buf, 2, "<audio>");
    push_line(&mut buf, 3, "<atmos_mezz version=\"1\">");
    push_line(
        &mut buf,
        4,
        &format!("<file_name>{}</file_name>", xml_escape(&input_files)),
    );
    push_line(
        &mut buf,
        4,
        &format!(
            "<timecode_frame_rate>{}</timecode_frame_rate>",
            xml_escape(&job.filter.timecode_frame_rate)
        ),
    );
    push_line(&mut buf, 4, &format!("<offset>{}</offset>", "auto"));
    push_line(&mut buf, 4, &format!("<ffoa>{}</ffoa>", "auto"));
    push_line(&mut buf, 4, "<storage>");
    push_line(&mut buf, 5, &format!("<{storage_tag}>"));
    push_line(
        &mut buf,
        6,
        &format!("<path>{}</path>", xml_escape(&job.input.storage_path)),
    );
    push_line(&mut buf, 5, &format!("</{storage_tag}>"));
    push_line(&mut buf, 4, "</storage>");
    push_line(&mut buf, 3, "</atmos_mezz>");
    push_line(&mut buf, 2, "</audio>");
    push_line(&mut buf, 1, "</input>");

    push_line(&mut buf, 1, "<filter>");
    push_line(&mut buf, 2, "<audio>");
    push_line(&mut buf, 3, "<encode_to_atmos_ddp version=\"1\">");
    push_line(&mut buf, 4, "<loudness>");
    push_line(&mut buf, 5, "<measure_only>");
    push_line(
        &mut buf,
        6,
        &format!(
            "<metering_mode>{}</metering_mode>",
            xml_escape(&job.filter.metering_mode)
        ),
    );
    push_line(
        &mut buf,
        6,
        &format!(
            "<dialogue_intelligence>{}</dialogue_intelligence>",
            bool_to_xml(job.filter.dialogue_intelligence)
        ),
    );
    push_line(
        &mut buf,
        6,
        &format!(
            "<speech_threshold>{}</speech_threshold>",
            job.filter.speech_threshold
        ),
    );
    push_line(&mut buf, 5, "</measure_only>");
    push_line(&mut buf, 4, "</loudness>");

    push_line(
        &mut buf,
        4,
        &format!("<data_rate>{}</data_rate>", job.filter.data_rate),
    );
    push_line(
        &mut buf,
        4,
        &format!(
            "<timecode_frame_rate>{}</timecode_frame_rate>",
            xml_escape(&job.filter.timecode_frame_rate)
        ),
    );
    push_line(
        &mut buf,
        4,
        &format!("<start>{}</start>", xml_escape(&job.filter.start)),
    );
    push_line(
        &mut buf,
        4,
        &format!("<end>{}</end>", xml_escape(&job.filter.end)),
    );
    push_line(
        &mut buf,
        4,
        &format!(
            "<time_base>{}</time_base>",
            xml_escape(&job.filter.time_base)
        ),
    );
    push_line(
        &mut buf,
        4,
        &format!(
            "<prepend_silence_duration>{}</prepend_silence_duration>",
            xml_escape(&job.filter.prepend_silence_duration)
        ),
    );
    push_line(
        &mut buf,
        4,
        &format!(
            "<append_silence_duration>{}</append_silence_duration>",
            xml_escape(&job.filter.append_silence_duration)
        ),
    );

    push_line(&mut buf, 4, "<drc>");
    push_line(
        &mut buf,
        5,
        &format!(
            "<line_mode_drc_profile>{}</line_mode_drc_profile>",
            xml_escape(&job.filter.line_mode_drc_profile)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<rf_mode_drc_profile>{}</rf_mode_drc_profile>",
            xml_escape(&job.filter.rf_mode_drc_profile)
        ),
    );
    push_line(&mut buf, 4, "</drc>");

    push_line(&mut buf, 4, "<downmix>");
    push_line(
        &mut buf,
        5,
        &format!(
            "<loro_center_mix_level>{}</loro_center_mix_level>",
            xml_escape(&job.filter.loro_center_mix_level)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<loro_surround_mix_level>{}</loro_surround_mix_level>",
            xml_escape(&job.filter.loro_surround_mix_level)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<ltrt_center_mix_level>{}</ltrt_center_mix_level>",
            xml_escape(&job.filter.ltrt_center_mix_level)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<ltrt_surround_mix_level>{}</ltrt_surround_mix_level>",
            xml_escape(&job.filter.ltrt_surround_mix_level)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<preferred_downmix_mode>{}</preferred_downmix_mode>",
            xml_escape(&job.filter.preferred_downmix_mode)
        ),
    );
    push_line(&mut buf, 4, "</downmix>");

    push_line(&mut buf, 4, "<custom_trims>");
    push_line(
        &mut buf,
        5,
        &format!(
            "<surround_trim_5_1>{}</surround_trim_5_1>",
            xml_escape(&job.filter.surround_trim_5_1)
        ),
    );
    push_line(
        &mut buf,
        5,
        &format!(
            "<height_trim_5_1>{}</height_trim_5_1>",
            xml_escape(&job.filter.height_trim_5_1)
        ),
    );
    push_line(&mut buf, 4, "</custom_trims>");
    push_line(
        &mut buf,
        4,
        &format!(
            "<custom_dialnorm>{}</custom_dialnorm>",
            job.filter.custom_dialnorm
        ),
    );

    if let Some(v) = &job.filter.encoding_backend {
        push_line(
            &mut buf,
            4,
            &format!("<encoding_backend>{}</encoding_backend>", xml_escape(v)),
        );
    }
    if let Some(v) = &job.filter.encoder_mode {
        push_line(
            &mut buf,
            4,
            &format!("<encoder_mode>{}</encoder_mode>", xml_escape(v)),
        );
    }

    push_line(&mut buf, 3, "</encode_to_atmos_ddp>");
    push_line(&mut buf, 2, "</audio>");
    push_line(&mut buf, 1, "</filter>");

    push_line(&mut buf, 1, "<output>");
    push_line(&mut buf, 2, "<ec3 version=\"1\">");
    push_line(
        &mut buf,
        3,
        &format!("<file_name>{}</file_name>", xml_escape(&output_files)),
    );
    push_line(&mut buf, 3, "<storage>");
    push_line(&mut buf, 4, &format!("<{storage_tag}>"));
    push_line(
        &mut buf,
        5,
        &format!("<path>{}</path>", xml_escape(&job.output.storage_path)),
    );
    push_line(&mut buf, 4, &format!("</{storage_tag}>"));
    push_line(&mut buf, 3, "</storage>");
    push_line(&mut buf, 2, "</ec3>");
    push_line(&mut buf, 1, "</output>");

    push_line(&mut buf, 1, "<misc>");
    push_line(&mut buf, 2, "<temp_dir>");
    push_line(
        &mut buf,
        3,
        &format!(
            "<clean_temp>{}</clean_temp>",
            bool_to_xml(job.misc.clean_temp)
        ),
    );
    push_line(
        &mut buf,
        3,
        &format!("<path>{}</path>", xml_escape(&job.misc.temp_dir)),
    );
    push_line(&mut buf, 2, "</temp_dir>");
    push_line(&mut buf, 1, "</misc>");

    push_line(&mut buf, 0, "</job_config>");
    buf
}

fn push_line(buf: &mut String, indent: usize, value: &str) {
    let spaces = "  ".repeat(indent);
    buf.push_str(&spaces);
    buf.push_str(value);
    buf.push('\n');
}

fn bool_to_xml(v: bool) -> &'static str {
    if v { "true" } else { "false" }
}

fn render_file_name_list(files: &[String]) -> String {
    files
        .iter()
        .map(|f| {
            if f.contains(' ') {
                format!("\"{}\"", f)
            } else {
                f.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use crate::config::{
        AtmosMode, DEFAULT_TEMPLATE_ID, JobMode, Profile, ResolvedFilter, ResolvedIo, ResolvedJob,
        ResolvedMisc, RunSpec,
    };

    use super::render_xml;

    fn sample_job(mode: AtmosMode, job_mode: JobMode) -> ResolvedJob {
        ResolvedJob {
            template_id: DEFAULT_TEMPLATE_ID.to_string(),
            profile: Profile::Standard,
            job_mode,
            atmos_mode: mode,
            input: ResolvedIo {
                storage_path: "Y:/input".to_string(),
                file_names: if matches!(job_mode, JobMode::Single) {
                    vec!["input.atmos".to_string()]
                } else {
                    vec!["a.atmos".to_string(), "b.atmos".to_string()]
                },
            },
            output: ResolvedIo {
                storage_path: "Y:/output".to_string(),
                file_names: if matches!(job_mode, JobMode::Single) {
                    vec!["output.ec3".to_string()]
                } else {
                    vec!["a.ec3".to_string(), "b.ec3".to_string()]
                },
            },
            misc: ResolvedMisc {
                temp_dir: "Y:/tmp".to_string(),
                clean_temp: true,
            },
            filter: ResolvedFilter {
                metering_mode: "1770-4".to_string(),
                dialogue_intelligence: true,
                speech_threshold: 15,
                data_rate: if matches!(mode, AtmosMode::Bluray) {
                    1280
                } else {
                    448
                },
                timecode_frame_rate: "not_indicated".to_string(),
                start: "first_frame_of_action".to_string(),
                end: "end_of_file".to_string(),
                time_base: "file_position".to_string(),
                prepend_silence_duration: "0.0".to_string(),
                append_silence_duration: "0.0".to_string(),
                line_mode_drc_profile: "film_light".to_string(),
                rf_mode_drc_profile: "film_light".to_string(),
                loro_center_mix_level: "-3".to_string(),
                loro_surround_mix_level: "-3".to_string(),
                ltrt_center_mix_level: "-3".to_string(),
                ltrt_surround_mix_level: "-3".to_string(),
                preferred_downmix_mode: "loro".to_string(),
                surround_trim_5_1: "auto".to_string(),
                height_trim_5_1: "auto".to_string(),
                custom_dialnorm: 0,
                encoding_backend: if matches!(mode, AtmosMode::Bluray) {
                    Some("atmosprocessor".to_string())
                } else {
                    None
                },
                encoder_mode: if matches!(mode, AtmosMode::Bluray) {
                    Some("bluray".to_string())
                } else {
                    None
                },
            },
            run: RunSpec::default(),
        }
    }

    #[test]
    fn renders_bluray_extensions_when_set() {
        let xml = render_xml(&sample_job(AtmosMode::Bluray, JobMode::Single));
        assert!(xml.contains("<encoding_backend>atmosprocessor</encoding_backend>"));
        assert!(xml.contains("<encoder_mode>bluray</encoder_mode>"));
    }

    #[test]
    fn renders_album_with_local_multi_path() {
        let xml = render_xml(&sample_job(AtmosMode::Streaming, JobMode::Album));
        assert!(xml.contains("<local_multi_path>"));
        assert!(xml.contains("<file_name>a.atmos b.atmos</file_name>"));
    }
}
