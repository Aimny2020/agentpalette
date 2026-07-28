# Changelog

[English](./CHANGELOG.md) | 简体中文

本文件记录 AgentPalette 的重要变更。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本遵循 [Semantic Versioning](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### 计划

- 完成内部预览安装验证后评估 Linux 构建、正式代码签名和自动更新。

## [1.1.0] - 2026-07-28

### 新增

- 新增英语界面与“跟随系统 / 简体中文 / English”显示语言选择，可立即生效并保存在本机。
- 为全局 Harness 模板及项目 Harness 工作流提供英语语言支持与英文模板内容。
- 新增英文 README、英文变更记录以及与中文文档对应的截图。

### 变更

- 统一 AgentPalette 的应用标识、数据库文件、全局 Skills 与 Harness 数据目录，以及主题存储键。
- GitHub Release 使用英语和简体中文的双语发布说明。

## [1.0.0] - 2026-07-21

### 新增

- 公开发布 AgentPalette，并提供 macOS（Apple Silicon、Intel）与 Windows x64 安装包。
- 项目管理、本机 Agent 检测与维护、全局 Skills 管理、全局 Harness 模板管理，以及项目级 Skills 与 Harness 配置。
- 平台启动偏好、主题设置及项目内启动本机 Agent 的工作流。

### 变更

- 项目许可证调整为 Apache License 2.0，并新增贡献与品牌使用说明。
- GitHub tag 发布流程调整为创建公开正式 Release。

### 限制

- macOS 安装包使用 ad-hoc signing，尚未完成 Apple notarization；Windows 安装包尚未进行代码签名。
- 暂不提供 Linux、MSI、移动端和自动更新支持。

## [0.2.1] - 2026-07-13

### 变更

- 改进项目 Harness 的导航、编辑布局、文件扫描范围与切换状态管理。

## [0.1.0] - 2026-07-07

### 新增

- 基于 Tauri v2、React、TypeScript 和 Rust 的桌面应用基础架构。
- 项目导航、主题切换、共享 UI 状态和页面框架。
- SQLite 迁移、领域端口、应用服务和类型化 Tauri IPC。
- 本地技能与技能包扫描、Markdown 元数据解析及可执行内容风险检测。
- 本地目录与 Git 来源的技能导入检查、来源记录和信任状态展示。
- 技能详情、技能包成员展示及带依赖保护的删除确认流程。
- Vitest 前端测试、Rust 单元测试和 Clippy 检查。
- GitHub CI 与 tag 驱动的 macOS/Windows Draft Release 流程。

### 限制

- 本版本仅供私有内部预览，未授权公开分发。
- macOS 仅使用 ad-hoc signing，未进行 notarization。
- Windows 安装包未签名。
- 暂不提供 Linux、MSI、移动端和自动更新支持。

[Unreleased]: https://github.com/Aimny2020/agentpalette/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v1.1.0
[1.0.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v1.0.0
[0.2.1]: https://github.com/Aimny2020/agentpalette/releases/tag/v0.2.1
[0.1.0]: https://github.com/Aimny2020/agentpalette/releases/tag/v0.1.0
