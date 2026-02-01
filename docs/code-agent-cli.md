## `beautiful-mermaid-rs`（CLI）使用说明

功能:把 Mermaid 文本渲染成 SVG 或 ASCII/Unicode。

### 1. 最关键的事实：它默认只从 stdin 读 Mermaid

这个 CLI **不接受“文件路径参数”**。
你必须用下面两种方式之一喂输入：

- 管道：`cat diagram.mmd | beautiful-mermaid-rs ...`
- 重定向：`beautiful-mermaid-rs ... < diagram.mmd`

输出默认写到 stdout。
因此写文件用重定向即可：`> out.svg`。

#### 用法

- 从 stdin 读取 Mermaid 文本并输出 SVG

  beautiful-mermaid-rs < diagram.mmd > diagram.svg

- 输出 Unicode 线条字符（更好看，适合终端）

  beautiful-mermaid-rs --ascii < diagram.mmd

- 输出纯 ASCII 字符集（兼容性最好）

  beautiful-mermaid-rs --ascii --use-ascii < diagram.mmd

#### 选项

  --ascii         输出 ASCII/Unicode 文本（默认输出 SVG）
  --use-ascii     仅在 --ascii 模式下生效：强制使用纯 ASCII 字符
  -h, --help      输出帮助并退出
  -V, --version   输出版本并退出

#### `beautiful-mermaid-rs` 的退出码约定

- `0`：成功（或 pipe 下游提前关闭导致 BrokenPipe，按 Unix 习惯也视为成功退出）
- `1`：渲染失败 / 读取 stdin 失败
- `2`：参数或用法错误（例如 stdin 为空、未知参数、`--use-ascii` 没配 `--ascii`）

#### 建议 agent 的处理策略

- 先检查 exit code。
- 失败时优先把 stderr 原样回显（里面有原因）。
- 如果是 `2`，直接改用法，不要继续“猜测式重试”。
