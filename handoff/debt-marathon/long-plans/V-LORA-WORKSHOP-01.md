# V-LORA-WORKSHOP-01（manual partial）

runner=skip · 自动债务马拉松仍不得擅自扩面。2026-07-24 用户显式立项后，`slot.lora.apply` 的 directory LLM 选择与 Stable completion 消费已人工交付。Stage0：skip。

## 已冻结的产品与架构决策

- OCLive 不复制 LLaMA-Factory、Unsloth 等上游 WebUI，也不在内核重写 LoRA/QLoRA 训练算法。
- 后续「角色炼丹炉」是独立创作者产品层：通过 `TrainingProvider` 封装外部训练引擎；首个 provider 优先 LLaMA-Factory CLI，可选 Unsloth 加速，后续可替换为 Axolotl 或远端 GPU。
- OCLive 拥有角色领域能力：角色包到训练/评测语料的转换、角色训练预设、任务状态、Base/LoRA/LoRA+专家路由对照评测、`.ocadapter` 打包。
- `expert_routing` 只负责何时选择哪个已安装 adapter；`.ocadapter` 导入器负责兼容性与完整性；directory 推理插件负责权重加载、显存和热切换；内核只消费统一 LLM 文本/流式文本。
- 在 T0 契约冻结前不向内核加入无消费者的训练抽象，避免为未来猜测接口。

## 当前已经准备好的发挥空间

| 能力链 | 状态 |
|--------|------|
| `slot.lora.apply` → 会话选择 | Done |
| 仅允许角色预声明、授权的 Experimental directory LLM | Done |
| LoRA 结果回流 Stable completion | Done |
| `llm.generate_stream` NDJSON 与 manifest 能力探测 | Done |
| 首 token 前回退、部分流失败不重复回复 | Done |
| llama.cpp/OpenAI-compatible adapter 模型别名示例 | Done |
| 确定性整段、流式、失败集成测试 | Done |
| llama.cpp GGUF / `.ocadapter` v1、导入管理、启用回滚 | Done（本地 performance；V-LORA-PACK-03 Partial） |
| 完整兼容指纹、签名、多 adapter 与专家绑定 | Deferred（V-LORA-PACK-03） |
| Hugging Face/PEFT 导入转换 | Deferred（独立插件 V-LORA-PEFT-04） |
| 角色炼丹炉与 `TrainingProvider` | Deferred（V-LORA-FORGE-02） |
| 真实模型 Base/LoRA/LoRA+专家路由评测矩阵 | OPEN（T3） |

## 后续解冻顺序

1. T0（已完成本地 v1）：`.ocadapter` 基础契约、GGUF/SHA/architecture 校验、导入/安装/回滚与管理 UI。
2. T0.5：补真实模型远端 CI smoke、完整兼容指纹与包签名。
3. T1：实现角色/专家路由绑定 UI；PEFT 仅由独立转换插件产出 v1 包。
4. T2：实现独立角色炼丹炉，以 `TrainingProvider` 接入成熟开源训练引擎。
5. T3：建立角色一致性、重复、Prompt 泄漏和基础能力退化的可复现对照评测。
