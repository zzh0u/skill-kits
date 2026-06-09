# Skill-kits

Skill-kits 是一个本地优先的 AI Agent Skills 管理器。它用一个 Rust 单二进制文件，把不同智能体实际读取的 Skill 目录扫描出来，并清楚展示每个 Skill 当前是启用、禁用、无效还是只读。

它的核心价值很直接：

- **单二进制文件**：CLI 和原生 GUI 都在同一个 `skill-kits` 可执行文件里。
- **零运行时依赖**：不需要 Electron、Node.js、Python、WebView 或后台服务。
- **多智能体管理**：统一查看 Codex、Claude Code、Gemini CLI 和自定义 Agent 的 Skill 状态。
- **不绑定任何 LLM**：Skill-kits 管理的是 Agent 可读取的本地 Skill 文件系统，因此不依赖某个模型、API 或供应商。

```text
Agent reads it there, Skill-kits manages it there.
```

只要智能体从本地目录读取 Skills，Skill-kits 就可以围绕这些目录提供扫描、检查、启用、禁用和项目级管理能力。

## 为什么需要它

现在的 AI 编程工作流通常不是只有一个智能体。开发者可能同时使用 Codex、Claude Code、Gemini CLI，也可能在不同项目里放置不同的本地 Skills。每个 Agent 的目录结构、启用规则和项目级约定都不完全一样。

没有统一工具时，常见问题是：

- 不知道某个 Agent 现在到底能看到哪些 Skills。
- 同名 Skill 在不同 Agent 或不同项目里有多份副本，容易误操作。
- 不清楚一个 Skill 是启用、禁用、只读、漂移还是无效状态。
- Codex 插件里的 Skills 和原生 Agent Space Skills 容易混在一起。
- 本地调试、项目迁移、团队共享 Skill 配置时缺少可重复的检查入口。

Skill-kits 把这些状态集中到一个本地工作台里。它不做云同步，不接管模型调用，也不把你的 Skill 文件搬进一个隐藏运行时。它只管理本机文件系统中 Agent 实际读取的位置。

## 主要好处

### 一个文件，带走整个工具

Skill-kits 是单个 Rust binary。构建完成后，你只需要分发或安装一个 `skill-kits` 文件：

```bash
skill-kits status
skill-kits list
skill-kits --gui
```

这意味着：

- 服务器、开发机、临时环境都更容易部署。
- 没有前端依赖安装、包管理器版本冲突或运行时启动链路。
- CLI 和 GUI 使用同一套 Rust core，结果一致。
- 卸载也干净，删除二进制即可；本地配置仍然留在明确的 `~/.skill-kits` 目录中。

### 零运行时

Skill-kits 不要求用户安装额外运行时：

- 不需要 Node.js。
- 不需要 Python。
- 不需要 Electron。
- 不需要浏览器 WebView。
- 不需要常驻 daemon。

这让它适合放在本地开发机、受限工作环境、离线环境，以及只希望保留最少工具链的团队机器上。

### 面向任何 LLM 工作流

Skill-kits 不直接绑定某个 LLM。它不关心你用的是哪家模型，也不需要读取模型 API key。

它关注的是更底层、更稳定的事实：Agent 从哪里读取 Skills。

因此它可以服务于多种 LLM 工作流：

- Codex 读取自己的 Skill 和插件能力。
- Claude Code 读取 `~/.claude/skills` 和项目级 `.claude/skills`。
- Gemini CLI 读取 `~/.gemini/skills` 和项目级 `.gemini/skills`。
- 自定义 Agent 可以通过配置项目 Skill 目录接入。

模型可以变化，供应商可以变化，但只要 Agent 使用本地 Skill 文件，Skill-kits 就能帮助你检查和管理这些文件。

### 多智能体统一视图

Skill-kits 的视角是多 Agent 的。它不会假设你只使用一个工具，也不会把不同 Agent 的同名 Skill 合并成一个抽象条目。

它使用 **Skill Instance** 模型：一个物理 Skill 目录就是一个实例。

同一个 `frontend-design` Skill 如果同时存在于 Codex、Claude Code 和 Gemini CLI 中，会显示为不同实例。这样做的好处是：

- 每一行都对应真实路径。
- 启用和禁用只影响被选中的那一份。
- 项目级 Skill 和全局 Skill 不会混淆。
- 插件缓存、供应商目录等只读来源不会被误当成可修改 Skill。

## Skill-kits 如何判断启用状态

Skill-kits 以文件系统为准。

| Skill 目录中的文件 | 状态 |
| --- | --- |
| 只有 `SKILL.md` | 已启用 |
| 只有 `SKILL.md.disabled` | 已禁用 |
| 两个文件都存在 | 无效 |
| 两个文件都不存在 | 缺失或索引过期 |

启用和禁用只是重命名当前选中的实例：

```text
启用:  SKILL.md.disabled -> SKILL.md
禁用:  SKILL.md -> SKILL.md.disabled
```

