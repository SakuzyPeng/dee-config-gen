use anyhow::{Result, bail};

use crate::media::InputMediaInfo;

pub(crate) fn validate_truehd_mixed_media_alignment(
    template_id: &str,
    atmos_mezz: &InputMediaInfo,
    secondary: &InputMediaInfo,
) -> Result<()> {
    if atmos_mezz.sample_rate != secondary.sample_rate {
        bail!(
            "template_id '{template_id}' requires matching sample_rate between atmos_mezz and secondary input; got {} and {}",
            atmos_mezz.sample_rate,
            secondary.sample_rate
        );
    }
    if atmos_mezz.bits_per_sample != secondary.bits_per_sample {
        bail!(
            "template_id '{template_id}' requires matching bits_per_sample between atmos_mezz and secondary input; got {} and {}",
            atmos_mezz.bits_per_sample,
            secondary.bits_per_sample
        );
    }

    Ok(())
}

pub(crate) fn validate_truehd_mixed_offset_start_guard(
    template_id: &str,
    input_label: &str,
    offset: &str,
    ffoa: &str,
    start: &str,
) -> Result<()> {
    if (offset != "auto" || ffoa != "auto") && start == "first_frame_of_action" {
        bail!(
            "template_id '{template_id}' requires an explicit start value when offset or ffoa is set on the {input_label}; 'first_frame_of_action' follows the atmos_mezz boundary and can fail at runtime"
        );
    }

    Ok(())
}
