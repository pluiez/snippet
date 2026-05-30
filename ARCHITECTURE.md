# Snippet — 技术架构 (ARCHITECTURE)

> 描述实现层的架构决定：栈选型、进程边界、模块组织、关键依赖、关键时序约束。SPEC.md 描述"做什么"，本文档描述"在什么基础上、怎么搭骨架"。具体类型定义、函数签名、文件命名、代码组织细节由 coding agent 在实现阶段确定。

---

## 1. 技术栈

### 1.1 核心选型

| 层 | 选型 |
|---|---|
| 桌面应用框架 | Tauri v2 |
| 目标平台 | Windows 10/11（v1）；macOS 后续 |
| 后端语言 | Rust (latest stable) |
| 前端框架 | React 19 + TypeScript |
| 构建工具 | Vite |
| 样式 | Tailwind CSS |
| 组件 / design system | shadcn/ui |
| 状态管理 | Zustand |
| 动画 | Framer Motion |
| 双端类型同步 | `ts-rs`（Rust 类型生成 TS 定义） |

### 1.2 关键理由

- **Tauri vs Electron**：内存占用与唤起延迟显著优于 Electron，对 palette 体验关键。Windows 上 auto-paste 等系统集成无需 Accessibility 权限。
- **React 19**：AI coding 工具的训练数据密度最高，agentic 开发流程下输出质量最稳定；shadcn/ui 是组件模式与 design system 的事实标准；Tauri 生态中可参考的在野案例最多。
- **Zustand**：轻量、API 干净，对桌面应用规模的状态够用，AI 工具熟悉度高。
- **OKLCh on WebView2**：Windows 上 Tauri 使用 WebView2 (Chromium)，原生支持 `oklch()` CSS 函数。SPEC 中颜色字符串直接作为 CSS 消费，无需转换。

---

## 2. 进程与模块边界

### 2.1 职责划分

**Rust 后端是数据的事实来源**。前端是它的视图与控制层。

后端负责：

- 数据持久化（模板、颜色 map、设置、上次使用值）
- 全局热键监听 + 焦点窗口捕获
- 剪贴板读写 + 自动粘贴模拟
- Pinyin 索引 + Fuzzy 匹配 + 加权评分
- 颜色生成（OKLCh 随机 + 对比度校验）
- 颜色 map 周期性 GC
- 外部文件变更监听
- IPC 命令处理

前端负责：

- 所有 UI 渲染（palette / 主窗口 / 填充对话框 / 编辑模式 / 设置 / 颜色管理 / onboarding）
- 键盘交互、表单输入的本地 transient 状态
- 实时渲染预览
- 调用后端 IPC、监听后端事件
- 主题切换的 CSS 层切换

前端**不**直接读写文件、**不**直接操作系统 API。

### 2.2 IPC 模型

Tauri v2 提供两套通道：

- `invoke`：前端 → 后端的请求-响应（async）
- `emit` / `listen`：双向事件总线

设计原则：

- **命令粒度是"业务动作"**，不是"文件操作"。例如 `save_template`、`render_template`，不是 `write_file`
- **类型契约双端共享**：Rust 端定义 schema 并 derive `ts-rs`，前端 import 生成产物，CI 校验同步

---

## 3. 模块组织

### 3.1 Rust 后端的功能域

按职责切分（具体文件、类型、内部函数由实现确定）：

- **storage**：模板与各 map 的读写、内存索引、原子写入、schema 迁移
- **commands**：所有 IPC 命令的入口分组
- **hotkey**：全局热键注册、焦点窗口 (HWND) 捕获
- **paste**：剪贴板写入、输入事件模拟
- **search**：pinyin 索引、fuzzy 匹配、加权评分。body 字段的索引建在变量占位符替换为 displayName 后的形态上（按 SPEC 7.1）；storage 模块完成模板写入后必须先触发该模板的索引重建再返回 IPC 响应
- **color**：颜色生成、对比度校验
- **gc**：颜色 map 周期性 GC
- **watcher**:数据文件夹外部变更监听
- **app_state**:跨模块共享的运行时状态（如缓存的 HWND）

### 3.2 前端的功能域

