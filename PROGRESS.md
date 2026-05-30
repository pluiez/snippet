# PROGRESS

实施进度。切片定义见 `TASKS.md`。

## Phase 1 — 脚手架

### Slice 0 — 项目初始化 ✅

收尾日期：2026-05-30。

**已落地**：

- Tauri v2 + React 19 + TS + Tailwind v4 + ts-rs 工程骨架
- pnpm 包管理；Vite dev server
- `tauri-plugin-single-instance`（双击 release `.exe` 两次验证：第二个进程立刻退出，第一个窗口被前置）
- 系统托盘图标（默认 Tauri logo，待替换）
- ts-rs 双端类型管线（示例类型 `AppInfo`；`pnpm bindings` 触发 `cargo test` 重新生成）
- tracing 日志（控制台输出）
- 主窗口 close → hide 拦截（按 ARCHITECTURE §7 关闭流程，X 不退出 app；dev 期间用 Ctrl+C 退出，生产期间用 Task Manager；显式"退出"UI 留 Slice 7）
- CLAUDE.md 记录 dev 命令

**仓库位置**：`C:\dev\snippet`（Windows 原生盘）；从 WSL 走 `/mnt/c/dev/snippet/`。WSL ext4 上跑 pnpm/tauri 因 UNC 限制 + 9p 慢已被排除（见 memory `project-layout`）。

### Slice 0.5 — 数据层骨架 ✅

收尾日期：2026-05-30。

**已落地**：

- 所有数据 schema（`Template` / `Variable` / `VariableType` / `Settings` / `ThemePreference` / `Bootstrap` / `VariableColorMap` / `TagColorMap` / `LastUsed`）— 全部 `#[derive(TS)]` + serde camelCase
- `TemplateSummary` 列表项类型（仅 IPC 返回，不持久化）
- 模块切分：`schema.rs` / `paths.rs` / `storage.rs` / `state.rs` / `commands.rs`
- bootstrap.json + dataFolder 解析（按 A1 决议：bootstrap 在 `<OS config>/Snippet/`，dataFolder 默认在 `<OS data>/Snippet/`；用 productName 做子目录而非 bundle identifier，详见 memory `tauri-app-paths`）
- 文件原子写（tmp + rename）
- `schemaVersion` 检查 + 损坏 / 版本错文件移到 `templates/.invalid/`
- 启动流程：bootstrap → dataFolder → ensure 结构 → load_or_init 各 config 文件（settings / 颜色 maps / last-used）→ 扫描模板入内存索引
- `AppState` 持有 `Mutex<HashMap<Uuid, Template>>`
- IPC 命令：`list_templates` / `get_template` / `save_template` / `delete_template`
- 主窗口最简列表 UI（pinned 在前、字母序、空状态引导、loading 态）
- 仓库根目录 `samples/` 下两份示例模板

**已验证**：

