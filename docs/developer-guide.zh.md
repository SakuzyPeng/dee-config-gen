# 开发者文档

[用户首页](../README.md) | [English](developer-guide.en.md) | [覆盖与实验](coverage-and-experiments.zh.md)

本文档面向维护者和贡献者，重点说明架构、目录、开发命令和文档维护规则。

## 架构概览

当前核心流程：

1. `config` 负责输入 serde 和路径归一化
2. `resolve` 负责默认值合并、参数校验、约束求值
3. `template` 负责模板注册、模板 schema、模板 XML 结构
4. `render` 负责 `XmlNode` 到 XML 文本的渲染
5. `runner` 负责把生成的 XML 交给外部命令执行
6. `media` 负责输入媒体探测，支撑 input-sensitive 校验

## 模板系统

当前正式模板：
- `atmos_ec3_v1`
  - mode: `streaming` / `bluray`
- `pcm_ddp_v1`
  - mode: `dd` / `ddp` / `ddp71` / `bluray`
- `thd_v1`
- `thd_wav_v1`
- `thd_wav_list_v1`
  - mode: `mlp`

模板模块负责：
- 参数 schema
- 默认值
- 约束
- 模板专属 filter struct
- XML 结构生成
- 必要的 runtime compatibility guard

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
- [`../src/template/atmos_ec3_v1/`](../src/template/atmos_ec3_v1)
- [`../src/template/pcm_ddp_v1/`](../src/template/pcm_ddp_v1)
- [`../src/template/thd_v1/`](../src/template/thd_v1)
- [`../docs/parameter_matrix.atmos_ec3_v1.yaml`](../docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`../docs/parameter_matrix.pcm_ddp_v1.yaml`](../docs/parameter_matrix.pcm_ddp_v1.yaml)
- [`../docs/parameter_matrix.thd_v1.yaml`](../docs/parameter_matrix.thd_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_v1.yaml`](../docs/parameter_matrix.thd_wav_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_list_v1.yaml`](../docs/parameter_matrix.thd_wav_list_v1.yaml)

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
