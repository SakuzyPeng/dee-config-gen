# Upstream Parameter Observations

Generated: 2026-03-08T15:02:42Z

## deew (MIT)

### Bluray/bitrate hints
```text
upstream/deew/deew/bitrates.py:9:    'ddp_71_bluray': [768, 1024, 1280, 1536, 1664],
upstream/deew/deew/bitrates.py:10:    'ddp_71_combined': [384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024, 1280, 1536, 1664],
upstream/deew/deew/__main__.py:169:parser.add_argument('-fb', '--force-bluray',
upstream/deew/deew/__main__.py:737:                bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_71_bluray'])
upstream/deew/deew/__main__.py:739:                bitrate = find_closest_allowed(bitrate, allowed_bitrates['ddp_71_combined'])
upstream/deew/deew/__main__.py:754:            xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp'
upstream/deew/deew/__main__.py:756:                xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp71'
upstream/deew/deew/__main__.py:758:                    xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'bluray'
upstream/deew/deew/__main__.py:760:                    xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'ddp71'
upstream/deew/deew/__main__.py:762:                    xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'bluray'
upstream/deew/deew/__main__.py:764:            xml_base['job_config']['filter']['audio']['pcm_to_ddp']['encoder_mode'] = 'dd'
upstream/deew/deew/__main__.py:769:        xml_base['job_config']['filter']['audio']['pcm_to_ddp']['custom_dialnorm'] = args.dialnorm
upstream/deew/deew/__main__.py:786:        xml_base['job_config']['filter']['audio']['encode_to_dthd']['custom_dialnorm'] = args.dialnorm
```

## DeeZy (MIT)

### Atmos bluray mode + extended bitrates
```text
upstream/DeeZy/example_json_flows/atmos-ec3-bluray.json:53:          "encoding_backend": "atmosprocessor",
upstream/DeeZy/example_json_flows/atmos-ec3-bluray.json:54:          "encoder_mode": "bluray"
upstream/DeeZy/deezy/enums/atmos.py:7:class AtmosMode(CaseInsensitiveEnum):
upstream/DeeZy/deezy/enums/atmos.py:9:    BLURAY = "bluray"
upstream/DeeZy/deezy/enums/atmos.py:16:        if self is AtmosMode.STREAMING:
upstream/DeeZy/deezy/enums/atmos.py:19:                choices=(
upstream/DeeZy/deezy/enums/atmos.py:33:                choices=(1152, 1280, 1408, 1512, 1536, 1664),
upstream/DeeZy/deezy/enums/atmos.py:37:        if self is AtmosMode.STREAMING:
upstream/DeeZy/deezy/enums/atmos.py:43:        if self is AtmosMode.STREAMING:
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:8:from deezy.enums.atmos import AtmosMode
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:73:        filter_section["encoder_mode"] = dd_mode.get_encoder_mode()
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:131:        atmos_mode: AtmosMode,
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:165:        # add encoding_backend and encoder mode if atmos 7.1 (bluray)
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:166:        if atmos_mode is AtmosMode.BLURAY:
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:167:            filter_section["encoding_backend"] = "atmosprocessor"
upstream/DeeZy/deezy/audio_encoders/dee/json/dee_json_generator.py:168:            filter_section["encoder_mode"] = "bluray"
```

## Notes

- This report is a metadata-only observation snapshot.
- Do not copy upstream implementation logic verbatim into this project.
- Runtime validation source of truth remains src/registry.rs + docs/parameter_matrix.atmos_ec3_v1.yaml.