- ✅ 首次启动空状态、bootstrap.json + 默认 config 文件自动创建
- ✅ 把 `samples/` 两个 JSON 拷到 `%APPDATA%\Snippet\templates\` → 重启 → 主窗口显示"邮箱"（pinned 在前，ID 短哈希 `11111111`）+ "翻译模板"（ID `550e8400`），共 2 条

**未单独跑（实现已就位、风险低）**：

- 故意损坏 JSON 移到 `.invalid/`
- save_template IPC 持久化往返

**实施期间发现 + 修的问题**：

- Tauri 2 的 `app.path().app_data_dir()` / `app_config_dir()` 默认用 bundle identifier 作子目录（产生 `app.snippet/` 这种用户可见的反向域名目录）。改用 base `config_dir()` / `data_dir()` + 手动 `APP_SUBDIR = "Snippet"` 子目录（`paths.rs`），符合 SPEC §11 的 `%APPDATA%\<app>\` 预期。

**已知坑**：

- 初次跑前，`src/lib/bindings/` 下只有 `AppInfo.ts` 和 `TemplateSummary.ts` 的 placeholder。第一次 `pnpm bindings` 后会生成完整的 `Template.ts` / `Variable.ts` / `VariableType.ts` / `Settings.ts` / `ThemePreference.ts` / `Bootstrap.ts` / `VariableColorMap.ts` / `TagColorMap.ts` / `LastUsed.ts`。
- Slice 0.5 不消费 settings / 颜色 map / last-used，只是确保文件在磁盘上存在 + schema 版本对得上。Slice 1+ 才把它们装入 AppState 真正用。

## Phase 2 — 核心功能

### Slice 1 — 主窗口模板列表 + 新建 / 复制 / 删除 ✅

收尾日期：2026-05-30。

**已落地（后端）**：

- 新 IPC：`duplicate_template(sourceId) -> Template`（新 UUID、displayName 后缀 " 副本"、`isPinned` 重置 false、`createdAt/updatedAt = now`、`useCount` 归零）
- `save_template` 服务端自动 bump `updatedAt`，前端不用记得
- save / delete / duplicate 三个 mutate 命令统一 emit `templates-changed` 事件
- + `chrono` 依赖（用于服务端时间戳）

**已落地（前端）**：

- `App.tsx` 拆 view switcher（list ↔ edit），顶层 `listen("templates-changed")` 自动重拉 list_templates
- `TemplateList.tsx`：左侧 nav（占位"全部模板"）+ 主区列表 + 顶部"新建"按钮 + 每行 hover 显隐的 编辑 / 复制 / 删除 按钮
- `TemplateEditor.tsx`：displayName 输入 + body textarea + Save / Cancel + Cmd/Ctrl+Enter / Esc 快捷键（含 IME composition 检测）
- `ConfirmDialog.tsx`：简版 Tailwind modal，支持 destructive 样式、Esc 取消
- `Template.ts` / `Variable.ts` / `VariableType.ts` ts-rs binding placeholder（首次 `pnpm bindings` 后被覆盖）

**已验证**：

- ✅ 新建 / 编辑 / 复制 / 删除 / 重启持久 / `templates-changed` 事件自动刷新 — 全部符合预期

**已知坑 / 留给后续切片**：

- 编辑器只能改 displayName + body；变量编辑（Slice 2-3）、tag 编辑、isPinned 切换暂不可
- 列表排序由后端给定（pinned 在前 + 字母序），前端不再二次排序

### Slice 2 — 变量填充对话框 🚧 实现完成，待验证

实施日期：2026-05-30。

**已落地（后端）**：

- 新模块 `render.rs`：纯函数 `render(body, values) -> String` + `order_variables_by_body_appearance`，孤儿 placeholder 渲染为空
- 新依赖：`tauri-plugin-clipboard-manager`（读 / 写剪贴板）、`regex`（占位符匹配）
- `state.rs`：`AppState` 加 `last_used: Mutex<LastUsed>`
- `lib.rs`：注册 clipboard 插件；启动时把 LastUsed 装进 AppState
- 新 IPC `prepare_fill_dialog(id) -> FillDialogState`：调用瞬间读剪贴板（SPEC §4.5 时机）；按 SPEC §4.5 优先级（剪贴板 → last-used → staticDefault → 空）算每变量初值；enum 在每一步都校验 options 内成员（SPEC 只显式说 last-used 失效回退，这里扩展到剪贴板和 staticDefault，UX 更稳）；返回 body-first-appearance 排好序的变量列表
- 新 IPC `apply_template(id, values)`：Rust render → 写剪贴板 → 更新 lastUsedAt + useCount（不动 updatedAt，apply 不算编辑）→ 写 last-used.json（仅 rememberLastUsed 的变量、值非空）→ emit `templates-changed`

**已落地（前端）**：

- `App.tsx`：View 加 `fill` 分支 + toast 状态
- `TemplateList.tsx`：行内加"试用"按钮（Play icon）。零变量模板点击直接 apply_template + toast，不开对话框（SPEC §4.4）
- `TemplateFillDialog.tsx`（新）：左表单 / 右只读 live preview。text → 多行 textarea，enum → select。Cmd/Ctrl+Enter 提交，Esc 取消。required + 空 → "复制"按钮 disabled
- `Toast.tsx`（新）：底部居中 transient banner，2.5 秒淡出
- `src/lib/render.ts`（新）：JS 镜像 render，preview 用，避免每键一次 IPC
- ts-rs binding placeholder：`FillDialogState.ts`

**待用户验证**：

1. `pnpm bindings` 跑通（cargo test 顺便编译 chrono / regex / clipboard plugin / new commands）
2. 用 sample 的"翻译模板"（带 enum + text 变量）：点试用 → 弹对话框 → Language 默认空（首次：剪贴板 / last-used / staticDefault 都空）→ 选"日文"+ text 填"hello world" → 右栏实时显示"翻译成 日文：hello world" → Cmd+Enter → toast "已复制：翻译模板" → 剪贴板里是该字符串
3. 再次点同模板试用 → Language 默认变"日文"（last-used 生效）
4. 改 sample 把 Language options 改成 `["简体", "繁体"]` → 重启 → 试用 → Language 默认为空（"日文"不在新 options 里，SPEC §13 不变量 3 验证）
5. 复制一段文字到剪贴板，再试用同模板 → text 变量预填剪贴板内容（fillFromClipboard=true）
6. 用"邮箱"模板（无变量、pinned）：点试用 → 不开对话框，直接复制 + toast
7. 提交后查看 `%APPDATA%\Snippet\templates\<uuid>.json`：useCount 增加、lastUsedAt 有值；`%APPDATA%\Snippet\last-used.json` 有 `{"language": "日文"}` 之类
8. dev console 看到每次 apply 后都 emit `templates-changed` + 自动 list_templates

**实施期间发现 + 修的问题**：

- Slice 1 漏的 SPEC §2.2 偏差：编辑器里 body 显示原始 `{<guid>}` 占位符，应该按 SPEC §2.2 显示 `{<displayName>}`。新增 `src/lib/body.ts` 提供 `bodyToDisplay` / `bodyToStorage` 双向转换，`TemplateEditor.tsx` 加载时转显示形式、保存时转存储形式。Slice 3 完整 body 编辑器在此基础上加变量增删改 + 校验。

**已知坑 / 留给后续切片**：

- 变量结构编辑（增 / 删 / 改名 / 改类型 / 改 options / 改属性）属 Slice 3；现在只能改 displayName + body
- 剪贴板互斥 UI 强制（同模板一次最多一个 fillFromClipboard）属 Slice 3 编辑器范围
- enum 在 clipboard / staticDefault 步骤的失效回退是 SPEC §4.5 没明说但 Slice 2 顺逻辑实现的；不喜欢这个语义后续可调
- 渲染逻辑 dual-implemented（Rust + JS），改 placeholder 规则要两边同步
- `bodyToStorage` 对未匹配的 `{xxx}` 保留为字面文本（不自动建变量、不报错）；Slice 3 编辑器需要正面回应 B5

### Slice 3 — 模板编辑模式 + 变量 GUID 体系 ✅

收尾日期：2026-05-30。**后端不动**（save_template 已通用），全部前端工作。

**新组件**：

- `VariableEditor.tsx`：单变量卡片（displayName / type / options / required / fillFromClipboard / rememberLastUsed / staticDefault + 删除）
- `VariableList.tsx`：variables 数组 + "+ 添加变量" + 剪贴板互斥实施 + 删除变量回调
- `TagInput.tsx`：chip-style 标签输入（按 SPEC §2.3 大小写不敏感去重）
- `OptionsInput.tsx`：enum options chip 输入；删项触发 staticDefault 失效检查
- `lib/fill.ts`：`mergeFillValues` helper，按 GUID 复用旧 fill 值

**修改**：

- `TemplateEditor.tsx`：整合上面 + amber "编辑模式" banner + 主操作"保存模板"（amber 配色）；validation：displayName 唯一 + enum options 非空 + 显示名非空；变量 rename 同步改 body 显示形态；变量删除清 body 占位符
- `TemplateFillDialog.tsx`：加"解锁编辑"按钮 + 灰 "填充模式" banner（对照编辑模式 amber）
- `App.tsx`：View 加 returnTo。fill → edit 时缓存 state + 当前 values；edit save → re-prepare_fill_dialog → mergeFillValues → 回 fill；edit cancel → 直接回 fill 用缓存 values

**B 系列决议落地**：

- **B2** staticDefault 在 options 改动 / type 切换时自动清空（不报错）
- **B3** text ↔ enum 切换清空 staticDefault
- **B4** displayName 同模板内强制唯一 → inline 错误 + 阻止 save
- **B5** body 编辑器未匹配 `{xxx}` 保留为字面文本（Slice 2 punch-list 已落）
- **B6** 剪贴板互斥的清除是 transient — fill ↔ edit cancel 路径下用户回 fill 时变量是缓存版

**SPEC §13 不变量保证**：

- 不变量 1（GUID 稳定）：variables 编辑只改字段、GUID 不变；rename 时 body 显示同步改但存储仍是 GUID
- 不变量 2（删除清理）：VariableList 删除变量时同步清 body 里的 `{<displayName>}`
- 不变量 4（剪贴板互斥）：勾 B 时清 A + Toast "已从 A 转移"

**待用户验证**：

1. 从主窗口列表进编辑：amber banner + 黄色"保存模板"按钮，跟试用模式（灰 banner + 黑色"复制"按钮）视觉明显不一样
2. 编辑"翻译模板"重命名变量 Language → 语言：body 实时显示 `{语言}` → 保存 → 重启 → 试用，仍正常工作（GUID 稳定）
3. 删除变量 text → body 中 `{text}` 自动消失 → 保存 → 试用 → 没有 text 字段
4. 加新变量 + 在 body 里手敲 `{新变量}` 引用 → 保存 → 试用 → 新变量字段出现
5. 切 type text→enum 或 enum→text → staticDefault 自动清空
6. enum options 删除某项时若它是 staticDefault → staticDefault 自动清空
7. 同模板加两变量同名 → 红框 + 错误信息 + "保存"按钮 disabled
8. 给 B 勾"从剪贴板填充"，A 已勾 → A 自动清 + toast "已从 A 转移"
9. fill ↔ edit 往返：
   - 试用 → 填值 → 解锁编辑 → 改某变量名 → 保存 → 回 fill，刚填的值跟到新名字下（GUID）
   - 试用 → 填值 → 解锁编辑 → 改东西 → 取消 → 回 fill，所有改动丢弃但 filled 值还在
   - 试用 → 填值 → 解锁编辑 → 新增变量 → 保存 → 回 fill，新变量按默认值级联（多半为空）

**实施期间发现 + 修的问题**：

- 重命名变量时 body 状态被错误重写：原方案在编辑器里把 body 存为 `{<displayName>}` 显示形式，rename 时强制重写 body 字符串，遇到名字冲突会让多个 `{X}` 占位符全部解析到同一个 GUID，第二个变量从 body 里彻底消失（试用时被 prepare_fill_dialog 过滤掉）。**改为 body 状态始终保持 `{<guid>}` 存储形式，textarea 显示靠每次渲染派生**。这样 SPEC §13 不变量 1 自然成立——rename 完全不动 body 状态，displayName 改变 textarea 自动反映。删除变量按 GUID（不是 displayName）从 body 清占位符，名字冲突也不会误删。
- 验证错误位置不对：第一版只标第二个重复项；第二版改成两个都标，但用户反馈"应该只标当前编辑的那个"才符合直觉。最终：追加 `lastEditedGuid` 状态（仅在 displayName 改动时更新），重复错误只展示在它身上；保存阻止逻辑独立用 `hasBlockingErrors` 检查任意重复，确保即使没有错误展示也能拦住保存。

**已知坑 / 留给后续切片**：

- 编辑器不过滤孤儿变量（variables 数组有但 body 不引用的）— 用户可在编辑器中看到并手动清理；prepare_fill_dialog 在试用路径上已过滤
- 解锁编辑后取消 = 丢弃全部 template 改动；如果用户改了很久误按 Esc 会很痛。Phase 3 工作流 B 加 dirty 检测 + 二次确认
- amber 视觉差异化将来做主题适配（明 / 暗）时要确保两个主题下都明显
- textarea 是普通文本框，没有变量引用的"原子化"概念。用户在 `{Language}` 中间删字符会破坏引用（变成字面文本）；要彻底解决得换成 chip-style 富文本编辑器，超出本切片范围

### Slice 4 — Palette + 全局热键 + 搜索 + 排序 ✅

收尾日期：2026-05-30。

**新依赖**：

- `tauri-plugin-global-shortcut` v2 — 全局热键
- `nucleo-matcher` v0.3 — fuzzy 评分
- `pinyin` v0.10 — 中文转拼音
- `windows` v0.58（Windows-only target）— `GetForegroundWindow`

**已落地（后端）**：

- 新模块 `palette.rs`：hotkey handler（**HWND 第一步同步捕获**，再做窗口互斥）+ show/hide palette + show_main_window
- 新模块 `search.rs`：SPEC §7 实现。空 query → pinned-first + lastUsedAt desc。非空 → 每字段（name / tag / body）跑直接 + 全拼 + 首字母三种 haystack，取最高分；模板总分 = `max(name×1.0, tag×0.8, body×0.3)`；同分 lastUsedAt desc
- `render.rs::body_for_search` — body 里 `{<guid>}` 替换为 `{<displayName>}` 给搜索用（A2）
- `state.rs` + `cached_hwnd: AtomicIsize`（Slice 6 autoPaste 待用）
- `commands.rs` + `search_templates` / `set_pinned` / `show_palette` / `hide_palette` / `show_main_window`
- `lib.rs` 注册 global-shortcut 插件 + `Ctrl+Alt+Space` 默认热键 + 新命令；tray 与 single-instance 改用 `palette::show_main_window`（隐藏 palette + 前置 main）
- 所有窗口 close → hide（main + palette）

**已落地（前端）**：

- `main.tsx` 按 `getCurrentWindow().label` 路由：main → `<App />`，palette → `<Palette />`
- 新 `Palette.tsx`：左 40% 搜索 + 列表，右 60% preview。键盘 ↑↓ Enter Esc + Cmd/Ctrl+↑↓（preview 滚动）。view stack：`search` / `fill` / `edit`，fill / edit 复用 Slice 2-3 的组件
- 顶部 6px drag region（borderless 窗口可拖）
- listen `palette-shown` 事件 → 重置 search view + 清空 query（仅 hotkey 重新打开时触发，已开则只 refocus 不重置——SPEC §4.9 "保持当前状态"）
- `App.tsx` listen `main-window-glow` → 0.9s 内层 amber 4px ring（SPEC §4.9 视觉反馈）
- `TemplateList.tsx`：行首 pin 图标改成可点按钮，调 `set_pinned`

**新 Tauri 窗口**：

- `palette` label：800×520、decorations: false、alwaysOnTop、center、visible: false、skipTaskbar、resizable: false
- `capabilities/default.json` windows 数组加 palette

**SPEC §13 不变量保证**：

- 7（搜索权重）：硬编码 1.0 / 0.8 / 0.3 + max
- 8（排序稳定）：同分 lastUsedAt desc
- 9（窗口互斥）：hotkey / tray / single-instance 三路径都按 §4.9 表
- 12（pinyin 多音字）：依赖 pinyin crate 单字默认音

**ARCHITECTURE §6 时序契约**：hotkey callback 第一行同步 `GetForegroundWindow()` → 存 `cached_hwnd` → 之后才做窗口互斥 / show palette / emit 事件。

**待用户验证**：

1. `pnpm bindings` 跑通（同时编译验证 windows / pinyin / nucleo / global-shortcut 依赖）
2. `pnpm tauri dev` → 任意应用按 `Ctrl+Alt+Space` → palette 弹出（无边框、置顶、屏幕中央）
3. 搜索：
   - "翻译" → "翻译模板" 命中（displayName 直接）
   - "fanyi" → 同样命中（拼音全拼）
   - "fy" → 同样命中（首字母）
   - 空 query → pinned 在前（如"邮箱"），其它按 lastUsedAt 倒序
4. ↑↓ 切换选中，preview 联动；Cmd/Ctrl+↑↓ preview 滚动
5. Enter 选中零变量模板 → 直接复制 + palette 关闭
6. Enter 选中带变量模板 → palette 内变身为填充对话框
7. 填充对话框中"解锁编辑"→ 编辑模式（amber banner）→ 改变量保存 → 回填充，值跟到新名字下
8. 主窗口打开时按热键 → 主窗口前置 + 内层 amber ring 闪 0.9s，palette **不**弹
9. palette 打开时点托盘 → palette 关闭、主窗口前置
10. 主窗口列表行首 pin 图标点击 → 切换 pinned + 列表自动重排（pinned 上浮）

**已知坑 / 留给后续切片**：

- Pinyin 多音字依赖单字默认音；compound word（"行业 hangye"）匹配靠 nucleo fuzzy 兜底，可能不严格
- palette drag region 只有 6px 顶部薄条；多显示器场景按 OS 默认（SPEC v1 不要求）
- 颜色 map 仍未消费（Slice 5 真做）：palette 预览 tag pill 灰底
- autoPaste 仍未实现（Slice 6）；HWND 已捕获待用
- 自定义热键 UI（Slice 7）；当前热键写死，settings.json 里的 hotkey 字段未消费

### Slice 5 — 颜色系统 + 中央 map + 管理页 ✅

收尾日期：2026-05-30。

**新依赖**：

- `rand = "0.8"`（OKLCh 随机生成）

**已落地（后端）**：

- 新模块 `color.rs`：`random_oklch()` 按 SPEC §6.2 生成（L 0.45-0.65 / C 0.10-0.20 / H 0-360）+ OKLab→sRGB 转换 + WCAG 对白对比度 ≥ 4.5:1 校验，不达标重采样。`reconcile_colors()` 双向对齐：补 templates 里有但 map 里没有的条目（覆盖手动放置或外部迁移的模板）+ 删 map 里有但 templates 不引用的孤儿条目（SPEC §6.6）
- `state.rs` + `variable_colors: Mutex<VariableColorMap>` + `tag_colors: Mutex<TagColorMap>` + 对应 `_path()` 助手
- `TemplateSummary` 类型加 `tags: Vec<String>` 字段（让主窗口列表能渲染 tag pill + 触发筛选）；`list_templates` / `search_templates` 都填充
- `commands.rs` + `get_variable_colors` / `get_tag_colors` / `save_variable_colors(map)` / `save_tag_colors(map)` / `random_color() -> String`；`save_template` / `duplicate_template` 持久化后调 `ensure_colors_for_template`，新增条目即落盘 + emit `colors-changed`
- `lib.rs` 注册新命令；`init_app_state` 末尾跑一次 GC（启动）；`run()` 改用 `.build() + .run(closure)` 模式，在 `RunEvent::ExitRequested` 跑关闭 GC（SPEC §6.6 + ARCHITECTURE §7）

**已落地（前端）**：

- 新 `lib/colors.tsx`：`ColorMapsProvider` Context Provider + `useColorMaps()` hook + `tagColor` / `variableColor` helpers。每窗口一个实例，listen `colors-changed` 自动 refresh
- `main.tsx` 在 `<Component />` 外包 `<ColorMapsProvider>`，main 和 palette 两个窗口各自 wrap
- 新 `TagPill.tsx`：`onClick` 可选 prop —— 主窗口 = 可点（cursor pointer + hover dim），palette = 静态（cursor default）
- 新 `BodyWithVariableChips.tsx`：解析 body 中 `{<guid>}`，渲染成 `rounded-sm` 方角色块 chip（区别于 tag 的 `rounded-full` pill），无 hover；palette 预览专用
- 新 `ColorManagement.tsx`：两 tab（变量颜色 / tag 颜色）+ 暂存模式（dirty 状态分别追踪 vars/tags）+ 单项刷新（调 `random_color`）+ 自定义（HTML `<input type="color">`，hex 形式）+ 重置全部 + 保存 / 取消
- `App.tsx`：View 加 `colors` 分支；新 `MainNav` 组件（"全部模板" + tag filter chip + "颜色管理"）；`tagFilter` 状态；`templates` 用 `useMemo` 按 tag 过滤；left nav 跨 list 和 colors 视图共享
- `TemplateList.tsx`：每行加 tag pills（`TagPill` 互动版，传 `onTagClick` 触发 filter）；nav 移到 App.tsx；超过 3 个 tag 显示 "+N"
- `TagInput.tsx`：chip 改用 tagColor 着色（白字 + 颜色背景），删除按钮改 `text-white/70 hover:text-white`
- `TemplateFillDialog.tsx`：变量 label 旁加 8px 圆色点（按 SPEC §4.5 "字段标签的颜色按 variableColorMap"）
- `Palette.tsx` Preview：tag 用 `TagPill` 静态变体（无 onClick）；body 用 `BodyWithVariableChips` 渲染色块

**SPEC §13 不变量保证**：

- 5（GC 不误删）：`run_gc` 只删 key 不在 used 集合的条目
- 6（GC 删孤儿）：实现 + 落盘
- 11（GC 收敛）：启动 + 关闭循环跑无 side-effect

**待用户验证**：

1. `pnpm bindings` 跑通（编译验证 rand 依赖 + 新命令 + RunEvent 重构）
2. `pnpm tauri dev` 启动，主窗口左 nav 看到 "全部模板" + "颜色管理"；点 "颜色管理" 进入管理页
3. 编辑某模板，加新变量名（例如 "tone"）+ 新 tag（例如 "test"）→ 保存 → 颜色管理页两 tab 中都能看到对应条目带颜色
4. 主窗口列表里：每行尾部显示 tag pills（带颜色），点 tag pill → 列表筛选；左侧 nav 出现 amber tag chip + ✕，点 ✕ 清除筛选
5. 试用带变量模板：填充对话框里每个变量 label 旁有色点；palette 预览中 body 显示 `{Language}` 这种色块 chip（方角，跟 tag 圆角 pill 视觉区别明显）
6. 颜色管理页：刷新单项 → 颜色变；自定义 → 弹系统色板；重置全部 → 全部刷新；保存 → 主窗口和 palette 所有 UI 同步刷新；取消 → 改动丢弃
7. 删除某 tag 的所有引用模板 → 重启 → GC 跑掉该 tag 颜色（在管理页确认）
8. palette 中 tag 不可点击（cursor: default、hover 无变化），主窗口 tag 可点击（cursor: pointer + hover dim）

**已知坑 / 留给后续切片**：

- 颜色 map 的 hex（来自系统色板）和 oklch（随机生成）混存；CSS 都接受，用户可见无差异
- 明暗主题适配 v2 候选（不在本切片范围）
- 编辑器变量卡片的 displayName 没加色点（仅 fill 对话框 label 加了）；如果需要可在 Slice 5 punch-list 补
- 颜色管理页用系统色板（`<input type="color">`）只能选 sRGB hex；要选 oklch 需要自己写色板（v2）
- run_gc 在 ExitRequested 触发；用户用 Task Manager 强杀时不跑 GC，下次启动跑一次会补上
- 我修改 `lib.rs` 的 `.run()` 结构（从一行变成 build+run callback），如果 Tauri 2 版本里 RunEvent enum 名字略不同（`Exit` / `ExitRequested` / `WindowEvent::Close`），编译会报错——按报错改

### Slice 6 — 输出（剪贴板 + 自动粘贴） ✅

收尾日期：2026-05-30。已验证 Notepad / 浏览器地址栏 / Google 搜索框；Sublime Text 是已知不兼容（详见已知坑）。

**新依赖**：

- `enigo = "0.3"`（Windows-only target）：模拟键盘输入

**已落地（后端）**：

- 新模块 `auto_paste.rs`（`#[cfg(target_os = "windows")]`）：`paste_into(hwnd_raw)` 调 `SetForegroundWindow` → sleep 50ms 让焦点 settle → `enigo` 发 Ctrl+V；非 Windows 平台 stub 返回错误（caller 自动降级为 clipboard-only）
- `state.rs` + `settings: Mutex<Settings>`（Slice 0.5 加载到 `_settings`，现在真正装入 state）+ `settings_path()`
- `commands.rs`:
  - 新类型 `ApplyOutcome { pasted: bool, reason: Option<String> }`，reason 为 `"disabled"` / `"failed"` / null
  - `apply_template` 扩展：写完剪贴板 + 更新 lastUsedAt / use_count / last-used.json 后，读 `settings.auto_paste`；若 enabled，从 `cached_hwnd` 取 HWND，调 `auto_paste::paste_into`；失败 warn 日志 + 降级 clipboard-only。返回 `ApplyOutcome`
  - 新 `get_settings` / `save_settings`（save 后 emit `settings-changed`）
