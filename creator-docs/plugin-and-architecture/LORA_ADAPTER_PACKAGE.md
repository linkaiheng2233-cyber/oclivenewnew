# llama.cpp LoRA GGUF 与 `.ocadapter` 契约

**状态**：v1 已实现（本地 performance 运行时）
**中文 SSOT**；[English mirror](../../creator-docs-en/plugin-and-architecture/LORA_ADAPTER_PACKAGE.md)

## 1. 边界

当前主程序只加载已经转换为 **llama.cpp LoRA GGUF** 的适配器。它不读取或转换 Hugging Face/PEFT 的 `adapter_config.json`、`.safetensors`、tokenizer 或训练框架私有状态。

- 主程序负责：本地导入、完整性校验、受管存储、成人内容确认、启用回滚和 `llama-server --lora`。
- `llama-server` 负责：权重加载、张量与基础模型的最终兼容检查、推理。
- 后续独立 PEFT 插件负责：下载/导入 PEFT 包、选择转换工具、生成符合本文的 GGUF 或 `.ocadapter`。该插件不得把 Python/Transformers 运行时耦合进稳定内核。
- 现有 `expert_routing` 仍负责选择预声明的 directory LLM；本地模型设置中的单适配器选择不改写专家路由语义。

## 2. 原始 GGUF 导入

用户可在“本地模型设置 → llama.cpp LoRA 适配器”选择 `.gguf`。导入器必须确认：

1. GGUF 版本为 2 或 3；
2. `general.type` 为 `adapter`；
3. `adapter.type` 为 `lora`；
4. 文件不超过 16 GiB；
5. 导入过程流式计算 SHA-256。

原始文件导入后使用 `local.lora.<sha256 前 16 位>` 作为确定性 ID，版本为 `0.0.0-local`。

## 3. `.ocadapter` v1

`.ocadapter` 是 ZIP 容器，根目录必须恰有一个 `adapter.json`。清单引用恰好一个包内 GGUF：

```json
{
  "schemaVersion": 1,
  "id": "com.example.mumu-style",
  "name": "Mumu Style",
  "version": "1.0.0",
  "format": "llama.cpp-lora-gguf",
  "adapterFile": "weights/adapter.gguf",
  "adapterSha256": "64-character-lowercase-sha256",
  "baseModel": "optional human-readable base model",
  "architecture": "llama",
  "contentRating": "general",
  "description": "optional",
  "license": "optional SPDX id or short label",
  "source": "optional source URL"
}
```

`baseModel` 只是人类可读的训练来源或导入上下文，用于兼容性判断与追踪；它不建立适配器与某个本地基座文件的固定绑定。实际启用仍以 GGUF architecture、完整性校验和 `llama-server` 加载结果为准。

字段规则：

| 字段 | 规则 |
|------|------|
| `schemaVersion` | 必须为 `1` |
| `id` | 1–96 字节；仅 ASCII 字母、数字、`.`、`_`、`-`；不得以 `.` 开头 |
| `name` | 1–160 个字符 |
| `version` | 1–64 个字符；推荐 SemVer |
| `format` | 必须为 `llama.cpp-lora-gguf` |
| `adapterFile` | 相对 `.gguf` 路径；禁止绝对路径、`..` 和重复条目 |
| `adapterSha256` | 64 位十六进制 SHA-256 |
| `contentRating` | `general` 或 `adult`；缺省为 `general` |
| `architecture` | 可选；若同时存在 GGUF 元数据，二者必须一致 |

`installedAt` 是安装时由 OCLive 写入的托管字段，发布包可省略。

## 4. 托管布局与替换

导入结果固定放在：

```text
<canonical models>/
└── adapters/
    └── <adapter-id>/
        ├── adapter.json
        └── adapter.gguf
```

导入先写入同一文件系统下的 `.import-<uuid>` 暂存目录，验证完成后再原子改名。同 ID 默认拒绝覆盖；只有用户显式选择“同 ID 时替换”才会先备份旧目录，新目录提交失败时恢复备份。

## 5. 启用、兼容与回滚

- 只有启用了托管 performance runtime 的发行版可以启用本地 LoRA。
- 必须先保存 GGUF 基础模型；适配器与基础模型的 `general.architecture` 必须一致（两边都有值时）。
- 启用前重新计算已安装适配器的 SHA-256，并再次验证 GGUF 元数据。
- `adult` 适配器每次从未启用状态切换为启用时都要求显式成人内容确认。
- OCLive 以 `llama-server -m <base.gguf> --lora <adapter.gguf>` 重启自己托管的进程。
- 外部占用同一端点的 `llama-server` 不能被宣称已加载适配器；用户须先停止外部进程。
- 新选择启动失败时，适配器 ID、路径、进程选择和环境变量全部恢复到上一个选择。
- 活跃适配器不可删除；必须先停用。

环境变量 `OCLIVE_LOCAL_LLM_LORA_PATH` 是数据库选择同步到托管进程的内部桥接，不是推荐的手工配置入口。

## 6. 独立基础模型与内容分级

放在 canonical `models/` 一级子目录中的独立基座，通过与 GGUF 同名的 `<file>.ocmodel.json` 进入模型列表。根目录中的松散 GGUF/BIN 继续按通用基座扫描；一级子目录中的文件必须有 sidecar，`adapters/` 与 `downloads/` 永不作为基础模型扫描。

```json
{
  "schemaVersion": 1,
  "kind": "oclive.local-base-model",
  "fileName": "example.Q4_K_M.gguf",
  "name": "Example full base",
  "contentRating": "adult",
  "description": "optional",
  "license": "Apache-2.0",
  "source": "optional source URL",
  "sha256": "optional 64-character SHA-256"
}
```

- `fileName` 必须与 sidecar 相邻的模型文件一致；`contentRating` 仅接受 `general` / `adult`；声明 `sha256` 时，切换到该基座前由后台线程重新计算并强制匹配。
- 选择新的 `adult` 完整基座时，前端提示与内核校验都要求显式成人内容确认。
- **独立基座与 LoRA 不建立固定绑定。** 用户可以自由选择任何通过架构、文件完整性与运行时加载验证的组合；来源信息用于追踪而非绑定。“第一套 / 第二套”等称呼只用于当前测试区分，不进入持久化契约或产品语义。
- **基础模型路径发生变化时，当前 LoRA 一律自动停用。** 用户必须在新基座保存成功后再显式启用目标适配器。这避免架构相同但训练来源不同的 LoRA 被静默带入另一套组合，也避免把“消融 LoRA”重复叠加到“消融主模型”。
- sidecar 声明的是模型来源与分级，不证明第三方权重的企业授权；交付方仍须核验许可证、数据权利与适用法律。

## 7. 尚未纳入 v1

- Hugging Face Hub 下载、PEFT/safetensors 转换与 Python 依赖；
- 多适配器叠加、动态 scale 与请求级切换；
- base revision、tokenizer、chat-template 的可验证指纹；
- 角色包/专家路由对本地全局适配器的绑定 UI；
- 签名、发布者信任链与远程仓库。

这些事项继续在 `V-LORA-PACK-03` 与独立的 `V-LORA-PEFT-04` 跟踪。
