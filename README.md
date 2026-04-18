# dee-config-gen

[![CI](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/SakuzyPeng/dee-config-gen)](LICENSE)

中文说明 | [English](README.en.md)

将 YAML/JSON 任务描述转换为 [Dolby Encoding Engine](https://professional.dolby.com/) (DEE) 的 XML/JSON 配置，并可选调用外部 runner 执行编码。告别手改 XML 模板——用结构化参数描述你想要的编码任务，让工具处理路径转换、参数校验和默认值注入。

## 安装

```bash
# 从源码构建
cargo build --release

# 或直接从仓库安装
cargo install --path .
```

## 快速上手

```bash
# 校验任务参数
dee-config-gen validate -i job.yaml

# 生成 DEE XML 配置
dee-config-gen generate -i job.yaml -o job.xml

# 生成 JSON 配置
dee-config-gen generate -i job.yaml --format json -o job.json

# 生成配置并调用 runner 执行编码
dee-config-gen run -i job.yaml --runner-cmd "dee" --keep-config
```

最小输入示例（`job.yaml`）：

```yaml
template_id: atmos_ec3_v1
profile: standard
job_mode: single
encode_mode: streaming
input:
  storage_path: ./input
  file_names:
    - testADM.wav
output:
  storage_path: ./output
  file_names:
    - output.ec3
misc:
  temp_dir: ./tmp
```

更多样例见 [`examples/`](examples)。

## 支持矩阵

| 模板 | 用途 | 编码模式 |
|---|---|---|
| `ac4_ims_atmos_v1` | AC-4 IMS（Atmos 沉浸式输入） | `ac4` |
| `ac4_ims_pcm_v1` | AC-4 IMS（PCM wav/wav_list 输入） | `ac4` |
| `atmos_ec3_v1` | Atmos DDP | `streaming` `bluray` |
| `pcm_ddp_v1` | PCM → DD/DDP | `dd` `ddp` `ddp71` `bluray` |
| `thd_v1` | TrueHD（Atmos mezz 输入） | `mlp` |
| `thd_wav_v1` | TrueHD（单 WAV 输入） | `mlp` |
| `thd_wav_list_v1` | TrueHD（mono stems 输入） | `mlp` |
| `thd_atmos_wav_v1` | TrueHD（Atmos + WAV 混合输入） | `mlp` |
| `thd_atmos_wav_list_v1` | TrueHD（Atmos + stems 混合输入） | `mlp` |

全部 9 个模板均支持 XML 和 JSON 输出，且已通过真实 DEE runtime 验证。

不确定该用哪个模板？见[模板选择指南](docs/template-guide.zh.md)。

## 集成方式

除 CLI 外，还提供：

- **Rust Library API** — `read_job` / `parse_job_str` → `generate_config`，详见 [API 文档](https://docs.rs/dee-config-gen)
- **C ABI (FFI v1)** — `dcg_validate_job` / `dcg_generate_config`，稳定错误码协议，详见 [FFI 文档](docs/ffi.zh.md)
- **UniFFI Python bundle** — 跨平台可下载，详见 [`ffi/example_python_uniffi`](ffi/example_python_uniffi)

## 注意事项

- `profile=music` 会锁定一组固定参数值；需要覆盖时使用 `--allow-fixed-override`
- 生成的 XML 路径自动转换为 Windows 格式（默认 `Y:` 盘符，`--win-drive` 可改）
- `--runner-cmd` 优先级最高；其次是环境变量 `DEE_RUNNER_CMD`；最后回退到 `dee`
- `--format json` 时 runner 自动注入 `--json` flag

## 文档

| 类别 | 中文 | English |
|---|---|---|
| 模板选择 | [template-guide.zh.md](docs/template-guide.zh.md) | [template-guide.en.md](docs/template-guide.en.md) |
| 开发者指南 | [developer-guide.zh.md](docs/developer-guide.zh.md) | [developer-guide.en.md](docs/developer-guide.en.md) |
| FFI 桥接 | [ffi.zh.md](docs/ffi.zh.md) | [ffi.en.md](docs/ffi.en.md) |
| JSON 输出 | [json-output.zh.md](docs/json-output.zh.md) | [json-output.en.md](docs/json-output.en.md) |
| 覆盖与实验 | [coverage-and-experiments.zh.md](docs/coverage-and-experiments.zh.md) | [coverage-and-experiments.en.md](docs/coverage-and-experiments.en.md) |
| 参数矩阵 | [`docs/parameter_matrix.*.yaml`](docs/) | |
| 覆盖矩阵 | [`docs/coverage_matrix.full.yaml`](docs/coverage_matrix.full.yaml) | |

## License

[MIT](LICENSE)

运行时依赖的 Dolby DEE、`dee-win`、上游实验样本与第三方工具链，许可证和使用条件各自独立，请自行确认。
