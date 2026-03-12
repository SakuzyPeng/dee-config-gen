# dee-config-gen

中文说明 | [English README](README.en.md) | [开发者文档](docs/developer-guide.zh.md) | [覆盖与实验](docs/coverage-and-experiments.zh.md)

`dee-config-gen` 是一个用于生成、校验并可选调用外部 DEE 运行的 Rust CLI。

适合的使用场景：
- 你已经有 YAML/JSON 任务描述，希望生成 DEE XML
- 你想在本地先做参数校验，再交给 `dee` 或自定义 runner 执行
- 你想稳定管理 Atmos、PCM DDP 和 TrueHD 三类模板

## 快速能力总览

当前支持：
- 模板：`atmos_ec3_v1`、`pcm_ddp_v1`、`thd_v1`
- 输入：YAML、JSON
- 命令：`validate`、`generate`、`run`
- Atmos 模式：`streaming`、`bluray`
- PCM 模式：`dd`、`ddp`、`ddp71`、`bluray`
- TrueHD 模式：`mlp`

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

生成并调用外部 runner：

```bash
cargo run -- run \
  -i examples/atmos_ec3_single.streaming.yaml \
  --runner-cmd "dee" \
  --keep-xml
```

说明：
- `run` 不内置容器或 Wine 逻辑，只负责生成 XML 并调用外部命令
- `--runner-cmd` 优先级最高；其次是环境变量 `DEE_RUNNER_CMD`；最后回退到 `dee`

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

### `atmos_ec3_v1`

适合：
- 生成 Atmos DDP XML
- 使用 `streaming` 或 `bluray` 两种 Atmos 路径

样例：
- [`examples/atmos_ec3_single.streaming.yaml`](examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](examples/atmos_ec3_single.bluray.yaml)

### `pcm_ddp_v1`

适合：
- 使用 `pcm_to_ddp` 路径生成 DD / DDP XML
- 管理 `dd`、`ddp`、`ddp71`、`bluray` 四种 PCM 编码模式

样例：
- [`examples/pcm_ddp_single.dd.yaml`](examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](examples/pcm_ddp_single.bluray.yaml)

### `thd_v1`

适合：
- 使用官方 `encode_to_dthd` 路径生成 TrueHD MLP XML
- 输入是 `atmos_mezz`，输出是 `mlp`

样例：
- [`examples/thd_single.mlp.yaml`](examples/thd_single.mlp.yaml)

## 常见注意事项

- `profile=music` 默认会锁定一组固定值；如果你确实要覆盖，使用 `--allow-fixed-override`
- 生成 XML 时，路径会被规范成 Windows 风格路径；默认盘符是 `Y:`，可用 `--win-drive` 调整
- `atmos_ec3_v1`、`pcm_ddp_v1`、`thd_v1` 是三套独立模板，不要混用参数

## 文档导航

面向普通用户：
- 参数矩阵：[`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.pcm_ddp_v1.yaml`](docs/parameter_matrix.pcm_ddp_v1.yaml)
- 参数矩阵：[`docs/parameter_matrix.thd_v1.yaml`](docs/parameter_matrix.thd_v1.yaml)

面向开发者与维护者：
- 开发者文档：[`docs/developer-guide.zh.md`](docs/developer-guide.zh.md)
- 覆盖与实验：[`docs/coverage-and-experiments.zh.md`](docs/coverage-and-experiments.zh.md)

英文文档：
- 用户首页：[`README.en.md`](README.en.md)
- Developer Guide: [`docs/developer-guide.en.md`](docs/developer-guide.en.md)
- Coverage & Experiments: [`docs/coverage-and-experiments.en.md`](docs/coverage-and-experiments.en.md)

## License

项目本身遵循仓库内的许可证约定。

注意：运行时依赖的 Dolby DEE、`dee-win`、上游实验样本与第三方工具链，许可证和使用条件各自独立，请自行确认。
