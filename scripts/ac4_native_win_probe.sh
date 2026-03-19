#!/usr/bin/env bash
set -euo pipefail

HOST="${1:-win-pc}"
DEE_DIR="${2:-F:\\dee\\dolby_encoding_engine_windows_5.2.1\\dolby_encoding_engine}"
PROBE_DIR="${3:-F:\\dee\\temp\\ac4_native_probe}"

run_ps() {
  local command="$1"
  local wrapped
  local encoded
  wrapped="\$ProgressPreference='SilentlyContinue'; \$ErrorActionPreference='Stop'; ${command}"
  encoded="$(
    printf '%s' "${wrapped}" \
      | iconv -f UTF-8 -t UTF-16LE \
      | base64 \
      | tr -d '\r\n'
  )"
  ssh -T "${HOST}" "powershell -NoProfile -NonInteractive -EncodedCommand ${encoded}"
}

echo "[ac4-native-win-probe] host=${HOST}"
echo "[ac4-native-win-probe] dee_dir=${DEE_DIR}"
echo "[ac4-native-win-probe] probe_dir=${PROBE_DIR}"

run_ps "if (-not (Test-Path -LiteralPath '${DEE_DIR}\\dee.exe')) { throw 'dee.exe not found under ${DEE_DIR}' }"
run_ps "if (-not (Test-Path -LiteralPath '${DEE_DIR}\\mp4muxer.exe')) { throw 'mp4muxer.exe not found under ${DEE_DIR}' }"
run_ps "if (-not (Test-Path -LiteralPath '${PROBE_DIR}\\atmos_ac4.xml')) { throw 'missing ${PROBE_DIR}\\atmos_ac4.xml' }"
run_ps "if (-not (Test-Path -LiteralPath '${PROBE_DIR}\\atmos_mp4.xml')) { throw 'missing ${PROBE_DIR}\\atmos_mp4.xml' }"

echo "[ac4-native-win-probe] shortcut target (if present)"
run_ps "\$lnk = Get-ChildItem -LiteralPath '${DEE_DIR}' -Filter '*.lnk' | Select-Object -First 1; if (\$null -eq \$lnk) { Write-Output 'LINK_TARGET=<none>' } else { \$target = (New-Object -ComObject WScript.Shell).CreateShortcut(\$lnk.FullName).TargetPath; Write-Output ('LINK_TARGET=' + \$target) }"

echo "[ac4-native-win-probe] cleaning output/log directories"
run_ps "Remove-Item -Path '${PROBE_DIR}\\out\\*' -Force -ErrorAction SilentlyContinue; Remove-Item -Path '${PROBE_DIR}\\logs\\*' -Force -ErrorAction SilentlyContinue"

echo "[ac4-native-win-probe] running AC-4 direct output"
run_ps "& '${DEE_DIR}\\dee.exe' --disable-xml-validation -x '${PROBE_DIR}\\atmos_ac4.xml' --log-file '${PROBE_DIR}\\logs\\ac4_direct.log'; Write-Output ('AC4_EXITCODE=' + \$LASTEXITCODE); if (\$LASTEXITCODE -ne 0) { exit \$LASTEXITCODE }"

echo "[ac4-native-win-probe] running MP4 direct output"
run_ps "& '${DEE_DIR}\\dee.exe' --disable-xml-validation -x '${PROBE_DIR}\\atmos_mp4.xml' --log-file '${PROBE_DIR}\\logs\\mp4_direct.log'; Write-Output ('MP4_EXITCODE=' + \$LASTEXITCODE); if (\$LASTEXITCODE -ne 0) { exit \$LASTEXITCODE }"

echo "[ac4-native-win-probe] running manual mux from generated output.ac4"
run_ps "if (-not (Test-Path -LiteralPath '${PROBE_DIR}\\out\\output.ac4')) { throw 'output.ac4 missing after AC-4 direct run' }; & '${DEE_DIR}\\mp4muxer.exe' -i '${PROBE_DIR}\\out\\output.ac4' -o '${PROBE_DIR}\\out\\manual_mux.mp4' --overwrite; Write-Output ('MANUAL_MUX_EXITCODE=' + \$LASTEXITCODE); if (\$LASTEXITCODE -ne 0) { exit \$LASTEXITCODE }"

echo "[ac4-native-win-probe] output files"
run_ps "Get-ChildItem -LiteralPath '${PROBE_DIR}\\out' | Select-Object Name,Length,LastWriteTime | Format-Table -AutoSize"

echo "[ac4-native-win-probe] MP4 log tail"
run_ps "Get-Content -LiteralPath '${PROBE_DIR}\\logs\\mp4_direct.log' -Tail 40"

echo "[ac4-native-win-probe] Access violation scan"
run_ps "\$hits = Select-String -Path '${PROBE_DIR}\\logs\\mp4_direct.log' -Pattern 'Access violation|exception|fatal'; if (\$hits) { \$hits } else { Write-Output '<none>' }"

echo "[ac4-native-win-probe] done"
