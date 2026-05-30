# Snippet — 产品规格 (SPEC)

> 本文档是产品决策的事实来源。所有 UI 实现、技术选型、测试用例、AI coding 工具的输入都以本文档为准。

---

## 1. 项目概述

### 1.1 产品定位

`Snippet` 是一个桌面端文本片段与模板管理器，面向有大量"可复用文本"需求的个人用户。核心场景两类：

- **A. 静态片段复用**：邮箱、手机号、姓名、地址等高频键入的固定文本。
- **B. 带变量的模板**：与 AI / LLM 交互、代码处理等场景中反复使用、但每次仅参数不同的文本模板。例如：
  - `翻译成 {Language}：{text}`
  - `整理一下视频中的信息：{url}`
  - 代码块包裹：用 markdown 三反引号加 `{language}` 加换行加 `{code}` 加换行加三反引号

两者在数据模型上**统一为一种结构**：静态片段就是"零变量的模板"。

### 1.2 核心使用流程

1. 用户按全局热键唤起 palette
2. 模糊搜索定位到目标模板
3. 若有变量，填表单产出最终文本；若无变量，直接复制
4. 渲染结果进入剪贴板（或自动粘贴回原焦点窗口，可选）

### 1.3 v1 范围与非范围

**v1 范围**：

- 单机桌面应用，本地文件存储
- Windows 10/11 平台（macOS 后续支持）
- 两种变量类型：text、enum
- 静态颜色系统（不为明暗主题自动适配）
- 借文件级同步盘（Dropbox / iCloud Drive / Syncthing 等）实现跨设备同步，app 不感知

**v1 不做**：

- 内置云同步 / 账号系统
- 团队共享 / 协作
- 文本扩展（typed abbreviation，如 `;email` 自动展开）
- 输入格式校验（URL / email 等）
- 模板版本历史 / 多步 undo
- 模板级输出行为覆盖
- 频率排序（v2 候选）
- 多语言 UI（v1 单语言，待定中 / 英）

---

## 2. 核心概念

### 2.1 模板 (Template)

模板是 app 的核心实体。包含：

- 唯一 ID（UUID v4，同时也是文件名）
- 显示名 displayName
- 模板正文 body（含 0 个或多个变量占位符）
- 0 个或多个 tag
- 0 个或多个变量定义
- 元数据：createdAt、updatedAt、lastUsedAt、useCount、isPinned、schemaVersion

### 2.2 变量 (Variable)

变量是模板正文中的占位符。每个变量有：

- 稳定 ID（GUID，模板内唯一）—— 占位符的真正标识
- 显示名 displayName —— 用户可见、可改名的标签
- 类型 (text / enum)
- enum 类型的 options 列表
- 是否必填 required
- 是否从剪贴板填充 fillFromClipboard
- 是否记住上次使用值 rememberLastUsed
- 静态默认值 staticDefault（可空）

**关键设计点**：模板正文存储时使用 `{<guid>}` 形式表示变量位置；用户在编辑器中看到的是 `{<displayName>}`。重命名 display name 不影响 body 里的 GUID 占位符，因此已填充值不丢失。

### 2.3 Tag

模板的分类标签。一个模板可以有多个 tag。Tag 名是字符串，比较时不区分大小写（lowercased），显示时保留原始大小写。

### 2.4 中央颜色映射 (Central Color Maps)

两个全局独立的 map：

- `variableColorMap`: 变量 display name → 颜色
- `tagColorMap`: tag → 颜色

**两个 map 不共享内容**（避免 `language` tag 和 `Language` 变量同色等意外）。规则相同：display name lowercased 后作为 key。

颜色由 app 在 display name 首次出现时随机生成；用户可以在颜色管理页手动改色。

---

## 3. 数据模型 / 数据契约

### 3.1 模板文件 JSON Schema

每个模板一个 JSON 文件，文件名为 `<uuid>.json`，存储于 `<dataFolder>/templates/`。