- `lib.rs` 把 `Settings` 真正传入 `AppState::new` + 注册新命令

**已落地（前端）**：

- 新 `lib/settings.tsx` Context Provider（类似 colors），listen `settings-changed`
- `main.tsx` 在 `<ColorMapsProvider>` 内嵌套 `<SettingsProvider>`
- 新 `Settings.tsx` 极简页：只有 autoPaste 复选框 + 简短说明 + 暂存模式（dirty 比较 JSON.stringify）+ 保存 / 取消；底部注释"其它设置（热键 / 主题 / dataFolder）留待 Slice 7"
- `App.tsx`：View 加 `settings` 分支；MainNav 加"设置"项（齿轮图标）；apply 调用处现在拿 `ApplyOutcome` 决定 toast 文案：
  - `pasted=true` → 不显示 toast（焦点已离开主窗口，没人看）
  - `pasted=false, reason="disabled"` → "已复制：{name}"（autoPaste 关 = 用户默认行为）
  - `pasted=false, reason="failed"` → "已复制：{name}，请手动粘贴"（SPEC §4.6）
- `Palette.tsx`：apply 后用 `finalizeApply()` 处理 outcome —— `pasted=true` 或 `reason="disabled"` 立即 `hide_palette`；`reason="failed"` 显示 toast 1.5s 后再 hide。新 Toast 实例（palette 自己的）

