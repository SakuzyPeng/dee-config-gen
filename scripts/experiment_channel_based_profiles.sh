#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_DEE_WIN_ROOT="$ROOT_DIR/../../dee-win"
DEE_WIN_ROOT="${DEE_WIN_ROOT:-$DEFAULT_DEE_WIN_ROOT}"
DEE_EXE="${DEE_EXE:-$DEE_WIN_ROOT/dolby_encoding_engine/dee.exe}"
SOURCE_WAV="${SOURCE_WAV:-$DEE_WIN_ROOT/testADM.wav}"
EXP_DIR="${EXP_DIR:-$ROOT_DIR/tmp/channel_profile_$(date +%Y%m%d_%H%M%S)}"
WINEPREFIX="${WINEPREFIX:-$ROOT_DIR/tmp/wineprefix_native}"

XML_STREAM="$DEE_WIN_ROOT/dolby_encoding_engine/xml_templates/encode_to_atmos_ddp/wav_encode_to_atmos_ddp_ec3.xml"
XML_PCM_DDP71="$DEE_WIN_ROOT/dolby_encoding_engine/xml_templates/pcm_to_ddp/wav_pcm_to_ddp_ec3_71.xml"

usage() {
  cat <<'EOF'
Usage: scripts/experiment_channel_based_profiles.sh [options]

Options:
  --dee-win-root PATH  dee-win workspace root (default: ../../dee-win from this repo)
  --source-wav PATH    Source wav path (default: <dee-win-root>/testADM.wav)
  --exp-dir PATH       Output experiment directory (default: ./tmp/channel_profile_<timestamp>)
  --wineprefix PATH    Wine prefix path (default: ./tmp/wineprefix_native)
  --dee-exe PATH       dee.exe path (default: <dee-win-root>/dolby_encoding_engine/dee.exe)
  -h, --help           Show this help
EOF
}

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd" >&2
    exit 1
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dee-win-root)
      DEE_WIN_ROOT="$2"
      DEE_EXE="${DEE_WIN_ROOT}/dolby_encoding_engine/dee.exe"
      SOURCE_WAV="${DEE_WIN_ROOT}/testADM.wav"
      XML_STREAM="$DEE_WIN_ROOT/dolby_encoding_engine/xml_templates/encode_to_atmos_ddp/wav_encode_to_atmos_ddp_ec3.xml"
      XML_PCM_DDP71="$DEE_WIN_ROOT/dolby_encoding_engine/xml_templates/pcm_to_ddp/wav_pcm_to_ddp_ec3_71.xml"
      shift 2
      ;;
    --source-wav)
      SOURCE_WAV="$2"
      shift 2
      ;;
    --exp-dir)
      EXP_DIR="$2"
      shift 2
      ;;
    --wineprefix)
      WINEPREFIX="$2"
      shift 2
      ;;
    --dee-exe)
      DEE_EXE="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

require_cmd wine64
require_cmd ffmpeg
require_cmd mediainfo

if [[ ! -f "$DEE_EXE" ]]; then
  echo "dee.exe not found: $DEE_EXE" >&2
  echo "Hint: pass --dee-win-root /path/to/dee-win" >&2
  exit 1
fi

if [[ ! -f "$SOURCE_WAV" ]]; then
  echo "Source wav not found: $SOURCE_WAV" >&2
  exit 1
fi

if [[ ! -f "$XML_STREAM" || ! -f "$XML_PCM_DDP71" ]]; then
  echo "Required XML templates not found under: $DEE_WIN_ROOT/dolby_encoding_engine/xml_templates" >&2
  exit 1
fi

mkdir -p "$EXP_DIR"/{xml,in,out,logs,tmp,mediainfo}
mkdir -p "$WINEPREFIX"

IN16="$EXP_DIR/in/16ch.wav"
IN8="$EXP_DIR/in/8ch.wav"
IN6="$EXP_DIR/in/6ch.wav"
ln -sf "$SOURCE_WAV" "$IN16"

ffmpeg -y -hide_banner -loglevel error -i "$IN16" -ac 8 -c:a pcm_f32le "$IN8"
ffmpeg -y -hide_banner -loglevel error -i "$IN16" -ac 6 -c:a pcm_f32le "$IN6"

XML_STREAM_BLURAY="$EXP_DIR/xml/encode_to_atmos_ddp_bluray.xml"
XML_PCM_BLURAY_1536="$EXP_DIR/xml/pcm_to_ddp_bluray_1536.xml"
XML_PCM_BLURAY_1664="$EXP_DIR/xml/pcm_to_ddp_bluray_1664.xml"

