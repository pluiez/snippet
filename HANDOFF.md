# 交接文档 — Snippet 项目

> 写给下一个 AI coding agent。
> 截止日期：2026-05-31。

---

## 一句话概述

Snippet 是一个 Windows-first（macOS later）Tauri v2 桌面应用：全局热键 → palette → 搜索模板 → 填变量 → 写剪贴板（可选自动粘贴）。**Phase 2（核心功能，Slice 0-7）和 Phase 3 工作流 A（动画）/ B（错误处理）已全部完成并验证**。下一步是工作流 C（测试与发布）。

---

## 当前状态速查

| 项目 | 状态 |
|---|---|
| Git 分支 | `main` |
| 最新 commit | `67e2b3a feat: §13 invariant unit tests + fill.rs / search::rank refactor (Phase 3 Workflow C)` |
| Tags | `slice-7a-onboarding` / `slice-7b-hotkey` / `slice-7c-theme` |
| 编译 | `pnpm bindings` / `cargo test --lib` 68 通过；`pnpm tauri dev` 可正常运行 |
| Phase 1 | Slice 0 + 0.5 ✅ |
| Phase 2 | Slice 1-7 (7a/7b/7c) ✅ 全部验证 |
| Phase 3 工作流 A | 动画与过渡 ✅ |
| Phase 3 工作流 B | 错误处理与边界态 ✅ |
| Phase 3 工作流 C | §13 不变量单测 ✅（10/12 覆盖，4/9 留集成/smoke）；剩 IPC 集成、.msi 打包、smoke test |

---

## 文档地图

按优先级阅读：

1. **`CLAUDE.md`** — 项目 AI agent 指南（dev 命令、架构约束、关键约定）。**先看这个。**
2. **`PROGRESS.md`** — 每个 slice 的详细落地记录：实现了什么、验证场景、已知坑、patch 历史。信息密度最高。
3. **`SPEC.md`** — 产品 spec（中文）。UI 行为、数据契约、交互流程、§13 核心不变量。**SPEC 是事实标准，其它文档跟它冲突时以 SPEC 为准。**
4. **`ARCHITECTURE.md`** — 实现骨架：技术栈、进程边界、模块域、§6 关键时序约束。
5. **`TASKS.md`** — 切片交付计划。Phase 3 的三个工作流（A/B/C）定义在此。

---

## 关键约束（必须遵守）

1. **Commit message 不许加 `Co-Authored-By` trailer** — 用户明确要求，绝对不能加。
2. **语言**：spec 和 UI 文字都是中文；代码注释/变量名英文。
3. **平台**：开发在 Windows native（`C:\dev\snippet`）。WSL 只做编辑（路径 `/mnt/c/dev/snippet/`）。`pnpm tauri dev` 从 PowerShell 跑。
4. **Backend 是 source of truth** — 前端是纯 view + control，不碰文件系统或 OS API。
5. **类型契约走 ts-rs** — 改了 Rust 类型（`#[derive(TS)]`）后要跑 `pnpm bindings` 重新生成。
6. **HWND 第一步同步捕获** — 热键 callback 第一行必须是 `GetForegroundWindow()`。
7. **两阶段启动** — `init_bootstrap`（Phase A，始终跑）→ `init_full_state`（Phase B，条件跑）。Onboarding 期间 AppState 未 manage。
8. **变量 GUID 稳定** — body 里存 `{<uuid>}`，不是 displayName。

---

## 代码结构

### 后端（`src-tauri/src/`）

| 模块 | 职责 |
|---|---|
| `lib.rs` | Tauri setup（两阶段启动）、close/tray/single-instance handler、startup warning 收集 |
| `schema.rs` | 所有数据 schema（Template/Variable/Settings/Bootstrap/ColorMaps/LastUsed/StartupWarning） |
| `state.rs` | `AppState`（Mutex-based in-memory store，含 startup_warnings） |
| `commands.rs` | 所有 IPC 命令（含 get_startup_warnings） |
| `paths.rs` | 数据目录路径解析 |
| `storage.rs` | 文件读写、atomic write、模板扫描、ensure 目录结构。load_or_init 返回 (T, bool=recovered) |
| `palette.rs` | hotkey handler + show/hide palette + show_main_window + 窗口互斥 |
| `search.rs` | SPEC §7 搜索实现（pinyin + fuzzy + 加权） |
| `render.rs` | 模板渲染（占位符替换 + body_for_search） |
| `color.rs` | OKLCh 生成 + 对比度校验 + GC |
| `hotkey.rs` | parse_hotkey + re_register_hotkey（12 个 unit test） |
| `onboarding.rs` | classify_path + needs_onboarding（6 个 unit test） |
| `auto_paste.rs` | Windows SetForegroundWindow + enigo Ctrl+V |

