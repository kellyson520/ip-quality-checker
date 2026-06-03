# IP Quality Checker

跨平台 IP 质量检测工具，基于 [IPQuality](https://github.com/xykt/IPQuality) 脚本封装。

## ✨ 特性

- 🖥️ **跨平台** — Windows / macOS / Linux 桌面应用 + Android APK
- 🎨 **美观 UI** — 暗色主题，渐变色彩，响应式设计
- 🔍 **全面检测** — IP 基础信息、风险评分、风险因子、流媒体解锁、邮局检测
- 📊 **多数据源** — IPinfo / ipregistry / ipapi / AbuseIPDB / IP2Location 等
- 🚀 **一键检测** — 点击按钮即可运行完整 IP 质量检查

## 📦 下载

前往 [Releases](https://github.com/kellyson520/ip-quality-checker/releases) 页面下载对应平台的安装包：

| 平台 | 格式 |
|------|------|
| Windows | `.msi` / `.exe` |
| macOS | `.dmg` |
| Linux | `.AppImage` / `.deb` |
| Android | `.apk` |

## 🛠️ 开发

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- Tauri CLI v2

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 📋 技术栈

- **前端**: React 18 + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri v2
- **脚本**: IPQuality Bash 脚本 (内嵌编译)
- **CI**: GitHub Actions (多平台自动构建)

## 📄 许可证

基于 [IPQuality](https://github.com/xykt/IPQuality) 项目，遵循 AGPL v3 许可证。