它不会删除 Skill 目录，不会修改同名的其他 Agent Skill，也不会把插件内置 Skill 当成原生 Skill 去改。

## 支持的 Agent Space

v0.1 内置支持：

| Agent | 全局 Skill 目录 | 项目级 Skill 目录 |
| --- | --- | --- |
| Codex | `~/.codex/skills` 和配置过的 Codex Skill roots | `<project>/.agents/skills` |
| Claude Code | `~/.claude/skills` | `<project>/.claude/skills` |
| Gemini CLI | `~/.gemini/skills` | `<project>/.gemini/skills` |

自定义 Agent 可以配置项目级 Skill 目录。Skill-kits 会把这些目录纳入项目视图。

## Codex 插件管理

Codex 插件和原生 Skills 是两类东西。

原生 Skill 的启用状态来自：

```text
SKILL.md / SKILL.md.disabled
```

Codex 插件的启用状态来自 Codex 配置：

```toml
[plugins."<plugin-name>@<provider>"]
enabled = true
```

Skill-kits 会单独展示 Codex 插件包、插件状态和插件提供的 runtime capabilities。插件内置的 Skills 会被视为父插件的只读能力，不会通过重命名插件包里的 `SKILL.md` 来启用或禁用。

## 功能概览

- 单二进制 CLI + 原生 GUI。
- 扫描全局和项目级 Agent Space。
- 展示 Skill、Agent、Scope、Status、Source、Path、Updated 等信息。
- 通过 `SKILL.md` / `SKILL.md.disabled` 安全启用和禁用。
- 检测无效 toggle 状态。
- 区分可写 Skill、只读插件缓存和供应商来源。
- 支持 Recent Projects 中的项目级 Skill 检查。
- 支持内置 Agent 和自定义 Agent 项目目录配置。
- 扫描 `~/.codex/plugins/cache` 下的 Codex 插件包。
- 展示插件状态和插件 runtime capabilities。
- 提供本地状态检查、scan 和 doctor 命令。
- 使用 TOML 存储本地配置和索引。

## CLI 使用

```bash
skill-kits --help
skill-kits status
skill-kits scan
skill-kits list
skill-kits enable <instance-id-or-query>
skill-kits disable <instance-id-or-query>
skill-kits doctor
```

项目级命令：

```bash
skill-kits project status --project /path/to/project
skill-kits project enable <skill> --agent codex --project /path/to/project
skill-kits project disable <skill> --agent codex --project /path/to/project
```

Codex 插件命令：

```bash
skill-kits plugin list
skill-kits plugin status <plugin-query>
skill-kits plugin enable <plugin-query>
skill-kits plugin disable <plugin-query>
skill-kits plugin scan
```

读取类命令默认输出表格，也可以使用 JSON：

```bash
skill-kits list --format json
skill-kits status --format json
skill-kits plugin list --format json
```

## GUI 使用

启动原生工作台：

```bash
skill-kits --gui
```

GUI 包含：

- Dashboard：Agent Space 数量、Recent Projects、本地健康状态。
- Skill：原生 Skill 实例列表、过滤器和右侧 inspector。
- Plugins：Codex 插件包和只读 runtime capabilities。
- Agent：内置和自定义 Agent 的项目目录配置。
- Project：项目级 Skill 实例和安全操作入口。

界面目标是本地开发工具，而不是营销页：紧凑表格、稳定分栏、清晰路径、明确操作文案。

## 从源码安装

要求：

- Rust 1.80 或更高版本。
- 当前 GUI 首发目标是 macOS。

从仓库根目录构建：

```bash
cargo build
cargo run -- status
cargo run -- --gui
```

安装到本机：

```bash
cargo install --path .
skill-kits --version
skill-kits status
```

构建 release binary：

```bash
cargo build --release
./target/release/skill-kits status
./target/release/skill-kits --gui
```

## 本地状态

Skill-kits 的本地配置和索引存放在：

```text
~/.skill-kits/
|-- config.toml
|-- registry/
|   `-- skill_instances.toml
|-- cache/
`-- locks/
```

Skill Instance index 只是扫描缓存，不是最终真相。如果索引和磁盘不一致，以磁盘为准，重新扫描对应范围即可。

## 开发

常用检查：

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

常用 smoke commands：

```bash
cargo run -- --help
cargo run -- status
cargo run -- list --format json
cargo run -- plugin list
cargo run -- doctor
```

相关文档：

- `RELEASE.md`
- `RELEASE_NOTES-v0.1.0.md`
- `skill-kits-prd/`

## 当前限制

- v0.1 是 offline-first，没有远程 marketplace 或网络安装流程。
- macOS 是首发目标，Windows 和 Linux 是后续目标。
- GUI 是 Rust/egui 原生应用，不是 Electron 或 Web app。
- Skill-kits 不会自动编辑项目 `.gitignore`。
- 插件管理目前聚焦 Codex 插件。
- 安全扫描是 advisory，不会阻塞操作。

## License

当前仓库还没有包含 license 文件。
