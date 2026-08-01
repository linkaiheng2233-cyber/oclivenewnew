# 模糊测试（AB5）

## 目标

对 **OOCP 形 JSON**、**manifest.json**、**settings.json** 等外部输入做随机变异，确保解析路径**不 panic**。

## 方式 A：`proptest`（默认，Nightly `fuzz` job）

```bash
cargo test -p oclive_validation --test proptest_fuzz_parsing
```

每个属性用例默认 **2048** 次变异（可在本地提高 `ProptestConfig::with_cases`）。

## 方式 B：`cargo-fuzz`（libFuzzer，需 nightly）

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cargo fuzz list --fuzz-dir kernel/fuzz
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_manifest_load -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_settings_parse -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_oocp_message -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_blueprint_v2 -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_oclive_validation -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_function_call_parser -- -runs=100000
cargo fuzz run --fuzz-dir kernel/fuzz fuzz_role_pack_loader -- -runs=100000
```

### `fuzz_oclive_validation`

对随机 UTF-8 / JSON 依次调用 **`validate_blueprint_v2_json`**、**`validate_manifest_top_level_keys`**、**`validate_settings_top_level_keys`**，断言不 panic。

### `fuzz_function_call_parser`

对随机字符串调用 **`parse_from_llm_response`**（OpenAI `tool_calls` / `function_call` 解析），断言不 panic。

### `fuzz_role_pack_loader`

将随机字节写入临时文件并调用 **`peek_role_pack_manifest`**（ZIP / 损坏输入），断言不 panic。

以上命令统一从仓库根目录运行；显式指定 `--fuzz-dir kernel/fuzz`，避免 Cargo 工作区将 fuzz 清单误解析为根目录下的 `fuzz/Cargo.toml`。

独立 **`.github/workflows/nightly-advisory.yml`** 的 **`fuzz`** job 在 proptest 之后运行 **256 轮** libFuzzer 冒烟。失败会使 Nightly 自身变红并上传最小化 failure artifact，但不阻塞 main 合并门禁。

## 复现崩溃

1. libFuzzer 会在 `kernel/fuzz/artifacts/<target>/` 留下最小输入。
2. 将字节写入文件后：`cargo test -p oclive_validation --test proptest_fuzz_parsing -- --exact <case>` 或单元测试夹具。

## 与 `oclive test` 的关系

无单独子命令；发版前建议跑 **proptest** 全量 + 可选 **cargo-fuzz** 过夜。

[English mirror](../../creator-docs-en/testing/FUZZING.md)
