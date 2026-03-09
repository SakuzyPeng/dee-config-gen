# Channel-Based Mode Experiment (using dee-win runtime)

This experiment captures runtime behavior needed for `dee-config-gen` mode modeling.  
Runtime dependency is `dee-win` (for `dee.exe` and official XML templates).

## Run

```bash
cd /path/to/dee-config-gen
scripts/experiment_channel_based_profiles.sh --dee-win-root /path/to/dee-win
```

Outputs:

- `tmp/channel_profile_<timestamp>/summary.tsv`
- `tmp/channel_profile_<timestamp>/logs/*`
- `tmp/channel_profile_<timestamp>/mediainfo/*`

## Key findings

1. `encode_to_atmos_ddp + streaming + channel-based(16ch)` can produce Atmos
   - `Format: E-AC-3 JOC`
   - `Commercial name: Dolby Digital Plus with Dolby Atmos`
2. `encode_to_atmos_ddp + bluray + channel-based` (CBI) fails
   - Error: `Blu-ray mode not supported for CBI input.`
3. `pcm_to_ddp + encoder_mode=ddp71` produces non-Atmos DDP (`E-AC-3`)
   - `8ch` input passes;
   - `6ch` input also passes and still outputs `8ch`.
4. `pcm_to_ddp + encoder_mode=bluray + 8ch + 1536/1664` produces non-Atmos output with Blu-ray profile
   - `Format: E-AC-3`
   - `Format profile: Blu-ray Disc`
   - `Commercial name: Dolby Digital Plus`

## Model impact

For `dee-config-gen`, this supports a model with:

1. `pipeline`: `encode_to_atmos_ddp | pcm_to_ddp`
2. `encoder_mode`: constrained by selected pipeline
3. validation matrix:
   - `encode_to_atmos_ddp + wav` input channels limited to `2/6/10/12/16`
   - `encode_to_atmos_ddp + bluray + channel-based` rejected
   - `pcm_to_ddp + ddp71` enforced as effective 8ch path
   - `pcm_to_ddp + bluray` currently constrained to `8ch + 1536/1664`
