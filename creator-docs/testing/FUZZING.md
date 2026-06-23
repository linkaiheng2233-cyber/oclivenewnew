# 模糊测试（AB5）

## 目标

对 **OOCP 形 JSON**、**manifest.json**、**settings.json** 等外部输入做随机变异，确保解析路径**不 panic**。

## 方式 A：`proptest`（默认，CI `fuzz` job）

```bash
cargo test -p oclive_validation --test proptest_fuzz_parsing
```

每个属性用例默认 **2048** 次变异（可在本地提高 `ProptestConfig::with_cases`）。

## 方式 B：`cargo-fuzz`（libFuzzer，需 nightly）

```bash
rustup toolchain install nightly
cargo install cargo-fuzz
cd fuzz
cargo fuzz list
cargo fuzz run fuzz_manifest_load -- -runs=100000
cargo fuzz run fuzz_settings_parse -- -runs=100000
cargo fuzz run fuzz_oocp_message -- -runs=100000
cargo fuzz run fuzz_blueprint_v2 -- -runs=100000
cargo fuzz run fuzz_oclive_validation -- -runs=100000
cargo fuzz run fuzz_function_call_parser -- -runs=100000
cargo fuzz run fuzz_role_pack_loader -- -runs=100000
```

### `fuzz_oclive_validation`

对随机 UTF-8 / JSON 依次调用 **`validate_blueprint_v2_json`**、**`validate_manifest_top_level_keys`**、**`validate_settings_top_level_keys`**，断言不 panic。

### `fuzz_function_call_parser`

对随机字符串调用 **`parse_from_llm_response`**（OpenAI `tool_calls` / `function_call` 解析），断言不 panic。

### `fuzz_role_pack_loader`

将随机字节写入临时文件并调用 **`peek_role_pack_manifest`**（ZIP / 损坏输入），断言不 panic。

```bash
cd fuzz
cargo fuzz run fuzz_blueprint_v2 -- -runs=100000
```

CI **`fuzz`** job 在 proptest 之后会尝试 **256 轮** libFuzzer 冒烟（`continue-on-error`）。

## 复现崩溃

1. libFuzzer 会在 `kernel/fuzz/artifacts/<target>/` 留下最小输入。
2. 将字节写入文件后：`cargo test -p oclive_validation --test proptest_fuzz_parsing -- --exact <case>` 或单元测试夹具。

## 与 `oclive test` 的关系

无单独子命令；发版前建议跑 **proptest** 全量 + 可选 **cargo-fuzz** 过夜。

[English mirror](../../creator-docs-en/testing/FUZZING.md)
