# DEE JSON 输出
[用户首页](../README.md) | [English](json-output.en.md) | [开发者文档](developer-guide.zh.md)

## 概览
`dee-config-gen` 现在支持直接生成 Dolby DEE 可执行的 JSON 配置文件。

当前行为：
- 默认输出仍是 XML
- 通过 `--format json` 切换到 JSON
- `validate`、`generate`、`run` 三个命令都支持 JSON
- `run --format json` 会自动向 DEE runner 注入 `--json`

## 当前支持的模板
- `atmos_ec3_v1`
- `pcm_ddp_v1`
- `thd_v1`
- `thd_wav_v1`
- `thd_wav_list_v1`
- `thd_atmos_wav_v1`
- `thd_atmos_wav_list_v1`

## 当前不支持 JSON 的模板
- `ac4_ims_atmos_v1`
  - 当前只支持 `inputs.atmos_mezz -> output/ac4` 的 XML 路径，官方文档尚未整理出独立 JSON 证据
- `ac4_ims_pcm_v1`
  - 当前只支持 `inputs.wav` 或 `inputs.wav_list -> output/ac4` 的 XML 路径，官方文档尚未整理出独立 JSON 证据

迁移说明：
- `ac4_v1` 已移除；AC-4 immersive stereo 现拆分为 Atmos 输入模板和 PCM 输入模板

如果模板暂不支持 JSON，工具会直接报错，不会进入半支持状态。

## 快速示例
生成 JSON：

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml --format json -o job.json
```

校验 JSON 渲染能力：

```bash
cargo run -- validate -i examples/pcm_ddp_single.dd.yaml --format json
```

直接运行 JSON：

```bash
cargo run -- run -i examples/thd_single.mlp.yaml --format json --runner-cmd "dee"
```

## 输出语义
- 参数校验、默认值、模板约束仍然完全复用现有 `resolve`
- JSON 只是另一种 DEE 配置序列化形式，不是另一套参数系统
- XML 继续是默认输出格式，JSON 是并行能力

## 与 XML 的关系
- 结构上，JSON 与 XML 对应同一个 `job_config`
- 支持面以本机 `DEE 5.2.1` 的真实接受面为准
- 如果某模板的 JSON 路线未来出现与 XML 不同的 runtime 差异，会进入：
  - `tests/fixtures/upstream_knowledge.json`
  - 必要时进入对应 pitfall fixture

## 相关文档
- 覆盖总表：[`coverage_matrix.full.yaml`](coverage_matrix.full.yaml)
- 覆盖与实验：[`coverage-and-experiments.zh.md`](coverage-and-experiments.zh.md)
- 开发者文档：[`developer-guide.zh.md`](developer-guide.zh.md)