**bindings**：`Settings.ts` / `ThemePreference.ts` / `ApplyOutcome.ts` placeholders

**OS 集成实现细节**：

- HWND 转换：`HWND(hwnd_raw as *mut std::ffi::c_void)`（windows 0.58 的 HWND 是 newtype around `*mut c_void`）
- `SetForegroundWindow` 返回 `BOOL`，用 `.as_bool()` 判断成功 / 失败
- enigo Ctrl+V：`Key::Control` Press → `Key::Unicode('v')` Click → `Key::Control` Release。Control 修饰键持续期间 v 字符被 OS 翻译为 paste 快捷键
- 50ms sleep 在 focus → input 之间（焦点切换需要时间）

**待用户验证**：

1. `pnpm bindings` 跑通（编译验证 enigo + auto_paste + 新 IPC）
2. `pnpm tauri dev` 启动，主窗口左 nav 看到"设置"项；进设置页 → 看到"自动粘贴 [启用]"复选框 + 提示文字
3. autoPaste **关闭**状态下：palette 选模板复制 → toast "已复制：X"，剪贴板有内容（默认行为不变）
4. autoPaste **开启**状态：保存设置 → 切换到其它 app（如记事本）→ 按 `Ctrl+Alt+Space` 开 palette → 选模板回车 → 焦点切回记事本 + 内容自动粘贴（无 toast）
5. autoPaste 开启但失败场景（少见，触发条件如 cached_hwnd 失效）：toast 显示"已复制：X，请手动粘贴"
6. 主窗口"试用"路径下，autoPaste 开 + 焦点在主窗口本身：SetForegroundWindow 自己到自己，paste 到主窗口（如果输入框聚焦能粘贴）；可能行为奇怪但不崩
7. dev console 检查日志：每次 apply 应有 `pasted=true/false reason=...` 字段

