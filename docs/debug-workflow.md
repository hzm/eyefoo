# Debug Workflow: Background Process + stderr Capture

## 适用场景

- 桌面应用（Tauri/Rust、C++、Qt 等）运行但行为不符合预期
- 需要确认某段代码是否被执行、执行频率、变量值
- 程序无法交互式调试（日志只能打印到 stderr）
- 不想手动复制粘贴日志，想让 AI 直接看到输出

## 核心原理

- `eprintln!` / `fprintf(stderr)` 等输出到 stderr
- `2>&1` 把 stderr 重定向到 stdout
- Claude Code bash 执行命令后，stdout/stderr 直接显示在返回结果里
- 后台运行 `&` + `sleep N` + `taskkill` 组合实现"运行→留时间→收集日志→停止"

## Rust 项目命令（示例：L:\test\eyefoo）

```bash
# 完整流程：杀进程 → 编译 → 后台运行exe → 留时间 → 停止
taskkill //F //IM eyefoo.exe 2>&1
~/.cargo/bin/cargo.exe build 2>&1
L:/test/eyefoo/src-tauri/target/debug/eyefoo.exe 2>&1 &
sleep 5
taskkill //F //IM eyefoo.exe 2>&1
```

### Windows CMD 版本

```cmd
taskkill //F //IM eyefoo.exe
cd /d L:\test\eyefoo\src-tauri
%USERPROFILE%\.cargo\bin\cargo.exe build
start L:\test\eyefoo\src-tauri\target\debug\eyefoo.exe
timeout /t 5
taskkill //F //IM eyefoo.exe
```

### Linux/macOS bash 版本

```bash
pkill eyefoo || true
cd L:/test/eyefoo/src-tauri
~/.cargo/bin/cargo.exe build
L:/test/eyefoo/src-tauri/target/debug/eyefoo 2>&1 &
sleep 5
pkill eyefoo
```

## 每一步的作用

| 命令部分 | 作用 |
|---|---|
| `taskkill //F //IM eyefoo.exe` | 确保没有旧进程在跑（exe 被锁无法编译） |
| `cargo build 2>&1` | 编译，日志也重定向方便查看 |
| `eyefoo.exe 2>&1 &` | 后台运行，stderr 重定向到 stdout |
| `sleep 5` | 留 5 秒给程序运行，日志在这期间输出 |
| `taskkill //F //IM eyefoo.exe` | 强制停止，收集输出 |

## 调试图：加日志

在怀疑的代码位置加 `eprintln!`：

```rust
eprintln!("[DEBUG] state={}, paused={}", state.load(Ordering::SeqCst), paused);
```

编译 → 运行 → 看输出 → 定位问题 → 改代码 → 重复

## 不同语言的 stderr 输出

| 语言 | 函数 | 示例 |
|---|---|---|
| Rust | `eprintln!` | `eprintln!("state={}", x)` |
| C | `fprintf(stderr)` | `fprintf(stderr, "state=%d\n", x);` |
| Python | `print(..., file=sys.stderr)` | `print(x, file=sys.stderr)` |
| Go | `log.Printf` / `fmt.Fprintln(os.Stderr)` | `log.Printf("state=%d", x)` |
| Node.js | `console.error` | `console.error('state:', x)` |

## 进阶：调整运行时间

- `sleep 2`：快速验证，适合改动频繁的调试
- `sleep 10`：观察更长时间，确认行为持续
- `sleep 60`：观察一分钟以上的行为（如计时器逻辑）

## 进阶：只运行不杀进程

如果想观察更长时间，先不用 `taskkill`，直接让程序跑着，然后在 Claude Code 里按 Ctrl+C 停止即可。`&` 后台运行时 Claude Code 不会自动收集输出，所以还是要用 taskkill 来触发输出收集。

## 适用项目

所有能从命令行编译运行的项目：Rust、C++、Go、Python、Node.js……模式通用，不限语言。
