# dee-config-gen

中文说明 | [English README](README.en.md) | [JSON 输出](docs/json-output.zh.md) | [开发者文档](docs/developer-guide.zh.md) | [覆盖与实验](docs/coverage-and-experiments.zh.md)

`dee-config-gen` 是一个用于生成、校验并可选调用外部 DEE 运行的 Rust CLI。

现在也提供了正式的 Rust library API，推荐入口是：
- 文件输入：`read_job -> generate_config`
- 内存输入：`parse_job_str -> generate_config`
- 分阶段编排：`resolve_job -> render_config -> run_with_runner`

适合的使用场景：
- 你已经有 YAML/JSON 任务描述，希望生成 DEE XML/JSON 配置
- 你想在本地先做参数校验，再交给 `dee` 或自定义 runner 执行
- 你想稳定管理 Atmos、PCM DDP、TrueHD Atmos 输入和 TrueHD WAV 输入模板

## 快速能力总览

当前支持：
- 模板：`ac4_v1`、`atmos_ec3_v1`、`pcm_ddp_v1`、`thd_v1`、`thd_wav_v1`、`thd_wav_list_v1`、`thd_atmos_wav_v1`、`thd_atmos_wav_list_v1`
- 输入：YAML、JSON
- 命令：`validate`、`generate`、`run`
- 输出格式：默认 `xml`；`json` 当前支持 `atmos_ec3_v1`、`pcm_ddp_v1` 与全部 TrueHD 模板，`ac4_v1` 暂不支持
- AC-4 模式：`ac4`
- Atmos 模式：`streaming`、`bluray`
- PCM 模式：`dd`、`ddp`、`ddp71`、`bluray`
- TrueHD 模式：`mlp`

### 支持矩阵速览

| 模板 | XML 输出 | JSON 输出 | 真实 DEE runtime |
| --- | --- | --- | --- |
| `ac4_v1` | 支持 | 不支持 | 保守建模 |
| `atmos_ec3_v1` | 支持 | 支持 | 已验证 |
| `pcm_ddp_v1` | 支持 | 支持 | 已验证 |
| `thd_v1` | 支持 | 支持 | 已验证 |
| `thd_wav_v1` | 支持 | 支持 | 已验证 |
| `thd_wav_list_v1` | 支持 | 支持 | 已验证 |
| `thd_atmos_wav_v1` | 支持 | 支持 | 已验证 |
| `thd_atmos_wav_list_v1` | 支持 | 支持 | 已验证 |

更细的参数级覆盖和限制见：
- [`docs/coverage_matrix.full.yaml`](docs/coverage_matrix.full.yaml)
- [`docs/json-output.zh.md`](docs/json-output.zh.md)

## 5 分钟上手

构建：

```bash
cargo build
```

校验输入：

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
```

生成 XML：

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml -o job.xml
```

生成 JSON：

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml --format json -o job.json
```

生成并调用外部 runner：

```bash
cargo run -- run \
  -i examples/atmos_ec3_single.streaming.yaml \
  --runner-cmd "dee" \
  --keep-config
```

说明：
- `run` 不内置容器或 Wine 逻辑，只负责生成配置文件并调用外部命令
- `--runner-cmd` 优先级最高；其次是环境变量 `DEE_RUNNER_CMD`；最后回退到 `dee`
- `--format json` 时会自动向 runner 注入 `--json`

## 作为库使用

从文件读取并生成 XML：

```rust
use dee_config_gen::{GenerateOptions, generate_config, read_job};

let spec = read_job("examples/atmos_ec3_single.streaming.yaml".as_ref())?;
let generated = generate_config(spec, &GenerateOptions::default())?;
assert!(generated.rendered.starts_with("<?xml version=\"1.0\"?>"));
# Ok::<(), anyhow::Error>(())
```

从字符串解析并生成 JSON：

```rust
use dee_config_gen::{GenerateOptions, RenderFormat, generate_config, parse_job_str};

let spec = parse_job_str(r#"{
  "template_id": "atmos_ec3_v1",
  "profile": "standard",
  "job_mode": "single",
  "encode_mode": "streaming",
  "input": {"storage_path": "./input", "file_names": ["testADM.wav"]},
  "output": {"storage_path": "./output", "file_names": ["output.ec3"]},
  "misc": {"temp_dir": "./tmp"}
}"#)?;