- **windows**：每个 Tauri 窗口对应一个 entry 组件（palette / main / onboarding）
- **routes**：主窗口内的子页面（模板列表 / 编辑 / 颜色管理 / 设置）
- **components**：跨页面复用的 UI 组件
- **stores**：Zustand stores，按数据域切分（templates / colors / settings / lastUsed）
- **lib**：IPC 调用封装、`ts-rs` 生成的 TS 类型、工具函数
- **styles**：design tokens（继承 shadcn/ui）+ 全局样式

### 3.3 数据持久化

文件级布局已在 SPEC 第 3.6 节定义。架构层面的要点：

- 所有 JSON 顶层带 `schemaVersion` 字段
- 写入采用 tmp + rename 的原子模式
- `bootstrap.json` 在 OS 用户配置目录，是唯一不在 dataFolder 内的 app 文件；其余文件均在 `<dataFolder>` 内
- 启动时全量加载到内存索引，运行时所有读取从内存
- 外部变更通过 `notify` crate 监听；app 自身写入触发的事件需要去重处理
- schema 迁移机制实现为版本号驱动的函数链；v1 实现为 stub

### 3.4 IPC 命令分组

按功能域组织，具体命令签名由实现确定：

- 模板 CRUD
- 模板渲染与使用记录更新
- 搜索
- 颜色 map 查询与修改
- 设置读写与热键重注册
- 输出（剪贴板 / 自动粘贴）
- 上次使用值查询
- Onboarding（数据文件夹检测 / 初始化 / 导入）
- 窗口控制

后端通过事件通知前端：模板增 / 改 / 删、颜色 map 更新、设置变更、热键冲突、数据文件夹路径变更等。

### 3.5 多窗口架构

Tauri 多窗口配置中至少包含：

- 主窗口（main）：按需创建
- Palette 窗口：无边框、置顶、屏幕居中、初始隐藏，按需显示
- Onboarding 窗口：首次启动专用，完成后销毁

窗口生命周期由后端管理，前端通过 IPC 请求显示 / 隐藏 / 销毁。窗口互斥逻辑（SPEC 4.9）的实现位于后端窗口控制 commands。

---

## 4. 关键依赖

### 4.1 Rust crate

| 用途 | crate |
|---|---|
| Tauri 主框架 | `tauri` v2 |
| 异步运行时 | `tokio` |
| 单实例锁 | `tauri-plugin-single-instance` |
| 全局热键 | `tauri-plugin-global-shortcut` |
| 剪贴板 | `tauri-plugin-clipboard-manager` 或 `arboard` |
| JSON 序列化 | `serde` + `serde_json` |
| 双端类型导出 | `ts-rs` |
| UUID | `uuid` |
| 文件监听 | `notify` |
| 输入模拟 | `enigo` |
| Windows API | `windows`（按需启用 Win32 子模块） |
| Pinyin 转写 | `pinyin` |
| Fuzzy 匹配 | `nucleo` |
| 错误处理 | `anyhow` + `thiserror` |
| 日志 | `tracing` + `tracing-subscriber` |
| 随机数 | `rand` |
| OKLCh 色彩计算 | `palette`（如需精确转换） |

### 4.2 前端 npm 包

| 用途 | 包 |
|---|---|
| 框架 | `react` 19 + `react-dom` |
| 构建 | `vite` + `@vitejs/plugin-react` |
| 样式 | `tailwindcss` |
| 组件 | shadcn/ui 系列 + `@radix-ui/*` |
| 图标 | `lucide-react` |
| 状态管理 | `zustand` |
| 动画 | `framer-motion` |
| Tauri SDK | `@tauri-apps/api` v2 + 按需 plugin clients |

---

## 5. 启动流程

按序：