**已知坑 / 留给后续切片**：

- Windows `SetForegroundWindow` 在"前台保护"机制下并非 100% 成功；如果用户在 palette 出现后切了其它 app，cached_hwnd 还是热键时刻的 HWND，paste 会粘到错误窗口或失败
- 第一次没按过热键就 试用（直接从主窗口列表）→ `cached_hwnd=0` → autoPaste 失败 → 降级 toast（这是符合预期的，热键 = 来源窗口的 implicit 契约）
- `Pasted` outcome 不显示 toast 的设计放弃了"已粘贴：X"的确认反馈；用户看到目标 app 已粘贴的内容即是反馈
- 完整设置页（热键、主题、dataFolder）留 Slice 7
- macOS / Linux 平台的 auto-paste 没实现（stub 返回错误，会全降级到 clipboard-only）
- 部分 app（Sublime Text、可能 Vim / 游戏 launcher / 自绘 Win32 工具）会忽略 `SendInput` 模拟的 Ctrl+V。enigo 在 API 层返回成功，backend `outcome.pasted=true`，前端不显示 toast，但目标 app 实际没收到 → false-positive。无法在 backend 检测。用户需手动 Ctrl+V（剪贴板内容还在）。这是 OS 级输入模拟的固有限制，非 bug

### Slice 7a — Onboarding 流程 + dataFolderPath 设置入口 🚧 实现完成，待验证

