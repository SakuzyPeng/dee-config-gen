# 覆盖与实验说明

[用户首页](../README.md) | [English](coverage-and-experiments.en.md) | [JSON 输出](json-output.zh.md) | [开发者文档](developer-guide.zh.md)

本文档面向维护和研究，说明覆盖语义、runtime 套件、XSD contract、knowledge/pitfall 与实验记录之间的关系。

## 覆盖语义

覆盖总表：[`coverage_matrix.full.yaml`](coverage_matrix.full.yaml)

状态定义：
- `covered`：该层行为已经被明确验证
- `conservative_gap`：本地建模刻意更保守，或 runtime 只做了部分验证
- `unsupported_or_hidden`：不属于官方 contract，或 runtime 已确认该路径/属性不支持

规则：
- coverage matrix 是覆盖状态的唯一总表
- 新的 runtime 结论必须同步更新 matrix
- 不允许把口头推断当成覆盖状态写进 matrix

## 手动 runtime 套件

AC-4 immersive stereo（atmos_mezz 输入）：

```bash
cargo test --test dee_runtime_ac4_ims_atmos -- --ignored --nocapture --test-threads=1
```

AC-4 immersive stereo（PCM 输入）：

```bash
cargo test --test dee_runtime_ac4_ims_pcm -- --ignored --nocapture --test-threads=1
```

Atmos：

```bash
cargo test --test dee_runtime_hidden_params -- --ignored --nocapture
```

PCM：

```bash
cargo test --test dee_runtime_pcm_ddp -- --ignored --nocapture
```

TrueHD：

```bash
cargo test --test dee_runtime_thd -- --ignored --nocapture
```

TrueHD WAV：

```bash
cargo test --test dee_runtime_thd_wav -- --ignored --nocapture
```

TrueHD WAV list：

```bash
cargo test --test dee_runtime_thd_wav_list -- --ignored --nocapture
```

TrueHD mixed input（atmos_mezz + wav）：

```bash
cargo test --test dee_runtime_thd_atmos_wav -- --ignored --nocapture
```

TrueHD mixed input：

```bash
cargo test --test dee_runtime_thd_atmos_wav_list -- --ignored --nocapture
```

用途：
- 验证真实 DEE 5.2.1 行为
- 固化 hidden extension、runtime normalization、known gaps
- 支撑 coverage matrix 与 knowledge fixture 更新

本地 AC-4 fixture 契约：
- atmos embedded-timecode 覆盖依赖 `testfiles/testADM.wav`
- PCM embedded-timecode 覆盖现在通过官方输入时码参数 `input_timecode_frame_rate`、`offset`、`ffoa` 配合运行时生成的 5.1 WAV 或 mono stems 完成，不再依赖额外本地 PCM fixture

## XSD contract 与 fixtures

官方 contract 相关文件：
- AC-4 IMS atmos contract：[`../tests/fixtures/xsd/contract.ac4_ims_atmos_v1.json`](../tests/fixtures/xsd/contract.ac4_ims_atmos_v1.json)
- AC-4 IMS PCM contract：[`../tests/fixtures/xsd/contract.ac4_ims_pcm_v1.json`](../tests/fixtures/xsd/contract.ac4_ims_pcm_v1.json)
- Atmos contract：[`../tests/fixtures/xsd/contract.atmos_ec3_v1.json`](../tests/fixtures/xsd/contract.atmos_ec3_v1.json)
- PCM contract：[`../tests/fixtures/xsd/contract.pcm_ddp_v1.json`](../tests/fixtures/xsd/contract.pcm_ddp_v1.json)
- TrueHD contract：[`../tests/fixtures/xsd/contract.thd_v1.json`](../tests/fixtures/xsd/contract.thd_v1.json)
- TrueHD WAV contract：[`../tests/fixtures/xsd/contract.thd_wav_v1.json`](../tests/fixtures/xsd/contract.thd_wav_v1.json)
- TrueHD WAV list contract：[`../tests/fixtures/xsd/contract.thd_wav_list_v1.json`](../tests/fixtures/xsd/contract.thd_wav_list_v1.json)
- TrueHD mixed-input WAV contract：[`../tests/fixtures/xsd/contract.thd_atmos_wav_v1.json`](../tests/fixtures/xsd/contract.thd_atmos_wav_v1.json)
- TrueHD mixed-input contract：[`../tests/fixtures/xsd/contract.thd_atmos_wav_list_v1.json`](../tests/fixtures/xsd/contract.thd_atmos_wav_list_v1.json)
- raw XSD（仅本地使用，不提交）：`../tests/fixtures/xsd/raw/`

抽取脚本：
- [`../scripts/extract_xsd_contract.py`](../scripts/extract_xsd_contract.py)

说明：
- XSD 负责 official contract
- runtime 负责真实外部行为
- `tests/fixtures/xsd/raw/` 下的官方导出 XSD 只允许本地存在，不应提交到仓库
- 两者不一致时，必须在 coverage/knowledge 中显式记录

## Knowledge / Pitfall / 实验索引

知识归档：
- [`../tests/fixtures/upstream_knowledge.json`](../tests/fixtures/upstream_knowledge.json)

Pitfall fixtures：
- [`../tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json`](../tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json)
- [`../tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json`](../tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json)
- [`../tests/fixtures/upstream_pitfalls.thd_v1.json`](../tests/fixtures/upstream_pitfalls.thd_v1.json)

参数矩阵：
- [`parameter_matrix.ac4_ims_atmos_v1.yaml`](parameter_matrix.ac4_ims_atmos_v1.yaml)
- [`parameter_matrix.ac4_ims_pcm_v1.yaml`](parameter_matrix.ac4_ims_pcm_v1.yaml)
- [`parameter_matrix.atmos_ec3_v1.yaml`](parameter_matrix.atmos_ec3_v1.yaml)
- [`parameter_matrix.pcm_ddp_v1.yaml`](parameter_matrix.pcm_ddp_v1.yaml)
- [`parameter_matrix.thd_v1.yaml`](parameter_matrix.thd_v1.yaml)
- [`parameter_matrix.thd_wav_v1.yaml`](parameter_matrix.thd_wav_v1.yaml)
- [`parameter_matrix.thd_wav_list_v1.yaml`](parameter_matrix.thd_wav_list_v1.yaml)
- [`parameter_matrix.thd_atmos_wav_v1.yaml`](parameter_matrix.thd_atmos_wav_v1.yaml)
- [`parameter_matrix.thd_atmos_wav_list_v1.yaml`](parameter_matrix.thd_atmos_wav_list_v1.yaml)
- [`ac4-official-notes.zh.md`](ac4-official-notes.zh.md)
- [`json-output.zh.md`](json-output.zh.md)

实验记录：
- [`upstream_parameter_observations.md`](upstream_parameter_observations.md)
- [`channel_based_mode_experiment.zh.md`](channel_based_mode_experiment.zh.md)
- [`channel_based_mode_experiment.md`](channel_based_mode_experiment.md)

## 当前维护约定

- 高频主路径参数优先追求 `covered`
- 低价值尾项允许保留 `conservative_gap`，但必须是“已知、可解释”的 gap
- 若 runtime 证明当前生产建模会稳定生成失败 XML，优先收敛代码；否则先记录差异