### 前端（`src/`）

| 文件 | 职责 |
|---|---|
| `main.tsx` | 按窗口 label 路由（main/palette/onboarding）+ Provider 嵌套 |
| `App.tsx` | 主窗口 view switcher（list/edit/fill/colors/settings）+ nav + AnimatePresence 视图过渡 + 启动警告 toast |
| `Palette.tsx` | 独立无边框窗口，搜索+列表+preview+fill/edit 堆叠 + 淡入淡出 + 空状态 UI |
| `Onboarding.tsx` | 首次启动三选一流程 + context-aware 路径校验提示 |
| `Settings.tsx` | 完整设置页（hotkey/autoPaste/dataFolder/theme） |
| `TemplateList.tsx` / `TemplateEditor.tsx` / `TemplateFillDialog.tsx` | CRUD + 填充 + dirty 检测 + 错误处理 |
| `ColorManagement.tsx` | 两 tab 颜色管理 + 空状态引导 |
| `VariableEditor.tsx` / `VariableList.tsx` | 变量编辑卡片 |
| `HotkeyInput.tsx` | 热键捕获输入组件（pause/resume + code-based） |
| `TagInput.tsx` / `TagPill.tsx` / `OptionsInput.tsx` | chip-style 输入 |
| `BodyWithVariableChips.tsx` | 模板 body 预览（变量色块） |
| `ConfirmDialog.tsx` | 通用确认弹窗（backdrop fade + card scale 动画） |
| `Toast.tsx` | 通用 toast（三变体 success/error/warning + 淡入淡出） |
| `lib/motion.ts` | 共享动画常量（DURATION / EASE） |
| `lib/theme.tsx` | ThemeApplier 纯 effect |
| `lib/colors.tsx` | ColorMapsProvider context |
| `lib/settings.tsx` | SettingsProvider context |
| `lib/body.ts` | bodyToDisplay / bodyToStorage 双向转换 |
| `lib/render.ts` | JS 镜像 render（preview 用） |
| `lib/fill.ts` | mergeFillValues helper |

---

## 工作流 B 实施总结（已完成 + 已验证）

已通过 commit `b6dbab9` 落地。涵盖后端 + 前端。

| 功能项 | 状态 | 关键文件 |
|---|---|---|
| Toast 多变体（success/error/warning） | ✅ 已验证 | `Toast.tsx` |
| 编辑器 dirty 检测 + Escape 确认 | ✅ 已验证 | `TemplateEditor.tsx` |
| 编辑器 save 失败 inline 错误 | ✅ 已实现 | `TemplateEditor.tsx` |
| 填充对话框 submit 错误 | ✅ 已实现 | `TemplateFillDialog.tsx` |
| 启动警告基础设施（损坏 config / 热键失败 / 写权限） | ✅ 已验证 | `schema.rs` / `state.rs` / `storage.rs` / `lib.rs` / `commands.rs` / `App.tsx` |
| Palette 空状态 UI | ✅ 已验证 | `Palette.tsx` |
| 颜色管理空状态 UI | ✅ 已验证 | `ColorManagement.tsx` |
| Onboarding context-aware 路径提示 | ✅ 已验证 | `Onboarding.tsx` |
| Palette apply toast 行为优化 | ✅ 已验证 | `Palette.tsx` |

**用户手动测试覆盖的场景**：

- 启动警告 toast（4-5-6 全部正常）
- 编辑器 dirty：修改后点取消 → 确认框 ✅；修改后 Escape → 确认框 ✅；无修改 → 直接退出 ✅
- Palette：autoPaste 关闭→立刻关闭 ✅；paste 失败→短暂 toast ✅

**未测试但已实现的场景**（低风险，代码路径清晰）：

- Save 失败 inline 错误（需人为制造后端错误）
- 填充对话框 submit 失败
- 数据文件夹写权限探测（需设目录为只读）

---

## Phase 3 — 下一步工作

### 工作流 C — 测试与发布（部分完成）

**已完成（2026-05-31）**：

- ✅ SPEC §13 核心不变量单元测试（10/12，4 和 9 性质所致跳过 — 详见下方"§13 核心不变量清单"）
- ✅ pinyin 多音字回归测试（`search.rs::invariant_12_*`）
- ✅ 重构副产品：`fill.rs` 模块抽离 + `search::rank` 纯函数化 + `state::AppState::for_test()` helper + `color::random_oklch` round-trip 精度 bug 修复（F1 决议，见 PROGRESS.md）

**剩余**：