实施日期：2026-05-30。**Slice 7 拆分为 7a / 7b / 7c**（详见 plan 与本文件下方"Spec 决议日志"E1）。

**新依赖**：

- `tauri-plugin-dialog` v2（Cargo + npm `@tauri-apps/plugin-dialog`）— 文件夹选择器
- `tempfile` v3（dev-deps）— `onboarding.rs` 单测用

**已落地（后端）**：

- 新模块 `src-tauri/src/onboarding.rs`：`classify_path()` + `needs_onboarding()` 纯函数 + 6 个 `#[test]`（DoesNotExist / Empty / ValidSnippet via templates dir / ValidSnippet via settings.json / OccupiedByOther / 文件不是目录）
- `schema.rs`:
  - `Bootstrap` 加 `onboarding_complete: bool`（默认 false），`#[serde(default = "default_onboarding_complete_for_legacy")]` 让旧 bootstrap.json（无此字段）反序列化为 `true`，避免老用户被强拉去重做 onboarding
  - 新 enum `DataFolderStatus`（`DoesNotExist` / `Empty` / `ValidSnippet` / `OccupiedByOther`），camelCase，PartialEq+Eq
- **lib.rs 两阶段启动改造**（本切片最大结构改动）：
  - `init_app_state` 拆为 `init_bootstrap`（Phase A，始终跑）+ `init_full_state`（Phase B，构 AppState）
  - `register_default_hotkey` 抽成独立函数（Slice 7b 替换为可配置版本）
  - 新 `pub fn complete_onboarding(app, set_data_folder)`：写 bootstrap → init_full_state → manage → register hotkey → hide onboarding 窗口。给 try_state 加 idempotency 保护
  - setup 中：先 init_bootstrap → 总是建 tray（tray click handler 在 AppState 未 manage 时 fallback 到显示 onboarding 窗口）→ 按 needs_onboarding 分支：是 → 显示 onboarding 窗口、Phase B 推迟；否 → 走 Phase B + 注册热键
  - close handler 加 `"onboarding"` 分支 → `app.exit(0)`（X = 退出 app；与 main/palette 的 hide 语义不同）
  - single-instance handler 在 AppState 未 manage 时也 fallback 到显示 onboarding（避免双开试图打开主窗口）
  - ExitRequested 已有 try_state 保护（Slice 5 已加），onboarding cancel 退出时不跑 GC