```json
{
  "schemaVersion": 1,
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "displayName": "翻译模板",
  "body": "翻译成 {a1b2c3d4-...}：{e5f6g7h8-...}",
  "variables": [
    {
      "guid": "a1b2c3d4-...",
      "displayName": "Language",
      "type": "enum",
      "options": ["中文", "日文", "英文"],
      "required": true,
      "fillFromClipboard": false,
      "rememberLastUsed": true,
      "staticDefault": null
    },
    {
      "guid": "e5f6g7h8-...",
      "displayName": "text",
      "type": "text",
      "options": null,
      "required": true,
      "fillFromClipboard": true,
      "rememberLastUsed": false,
      "staticDefault": null
    }
  ],
  "tags": ["AI", "翻译"],
  "isPinned": false,
  "createdAt": "2026-05-21T10:30:00Z",
  "updatedAt": "2026-05-21T10:30:00Z",
  "lastUsedAt": null,
  "useCount": 0
}
```

字段约束：

- `body` 中变量以 `{<guid>}` 形式占位，不是 `{<displayName>}`
- text 类型变量的 `options` 字段为 null
- enum 类型变量的 `options` 是非空字符串数组
- `staticDefault` 对 enum 类型必须是 options 中的一项或 null

### 3.2 变量颜色映射

文件 `<dataFolder>/variable-colors.json`：

```json
{
  "schemaVersion": 1,
  "map": {
    "language": "oklch(0.55 0.18 240)",
    "text": "oklch(0.55 0.15 30)"
  }
}
```

- `key` 是变量 display name 转小写
- `value` 是 OKLCh CSS 颜色字符串

### 3.3 Tag 颜色映射

文件 `<dataFolder>/tag-colors.json`，结构同 3.2，独立维护。

### 3.4 上次使用值

文件 `<dataFolder>/last-used.json`：

```json
{
  "schemaVersion": 1,
  "values": {
    "language": "日文"
  }
}
```

按变量 display name lowercased 索引，作用域全局（跨模板共享）。

### 3.5 设置

设置分两个文件：**Bootstrap 指针**（每设备本地、不同步）+ **同步设置**（在数据文件夹内、跨设备同步）。

**Bootstrap 指针** — `<OS 用户配置目录>/<app>/bootstrap.json`：

```json
{
  "schemaVersion": 1,
  "dataFolderPath": null
}
```

- 唯一不在 `<dataFolder>` 内的 app 文件，仅记录"数据文件夹在哪里"
- `dataFolderPath` 为 null 表示使用 OS 默认路径（Windows: `%APPDATA%\<app>\`）
- 每设备独立写入，不通过同步盘传播

**同步设置** — `<dataFolder>/settings.json`：

```json
{
  "schemaVersion": 1,
  "hotkey": "Ctrl+Alt+Space",
  "autoPaste": false,
  "theme": "system"
}
```

- 所有用户偏好（hotkey、autoPaste、theme）都在这里，跨设备同步
- `dataFolderPath` **不**出现在此文件，见上 bootstrap 指针

### 3.6 数据文件夹布局

```
<dataFolder>/
  settings.json
  variable-colors.json
  tag-colors.json
  last-used.json
  templates/
    <uuid1>.json
    <uuid2>.json
    ...
