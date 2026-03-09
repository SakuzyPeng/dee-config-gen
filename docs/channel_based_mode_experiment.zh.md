# Channel-based 模式实验（依赖 dee-win 运行时）

本实验用于验证 `dee-config-gen` 后续参数建模所需的关键行为，运行时依赖 `dee-win` 提供的 `dee.exe` 与官方 XML 模板。

## 运行脚本

```bash
cd /path/to/dee-config-gen
scripts/experiment_channel_based_profiles.sh --dee-win-root /path/to/dee-win
```

产出目录：

- `tmp/channel_profile_<timestamp>/summary.tsv`
- `tmp/channel_profile_<timestamp>/logs/*`
- `tmp/channel_profile_<timestamp>/mediainfo/*`

## 核心结论

1. `encode_to_atmos_ddp + streaming + channel-based(16ch)` 可产出 Atmos
   - `Format: E-AC-3 JOC`
   - `Commercial name: Dolby Digital Plus with Dolby Atmos`
2. `encode_to_atmos_ddp + bluray + channel-based`（CBI）失败
   - 错误：`Blu-ray mode not supported for CBI input.`
3. `pcm_to_ddp + encoder_mode=ddp71` 可产出非 Atmos DDP（`E-AC-3`）
   - `8ch` 输入可过；
   - `6ch` 输入也可过，输出仍为 `8ch`。
4. `pcm_to_ddp + encoder_mode=bluray + 8ch + 1536/1664` 可产出“非 Atmos + Blu-ray Disc profile”
   - `Format: E-AC-3`
   - `Format profile: Blu-ray Disc`
   - `Commercial name: Dolby Digital Plus`

## 对参数模型的直接影响

建议在 `dee-config-gen` 中优先按以下模型设计：

1. `pipeline`: `encode_to_atmos_ddp | pcm_to_ddp`
2. `encoder_mode`: 按 pipeline 约束可选值
3. 校验矩阵：
   - `encode_to_atmos_ddp + wav` 输入声道仅允许 `2/6/10/12/16`
   - `encode_to_atmos_ddp + bluray + channel-based` 拒绝
   - `pcm_to_ddp + ddp71` 强约束结果为 `8ch`
   - `pcm_to_ddp + bluray` 当前优先约束 `8ch + 1536/1664`
