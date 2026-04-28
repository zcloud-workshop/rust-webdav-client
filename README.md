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

  - 音频/视频（HTML5 播放器）
  - Word 文档（DOCX 渲染）
  - Excel 表格（XLSX 渲染为 HTML 表格）
- **在线编辑** — 直接编辑文本文件并保存回 WebDAV 服务器
- **国际化** — 支持中文 / English 切换，默认跟随系统语言
- **主题切换** — 支持浅色模式 / 深色模式 / 跟随系统三种主题
- **性能优化** — 所有网络请求添加超时保护，支持请求取消

## 技术栈

| 层级 | 技术 |
| ------ | ------ |
| 桌面框架 | Tauri 2 |
| 后端 | Rust |
| WebDAV 协议 | reqwest_dav 0.3 |
| 前端 | Svelte 5 + TypeScript |
| 样式 | Tailwind CSS 4 |
| 国际化 | svelte-i18n |
| 文件预览 | pdfjs-dist / docx-preview / SheetJS |

## 项目结构

```tree
src-tauri/src/              # Rust 后端
  lib.rs                    # Tauri 入口，命令注册
  error.rs                  # 统一错误类型
  webdav/
    client.rs               # WebDAV 客户端封装
    types.rs                # 数据类型定义
  commands/
    connection.rs           # 连接管理命令
    files.rs                # 目录列表
    upload.rs               # 上传
    download.rs             # 下载
    operations.rs           # 文件操作（删除/重命名/复制/移动）
    preview.rs              # 预览数据获取
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
    preview.svelte.ts       # 预览面板状态
    toast.svelte.ts         # 通知提示
    theme.svelte.ts         # 主题切换
  utils/file-types.ts       # 文件类型判断与格式化
  components/
    layout/                 # 布局组件（Sidebar/Toolbar/Breadcrumb）
    connection/             # 连接表单
    browser/                # 文件浏览器
    preview/                # 各格式预览器
    common/ContextMenu.svelte  # 右键菜单组件
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

## 发布

项目配置了 GitHub Actions 自动构建发布。推送 `v*` 标签即可触发：

```bash
git tag v0.1.0
git push origin v0.1.0
```

构建目标：

- macOS Apple Silicon (`.dmg`)
- macOS Intel (`.dmg`)
- Windows x64 (`.exe` 安装包)

构建完成后自动上传到 GitHub Releases，手动确认草稿后发布。

## 兼容性

支持标准 WebDAV 协议，已测试兼容：

- Nextcloud
- Apache mod_dav
- Nginx WebDAV 模块
- 群晖 WebDAV Server
- 其他标准 WebDAV 服务器

## License

MIT