1. 进程启动后**立即检测**是否已有实例运行；若有：通知现有实例前置窗口（按 SPEC 4.10），本进程立即退出
2. 读取 `bootstrap.json`（不存在则触发 onboarding；onboarding 完成后写回）
3. 由 `bootstrap.json` 的 `dataFolderPath` 决定数据文件夹位置（null → OS 默认）
4. 检测数据文件夹状态：空 / 不存在 → 启动 onboarding；schema 过低 → 跑 migration；正常 → 继续
5. 从 `<dataFolder>/settings.json` 加载同步设置（不存在则用默认值创建）
6. 加载颜色 map 和 last-used 值
7. 扫描所有模板 JSON 到内存索引；损坏文件移至 `.invalid/`，记 warning
8. 为所有模板生成 pinyin 索引和 body 字段索引（按 SPEC 7.1，body 索引建在 displayName 替换后的形态上）
9. 执行颜色 map GC（删除孤儿条目并落盘）
10. 注册全局热键;失败则通过事件通知前端，引导用户改键
11. 初始化系统托盘
12. 启动数据文件夹变更 watcher
13. App 进入空闲，等待事件

---

## 6. 关键时序约束

某些时序约束是正确性关键，实现时必须遵守：

- **HWND 捕获必须同步且最早**：全局热键回调中第一步同步调 `GetForegroundWindow` 并存入原子变量，任何 UI 切换、异步任务调度都必须在这之后。否则缓存到的窗口是 palette 自己。
- **剪贴板读取在填充对话框打开瞬间**，不在热键唤起时。否则用户在 palette 阶段（之前）复制的内容被错过。
- **写盘事件去重**：app 自身的写入会被 `notify` watcher 检测到，需要 ignore-set 或 debounce 机制避免与显式 emit 产生重复刷新。
- **颜色 map GC 周期**：启动与关闭各跑一次。不维护引用计数，依赖周期性扫描保持收敛。

---

## 7. 关闭流程

1. 收到关闭信号（用户主动或系统）
2. 注销全局热键
3. 等待所有 pending 写入完成
4. 执行颜色 map GC
5. 写回任何 dirty 内存数据
6. 停止 watcher
7. 进程退出

palette 窗口关闭不触发应用退出；仅托盘菜单"退出"或主窗口"退出"按钮触发。

---

## 8. 错误处理与降级

| 场景 | 处理 |
|---|---|
| 数据文件夹无写权限 | 启动时错误对话框，引导改路径 |
| 单个模板 JSON 损坏 | 跳过、记 warning、移至 `templates/.invalid/` |
| 颜色 map 文件损坏 | 备份原文件，初始化为空（颜色会重新生成） |
| 全局热键被占用 | 事件通知前端，设置页提示用户改键 |
| auto-paste 任一步失败 | 降级为仅剪贴板 + 非阻塞 toast |
| HWND 失效（窗口已关闭） | auto-paste 跳过模拟步骤，转 toast |
| WebView2 缺失 | 安装包 bootstrapper 引导安装 |

所有 user-facing 错误通过统一的 toast / dialog 通道展示，避免阻塞性 modal。

---

## 9. 测试策略

### 9.1 Rust 单元测试

覆盖 SPEC 第 13 节的核心不变量。最低范围：

- 模板存储 CRUD + 原子写入
- 模板渲染（占位符替换、可选变量空字符串）
- schema 迁移机制（v1 用 stub 测试机制本身）
- 颜色生成的对比度校验
- 颜色 map GC 不误删 + 删除孤儿
- 搜索权重生效 + 排序 tiebreaker
- pinyin 索引（含多音字案例）
- 变量重命名 / 删除时 body 占位符同步

### 9.2 前端测试

不强制覆盖率。如做，用 Vitest + React Testing Library 覆盖关键状态机：编辑模式 ↔ 填充模式往返时变量值保留、剪贴板互斥的 UI 行为。

### 9.3 集成 / E2E

v1 不在范围。SPEC 描述的核心流程通过手动 smoke test 覆盖。

---

## 10. 打包与发布

### 10.1 Windows 安装包

- Tauri build target: `windows-x86_64-msvc`
- 安装包格式：`.msi`（推荐）或 NSIS `.exe`
- WebView2 bootstrapper：默认 `downloadBootstrapper`（首次安装联网，安装包小）；可选 `embedBootstrapper`（离线可用，体积 +2MB 左右）

### 10.2 代码签名

v1 可选。未签名时 Windows SmartScreen 首次运行会拦截。后续应申请代码签名证书。

### 10.3 自动更新

v1 不实现内置更新机制。手动从 release 页面下载新版本。