```

---

## 4. 交互流程

### 4.1 全局热键 → Palette 唤起

1. 全局热键按下（默认 `Ctrl+Alt+Space`，可改）
2. **第一时间** Rust 后端调系统 API 拿当前最顶层窗口的句柄 (HWND)，缓存到内存
3. 检查主窗口状态：
   - 主窗口已打开 → 主窗口前置 + 加发光描边作为视觉反馈，palette **不**弹出，流程结束
   - 主窗口未打开 → palette 窗口弹出，左侧搜索框聚焦

### 4.2 Palette 形态

无边框、屏幕居中、置顶的独立窗口。左右分栏（默认 40 / 60，分割条可拖拽）：

- **左栏**：搜索输入框（顶部）+ 模板结果列表（下方）
- **右栏**：当前选中模板的预览面板

**空查询状态**（搜索框为空时）：

- 列表先展示置顶 (pinned) 模板（按字母序或 lastUsedAt 倒序），再展示最近使用（按 lastUsedAt 倒序）
- 右栏自动显示当前高亮模板的预览（默认第一条）

**有查询状态**：

- 列表按匹配分数倒序排（详见第 7 节）
- 高亮项随箭头键变化，右栏预览随之联动

**键盘操作**：

| 按键 | 行为 |
|---|---|
| ↑ / ↓ | 在列表选择中切换 |
| Enter | 选中当前高亮模板 |
| Esc | 关闭 palette |
| Cmd/Ctrl+↑ / Cmd/Ctrl+↓ | 在右栏预览面板内滚动（处理超长模板） |
| Tab | 不分配（保留焦点管理） |

palette 唤起后不响应除上述键和文本输入外的快捷键。

### 4.3 模板预览

预览面板展示：

1. 模板 displayName（顶部标题）
2. 模板 tag 列表（圆角胶囊 pill 形态，按 `tagColorMap` 着色，**在 palette 中不可点击**）
3. 模板正文 body，其中变量以**视觉上独立的标签**展示：
   - 视觉形态：方角矩形或尖角括号样式（与圆角的 tag pill 区别开）
   - 背景色按 `variableColorMap` 查表
   - cursor 是 default，**不响应任何 hover / click 交互**

### 4.4 选中模板后的分支

按下 Enter：

- **零变量模板** → 跳到 4.6 渲染输出
- **带变量模板** → palette **同窗口原地变换**为变量填充对话框（4.5）。窗口位置、整体框架保持不变，左栏列表淡出（不保留窄条），右栏内容从"预览"morph 为"表单 + 只读预览"

### 4.5 变量填充对话框

布局保留 palette 的整体形态，但左栏是表单字段，右栏是只读预览。

**表单（左栏）**：

- 每个变量一个字段，按变量在 body 中**首次出现**的顺序排列
- text 类型 → 多行 textarea（统一用 textarea 即使预期单行）
- enum 类型 → 下拉选择 select
- 字段标签（label）的颜色按 `variableColorMap` 与预览中变量标签同色
- 字段类型对应的输入控件：
  - text textarea 在 Enter 时默认换行
  - enum select 在 Enter 时不分配业务行为（保留浏览器默认）

**只读预览（右栏）**：

- 实时显示模板正文 body 中变量被当前字段值代入后的结果
- 每次字段值变化立即更新
- 不可编辑（不允许用户直接修改预览文本）
- 提供"解锁编辑"按钮 → 进入模板编辑模式（4.7）

**默认值层叠**（对话框首次打开时为每个字段填初值）：

按优先级从高到低：

1. **剪贴板**：仅对勾选了 `fillFromClipboard` 的变量；读取剪贴板的时机是**对话框打开瞬间**（不在 palette 唤起时读）
2. **上次使用值**：仅对勾选了 `rememberLastUsed` 的变量；从 `last-used.json` 按 display name lowercased 查
3. **静态默认值** staticDefault
4. 空

**enum 类型的 last-used 失效回退**：如果 step 2 取出的上次值不在当前 options 中，本次作废，回退到 step 3。

**键盘操作**：

| 按键 | 行为 |
|---|---|
| Tab / Shift+Tab | 字段间切换 |
| **Cmd/Ctrl+Enter** | 完成填充，触发输出（4.6） |
| Esc | 关闭对话框，不输出 |
| Enter（在 textarea 内） | 默认换行 |

**必填与可选**：

- `required: true` 且当前值为空 → "复制"按钮 disabled
- 可选变量未填 → 渲染时该位置为空字符串

### 4.6 渲染输出

完成填充后（或零变量模板被选中时）：

1. 后端用各变量值替换 body 中的 GUID 占位符，得到最终文本
2. 写入系统剪贴板
3. 关闭 palette / 对话框
4. 更新该模板的 `lastUsedAt`（当前时间）和 `useCount`（+1）
5. 更新涉及变量的 last-used 值（仅 `rememberLastUsed` 的变量）
6. **若 `autoPaste` 为 true**：
   - 调系统 API 把焦点切回缓存的 HWND
   - 模拟 Ctrl+V 键事件
   - 失败（HWND 失效、键事件被拒等）→ 静默降级为仅剪贴板 + 非阻塞 toast 提示"已复制，请手动粘贴"

剪贴板内容不做"粘贴后恢复旧内容"处理。

### 4.7 模板编辑模式（往返）

从填充对话框点击"解锁编辑"按钮进入。视觉与填充模式**必须明显区分**，至少在以下维度上让用户一眼分辨：

- 标题文本（标识当前是填充模式还是编辑模式）
- 整体色调或边框 accent
- 主操作按钮的文案与功能

具体的视觉表达（用什么颜色、什么措辞）留给 design 阶段决定。

主操作按钮的行为契约：填充模式触发"复制"，编辑模式触发"保存模板"。两种模式下提交快捷键都是 Cmd/Ctrl+Enter。

**编辑模式允许修改**：

- 模板 displayName
- 模板 body（用户看到 `{displayName}` 形式，存储转 `{guid}`）
- 变量：新增 / 删除 / 重命名 / 改类型 / 改 options / 改属性 (required / fillFromClipboard / rememberLastUsed / staticDefault)
- 模板 tag 列表

**保存语义**：

- "保存模板" → 改动永久写入模板文件 → 返回填充模式
- "取消" 或 Esc → 改动全部丢弃 → 返回填充模式

**变量值在往返时的保留规则**：

- 变量按 GUID 索引。重命名 display name 不丢失填充值
- 新增变量在表单中显示为空，待填写
- 删除变量则其填充值丢弃，body 中对应 GUID 占位符也一并清除
- enum options 改动后，对应字段的当前值如不在新 options 中则被清空

**剪贴板互斥**：每个模板最多一个变量勾选 `fillFromClipboard`。编辑模式给变量 B 勾选时，若 A 已勾选，**静默清除 A 的标记**，并在 B 旁短暂显示提示"已从 {A 的名字} 转移"（几秒后淡出）。

### 4.8 主窗口（管理模式）

主窗口提供模板的浏览、新建、编辑、删除、批量管理。包含若干子页面：

- 模板列表（默认页）：可按 tag 筛选、"全部" / "无 tag" / `<某 tag>` 等过滤视图
- 单个模板的编辑（与 4.7 编辑模式视觉一致，但不叠加在 palette 上，是主窗口的子页面）
- 颜色管理（两个独立子页面：变量颜色 / tag 颜色）
- 设置

**新建模板**入口：

- "从空白新建"：直接进入编辑视图，初始为空模板
- "复制现有模板新建"：从选中模板复制一份（新 UUID、displayName 加后缀如"副本"）

**Tag 在主窗口中**：

- 显示形态：圆角胶囊 pill，按 `tagColorMap` 着色
- 鼠标 hover：cursor 变 pointer，背景轻微变化或加下划线
- 点击：按该 tag 筛选当前模板列表

### 4.9 主窗口 ↔ Palette 互斥

应用任意时刻最多一个核心窗口活跃：

| 当前状态 | 全局热键触发 | 主窗口被唤起（托盘 / 任务栏） |
|---|---|---|
| 都未开 | 开 palette | 开主窗口 |
| 只 palette 开 | palette 焦点（保持当前状态） | 关 palette，开主窗口 |
| 只主窗口开 | 主窗口前置 + 发光描边反馈，**不**开 palette | 主窗口焦点 |
| 都开 | （状态不应出现，关 palette、前置主窗口） | （同左） |

多显示器：v1 不做特殊处理，按 OS 默认行为。

### 4.10 应用单实例

应用是 **single-instance** 的。任意时刻系统中只能存在一个 app 进程。

触发场景：用户重复启动（再次点击桌面快捷方式、托盘程序，或开机自启时已有实例运行等）。

处理：

- 第二个进程**立即退出**，不创建任何窗口
- 现有实例的窗口被前置 focus，按 4.9 节窗口互斥规则决定前置的是主窗口还是 palette；若两者皆未开，等同于"用户主动唤起主窗口"
- 第二次启动**不**重置任何状态（不重新走 onboarding、不清空内存索引、不重置设置）

---

## 5. 变量类型系统

### 5.1 支持的类型

仅两种：

- **text**：自由文本，统一用多行 textarea 承载
- **enum**：从一组预定义选项中下拉选择

不支持：number、boolean、date。数据 schema 中变量的 `type` 字段为枚举字符串，v2 扩展时新增枚举值。

### 5.2 enum options 的管理

`options` 是非空字符串数组，顺序即下拉显示顺序。编辑模式提供 UI 增删改 options。

**options 重命名 = 删 + 增**：没有 option ID 体系，无法识别"中文 → 简体中文"是改名还是替换。last-used 对应失效则按 4.5 节回退规则处理。

### 5.3 默认值层叠

每个变量可以同时配置：

- `fillFromClipboard: bool`
- `rememberLastUsed: bool`
- `staticDefault: string | null`

填充对话框打开时按 4.5 节优先级取初值。

### 5.4 剪贴板互斥

每个模板最多一个变量勾选 `fillFromClipboard`。编辑模式中切换该标记时执行 4.7 节互斥逻辑。

### 5.5 上次使用值的作用域

**按 display name 全局**记忆，不按模板隔离。任意模板里 `Language` 变量用过"日文"后，所有其他模板的 `Language` 变量默认值都是"日文"。

存储于 `last-used.json`，key 是 lowercased display name。

### 5.6 必填与可选

每个变量有 `required: bool`，默认 true。

- required 且当前值为空 → 复制按钮 disabled
- 可选变量未填 → 渲染结果中该位置为空字符串

**无格式校验**（v1 不引入 URL / email 等校验）。

### 5.7 表单字段排序

填充对话框中字段顺序 = 变量 GUID 在模板正文 (body) 中**首次出现**的顺序。同一变量多次出现按首次位置算。

---

## 6. 颜色系统

### 6.1 两个独立的中央映射

- `variableColorMap`：变量 display name → 颜色
- `tagColorMap`：tag → 颜色

两个 map **存为两份独立文件**，不共享。

### 6.2 颜色生成规则

新名字首次出现时随机生成一个颜色：

- 色彩空间：**OKLCh**（CSS 原生支持，感知均匀）
- L (lightness)：0.45–0.65 区间随机
- C (chroma)：0.10–0.20 区间随机
- H (hue)：0–360 完全随机
- 生成后对白色前景文字校验对比度 ≥ 4.5:1，不通过则重新采样（极少发生）

不维护固定调色板，不主动避免相近 hue（接受偶然碰撞）。

### 6.3 颜色查找

UI 渲染时从中央 map 查 display name (lowercased) 取颜色字符串，直接作为 CSS `background-color` 使用。

map 修改后所有引用立即更新（前端响应式 store + IPC 事件触发刷新）。

### 6.4 颜色管理

主窗口提供"颜色管理"子页面，分两个 tab（变量颜色 / tag 颜色）。每个 tab 内：

- 展示所有 display name + 当前颜色样例
- 点击 display name：随机刷新该颜色
- 右键 / 长按 / 显式按钮 → 打开取色板，可手动选择颜色
- "重置全部"按钮：所有颜色重新随机
- "保存"按钮：把暂存的改动落盘
- "取消"按钮：丢弃暂存改动

由于主窗口 ↔ palette 互斥（4.9），颜色管理页打开期间不会有变量被新增，因此不存在"未保存方案 vs 后台自动分配"的状态冲突。

### 6.5 明暗主题

**不**为明暗主题各维护一套颜色值。切换主题后，若某些颜色对比度过低、看不清，由用户手动到颜色管理页重置该颜色。系统不做自动适配。

### 6.6 颜色 map GC

在 app 启动和关闭时各执行一次：

1. 扫描所有模板的变量集合，取所有变量 display name (lowercased)
2. 扫描所有模板的 tag 集合，取所有 tag (lowercased)
3. 删除 `variableColorMap` 中 key 不在变量集合内的条目
4. 删除 `tagColorMap` 中 key 不在 tag 集合内的条目
5. 落盘

不维护引用计数。

副作用：删除后短期内再使用同名变量 / tag 会得到新颜色（不"恢复"旧色）。

---

## 7. 搜索与排序

### 7.1 匹配范围与权重

palette 搜索框查询时，对每条模板按以下字段做 fuzzy 匹配。每个字段的命中分数乘以对应权重，模板总分取**字段加权分数中的最大值**（不是加和）。

| 字段 | 权重 |
|---|---|
| displayName | 1.0 |
| tag | 0.8 |
| body | 0.3 |

**不**对变量 display name 做独立匹配（变量名通过 body 中的 `{xxx}` 字面文本间接被匹配覆盖）。

**body 字段的索引形态**：body 在磁盘上以 `{<guid>}` 占位符存储，但搜索索引建在**变量替换为 displayName 后的文本**上（占位符渲染为 `{<displayName>}` 字面文本）。这是上一段"变量名通过 body 间接被匹配"契约成立的前提。模板保存（含变量增 / 删 / 重命名）时，该模板的 body 索引及其 pinyin 衍生索引一并重建；变量是模板内私有的，重命名只影响该模板自身的索引。

### 7.2 Fuzzy 匹配算法

经典 fuzzy 子序列匹配：查询字符按顺序在目标字符串中出现即算命中，中间允许 gap。示例：

- `trsl` → `translate` ✓
- `tplt` → `template` ✓

不要求查询字符连续。具体打分（连续 bonus、首字符 bonus 等）由所选 fuzzy lib 决定（推荐 `nucleo` crate）。

### 7.3 Pinyin 模糊匹配

对包含中文字符的字段，**预先生成 pinyin 索引**：

- 全拼：例如"翻译模板" → "fanyimoban"
- 首字母：例如"翻译模板" → "fymb"

查询时同时对（原文、pinyin 全拼、pinyin 首字母）跑 fuzzy 匹配，取三者中最高分作为该字段的命中分。

- 多音字处理：完全交给 pinyin 转写库的内置词典（不自己处理）
- 声调统一忽略

预索引时机：模板加载到内存时；模板保存时增量更新。

### 7.4 排序

**空查询时**：

1. 置顶 (pinned) 模板在最上（pinned 之间按字母序或 lastUsedAt 倒序均可，由实现决定）
2. 非 pinned 按 lastUsedAt 倒序紧随其后

**有查询时**：

1. 按匹配分数倒序排
2. 相同匹配分数时按 `lastUsedAt` 倒序作 tiebreaker

频率（useCount）不进入 v1 排序逻辑，但 useCount 字段持续更新供 v2 使用。

---

## 8. 输出与剪贴板

### 8.1 默认行为

最终渲染结果**始终写入系统剪贴板**。palette / 填充对话框关闭。用户手动到目标位置粘贴。

**不**恢复粘贴前的旧剪贴板内容。

### 8.2 自动粘贴（可选）

设置 `autoPaste: true` 时：

1. 全局热键按下瞬间，Rust 后端调系统 API（Windows: `GetForegroundWindow`）拿当前最顶层窗口的 HWND，缓存到内存
2. 用户走完模板使用流程
3. 渲染结果写入剪贴板
4. 关闭 palette / 对话框
5. 调 `SetForegroundWindow(cached_hwnd)` 把焦点切回缓存窗口
6. 调输入模拟（`enigo` crate）发送 Ctrl+V 键事件

Windows 平台**无需** Accessibility 权限。macOS 后续支持时需要权限引导。

**失败降级**：任一步失败 → 仅保留剪贴板内容 + 非阻塞 toast 提示"已复制，请手动粘贴"。

### 8.3 没有模板级覆盖

输出行为是全局设置，不为单个模板配置不同行为。

---

## 9. 模板组织

### 9.1 平铺 + tag

所有模板在同一集合中，没有目录 / 文件夹层级。通过 tag 分类、通过搜索检索。

一个模板可以有 0 个或多个 tag。

### 9.2 主窗口的 tag 交互

- Tag 显示形态：圆角胶囊 pill
- 在主窗口模板列表中**可点击**（点击触发按该 tag 筛选）
- 主窗口左侧导航提供 "全部" / "无 tag" / `<某 tag>` 等过滤视图

### 9.3 Palette 中的 tag 交互

Palette 预览面板中 tag 同样用 pill 显示并按 tagColorMap 着色，但**不响应任何鼠标 / 键盘交互**（不可点击）。避免与键盘驱动搜索流程冲突。

### 9.4 Tag 与变量的视觉区分

| 维度 | Tag (pill) | 变量 (variable placeholder) |
|---|---|---|
| 形状 | 全圆角胶囊 | 方角矩形或尖角括号 |
| cursor | 主窗口中 pointer，palette 中 default | 始终 default |
| hover | 主窗口中有背景变化 / 下划线 | 完全无 hover 反馈 |
| 可点击 | 主窗口可，palette 不可 | 始终不可 |

---

## 10. Pinning

每个模板有 `isPinned: bool`。

主窗口模板列表提供"置顶 / 取消置顶"按钮。

Palette 中：

- 空查询时，置顶模板排在最上
- 置顶模板名旁有图钉图标作视觉识别

**pinned 之间无手动排序**：按 lastUsedAt 倒序或字母序即可。

---

## 11. Onboarding

数据文件夹不存在或为空时（首次启动），显示 onboarding 窗口，三选一：

1. **默认路径新建空库**：在 OS 标准 app data 目录（Windows: `%APPDATA%\<app>\`）下创建空数据文件夹结构
2. **指定路径新建空库**：文件选择器选自定义路径，初始化空库
3. **从已有路径导入**：选一个现存的、符合格式的数据文件夹，挂载使用

选定后写入 `bootstrap.json` 的 `dataFolderPath` 字段（默认路径模式下该字段为 null）。

---

## 12. 设置项

| Key | 类型 | 默认值 | 生效时机 | 说明 |
|---|---|---|---|---|
| `hotkey` | string | `"Ctrl+Alt+Space"` | 立即（重新注册全局热键） | 冲突时弹窗提示用户改键 |
| `dataFolderPath` | string \| null | null | 需要重启 app | 数据文件夹路径；存于 bootstrap.json（每设备本地），不在 settings.json |
| `autoPaste` | bool | false | 立即（下一次输出即按新值） | 是否自动粘贴到原焦点窗口 |
| `theme` | "light" \| "dark" \| "system" | "system" | 立即 | UI 主题；"system" 表示跟随 OS |

所有设置项在主窗口"设置"子页面修改，改动立即持久化到对应文件：`hotkey` / `autoPaste` / `theme` → `<dataFolder>/settings.json`；`dataFolderPath` → `<OS 用户配置目录>/<app>/bootstrap.json`。

---

## 13. 核心不变量（必须有单元测试覆盖）

以下行为是设计契约，对应单元测试是 v1 强制要求：

1. **变量 GUID 稳定性**：编辑模式中重命名变量 displayName，变量 GUID 不变，body 中占位符不变，已填充值不丢
2. **变量删除清理**：编辑模式删除变量后，对应已填充值丢弃，body 中该 GUID 占位符也一并清除
3. **enum last-used 失效回退**：若 last-used 值不在当前 options 中，落到下一优先级
4. **剪贴板互斥**：给变量 B 勾选 fillFromClipboard 时，同模板内变量 A 的该标记自动清除
5. **颜色 GC 不误删**：启动 / 关闭 GC 不删除当前实际被使用的 display name 对应的颜色条目
6. **颜色 GC 删除孤儿**：不再被任何模板引用的 display name 在下一次 GC 时被清除
7. **搜索权重生效**：相同子串在 displayName 命中分数 > tag 命中分数 > body 命中分数
8. **排序稳定**：相同匹配分数下按 lastUsedAt 倒序排，排序稳定
9. **窗口互斥**：主窗口打开时按热键不弹 palette；palette 打开时唤起主窗口会关闭 palette
10. **渲染输出正确**：渲染产物按 GUID 在 body 中的位置插入对应值；表单字段顺序 = GUID 在 body 中首次出现的顺序
11. **GC 收敛**：连续多次启动 / 关闭，颜色 map 收敛到稳定状态
12. **Pinyin 索引正确**：常用多音字（如"重庆"、"行业"、"中行"）按词典默认音处理；首字母 / 全拼匹配按预期命中

---

## 14. Schema 演进

所有 JSON 文件顶层包含 `schemaVersion: 1`。加载时：

- `schemaVersion < current` → 应用注册的 migration 函数序列升级，写回新版本
- `schemaVersion > current` → 拒绝加载，提示 "数据由较新版本 app 创建，请升级 app"
- `schemaVersion == current` → 直接使用

v1 实现时定义机制即可，不实际迁移任何东西（current = 1，无 migration 函数）。

---

## 15. 显式不做的事（v1 Out of Scope）

记录这些以避免实施时不小心做进去：

- 内置云同步、账号体系
- 团队协作 / 多用户
- 文本扩展（typed abbreviation 如 `;email` 自动展开）
- 输入格式校验（URL / email 验证等）
- 模板版本历史 / 多步 undo / redo
- 模板级输出行为覆盖（仅复制 / 仅自动粘贴）
- 频率排序
- 颜色明暗主题自动适配
- 颜色 map 引用计数（仅周期性 GC）
- pinned 之间手动排序
- 多语言 UI
- 多显示器特殊交互
- 模板分享 / 导出为可分发格式
- 模板内嵌另一个模板（模板组合）
- 变量类型扩展（number / boolean / date 等）

以上每一项 v2 可重新评估。