let generated = generate_config(
    spec,
    &GenerateOptions {
        format: RenderFormat::Json,
        ..GenerateOptions::default()
    },
)?;
assert!(generated.rendered.contains("\"job_config\""));
# Ok::<(), anyhow::Error>(())
```

## 最小输入示例

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

更多可直接运行的样例见：[`examples/`](examples)

## 模板怎么选

### `ac4_v1`

适合：
- 先生成最小可表达的 AC-4 XML 结构
- 当前仅覆盖 `input/audio/ac4` 与 `output/ac4` 的单文件路径
- 这是保守模板 v1，暂不暴露 AC-4 filter 参数，也不支持 JSON

样例：
- [`examples/ac4_single.ac4.yaml`](examples/ac4_single.ac4.yaml)

### `atmos_ec3_v1`

适合：
- 生成 Atmos DDP XML
- 使用 `streaming` 或 `bluray` 两种 Atmos 路径
- 当前支持 DEE JSON 输出

样例：
- [`examples/atmos_ec3_single.streaming.yaml`](examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](examples/atmos_ec3_single.bluray.yaml)

### `pcm_ddp_v1`

适合：
- 使用 `pcm_to_ddp` 路径生成 DD / DDP XML
- 管理 `dd`、`ddp`、`ddp71`、`bluray` 四种 PCM 编码模式
- 当前也支持 DEE JSON 输出

样例：
- [`examples/pcm_ddp_single.dd.yaml`](examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](examples/pcm_ddp_single.bluray.yaml)

### `thd_v1`

适合：
- 使用官方 `encode_to_dthd` 路径生成 TrueHD MLP XML/JSON
- 输入是 `atmos_mezz`，输出是 `mlp`

样例：
- [`examples/thd_single.mlp.yaml`](examples/thd_single.mlp.yaml)

### `thd_wav_v1`

适合：
- 使用单文件 `wav -> mlp` 的 TrueHD 路线
- 输入是一个 WAV 文件，不是 stem 列表

样例：
- [`examples/thd_wav_single.mlp.yaml`](examples/thd_wav_single.mlp.yaml)

### `thd_wav_list_v1`

适合：
- 使用 ordered mono stems 的 `wav_list -> mlp` TrueHD 路线
- `input.file_names` 采用固定顺序槽位
- 当前正式支持 `stereo`、`5.1`、`7.1`，不支持 `mono` 和 `-` 占位

样例：
- [`examples/thd_wav_list_single.mlp.yaml`](examples/thd_wav_list_single.mlp.yaml)

### `thd_atmos_wav_v1`

适合：
- 使用 mixed-input 的 `atmos_mezz + wav -> mlp` TrueHD 路线
- 输入通过 `inputs.atmos_mezz` 和 `inputs.wav` 两组分别声明

样例：
- [`examples/thd_atmos_wav_single.mlp.yaml`](examples/thd_atmos_wav_single.mlp.yaml)

### `thd_atmos_wav_list_v1`

适合：
- 使用 mixed-input 的 `atmos_mezz + wav_list -> mlp` TrueHD 路线
- 输入通过 `inputs.atmos_mezz` 和 `inputs.wav_list` 两组分别声明

样例：
- [`examples/thd_atmos_wav_list_single.mlp.yaml`](examples/thd_atmos_wav_list_single.mlp.yaml)

## 常见注意事项

- `profile=music` 默认会锁定一组固定值；如果你确实要覆盖，使用 `--allow-fixed-override`
- 生成 XML 时，路径会被规范成 Windows 风格路径；默认盘符是 `Y:`，可用 `--win-drive` 调整
- `ac4_v1`、`atmos_ec3_v1`、`pcm_ddp_v1`、`thd_v1`、`thd_wav_v1`、`thd_wav_list_v1`、`thd_atmos_wav_v1`、`thd_atmos_wav_list_v1` 是八套独立模板，不要混用参数

## 文档导航

面向普通用户：
- 参数矩阵：[`docs/parameter_matrix.ac4_v1.yaml`](docs/parameter_matrix.ac4_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.pcm_ddp_v1.yaml`](docs/parameter_matrix.pcm_ddp_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_v1.yaml`](docs/parameter_matrix.thd_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_wav_v1.yaml`](docs/parameter_matrix.thd_wav_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_wav_list_v1.yaml`](docs/parameter_matrix.thd_wav_list_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_atmos_wav_v1.yaml`](docs/parameter_matrix.thd_atmos_wav_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_atmos_wav_list_v1.yaml`](docs/parameter_matrix.thd_atmos_wav_list_v1.yaml)

面向开发者与维护者：
- 开发者文档：[`docs/developer-guide.zh.md`](docs/developer-guide.zh.md)
- 覆盖与实验：[`docs/coverage-and-experiments.zh.md`](docs/coverage-and-experiments.zh.md)
- JSON 输出：[`docs/json-output.zh.md`](docs/json-output.zh.md)

英文文档：
- 用户首页：[`README.en.md`](README.en.md)
- Developer Guide: [`docs/developer-guide.en.md`](docs/developer-guide.en.md)
- Coverage & Experiments: [`docs/coverage-and-experiments.en.md`](docs/coverage-and-experiments.en.md)
- JSON Output: [`docs/json-output.en.md`](docs/json-output.en.md)

## License

项目本身遵循仓库内的许可证约定。

注意：运行时依赖的 Dolby DEE、`dee-win`、上游实验样本与第三方工具链，许可证和使用条件各自独立，请自行确认。