- 新 IPC 命令（9 个，注册在 `invoke_handler`）：
  - `default_data_folder() -> String` —— 解析 OS 默认路径给 UI 显示
  - `current_data_folder() -> String` —— 返回当前正用的 dataFolder（仅 AppState 已 manage 时可用）
  - `validate_path_for_new(path) -> DataFolderStatus` / `validate_path_for_import(path) -> DataFolderStatus`
  - `complete_onboarding_default()` / `complete_onboarding_custom_new(path)` / `complete_onboarding_import(path)` —— 写 bootstrap + 调 `crate::complete_onboarding`
  - `set_data_folder_path(path: Option<String>)` —— 设置页改路径专用，只动 bootstrap 不重启
  - `exit_app()` —— 设置页改 dataFolderPath 后用户确认退出时用

**已落地（前端）**：

- 新 Tauri 窗口 `onboarding` (560×480, center, resizable: false, visible: false) in `tauri.conf.json`
- `capabilities/default.json` windows 加 `"onboarding"`，permissions 加 `"dialog:default"`
- 新 `src/Onboarding.tsx`：三 OptionCard 选 default / customNew / import，每张卡选中后展开内部 PathPicker（dialog 选目录 → validate → 显示路径 + 状态徽章），底部"开始使用" amber 按钮按校验状态启停
- `src/main.tsx`：label === "onboarding" 时只渲染 `<Onboarding />`，跳过 SettingsProvider / ColorMapsProvider（避免在 AppState 未 manage 时 spam `get_settings` 错误）
- `src/Settings.tsx`：从 Slice 6 桩扩展，加"数据文件夹"行（显示 `current_data_folder` + 更改按钮 → dialog → validate_path_for_import → ConfirmDialog → set_data_folder_path → 浏览器 confirm "是否立即退出" → `exit_app`）。改路径仅接受已有 Snippet 数据（创建新空库走 onboarding 流程，不在设置页里重复）
- `src/lib/bindings/DataFolderStatus.ts` placeholder（首次 `pnpm bindings` 后被覆盖；同时会新生成完整 `Bootstrap.ts`）

**SPEC §13 不变量保证**：

- 不变量 9（窗口互斥）：第三窗口 onboarding 仅在未完成 onboarding 时显示；main 不可见 + palette 未注册热键 → 互斥规则自然成立。完成 onboarding 后 onboarding 立即 hide，回到 main/palette 二者互斥的 Slice 4 状态

**待用户验证**：

⚠️ **首次跑前必做**：
1. `pnpm install`（安装 `@tauri-apps/plugin-dialog`）
2. `pnpm bindings`（编译 + 触发 ts-rs 重新生成 `Bootstrap.ts` / `DataFolderStatus.ts`，验证 dialog 插件、tempfile dev-dep、新模块、所有 IPC 命令)
3. `pnpm tauri dev` 启动

验收场景：