- 关键 IPC 命令族集成测试（`save_template` round-trip / `apply_template` outcome / `prepare_fill_dialog` 全 case）
- fuzzy 匹配评分排序的回归测试（rank 测试已覆盖主路径，可视需要扩展 nucleo 边界 case）
- Windows 安装包打包（`.msi`，含 `tauri.conf.json` bundle identifier 从占位 `app.snippet` 换为真实反向域名）
- 代码签名（v1 跳过 — unsigned `.msi`，SmartScreen 首次启动会有警告但功能完整）
- 手动 smoke test：完整安装 → onboarding → 使用流程 → 设置变更 → 卸载 → 重装（数据保留）

### 推迟到工作流 C 的项（从工作流 B 移出）

| 项目 | 理由 |
|---|---|
| WebView2 缺失引导安装 | 属打包阶段 |
| 设置页"指定空路径新建" | 需大量改动，优先级低 |
| 已知 OS 保留热键列表提示 | 优先级低 |

---

## 开发环境快速启动

```bash
# WSL 编辑路径
cd /mnt/c/dev/snippet

# Windows PowerShell 运行（从 C:\dev\snippet）
pnpm install          # 一次性
pnpm bindings         # 编译 Rust + 生成 TS 类型（= cd src-tauri && cargo test）
pnpm tauri dev        # 开发服务器 + Rust 热重建
pnpm tauri build      # 生产构建
```

首次启动前确保 `%APPDATA%\Snippet\bootstrap.json` 存在且 `onboardingComplete: true`（否则会弹 onboarding 窗口）。清空 `%APPDATA%\Snippet\` 可触发 onboarding 流程测试。

---

## 技术栈版本

- Tauri 2.x（`tauri = "2"`）
- React 19 + TypeScript
- Tailwind CSS v4（`@import "tailwindcss"`，`@custom-variant dark`）
- Vite
- framer-motion ^12（动画库）
- ts-rs（双端类型同步）
- 关键插件：`tauri-plugin-global-shortcut` / `tauri-plugin-dialog` / `tauri-plugin-clipboard-manager` / `tauri-plugin-single-instance`
- Rust 关键 crate：`nucleo-matcher`（fuzzy）、`pinyin`（中文）、`enigo`（键盘模拟）、`rand`（颜色生成）、`chrono`

---

## 容易踩的坑（前人经验）

1. **HotkeyInput 自抢先**：app 已注册的热键在 HotkeyInput 捕获态下仍被 OS 优先 dispatch。已通过 `pause_hotkey`/`resume_hotkey` IPC 解决。
2. **AppState 延迟 manage**：onboarding 期间所有业务 IPC 拿不到 State。所有新 IPC 如果可能在 onboarding 期间被调用，要用 `try_state` 保护。
3. **`e.code` vs `e.key`**：热键捕获用 `e.code`（物理键，layout 中立），不用 `e.key`（被输入法/布局翻译后的字符）。
4. **render 双实现**：Rust `render.rs` 和 JS `lib/render.ts` 是同逻辑的镜像实现，改占位符规则要两边同步。
5. **颜色 map 双文件**：`variableColorMap` 和 `tagColorMap` 是独立的两个文件，GC 也独立跑。同名 tag 和变量不会冲突。
6. **atomic_write**：所有文件持久化走 tmp → rename，不直接覆写。
7. **Tailwind v4 dark 语法**：用 `@custom-variant dark (&:where(.dark, .dark *));`，不是 v3 的 `darkMode: 'class'` JS 配置。
8. **Windows 系统级热键**：Win+L / Ctrl+Alt+Del 等被 OS 拦截，webview 的 keydown 收不到、global-shortcut 注册也会失败。无法在 app 内修复，是 OS 限制。
9. **SetForegroundWindow**：Windows 前台保护机制下不是 100% 成功。某些 app（Sublime Text、游戏 launcher）忽略 SendInput 模拟的 Ctrl+V。enigo 报成功但目标 app 没收到——已知 OS 级限制。
10. **serde 兼容**：Bootstrap 加新字段时用 `#[serde(default)]` 让老文件能反序列化。`onboarding_complete` 用专门的 `default_onboarding_complete_for_legacy` 函数返回 `true`，避免让老用户重做 onboarding。
11. **Palette 隐藏时序**：`requestHide()` 先 `setVisible(false)` 触发淡出动画，200ms 后才调 `invoke("hide_palette")`。快速连续按键由 `hideTimeoutRef` 去重。直接调 `invoke("hide_palette")` 会跳过淡出动画。
12. **AnimatePresence 需要 key**：每个条件渲染的 `motion.div` 需要唯一 `key` prop 才能正确触发 exit 动画。漏了 key 会导致卸载时无动画。
13. **TemplateEditor 全局 keydown**：Escape/Ctrl+Enter 用 `window.addEventListener("keydown")` + ref pattern（不是 div onKeyDown），因为 div 无 tabIndex 时事件冒泡不可靠。`showDiscardConfirm` 时 early return 避免与 ConfirmDialog 的 Escape handler 冲突。
14. **Toast variant**：Toast 支持 success/error/warning 三变体。Palette 的 finalizeApply 仅在 paste-failed 路径显示 toast，正常路径（autoPaste 关闭或粘贴成功）立即隐藏 palette 不显示 toast。
15. **Startup warnings**：`get_startup_warnings` IPC drain-and-clear 模式（读一次就清空）。App.tsx mount 时调一次，staggered toast 展示。

