# 校企合作全量镜像仓（School / Enterprise Fork）

> **状态**：待建仓（等校企仓 URL）  
> **范围**：**完整镜像** —— 新仓 = 当前 `oclivenewnew` 全量（`kernel/` + 全部 `distros/` + 根契约文档），作为校企二次开发基线；**本仓继续做官方上游**。  
> **与附录 A 区别**：附录 A 的 `oclive-chat-pro` 是官方产品拆仓决策门；本文件是**校企定制下游**，二者独立。

---

## 1. 推荐仓名

| 优先级 | GitHub 仓名 | 说明 |
|--------|-------------|------|
| **首选** | **`oclive-school`** | 短、好记；对外可说「校企实验室基线」 |
| 备选 | `oclive-campus` | 偏校园叙事 |
| 备选 | `oclive-partner-lab` | 偏企业联合实验室 |

若 org 为 `linkaiheng2233-cyber`，推荐完整路径：

`https://github.com/linkaiheng2233-cyber/oclive-school`

**下游 URL**：https://github.com/linkaiheng2233-cyber/oclive-school

---

## 2. 角色分工

```text
oclivenewnew (upstream · 官方上游)
    │  tag / merge 同步
    ▼
oclive-school (downstream · 校企二次开发)
    ├── 校企定制发行版 / 角色包 / 插件
    ├── 可选私有 CI / 内网模型端点
    └── 通用改进 → 提 PR 回 upstream（见 §5）
```

| 仓库 | 职责 | 谁 merge |
|------|------|----------|
| **oclivenewnew** | 官方内核、Chat Pro、AI Theater、契约文档、OOCP 门禁 | 内核作者 |
| **oclive-school** | 校企场景定制、教学实验、本地化交付 | 校企维护者（学生/导师） |

---

## 3. 首次建仓（维护者 · 你）

### 3.1 从上游打基线 tag（建议）

在 **oclivenewnew** 合入 `kernel/` + `distros/` 重组且 `dimension5 --ci` 绿之后：

```bash
git tag -a school-baseline-v0.4.0 -m "校企镜像基线：kernel+distros 重组完成"
git push origin school-baseline-v0.4.0
```

### 3.2 创建空仓并推送全量镜像

```bash
# 在 oclivenewnew 根目录
git clone --bare . ../oclive-school-bare.git
cd ../oclive-school-bare.git
git push --mirror https://github.com/linkaiheng2233-cyber/oclive-school.git
```

或普通 clone + 改 remote：

```bash
git clone https://github.com/linkaiheng2233-cyber/oclivenewnew.git oclive-school
cd oclive-school
git remote rename origin upstream
git remote add origin https://github.com/linkaiheng2233-cyber/oclive-school.git
git push -u origin main
```

### 3.3 在校企仓运行初始化脚本

```bash
cd oclive-school
node scripts/init-school-fork.mjs \
  --upstream https://github.com/linkaiheng2233-cyber/oclivenewnew \
  --baseline-tag school-baseline-v0.4.0
```

脚本会：校验 `upstream` remote、写入 `.oclive/school-fork.json`、打印 README 首屏横幅模板。

### 3.4 README 首屏横幅（校企仓必改）

在校企仓 `README.md` **标题下方**插入（勿删 Apache-2.0 / 上游链接）：

```markdown
> **校企二次开发基线** · 上游：[oclivenewnew](https://github.com/linkaiheng2233-cyber/oclivenewnew) · 基线 tag：`school-baseline-v0.4.0`  
> 本仓库为**下游镜像**，官方发行版与内核契约以 upstream 为准；定制改动见 `handoff/distros/SCHOOL_CUSTOMIZATIONS.md`（校企维护）。
```

可选：校企仓 `package.json` 的 `name` 改为 `@oclive/school` 或保留 `oclivenewnew` 以免破坏脚本——**推荐保留根 package 名**，仅在 README 标明校企身份。

