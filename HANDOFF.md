# 交接文档 — Snippet 项目

> 写给下一个 AI coding agent。
> 截止日期：2026-05-31。

---

## 一句话概述

Snippet 是一个 Windows-first（macOS later）Tauri v2 桌面应用：全局热键 → palette → 搜索模板 → 填变量 → 写剪贴板（可选自动粘贴）。**Phase 2 全部完成；Phase 3 工作流 A / B / C 主要工作均已完成**——含 §13 不变量单测、storage 层集成测试、Windows `.msi` + `.exe` 打包、GitHub Actions CI、公开仓库托管。**仅剩 task 17：手动 smoke test**（详见下方"smoke test checklist"段）。

---

## 当前状态速查

| 项目 | 状态 |
|---|---|
| Git 分支 | `main`（已与 `origin/main` 同步） |
| GitHub 仓库 | <https://github.com/pluiez/snippet>（public，MIT，双语 README） |
| 最新提交 | 见 `git log -1`（不再人工维护 SHA — self-referential 会永远滞后一格） |
| Tags（按时间倒序） | `workflow-c-package` / `workflow-c-storage-tests` / `workflow-c-invariant-tests` / `slice-7c-theme` / `slice-7b-hotkey` / `slice-7a-onboarding`（完整列表：`git tag --sort=-creatordate`） |
| 编译 | `cargo test --lib` 78 通过；`pnpm tauri dev` 可跑；`pnpm tauri build` 出 `.msi` + `.exe` |
| 打包产物 | `src-tauri/target/release/bundle/msi/Snippet_0.1.0_x64_en-US.msi`（4.6MB）<br>`src-tauri/target/release/bundle/nsis/Snippet_0.1.0_x64-setup.exe`（3.1MB）<br>unsigned（SmartScreen 首次启动会有警告） |
| GitHub Actions CI | `cargo test --lib` on ubuntu-latest，push 到 main / PR 自动跑（见 `.github/workflows/test.yml`） |
| Phase 1 | Slice 0 + 0.5 ✅ |
| Phase 2 | Slice 1-7 (7a/7b/7c) ✅ 全部验证 |
| Phase 3 工作流 A | 动画与过渡 ✅ |
| Phase 3 工作流 B | 错误处理与边界态 ✅ |
| Phase 3 工作流 C | §13 单测 ✅（10/12）+ storage 集成测试 ✅（10 测试）+ `.msi` / `.exe` 打包 ✅ + CI ✅；**剩 smoke test** |

---

## 交接快照（2026-05-31 本会话产出）

如果你是新接手 agent，先读这里再决定下一步。

### Commit 时间线（早 → 晚）

| Commit | 含义 | 落地 |
|---|---|---|
| `67e2b3a` | feat: §13 invariant unit tests + fill.rs / search::rank refactor | tag `workflow-c-invariant-tests` |
| `ef8a668` | docs: 同步 HANDOFF 最新 commit 字段 | — |
| `9e4a114` | docs: 明确 dev platform 是 Ubuntu on WSL 2 / Windows native | — |
| `a089a16` | docs: HANDOFF 顶部"最新 commit"字段改 placeholder（消 self-ref 循环） | — |
| `81a1cf6` | test: storage layer integration tests | tag `workflow-c-storage-tests` |
| `b90587f` | docs: 加双语 README + MIT LICENSE | — |
| `947c1f7` | docs: 拆双语 README 为 README.md + README.zh-CN.md | — |
| `b798ffa` | ci: GitHub Actions workflow（cargo test on ubuntu） | — |
| `8162073` | feat: tauri bundle metadata + first `.msi`/`.exe` build | tag `workflow-c-package` |

### 里程碑速览

1. **§13 不变量单测基础设施**（10/12 条由单测覆盖，4 + 9 性质所致跳过）— 见下方 §13 清单段
2. **storage 层集成测试**（10 测试，覆盖 atomic_write / load_or_init 三态恢复 / load_templates 损坏隔离 / save+load round-trip / delete / ensure_data_folder_structure 幂等）
3. **GitHub 公开仓库 + 双语 README + MIT LICENSE**：`https://github.com/pluiez/snippet`
4. **GitHub Actions CI**（`.github/workflows/test.yml`，跑 cargo test --lib on ubuntu-latest，push 到 main / PR 自动触发，README 顶部有 badge）
5. **Windows `.msi` + `.exe` 打包**：identifier 从占位 `app.snippet` 换为 `com.github.pluiez.snippet`；加 publisher / homepage / shortDescription / copyright

