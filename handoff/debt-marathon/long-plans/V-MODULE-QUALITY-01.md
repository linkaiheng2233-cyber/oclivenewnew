# V-MODULE-QUALITY-01 · 可复现模块质量对比台

**强制门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md) · 性能数字不得替代行为质量证据

| 字段 | 值 |
|------|-----|
| **债 ID** | V-MODULE-QUALITY-01 |
| **台账** | `TECHNICAL_DEBT_INVENTORY.md` · V-MODULE-QUALITY-01 OPEN |
| **标题** | 固定角色、场景与 replay 的 memory / emotion / prompt / LLM 质量对比 harness |
| **尺寸** | L |
| **Minimal / Full** | 本书推进 Full；各 Stage 必须保持可独立回滚 |
| **Owner** | main-repo |
| **runner** | auto |
| **状态** | Ready · Stage 1 Locally verified；Stage 2 待实施 |
| **更新** | 2026-07-27 |

## AI + OCLive

- **必读门禁：** [`../AI_AND_PIPELINE_GATES.md`](../AI_AND_PIPELINE_GATES.md)
- **流水线：** dev-pipeline 七阶段 + oclive-dev-pipeline；尺寸 L 不跳纪律、文档、模块兼容与总审
- **相关 G：** G3、G4、G7、G7b、G8、G9、G11、G14、G17
- **场景路径：** `AI_READING_INDEX.md` §9 技术债；复用既有 OOCP、HTTP mock、replay 与模块兼容入口
- **证据纪律：** 固定输入、评分合同、原始观察与汇总报告必须可追溯；本地通过只记 Locally verified；没有目标提交远程 CI 不得写 Done

<!-- oclive-marathon-contract
{
  "version": 1,
  "id": "V-MODULE-QUALITY-01",
  "runner": "auto",
  "planStatus": "ready",
  "parentDebtDisposition": "keep-open",
  "currentStage": 2,
  "prerequisites": [],
  "stages": [
    {
      "id": 0,
      "title": "Inventory reusable quality-test parts and freeze boundaries",
      "files": ["read-only"],
      "actions": ["Inventory OOCP, HTTP mock, replay, CLI bench and report-schema responsibilities", "Separate deterministic behavior scoring from performance benchmarking and live-model sampling"],
      "checks": [
        {"command": "npm run check:debt-marathon", "why": "The debt plan and queue must be consistent before implementation"},
        {"command": "rg -n \"OOCP|MOCK_LLM|replay|bench|quality\" examples scripts kernel creator-docs", "why": "Existing test and benchmark paths must be reused instead of duplicated"}
      ],
      "outputs": ["Reusable-parts inventory", "Non-overlapping harness boundary", "Ordered rollback-safe stages"],
      "rollback": "No production writes; keep the debt OPEN when the boundary is ambiguous"
    },
    {
      "id": 1,
      "title": "Versioned fixture contract and deterministic offline scorer",
      "files": [
        "scripts/module-quality-harness.mjs",
        "examples/module-quality-harness/fixtures/suite.v1.json",
        "examples/module-quality-harness/fixtures/observations.reference.v1.json"
      ],
      "actions": ["Define fixed role, scene and replay cases with explicit memory, emotion, prompt and LLM expectations", "Validate observation shape and score every dimension deterministically without network or model access", "Emit stable JSON with per-case findings and aggregate scores", "Add self-tests proving malformed input and a behavioral regression fail closed"],
      "checks": [
        {"command": "node scripts/module-quality-harness.mjs --self-test", "why": "Schema rejection, deterministic scoring and regression detection are the Stage 1 contract"},
        {"command": "node scripts/module-quality-harness.mjs --suite examples/module-quality-harness/fixtures/suite.v1.json --observations examples/module-quality-harness/fixtures/observations.reference.v1.json --json", "why": "The checked-in reference fixture must produce a reproducible all-dimension report"},
        {"command": "node scripts/check-debt-marathon.mjs --id V-MODULE-QUALITY-01 --stage 1 --require-ready", "why": "The implementation must remain inside the current declared Stage"}
      ],
      "outputs": ["Versioned fixed-input suite", "Validated observation contract", "Deterministic per-dimension JSON report", "Positive and negative self-test evidence"],
      "rollback": "Remove the isolated script and fixtures; no kernel, distro or public runtime behavior is changed"
    },
    {
      "id": 2,
      "title": "Existing-kernel observation adapter",
      "files": [
        "scripts/module-quality-runner.mjs",
        "examples/module-quality-harness/fixtures/",
        "examples/oocp-test-suite/run.mjs",
        "kernel/crates/oclive_kernel_host/src/api/",
        "kernel/crates/oclive_kernel_host/src/infrastructure/"
      ],
      "actions": ["Drive fixed replay cases through the existing HTTP and mock-provider path", "Collect only contract-approved module observations and redact secrets", "Keep production response DTOs unchanged unless an explicit debug-only contract is justified"],
      "checks": [
        {"command": "node examples/oocp-test-suite/run.mjs --required-only", "why": "The adapter must preserve the existing HTTP black-box contract"},
        {"command": "cargo test -p oclive_kernel_host --lib -j 1", "why": "Any trace adapter touching host orchestration must preserve kernel behavior"},
        {"command": "cargo test --workspace --doc -j 1", "why": "Public Rust contract changes, if any, require workspace doctest"}
      ],
      "outputs": ["Repeatable observation capture", "Mock-provider baseline", "Secret-safe raw artifacts"],
      "rollback": "Keep Stage 1 offline scoring usable and remove the adapter; do not add a second chat or replay implementation"
    },
    {
      "id": 3,
      "title": "Cross-module comparison report and creator documentation",
      "files": [
        "scripts/module-quality-runner.mjs",
        "scripts/check-module-compat.mjs",
        "creator-docs/testing/MODULE_QUALITY_HARNESS.md",
        "creator-docs-en/testing/MODULE_QUALITY_HARNESS.md",
        "creator-docs/testing/TEST_OUTPUT_SCHEMA.md",
        "creator-docs-en/testing/TEST_OUTPUT_SCHEMA.md",
        "package.json"
      ],
      "actions": ["Compare multiple declared module sets against the same captured suite", "Report behavior quality separately from latency and resource measurements", "Document author workflow, limits and interpretation without claiming subjective scores are universal truth", "Add a module-compat or Dimension5 ratchet only after runtime integration exists"],
      "checks": [
        {"command": "node scripts/check-module-compat.mjs", "why": "Third-party module comparison must not weaken compatibility validation"},
        {"command": "node scripts/check-doc-mirror.mjs", "why": "The public testing workflow is a mirrored documentation contract"},
        {"command": "node scripts/dimension5-acceptance.mjs --ci", "why": "A new release-facing quality gate must compose with the L-level acceptance suite"}
      ],
      "outputs": ["Comparable module-set report", "Documented score semantics and limitations", "Release-compatible invocation"],
      "rollback": "Retain raw Stage 2 artifacts and the Stage 1 scorer; remove only the comparison orchestration and gate registration"
    },
    {
      "id": 4,
      "title": "L-level evidence and honest closure",
      "files": [
        "handoff/debt-marathon/waves/",
        "handoff/TECHNICAL_DEBT_INVENTORY.md",
        "handoff/debt-marathon/MARATHON_QUEUE.md",
        "handoff/debt-marathon/COVERAGE.md"
      ],
      "actions": ["Run applicable release, module-compat and documentation gates", "Record the exact target commit, fixtures, report and remote CI", "Close only when at least two module configurations are reproducibly compared across all four dimensions"],
      "checks": [
        {"command": "npm run check:ci-local", "why": "The Full harness crosses scripts, kernel adapters, module compatibility and public documentation"},
        {"command": "gh run view <RUN_ID> --json headSha,conclusion,url", "why": "L-level Done requires successful remote CI for the exact target commit"}
      ],
      "outputs": ["Locally verified or Done-eligible Wave", "Updated queue and technical-debt state", "Exact reproducibility evidence"],
      "rollback": "Keep V-MODULE-QUALITY-01 OPEN or Partial and name the missing module set, dimension or CI evidence"
    }
  ]
}
-->