---

## SPEC §13 核心不变量清单（覆盖状态）

工作流 C 测试基础设施完成后的最新覆盖。10/12 由单测覆盖，4 和 9 性质所致不适合纯 Rust 单测（前端 UI 行为 / Tauri runtime 依赖），留集成测试 + smoke test 覆盖。

| # | 内容 | 单测覆盖 |
|---|---|---|
| 1 | 变量 GUID 稳定 | ✅ `render.rs::invariant_1_*`（2 测试，render lookup + body_for_search rename） |
| 2 | 删除变量清理 body | ✅ `render.rs::invariant_2_orphan_placeholder_renders_empty`（后端容错；前端清理仍在 `VariableList.tsx`） |
| 3 | enum last-used 失效回退 | ✅ `fill.rs::invariant_3_*`（3 测试：last-used 失效 / staticDefault 失效 / 双失效） |
| 4 | 剪贴板互斥 | ⏭️ 前端 `VariableList.tsx` UI 行为，后端 `save_template` 不强制；留集成测试 / smoke test |
| 5 | GC 不误删活引用 | ✅ `color.rs::invariant_5_gc_preserves_live_entries` |
| 6 | GC 删孤儿 | ✅ `color.rs::invariant_6_gc_removes_orphans` |
| 7 | 搜索权重 1.0/0.8/0.3（MAX 不 SUM） | ✅ `search.rs::invariant_7_*`（2 测试，displayName 与 tag 各自胜过 body） |
| 8 | 排序稳定（同分 lastUsedAt desc + displayName 兜底） | ✅ `search.rs::invariant_8_*`（2 测试）+ `empty_query_*`（2 测试） |
| 9 | 窗口互斥 | ⏭️ `palette.rs` 依赖 Tauri runtime（hotkey 注册 / window 管理）；留 smoke test |
| 10 | 颜色对比度 ≥ 4.5:1 | ✅ `color.rs::invariant_10_random_oklch_meets_contrast_against_white`（1000 sample sweep）。**实施时修了 round-trip 精度 bug**：原 `>= 4.5` 在生成器内满足，但 stored 形式（`{l:.3}` 截断）re-parse 可能掉到 4.499，违反实际渲染语义。改为 `>= 4.55`（`CONTRAST_TARGET + CONTRAST_GUARD`），详见 PROGRESS.md F1 决议 |
| 11 | GC 收敛（reconcile 二次幂等） | ✅ `color.rs::invariant_11_reconcile_converges_after_first_run` |
| 12 | pinyin 多音字（用 crate 默认音） | ✅ `search.rs::invariant_12_*`（3 测试：full / initial / 多音字默认音） |

补充测试（非 §13）：

- `hotkey.rs` 12 个 parser test（Slice 7b）
- `onboarding.rs` 6 个 `classify_path` test（Slice 7a）
- `fill.rs` 7 个补充：优先级 4 层 + lowercase key + empty skip + `is_valid_for_variable` rules
- `color.rs` 3 个补充：`contrast_against_white_anchor_values` + `reconcile_ensures_missing_entries_for_live_refs` + `gc_keys_are_lowercased`
- `search.rs` 3 个补充：`contains_chinese_detection` + `pinyin_passes_through_ascii` + `nonempty_query_excludes_zero_score`

`cargo test --lib` 当前 68 通过，0 失败。

---

## 未完成的文档债

| 项目 | 说明 |
|---|---|
| **Spec 决议日志 B/C/D 系列** | PROGRESS.md 末尾列了 B 系列（孤儿变量/staticDefault 失效等）、C 系列（必填+剪贴板死锁/watcher 并发等）、D 系列（措辞/测试稳定性）多条未决 spec 偏差。大部分 B 系列已在 Slice 3 代码中解决但措辞未写回 SPEC.md。C/D 系列属 Phase 3 工作流 C 范围，做的时候再决。 |
| **ARCHITECTURE.md 两阶段启动** | Slice 7a 引入了两阶段启动，应在 ARCHITECTURE.md §7 补一行说明。可选做。 |

---

## 总结

项目核心功能完整，动画 / 错误处理 / §13 不变量测试基础设施均已落地。工作流 C 仅剩 IPC 集成测试 + Windows `.msi` 打包 + 手动 smoke test。

祝接手顺利。