### 本会话修的关键 bug（已 commit，无需重做）

- **F1**：`color::random_oklch` round-trip 精度 bug。阈值 `>= 4.5` → `>= 4.55`（`CONTRAST_TARGET + CONTRAST_GUARD`）。原因：`{l:.3}` 截断 re-parse 可能掉到 4.499 违反不变量 10。
- **F2**：tauri.conf.json bundle identifier 改真实反向域名；补 4 个 bundle metadata 字段。
- **4 个 ts-rs binding 文件从未 commit**（`Bootstrap.ts` / `LastUsed.ts` / `VariableColorMap.ts` / `TagColorMap.ts`）：dev mode `import type` erase + Vite ESM lenient 没暴露；prod `tsc` 才报。手写按 ts-rs 格式补；以后 `cargo test` 重新生成应内容一致（不会冲突）。
- **3 个 TS strict-mode 错误**（dev 没暴露，prod build 才挂）：`src/lib/render.ts` 未用 `match` 参数（改 `_match`）；`src/VariableEditor.tsx` 从 `Variable.ts` 错误 import `VariableType`（拆为独立 import）；`src/lib/colors.tsx` ts-rs HashMap value 是 `string | undefined`，但 `ColorMaps.variables` 用 `Record<string, string>`（用 `as Record<string, string>` 强转保住 consumer 接口）。

### 现在可以接着做的事

按优先级：

1. **手动 smoke test**（task 17 pending）— 见下方"smoke test checklist"段，~30 分钟跑完
2. （optional）发 GitHub release v0.1.0 基于 tag `workflow-c-package`，附 `.msi` + `.exe` artifact
3. （optional）补 ARCHITECTURE.md §7 两阶段启动说明（文档债 1 条，可选做）
4. （optional）把 PROGRESS.md 末尾 B/D 系列 spec 决议措辞写回 SPEC.md（文档债，可选做）

### 已通过测试，**不要重复跑/重复验证**的项

- `cargo test --lib` = 78 通过（已在本地 + GitHub Actions 验证）
- `pnpm tauri build` 在 Windows PowerShell 已跑通，`.msi` + `.exe` 已 produce
- 所有 Slice 0-7c + Phase 3 工作流 A/B 的用户手动验证场景（见 PROGRESS.md 各 Slice 段，全部标 ✅ 已验证）
- §13 不变量 1/2/3/5/6/7/8/10/11/12 由单测覆盖（不变量 4/9 由 nature 留 smoke test）
- HotkeyInput pause/resume 自抢先 bug、TemplateEditor 全局 keydown、Palette 隐藏时序、AnimatePresence key 缺失、Toast variant 等 Workflow A/B 期间踩的坑（见下方"容易踩的坑"段）

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
3. **平台**：目标运行 Windows native（`C:\dev\snippet`）。编辑 Claude session 跑在 **Ubuntu on WSL 2**，走 `/mnt/c/dev/snippet/` 访问同一份文件。`pnpm tauri dev` / `pnpm tauri build` 从 Windows PowerShell 跑。WSL 1 不支持（9p / `/mnt/c` 假设）。
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
- ✅ storage 层集成测试（10 测试：`atomic_write` / `load_or_init` 三态恢复 / `load_templates` 损坏隔离 / `save_template`+`load_templates` 全字段 round-trip / `delete_template` / `ensure_data_folder_structure` 幂等）—— 覆盖了原计划的"`save_template` round-trip 集成测试"
- ✅ Windows `.msi` + `.exe` 打包（F2 决议：identifier 从 `app.snippet` 换为 `com.github.pluiez.snippet`；加 publisher / homepage / shortDescription / copyright metadata）。输出 `bundle/msi/Snippet_0.1.0_x64_en-US.msi` + `bundle/nsis/Snippet_0.1.0_x64-setup.exe`
- ✅ GitHub Actions CI（`cargo test --lib` on ubuntu-latest，push 到 main / PR 自动跑）
- ✅ GitHub public 仓库托管 + 双语 README + MIT LICENSE：https://github.com/pluiez/snippet
- ✅ 重构副产品：`fill.rs` 模块抽离 + `search::rank` 纯函数化 + `state::AppState::for_test()` helper + `color::random_oklch` round-trip 精度 bug 修复（F1 决议，见 PROGRESS.md）
- ✅ 打包过程踩坑修复：手写 4 个缺失 ts-rs binding 文件（`Bootstrap` / `LastUsed` / `VariableColorMap` / `TagColorMap` 从未 commit）+ 修 3 个 prod TS strict-mode 错误（unused var / 错误 import / HashMap value type 不匹配）—— 详见 PROGRESS.md 工作流 C 段