## Stage 0 结论

- OOCP 已覆盖 HTTP 黑盒与 MOCK_LLM，可作为后续采集执行器；它不负责定义模块质量。
- `oclive bench` 的既有合同是性能、压力与等价性；不把行为质量字段硬塞进性能报告。
- 当前 replay 主要服务会话/记忆恢复；后续适配必须复用它或调用同一公开链路，不复制 `process_message` 编排。
- 第一阶段先冻结“输入、观察、评分、报告”合同，并用离线 fixture 证明确定性；这一步不需要启动模型，也不改生产行为。

## 目标

- 同一份固定角色、场景和多轮 replay 输入可被不同 memory、emotion、prompt、LLM 模块组合重复消费。
- 每次运行保留模块声明、用例输入身份、原始观察、逐项 finding 与汇总分数，能从报告追到依据。
- 质量与性能分栏：质量报告回答行为是否符合明确期望，现有 bench 继续回答延迟、吞吐与资源问题。
- 支持第三方模块接入，但评分器对模块来源无特权，且失败时明确指出合同或行为差异。

## 非目标

- 不用一个总分宣称模型或角色“客观更好”，不把固定样例泛化为所有对话质量。
- 不在 Stage 1 修改聊天、记忆、情绪、prompt 或 LLM 生产路径。
- 不新增另一套角色包解析、场景解析、会话编排或模型路由。
- 不把 MOCK_LLM 通过、低延迟或人工观感冒充四类模块均已完成质量验证。
- 不在缺少远程 CI 与至少两套真实模块配置对比时关闭父债。

## 停止条件

- 评分规则无法由 fixture 明确表达，需要主观人工判断却没有独立标注流程。
- 采集必须泄露完整系统 prompt、密钥或隐私记忆才能工作。
- 后续适配要求复制内核 `process_message` 或改变普通用户响应合同。
- queue、plan、inventory、Git 或目标 CI 证据发生冲突。

## 下一跳

Stage 2：通过既有 HTTP / OOCP 与 mock-provider 链路采集同一套观察合同；不得复制聊天编排，也不得向普通响应泄露 prompt 或私密记忆。