---

## 4. 校企团队日常开发

### 4.1 环境（与上游相同）

见根目录 `CONTRIBUTING.md` · `human-docs/README.md`。物理布局：

- 内核：`kernel/crates/`
- Chat Pro：`distros/chat-pro/`
- AI Theater：`distros/theater/`
- Tauri：`distros/desktop-tauri/`

### 4.2 定制落点（推荐）

| 定制类型 | 建议目录 | 避免 |
|----------|----------|------|
| 校企专用角色包 | `distros/chat-pro/roles/<school>/` | 勿改 `kernel/` 编排除非计划 upstream |
| 校企 UI / 壳 | `distros/chat-pro/src/` 或新 `distros/school/`（可选） | 勿复制整棵 `kernel/` |
| 发行版 profile | `distros/desktop-tauri/resources/distro-profiles/` | 勿改 OOCP 契约字段名 |
| 实验文档 | `handoff/distros/SCHOOL_CUSTOMIZATIONS.md` | 勿改 `creator-docs/` 契约而不 PR 上游 |

### 4.3 门禁（每 PR 建议跑）

```bash
npm ci
node scripts/dimension5-acceptance.mjs --ci
cargo test -p oclive_kernel_host --lib
```

**供应链**：见 [`creator-docs/security/SUPPLY_CHAIN.md`](../../creator-docs/security/SUPPLY_CHAIN.md) — 要求组员 `npm ci && cargo build` 从源码跑通，勿只下未知二进制。

---

## 5. 与上游同步

### 5.1 拉取官方更新

```bash
git fetch upstream
git merge upstream/main
# 或按学期节奏：git merge upstream/main -X ours  # 仅当冲突策略已与导师约定
```

有 tag 时：

```bash
git fetch upstream tag school-baseline-v0.4.1
git merge school-baseline-v0.4.1
```

### 5.2 向上游贡献（通用改进）

1. 在 **oclivenewnew** 开 branch，**不要**把校企私有配置（API key、内网 URL、未脱敏数据）带进 PR。
2. 遵循 `CONTRIBUTING.md` · `handoff/BREAKING_CHANGE_PROCESS.md`。
3. 校企-only 功能：**不要**直接 PR 进 main；放校企仓或独立插件包。

### 5.3 许可证

- 上游与校企基线均为 **Apache-2.0**（见 `LICENSE`、`creator-docs/LICENSE_POLICY.md`）。
- 校企可闭源销售**自行编写的**插件/角色包；**fork 自上游的代码**仍受 Apache-2.0 约束（保留 NOTICE、声明修改）。

---

## 6. 禁止事项（避免双仓腐烂）

1. **禁止**校企仓长期漂移而不 merge upstream（每学期至少一次同步）。
2. **禁止**在校企仓改 `creator-docs/` 契约却不 PR 上游（会导致学生读两份矛盾 SSOT）。
3. **禁止**把校企仓当成新的「官方主仓」对外发布（品牌与 release 仍以 oclivenewnew 为准）。
4. **禁止**删除 `kernel/` 或 `scripts/dimension5-acceptance.mjs` 来「简化作业」——用 profile / 环境变量跳过 LLM 探测即可。

---

## 7. 决策记录模板

```markdown
## School fork decision
- Date:
- Downstream URL: https://github.com/.../oclive-school
- Baseline tag: school-baseline-v0.4.0
- Upstream at baseline: <commit sha>
- School maintainers:
- Sync cadence: monthly / per-semester
```

---

## 8. 相关文档

- 目录重组 RFC：[ARCHITECTURE_DECOUPLING_RFC.md](./ARCHITECTURE_DECOUPLING_RFC.md) · 附录 B
- 官方姊妹仓决策门（非校企）：RFC 附录 A `oclive-chat-pro`
- 许可证：[creator-docs/LICENSE_POLICY.md](../../creator-docs/LICENSE_POLICY.md)
