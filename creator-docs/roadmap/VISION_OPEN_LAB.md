# 平台愿景：开放实验场（摘要）

oclive 的长期方向，是在 **本地优先、可替换子系统、角色包为唯一对接面** 的前提下，把运行时做成 **创作者与玩家都能安全实验** 的桌面底座：模块可切换（内置 / v2 / 远程侧车）、契约与 CI 守住兼容边界，文档与示例降低接入成本。架构上见 **[单核双态构建架构](../getting-started/OCLIVE_ARCHITECTURE_OVERVIEW.md)**（外核态可替换 + 可选宏核态 Monolith 焊接）。**运行时双核**（Stable / Experimental 编排 + 可降级试验场）见 **[RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md](../rfc/RFC_OCLIVE_DUAL_CORE_DUAL_MODE.md)**（Opt-in Beta，默认关，不阻塞当前发布）。

这与 [VISION_ROADMAP_MONTHLY.md](VISION_ROADMAP_MONTHLY.md) 中的分阶段路线一致；本页只强调 **「开放实验」** 这一主轴，便于 README 与对外说明引用。

**已对齐的落实点（随主仓演进）**：HTTP JSON-RPC Remote 宿主路径、`plugin_backends` 与扩展点文档、目录式插件与整壳桥接、开源与多平台 CI。详见仓库根目录 [README.md](../../README.md) 的「路线图状态」与 [DOCUMENTATION_INDEX.md](../getting-started/DOCUMENTATION_INDEX.md)。

**仍属路线图**：包内知识库深化、启动器/市场联动、社区站形态等，以对应 roadmap 文档为准。
