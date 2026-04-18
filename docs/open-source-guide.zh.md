# 开源边界与复现指南

[用户首页](../README.md) | [English](open-source-guide.en.md) | [贡献指南](../CONTRIBUTING.md) | [安全策略](../SECURITY.md)

本文档说明仓库当前的开源分发边界，以及外部贡献者在没有本地 Dolby 工具链时可以稳定复现到什么程度。

## 默认可复现路径

公开贡献者默认应能完成：

```bash
cargo build
cargo test --workspace --quiet
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
cargo run -- generate -i examples/pcm_ddp_single.dd.yaml -o job.xml
```

这条路径不要求本地安装 Dolby DEE、`dee-win` 或私有音频样本。

## 手动 Runtime 套件边界

以下内容仍保留在仓库中，但属于手动、非默认的维护路径：

- `tests/dee_runtime_*.rs` 真实 DEE runtime 套件
- `scripts/ac4_native_win_probe.sh` 之类的本地实验脚本
- `docs/coverage-and-experiments.*` 中记录的 runtime 结论

约定：
- 真实 runtime 测试保持 `#[ignore]`
- 触发方式通常是 `cargo test --test <name> -- --ignored --nocapture`
- 这些套件可能依赖本地 `dee`、`ffmpeg`、`DEE_WORKSPACE_ROOT`
- AC-4 MP4 路径还可能依赖 AC-4 package 与 native mp4 muxer

示例：

```bash
cargo test --test dee_runtime_pcm_ddp -- --ignored --nocapture
cargo test --test dee_runtime_json -- --ignored --nocapture
```

## 不随仓库分发的内容

下列内容被视为本地或专有依赖，不应当期待仓库直接分发：

- Dolby DEE / `dee-win` 运行时与官方模板
- 本地大型音频样本与 `testfiles/` 目录
- 本地导出的 raw XSD、实验缓存、临时日志与输出目录
- 只有本地研究价值、没有公开分发必要的协作文档或草稿

当前仓库策略：
- 默认测试尽量用仓库内 fixture 或运行时生成的 WAV/stems
- 少量手动 runtime 覆盖仍依赖本地 Atmos 样本，例如 `testfiles/testADM.wav`
- crate 打包会显式排除内部协作文档和本地 temp/log 目录

## 贡献时需要注意

- 不要提交专有二进制、Dolby 文档、私有样本、临时日志或本地路径
- 如果改动影响模板行为，除了代码外还要同步更新 examples、parameter matrix、coverage matrix 和知识/陷阱 fixture
- 如果新增真实 runtime 结论，优先补 `#[ignore]` runtime 测试或覆盖文档，而不是只留在聊天记录里
- 如果改动触及 C ABI v1 或 UniFFI v1，对外兼容性需要显式评估

## 进一步阅读

- [贡献指南](../CONTRIBUTING.md)
- [开发者文档](developer-guide.zh.md)
- [覆盖与实验](coverage-and-experiments.zh.md)
- [FFI 桥接](ffi.zh.md)
