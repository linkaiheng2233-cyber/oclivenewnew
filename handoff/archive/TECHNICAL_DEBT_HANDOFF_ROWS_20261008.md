# 技术债接手现状原行快照 · 2026-10-08

**状态**：历史原文；基线 `6f11a54fef02281f45aef4aeb2301b6b5226d10c` 的三条权威行。当前状态只读 [技术债台账](../TECHNICAL_DEBT_INVENTORY.md)，修正原因见 [DCL-61](../debt-marathon/DEBT_CHANGELOG.md#dcl-20261008-61--技术债接手现状的三处过期表述收束)。

下面每个 fenced block 的行文本逐字保留；原行相对链接按原 `handoff/TECHNICAL_DEBT_INVENTORY.md` 所在目录解释。保全不表示旧待验或未接通描述仍有效；换行统一为 LF，不改变原行文字。

## K-EMO-01

```text
| **K-EMO-01** | 英文词表扩充通道暂放（VADER MIT / AFINN Apache-2.0 参考），中文完成后处理 | P3 | 中文词表完成后扩充英文词表 | **Partial · 直接词补充本地已验**（2026-10-08：沿现有英文 seed 的权重、边界与否定规则追加十二个直接词，旧 104 条逐项及顺序保持；公开 Base / 七维两种消费的正例、否定和标识符负例通过，runtime lib 273/273。未导入外部词库、改变评分/主体规则或取得真实模型语义质量结论；本片目标 SHA 正式验收仍待，K-EMO-07 独立保留，范围与原件见 [DCL-60](debt-marathon/DEBT_CHANGELOG.md#dcl-20261008-60--英文直接情绪词沿既有规则补充)） |
```

## D-CLI-BLUEPRINT-05

```text
| **D-CLI-BLUEPRINT-05** | CLI 的 `init` / `pack create` 仍生成参考宿主组合格式，不能用其蓝图或关系字段定义跨发行版最小角色。共享逻辑定义、本地快照、可选 PNG、CLI 显式文件校验及独立 Prompt Base 已实现；独立 Host 示例现在可从本地和纯内存两种来源装配。参考 Rust Host 已新增开发者准备句柄与基础文本入口；旧 `Role` 缓存、`PromptInput` 及丰富生命周期仍要求完整产品角色，HTTP/Tauri/UI 最小路径尚未接通 | **P1** | 按 [ROLE_PACK_BOUNDARY.md](ROLE_PACK_BOUNDARY.md) §0.1–0.9 保留跨发行版逻辑契约，由各发行版自行适配磁盘格式、资产与技术身份；不再以统一磁盘封装/生成器作为 Kernel 合规前置。保留旧参考 Host 接口、增量寻找最小角色的内部接入点，不能为通过旧 `Role` 校验注入关系、展示名、人格或蓝图默认值；缺失/损坏/能力不支持要分开返回。可选本地生成样例必须明确不是通用包标准；CLI 默认 `init` 不切换到完整 v4，也不复制 parser 或回合编排。真实发行版适配与参考 Host 接入分别验收，不互相冒充 | **Partial · 基础主流程与六槽共享消费已接**。同一最小定义的可替换六槽消费、独立 Host 装配及参考 Host 真实资源绑定见 [ROLE_PACK_BOUNDARY §0.14–0.19](ROLE_PACK_BOUNDARY.md#014-六槽可替换的最小角色消费)；当前基础聊天消费本绑定会话 Memory、当前原文 Emotion、Prompt 与正文，Event 按需、Agent 仅明确委托。已保留可返回完整角色上下文；具体任务操作、工具范围与结果消费仍待发行版选择，不代表普通聊天自动运行六阶段、真实模型质量或所有发行版已验。逐次实施与验收见 [DCL-36–42](debt-marathon/DEBT_CHANGELOG.md#dcl-20261004-36--同一最小角色的六槽可替换消费)；本轮凝缩前的完整原行已[逐字保全](archive/TECHNICAL_DEBT_MINIMAL_ROLE_STATUS_20261005.md)，旧快照不改。父债不转 Done；旧丰富生命周期不迁移，也不以假状态补齐扩展。 |
```

## K-SUPPLY-10

```text
| **K-SUPPLY-10** | 外部 Actions 的不可变来源与升级维护；仓库 workflow 和 CLI 生成模板须分别核对 | P2 | 所有直接外部 action 固定原上游完整 commit SHA，保留版本/执行语义并验证目标 SHA CI；升级维护必须覆盖实际引用，不能把 bot 配置存在当 PR 已验证 | **Partial · pinned references remote verified / maintenance in progress**（2026-10-01：先以 test-only 合同扫描 workflow 与两类 CLI 模板，拒绝单边漂移；再把真实 bot PR #184 的 14 处 workflow Rust Action 更新与五处 CLI 生产模板配对，定向正负合同 5/5、完整本地链 exit 0。本次上游增加 `--force-non-host`，属已声明 stable 路径的兼容更新，不写成完全等价；提交 `053ebdadf990c9c278bfc601c1e9b1640258c962` 的正式 CI `36823868603` 已 17/17 success，未来模板升级仍需人工同步，父债不转 Done。来源、边界与历史固定引用远端 17/17 证据见 [Actions Wave](debt-marathon/waves/WAVE-20260929-ACTIONS-PINS.md)） 2026-10-07 CLI 五个 Rust job 复用一个私有完整步骤，原两类生成 YAML 字节一致，原同步正负合同与真实 init/check 保持；本片仅本地预备，目标完整/正式出口待验，见 [DCL-58](debt-marathon/DEBT_CHANGELOG.md#dcl-20261007-58--cli-rust-action-的五处重复收束为一个步骤)。 |
```