perl -0pe \
  's#<data_rate>448</data_rate>#<data_rate>1536</data_rate>#; s#</encode_to_atmos_ddp>#        <encoding_backend>atmosprocessor</encoding_backend>\n        <encoder_mode>bluray</encoder_mode>\n      </encode_to_atmos_ddp>#' \
  "$XML_STREAM" > "$XML_STREAM_BLURAY"

perl -0pe \
  's#<encoder_mode>ddp71</encoder_mode>#<encoder_mode>bluray</encoder_mode>#; s#<data_rate>384</data_rate>#<data_rate>1536</data_rate>#' \
  "$XML_PCM_DDP71" > "$XML_PCM_BLURAY_1536"

perl -0pe \
  's#<encoder_mode>ddp71</encoder_mode>#<encoder_mode>bluray</encoder_mode>#; s#<data_rate>384</data_rate>#<data_rate>1664</data_rate>#' \
  "$XML_PCM_DDP71" > "$XML_PCM_BLURAY_1664"

SUMMARY="$EXP_DIR/summary.tsv"
echo -e "case\tpipeline\txml\tinput\trc\tsecs\tout_exists\tformat\tprofile\tcommercial\taudio_format\taudio_profile\taudio_channels\tbitrate" > "$SUMMARY"

run_case() {
  local case_id="$1"
  local pipeline="$2"
  local xml="$3"
  local input="$4"
  local out="$EXP_DIR/out/${case_id}.ec3"
  local log="$EXP_DIR/logs/${case_id}.log"
  local stdout_log="$EXP_DIR/logs/${case_id}.stdout.log"
  local tmp_dir="$EXP_DIR/tmp/${case_id}"

  mkdir -p "$tmp_dir"

  local t0 t1 rc
  local out_exists=0 fmt="" prof="" comm="" afmt="" aprof="" ach="" br=""
  t0="$(date +%s)"

  set +e
  WINEPREFIX="$WINEPREFIX" wine64 "$DEE_EXE" \
    --xml "$xml" \
    --input-audio "$input" \
    --output "$out" \
    --temp "$tmp_dir" \
    --log-file "$log" \
    --stdout --verbose info >"$stdout_log" 2>&1
  rc=$?
  set -e

  t1="$(date +%s)"

  if [[ -f "$out" ]]; then
    out_exists=1
    mediainfo "$out" > "$EXP_DIR/mediainfo/${case_id}.txt"
    fmt="$(awk -F': *' '/^Format[[:space:]]*:/ {print $2; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    prof="$(awk -F': *' '/^Format profile[[:space:]]*:/ {print $2; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    comm="$(awk -F': *' '/^Commercial name[[:space:]]*:/ {print $2; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    afmt="$(awk 'BEGIN{s=0} /^Audio$/ {s=1; next} s && /^Format[[:space:]]*:/ {sub(/^.*: */,""); print; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    aprof="$(awk 'BEGIN{s=0} /^Audio$/ {s=1; next} s && /^Format profile[[:space:]]*:/ {sub(/^.*: */,""); print; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    ach="$(awk 'BEGIN{s=0} /^Audio$/ {s=1; next} s && /^Channel\(s\)[[:space:]]*:/ {sub(/^.*: */,""); print; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
    br="$(awk -F': *' '/^Overall bit rate[[:space:]]*:/ {print $2; exit}' "$EXP_DIR/mediainfo/${case_id}.txt")"
  fi

  echo -e "${case_id}\t${pipeline}\t$(basename "$xml")\t$(basename "$input")\t${rc}\t$((t1-t0))\t${out_exists}\t${fmt}\t${prof}\t${comm}\t${afmt}\t${aprof}\t${ach}\t${br}" >> "$SUMMARY"
}

run_case "streaming_16ch" "encode_to_atmos_ddp" "$XML_STREAM" "$IN16"
run_case "bluray_cbi_16ch" "encode_to_atmos_ddp" "$XML_STREAM_BLURAY" "$IN16"
run_case "ddp71_8ch" "pcm_to_ddp" "$XML_PCM_DDP71" "$IN8"
run_case "ddp71_6ch" "pcm_to_ddp" "$XML_PCM_DDP71" "$IN6"
run_case "bluray_non_atmos_1536_8ch" "pcm_to_ddp" "$XML_PCM_BLURAY_1536" "$IN8"
run_case "bluray_non_atmos_1664_8ch" "pcm_to_ddp" "$XML_PCM_BLURAY_1664" "$IN8"

echo "Experiment done."
echo "Summary: $SUMMARY"
cat "$SUMMARY"
