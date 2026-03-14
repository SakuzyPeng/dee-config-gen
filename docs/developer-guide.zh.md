# 开发者文档

[用户首页](../README.md) | [English](developer-guide.en.md) | [JSON 输出](json-output.zh.md) | [覆盖与实验](coverage-and-experiments.zh.md)

本文档面向维护者和贡献者，重点说明架构、目录、开发命令和文档维护规则。

## 架构概览

当前核心流程：

1. `spec` 负责输入 serde 和路径归一化
2. `resolve` 负责默认值合并、参数校验、约束求值
3. `template` 负责模板注册、模板 schema、模板 XML/JSON 结构
4. `render` 负责按 `RenderFormat` 渲染 XML 或 JSON 文本
5. `runner` 负责把生成的配置文件交给外部命令执行
6. `media` 负责输入媒体探测，支撑 input-sensitive 校验

## 模板系统

当前正式模板：
- `ac4_ims_atmos_v1`
  - mode: `ac4`
- `ac4_ims_pcm_v1`
  - mode: `ac4`
- `atmos_ec3_v1`
  - mode: `streaming` / `bluray`
- `pcm_ddp_v1`
  - mode: `dd` / `ddp` / `ddp71` / `bluray`
- `thd_v1`
- `thd_wav_v1`
- `thd_wav_list_v1`
- `thd_atmos_wav_v1`
- `thd_atmos_wav_list_v1`
  - mode: `mlp`

模板模块负责：
- 参数 schema
- 默认值
- 约束
- 模板专属 filter struct
- XML 结构生成
- JSON 结构生成（如果该模板支持）
- 必要的 runtime compatibility guard

当前 JSON 输出策略：
- `RenderFormat::Xml` 与 `RenderFormat::Json` 并行
- XML 仍是默认值
- 当前 `atmos_ec3_v1`、`pcm_ddp_v1` 与全部 TrueHD 模板都实现了原生 JSON hook
- `ac4_ims_atmos_v1` 与 `ac4_ims_pcm_v1` 当前刻意保持 XML-only

核心入口：
- [`../src/template/mod.rs`](../src/template/mod.rs)
- [`../src/resolve.rs`](../src/resolve.rs)

## 目录结构

关键目录：
- `src/`：CLI、解析、模板、渲染、runner
- `examples/`：可直接运行的样例输入
- `docs/`：参数矩阵、覆盖表、实验记录、开发文档
- `tests/`：schema/XSD/example/runtime/pitfall 多层测试
- `scripts/`：XSD 抽取、upstream 同步、实验脚本

建议优先阅读：
- [`../src/template/ac4_ims_shared.rs`](../src/template/ac4_ims_shared.rs)
- [`../src/template/ac4_ims_atmos_v1/`](../src/template/ac4_ims_atmos_v1)
- [`../src/template/ac4_ims_pcm_v1/`](../src/template/ac4_ims_pcm_v1)
- [`../src/template/atmos_ec3_v1/`](../src/template/atmos_ec3_v1)
- [`../src/template/pcm_ddp_v1/`](../src/template/pcm_ddp_v1)
- [`../src/template/thd_v1/`](../src/template/thd_v1)
- [`../docs/parameter_matrix.ac4_ims_atmos_v1.yaml`](../docs/parameter_matrix.ac4_ims_atmos_v1.yaml)
- [`../docs/parameter_matrix.ac4_ims_pcm_v1.yaml`](../docs/parameter_matrix.ac4_ims_pcm_v1.yaml)
- [`../docs/ac4-official-notes.zh.md`](../docs/ac4-official-notes.zh.md)
- [`../docs/parameter_matrix.atmos_ec3_v1.yaml`](../docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`../docs/parameter_matrix.pcm_ddp_v1.yaml`](../docs/parameter_matrix.pcm_ddp_v1.yaml)
- [`../docs/parameter_matrix.thd_v1.yaml`](../docs/parameter_matrix.thd_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_v1.yaml`](../docs/parameter_matrix.thd_wav_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_list_v1.yaml`](../docs/parameter_matrix.thd_wav_list_v1.yaml)
- [`../docs/parameter_matrix.thd_atmos_wav_v1.yaml`](../docs/parameter_matrix.thd_atmos_wav_v1.yaml)
- [`../docs/parameter_matrix.thd_atmos_wav_list_v1.yaml`](../docs/parameter_matrix.thd_atmos_wav_list_v1.yaml)

## 常用开发命令

构建：

```bash
cargo build
```

快速测试：

```bash
cargo test
```

预提交检查：

```bash
scripts/precommit_checks.sh
```

单条命令冒烟：

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
cargo run -- generate -i examples/pcm_ddp_single.dd.yaml -o job.xml
```

## 测试分层

当前测试分成 5 层：
- schema / resolve
- XSD contract
- example regression
- runtime regression
- pitfall / knowledge

原则：
- schema/XSD/example 进入默认 `cargo test`
- 真实 DEE runtime 保持 `#[ignore]`，手动触发
- pitfall 和 knowledge 用于固化“已知差异”，避免靠记忆维护

runtime 入口见：
- [`coverage-and-experiments.zh.md`](coverage-and-experiments.zh.md)

## 文档维护规则

首页与开发文档职责分离：
- `README.md` / `README.en.md` 只面向普通用户
- 开发、覆盖、实验细节放在 `docs/`

维护时遵守：
- 参数真相源是模板 schema 与矩阵文档，不在 README 重复长表
- 新 runtime 结论必须同步更新 coverage matrix 和 knowledge
- 首页只保留“是什么、怎么用、去哪看更深内容”

### 模板级真相源清单

新增或修改正式模板时，至少同步这些文件：
- `docs/parameter_matrix.<template>.yaml`
- `docs/coverage_matrix.full.yaml`
- `tests/fixtures/upstream_knowledge.json`
- `docs/json-output.zh.md` / `docs/json-output.en.md`（支持面变化时同步）
- 至少一个 `examples/` 样例
- 对应的 XSD contract fixture 与 smoke test

按能力再补：
- 若有真实 runtime 结论：补 `#[ignore]` runtime 测试
- 若有 runtime 差异或限制：补 `upstream_pitfalls.*.json` 和对应测试

状态使用规则：
- `covered`
  - 需要 schema / XSD / runtime / knowledge 都已有证据
- `conservative_gap`
  - 至少要有 schema + knowledge
  - 只在 runtime 尚未完整验证，或本地刻意保持比 runtime 更宽/更窄时使用
- `unsupported_or_hidden`
  - 必须有 knowledge 证据
  - 若属于 runtime 已证实不支持，应该再补 pitfall 或等价 runtime 记录

仓库中有模板级一致性测试，会检查：
- `src/template/mod.rs` 里已注册的模板，必须同时出现在 coverage matrix、parameter matrix、XSD contract、example 和 knowledge 中
- `unsupported_or_hidden` 参数必须在 `upstream_knowledge.json` 里有对应描述