**剩余**：

- 代码签名（v1 跳过 — unsigned `.msi`，SmartScreen 首次启动会有警告但功能完整）
- 手动 smoke test：完整安装 → onboarding → 使用流程 → 设置变更 → 卸载 → 重装（数据保留）

`apply_template` 的剪贴板 / autoPaste runtime 路径 + `prepare_fill_dialog` 的剪贴板读取依赖 Tauri ClipboardExt，留 smoke test 覆盖。fuzzy 匹配回归已被 search rank 主路径覆盖。

### 推迟到工作流 C 的项（从工作流 B 移出）

| 项目 | 理由 |
|---|---|
| WebView2 缺失引导安装 | 属打包阶段 |
| 设置页"指定空路径新建" | 需大量改动，优先级低 |
| 已知 OS 保留热键列表提示 | 优先级低 |

---

## smoke test checklist（task 17 — 待跑）

`.msi` 在 `src-tauri/target/release/bundle/msi/Snippet_0.1.0_x64_en-US.msi`，可以直接装。预备：先清空 `%APPDATA%\Snippet\` 让首次启动走 onboarding。每个 ✅ 项目跑完打勾。

**阶段 1：装 + onboarding**

- [ ] 双击 `Snippet_0.1.0_x64_en-US.msi`。SmartScreen 出现警告（unsigned，正常）→ "更多信息" → "仍要运行" → 完成安装
- [ ] 安装后 Windows "已安装程序"列表能看到 `Snippet`，publisher = `pluiez`
- [ ] 通过开始菜单或快捷方式启动 Snippet
- [ ] **不变量 9 验证**：onboarding 窗口居中弹出（不是 main，不是 palette）；按 `Ctrl+Alt+Space` 应无反应（hotkey 还没注册）
- [ ] 三选一走"使用默认路径" → "开始使用" → onboarding 窗口关闭，主窗口出现，托盘有 icon
- [ ] 检查 `%APPDATA%\Snippet\bootstrap.json` 存在含 `onboardingComplete: true`；`templates/` 等子目录已 ensure

**阶段 2：基础使用**

- [ ] 主窗口左侧 nav 看到"全部模板"+"颜色管理"+"设置"。空状态（无模板）
- [ ] 顶部"新建模板" → 输入 displayName="测试" + body="hello {name}" → 加变量 name → 保存
- [ ] **不变量 1/2 验证**：编辑模板，改 name → username 显示名 → body 文本框自动显示 `{username}`，保存→ 列表仍正常
- [ ] 主窗口 list 点 pin 图标 → 该模板置顶（pinned 排序）
- [ ] 按 `Ctrl+Alt+Space` 唤起 palette → 输入"测试" → 命中 → 回车
- [ ] palette 内变身为填充对话框 → 填值 → Cmd+Enter → 提示已复制
- [ ] 切到任意文本编辑器粘贴 → 内容是"hello <你填的值>"
- [ ] 拼音搜索：palette 内输"ceshi" → 同样命中
- [ ] 首字母搜索：palette 输"cs" → 命中

**阶段 3：交互边界**

- [ ] **不变量 9 主窗口 ↔ palette 互斥**：主窗口可见时按 `Ctrl+Alt+Space` → 主窗口前置 + amber 描边脉冲 0.9s + palette 不弹
- [ ] **不变量 9 reverse**：palette 显示时点托盘 icon → palette 关、主窗口前置
- [ ] **不变量 4 剪贴板互斥**：模板加 A + B 两变量，A 勾"从剪贴板填充"；勾 B 同选项 → A 自动清 + toast"已从 A 转移"
- [ ] **不变量 3 enum 回退**：建带 enum 变量的模板，options=["简体","繁体"]，先试用选"简体" → 改 options 为 ["日文","英文"] → 重新试用 → enum 字段默认空（不是"简体"，因为不在新 options）

**阶段 4：设置变更**

- [ ] 设置页改 hotkey 为 `Ctrl+Alt+J` → 保存 → palette 关闭 → 按 `Ctrl+Alt+J` 唤起 ✓；按 `Ctrl+Alt+Space` 无反应 ✓
- [ ] 设置页 autoPaste 开启 → 切到记事本 → 按 hotkey → 选模板 → 内容自动粘贴到记事本
- [ ] 设置页改 theme 为"深色" → 主窗口 + palette 立即变深色（无白屏闪烁）
- [ ] 改回"跟随系统" → OS 切 dark mode → 应用同步切换

**阶段 5：颜色管理**

- [ ] **不变量 5/6/10/11 验证**：颜色管理页两 tab 都有条目（变量 + tag 各种），每个色块对比度足够（字白色清晰）；改一个颜色 → 保存 → 主窗口列表的 tag pill 和 palette 内的色点同步刷新
- [ ] 删除引用某 tag 的所有模板 → 重启 app → 颜色管理页该 tag 不再出现（GC 跑过）

**阶段 6：卸载 + 重装**

- [ ] Windows "已安装程序"列表卸载 Snippet（或跑 uninstaller）
- [ ] 卸载后 `%APPDATA%\Snippet\` 应**保留**（这是设计——用户数据不随 app 卸载丢）
- [ ] 重新双击 `Snippet_0.1.0_x64_en-US.msi` 安装
- [ ] 启动 app → **不弹 onboarding**（bootstrap.json 还在，`onboardingComplete: true`）
- [ ] 主窗口列表展示之前的模板 + tag + 颜色保持

**阶段 7（optional）：导入流程**

- [ ] 清空 `%APPDATA%\Snippet\` 让 onboarding 重新触发
- [ ] 把上面"卸载前"的 `%APPDATA%\Snippet\` 备份恢复到 `D:\backup\snippet\`
- [ ] 重启 app → onboarding 选"从已有路径导入" → 选 `D:\backup\snippet` → 进入主窗口看到旧模板

**完成后**：

- 在 PROGRESS.md 工作流 C 段加一行 "smoke test 已通过（YYYY-MM-DD）"
- HANDOFF.md 顶部速查表 Phase 3 工作流 C 行改为 ✅ 全部完成
- 打 tag `phase-3-complete` 或 `v0.1.0`，commit + push
- 可选发 GitHub release v0.1.0

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

`cargo test --lib` 当前 **78 通过**，0 失败（含 storage 集成测试 10 个 + 非 §13 的 hotkey 12 / onboarding 6 / ts-rs export_bindings 15）。GitHub Actions CI 上同样跑通。

---

## 未完成的文档债

| 项目 | 说明 |
|---|---|
| **Spec 决议日志 B/C/D 系列** | PROGRESS.md 末尾列了 B 系列（孤儿变量/staticDefault 失效等）、C 系列（必填+剪贴板死锁/watcher 并发等）、D 系列（措辞/测试稳定性）多条未决 spec 偏差。大部分 B 系列已在 Slice 3 代码中解决但措辞未写回 SPEC.md。C/D 系列属 Phase 3 工作流 C 范围，做的时候再决。 |
| **ARCHITECTURE.md 两阶段启动** | Slice 7a 引入了两阶段启动，应在 ARCHITECTURE.md §7 补一行说明。可选做。 |
| **F1/F2 决议在 SPEC 措辞** | PROGRESS.md 末尾的 F1（`random_oklch` 阈值 +0.05 guard）和 F2（bundle identifier 改 `com.github.pluiez.snippet` + 加 metadata）属实现细节，未触发 SPEC.md 措辞更新。如未来要严格按 SPEC §13 不变量 10 字面化（contrast ≥ 4.5 hard line），可加一句说明 stored 形式必须 round-trip 后仍 ≥ 4.5。可选做。 |
| **ts-rs binding 文件 git 化** | 本会话发现 4 个 binding 文件曾长期 untracked（dev mode 不报）。已手补并 commit。CI 跑 `cargo test --lib` 会重新生成同样内容，与 commit 不冲突。如果接手 agent 想严格起来，可加 CI step `git diff --exit-code src/lib/bindings/` 验证 bindings 实时同步（当前 CI 未做）。 |

---

## 总结

项目核心功能完整，动画 / 错误处理 / §13 不变量测试基础设施 / storage 层集成测试 / Windows `.msi`+`.exe` 打包 / GitHub 公开托管 / CI 均已落地。工作流 C 仅剩手动 smoke test。

祝接手顺利。
