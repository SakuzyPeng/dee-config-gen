#[derive(Debug, Clone)]
pub struct ThdV1Filter {
    pub metering_mode: String,
    pub dialogue_intelligence: bool,
    pub speech_threshold: u8,
    pub timecode_frame_rate: String,
    pub start: String,
    pub end: String,
    pub time_base: String,
    pub prepend_silence_duration: String,
    pub append_silence_duration: String,
    pub custom_dialnorm: i8,
    pub atmos_presentation_drc_profile: String,
    pub presentation_8ch_drc_profile: String,
    pub presentation_6ch_drc_profile: String,
    pub presentation_2ch_drc_profile: String,
}