1. **首次启动 (default 路径)**：清空 `%APPDATA%\Snippet\` → 启动 → onboarding 窗口出现，三个 OptionCard 默认选中"使用默认路径"+ 显示默认路径文字 → 点"开始使用" → 窗口关闭 → 托盘有图标 → 点托盘 → 主窗口空状态 → `%APPDATA%\Snippet\bootstrap.json` 含 `"onboardingComplete": true, "dataFolderPath": null` + `templates/` 等空结构生成
2. **首次启动 (自定义新建)**：清空 → 启动 onboarding → 选"指定路径新建" → 选 `D:\test-snippet`（空目录）→ 显示绿色 "目录为空，可以使用" → "开始使用" → 主窗口启动 → `bootstrap.json` 含 `"dataFolderPath": "D:\\test-snippet"`
3. **首次启动 (自定义新建-冲突拒绝)**：选"指定路径新建" → 选 `D:\Downloads`（有内容）→ 显示红色 "目标路径已有内容..." → 按钮 disabled
4. **首次启动 (导入)**：先在 `D:\backup\` 手动放 sample 模板 + `templates/` 目录 → 清空 `%APPDATA%\Snippet\` 启动 → onboarding → 选"从已有路径导入" → 选 `D:\backup\` → 绿色 "检测到 Snippet 数据" → "开始使用" → 主窗口显示导入的模板
5. **导入-路径无效**：选"从已有路径导入" → 选空目录 → 红色 "该路径不是 Snippet 数据文件夹..." → disabled
6. **onboarding X 掉**：onboarding 期间点 X → app 直接退出（dev console: 无 GC 日志）→ 重启仍弹 onboarding
7. **设置页改路径**：完成 onboarding 后，主窗口设置页看到"数据文件夹"行显示当前路径 → 点"更改..." → 选另一个已有 Snippet 路径 → ConfirmDialog "更改数据文件夹" 弹出 → 点"保存" → 浏览器 confirm "是否立即退出" → 选"确定" → app 退出 → 手动重启 → 使用新路径
8. **旧 bootstrap.json 兼容**：现有开发环境的 bootstrap.json（不含 onboardingComplete 字段）→ 启动 → **不弹** onboarding（serde 默认值是 true）→ 正常进入主窗口 → Slice 1-6 全部功能照常工作

**实施期间发现 + 修的问题**：

- Onboarding 期间 AppState 未 manage，但 single-instance 二次启动 handler 默认会调 `palette::show_main_window`（其内部用 try_state，不会 panic 但语义不对）。改为先检测 try_state，未 manage 时 fallback 显示 onboarding 窗口。tray click 也同处理
- ts-rs 的 `DataFolderStatus` 在 camelCase 下变成 `"doesNotExist" | "empty" | "validSnippet" | "occupiedByOther"`，TS 字符串字面量类型 —— 校验时不用 import enum，直接用 string compare（如 `customStatus === "validSnippet"`）

**已知坑 / 留给后续切片**：

- 设置页改 dataFolderPath 只接受已有 Snippet 数据。如果想"指定空路径新建作为新数据集"，目前只能：清掉 bootstrap.json → 重启 → 走 onboarding。这个 punch-list 可放 Phase 3 工作流 B
- onboarding 期间 dev console 还是会因 React StrictMode 双 render 而看到 `default_data_folder` 调两次。无害
- `exit_app` IPC 是新加的（plan 没列），Slice 7b 也会需要它做"改热键失败需重启"路径（实际上 7b 是即时 re-register 不需要重启，所以可能不会用到）
- `validate_path_for_import` 检测 Snippet 结构靠几个 marker 文件（templates/、settings.json 等任一）；如果用户的备份目录刚好只有空 templates/ 子目录，也会被认作 ValidSnippet（合理 —— 那个目录确实"是" Snippet folder，只是没模板）

## Phase 3 — 打磨与发布

未开始。

---

## Spec 决议日志

记录在写实现前对 SPEC / ARCHITECTURE 的修订。

- **A1**（2026-05-30）：settings 拆 `bootstrap.json`（每设备本地）+ `settings.json`（在 dataFolder 内、跨设备同步）。已写回 `SPEC.md` §3.5 / §11 / §12，`ARCHITECTURE.md` §3.3 / §5。
- **A2**（2026-05-30）：body 搜索索引建在变量替换为 displayName 后的形态上；模板保存时该模板的索引重建。已写回 `SPEC.md` §7.1，`ARCHITECTURE.md` §3.1 / §5。
- **B 系列**（孤儿变量 / staticDefault 失效 / 类型变更迁移 / displayName 唯一 / body 编辑器未识别占位符 / 剪贴板互斥 transient 语义）：未决，留到对应切片处理。
- **C 系列**（必填+剪贴板空死锁 / watcher+颜色页并发 / autoPaste 失败检测 / migration 失败 / 剪贴板读失败回退）：边做边补。
- **D 系列**（"对话框" 措辞、§13 不变量 7 测试稳定性、ARCHITECTURE §6 GC 周期归类）：未处理。
- **E1**（2026-05-30，Slice 7a 开工时）：原 Slice 7 拆为 7a / 7b / 7c —— 7a Onboarding + dataFolderPath、7b 完整设置页 hotkey + autoPaste 整合、7c 主题切换 + 全组件 dark 适配。原因：原切片范围过大（~2 天），单切完成前无法分段 demo / 回滚；blast radius 集中在三块独立基础设施，拆开后每块可独立 tag、独立验收。Onboarding 引入两阶段启动（Phase A bootstrap → Phase B AppState lazy manage）作为整个 7 系列的结构前置，落地在 lib.rs。TASKS.md 拆分写入推迟到 7c 完成后一次性更新（避免文档/代码不同步窗口）。
