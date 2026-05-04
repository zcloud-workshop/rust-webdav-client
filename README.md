# WebDAV Client

基于 Tauri 2 + Svelte 5 构建的跨平台 WebDAV 桌面客户端，支持文件浏览、全格式预览、在线编辑和上传下载。

## 功能

- **连接管理** — 保存多个 WebDAV 服务器配置，一键切换连接，支持连接测试、编辑、删除
- **文件浏览** — 目录列表、面包屑导航、排序（名称/大小/修改时间）、复选框全选/多选，支持中文路径和 URL 编码
- **文件操作** — 上传、下载、新建文件夹、删除（二次确认）、重命名、复制、移动，支持批量操作和根目录操作
- **右键菜单** — 支持文件右键快捷操作（重命名/复制/移动/下载/删除）
- **文件预览**

  - 文本/代码文件
  - 图片（JPG/PNG/GIF/WebP/SVG），支持大文件预览大小限制
  - PDF（翻页浏览）
  - 音频/视频（本地 HTTP 流式代理，支持大文件实时播放、拖拽进度条）
  - Word 文档（DOCX 渲染）
  - Excel 表格（XLSX 渲染为 HTML 表格）
- **在线编辑** — 直接编辑文本文件并保存回 WebDAV 服务器
- **自动更新** — 启动时自动检查 GitHub Releases 更新，设置中可手动检查
- **全局设置** — 语言切换、主题切换、自动更新开关、关于与 License 查看
- **国际化** — 支持 7 种语言：English / 简体中文 / 繁體中文 / 日本語 / 한국어 / Deutsch / Русский
- **主题切换** — 支持浅色模式 / 深色模式 / 跟随系统三种主题
- **侧边栏** — 可拖拽调整宽度、折叠/展开，右键菜单管理连接
- **图标** — 全部使用 lucide-svelte 图标组件
- **性能优化** — 所有网络请求添加超时保护，支持请求取消

## 技术栈

| 层级 | 技术 |
| ------ | ------ |
| 桌面框架 | Tauri 2 |
| 后端 | Rust |
| WebDAV 协议 | reqwest_dav 0.3 |
| 前端 | Svelte 5 + TypeScript |
| 样式 | Tailwind CSS 4 |
| 图标 | lucide-svelte |
| 国际化 | svelte-i18n |
| 文件预览 | pdfjs-dist / docx-preview / SheetJS |

## 项目结构

```tree
src-tauri/src/              # Rust 后端
  lib.rs                    # Tauri 入口，命令注册
  error.rs                  # 统一错误类型
  streaming.rs              # 本地 HTTP 流式代理（视频播放）
  webdav/
    client.rs               # WebDAV 客户端封装
    types.rs                # 数据类型定义
    mod.rs                  # AppState / StreamState
  commands/
    connection.rs           # 连接管理命令
    files.rs                # 目录列表
    upload.rs               # 上传
    download.rs             # 下载
    operations.rs           # 文件操作（删除/重命名/复制/移动）
    preview.rs              # 预览数据获取、视频流管理
    edit.rs                 # 文本编辑读写

src/lib/                    # Svelte 前端
  api.ts                    # Tauri invoke 封装
  types.ts                  # TypeScript 类型定义
  i18n/                     # 国际化（中文 / English）
    en.json
    zh-CN.json
  stores/                   # Svelte 5 runes 状态管理
    browser.svelte.ts       # 文件浏览器状态
    connections.svelte.ts   # 连接管理状态
    preview.svelte.ts       # 预览面板状态（含视频流管理）
    toast.svelte.ts         # 通知提示
    theme.svelte.ts         # 主题切换
    dialog.svelte.ts        # 对话框状态
    update.svelte.ts        # 自动更新检查
    version.ts              # 版本号
  utils/file-types.ts       # 文件类型判断与格式化
  components/
    layout/                 # 布局组件（Sidebar/Toolbar/Breadcrumb）
    connection/             # 连接表单
    browser/                # 文件浏览器
    preview/                # 各格式预览器（含 VideoPreview 自定义控件）
    common/                 # 通用组件（ContextMenu/ConfirmDialog/SettingsModal）
```

## 开发

**环境要求：**

- Node.js >= 18
- Rust >= 1.77
- pnpm

```bash
# 安装依赖
pnpm install

# 启动开发模式
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```
