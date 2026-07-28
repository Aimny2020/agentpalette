# AgentPalette GitHub 内部预览发布设计

> Tauri identifier 为 `com.lemon.agentpalette`，与 AgentPalette 产品名称保持一致。

## 目标

将 AgentPalette `0.1.0` 准备为 GitHub 私有仓库中的内部预览版本。GitHub Actions 必须为 macOS ARM64、macOS x64 和 Windows x64 生成可供人工安装验证的 Draft Release 产物；本阶段不提供公开分发承诺、正式代码签名、notarization 或自动更新。

## 已选方案与替代方案

发布流程采用“tag 驱动的 Draft Release”：普通 push 和 pull request 只执行 CI；推送与应用版本一致的 `v*.*.*` tag 才触发安装包构建。相比 release 分支驱动方案，tag 能直接表达不可歧义的版本；相比手动 `workflow_dispatch`，tag 也能在 Git 历史和 GitHub Release 之间建立稳定对应关系。`workflow_dispatch` 不作为首发入口，避免手工输入版本造成漂移。

产物采用 macOS DMG 和 Windows NSIS `.exe`。不生成 MSI，因为内部预览不需要企业部署能力；不构建 Linux，因为首轮验证资源集中在 macOS 和 Windows，且 Linux 的 WebKitGTK 与发行版兼容性会扩大当前范围。

macOS 使用 ad-hoc signing，Windows 保持未签名。替代方案是立即配置 Apple notarization 与 Windows 商业证书，但这需要外部账号、证书和 Secret 管理，不符合内部预览边界。Release Notes 和 README 必须说明 Gatekeeper 与 SmartScreen 可能出现的提示。

## 仓库与产品元数据

- GitHub 仓库：`https://github.com/Aimny2020/agentpalette.git`
- 默认分支：`main`
- 仓库可见性：private
- 产品名：`AgentPalette`
- 首发版本：`0.1.0`
- Git tag：`v0.1.0`
- Tauri identifier：`com.lemon.agentpalette`
- 作者：`Aimny2020`
- 许可证：私有预览阶段不添加 LICENSE，README 明确“未授权公开分发”

`package.json`、`src-tauri/Cargo.toml` 与 `src-tauri/tauri.conf.json` 继续保留各自版本字段。新增一个零依赖 Node.js 校验脚本，读取三处版本并在不一致时非零退出；传入 tag 时还必须验证去掉 `v` 后等于应用版本。这样既不引入版本同步工具，也能阻止不一致的发布。

## Git 内容边界

`.agents/` 继续整体忽略，它属于本地开发辅助内容。未来需要随产品分发的内置资源应进入明确的源码资源目录，并通过 Tauri `bundle.resources` 声明。

现有的 `node_modules/`、`dist/`、`src-tauri/target/`、`src-tauri/gen/`、数据库文件、环境变量文件、`.superpowers/` 和系统垃圾文件继续忽略。补充常见编辑器、日志、覆盖率和本地签名材料的忽略规则，但不使用可能吞掉源码或发布配置的宽泛模式。

`stitch_agentpalette_project_manager (1)/` 下三个已缺失的旧设计导出文件正式删除。删除记录保留在工作树供用户审阅，本次不 commit。

## CI 设计

`.github/workflows/ci.yml` 在 `main` 的 push 和所有 pull request 上运行。单个 Ubuntu job 使用 Node.js 20、`npm ci` 和 stable Rust，安装 Tauri v2 所需 Linux 系统依赖，并执行：

1. `npm run lint`
2. `npm run test:run`
3. `npm run build`
4. `npm run version:check`
5. `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
6. `cargo test --manifest-path src-tauri/Cargo.toml`
7. `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`

Cargo 和 npm 缓存只用于提速，不影响 lockfile 驱动的可重复安装。工作流使用最小权限 `contents: read`。

## Draft Release 设计

`.github/workflows/release.yml` 只响应 `v*.*.*` tag。发布 job 首先执行与 CI 等价的版本和质量检查；只有检查通过才运行三项矩阵构建：

- `macos-latest` + `aarch64-apple-darwin`
- `macos-latest` + `x86_64-apple-darwin`
- `windows-latest` + `x86_64-pc-windows-msvc`

工作流使用 `tauri-apps/tauri-action` 创建 `AgentPalette v__VERSION__` Draft Release，上传 DMG 或 NSIS 产物，关闭 updater JSON 上传，并赋予 job `contents: write`。不配置长期 Secret；GitHub 自动提供的 `GITHUB_TOKEN` 足够创建 Draft Release。

为了让各平台只生成所需包，Tauri 主配置不再使用 `targets: "all"`。平台差异通过 `tauri.macos.conf.json` 和 `tauri.windows.conf.json` 表达：macOS 指定 `dmg` 与 ad-hoc identity，Windows 指定 `nsis`。主配置保留跨平台公共 bundle 设置。

## 安全配置

将 `app.security.csp` 从 `null` 改为显式策略。当前前端没有外部脚本、样式、字体、图片请求或 HTTP API，因此策略以本地资源为边界：默认、脚本、样式、字体和图片均限制为应用自身所需来源，`connect-src` 仅允许 Tauri IPC 所需协议。配置后必须通过生产构建和桌面应用启动检查，避免 CSP 阻断 Tauri IPC 或 Vite 产物。

不引入 updater 插件、公钥或 endpoint；不新增任何证书、私钥或 Secret 示例值。

## 文档

README 使用中文，包含：私有预览声明、当前功能、架构、环境要求、开发与验证命令、支持的安装包、版本/tag 发布流程、未签名安装风险、自动更新暂未启用，以及敏感信息不得提交的说明。内容必须反映当前已实现的技能扫描/导入/删除能力，不再保留“第一阶段不包含项目扫描”等过期表述。

新增 `CHANGELOG.md`，采用 Keep a Changelog 结构，包含 `[Unreleased]` 与 `[0.1.0]`。`0.1.0` 记录当前可验证功能和内部预览限制，不虚构尚未实现的能力。

## 测试与验收

版本校验脚本使用 Node 内置测试运行器覆盖：三个版本一致、任一版本不一致、合法 tag 一致、tag 不一致四类行为，并按 red-green-refactor 实现。

配置和工作流通过静态解析、Tauri config 校验与实际命令验证。最终验收至少执行：

- `npm run version:check`
- `npm run test:run`
- `npm run lint`
- `npm run build`
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `npm run tauri -- build --bundles app`（本机 macOS 应用包验证）

本机 DMG 已知在 `bundle_dmg.sh` 阶段失败且缺少有效错误细节。远程 macOS runner 的 DMG 构建是发布工作流的最终判定；本地验收不会把该环境相关失败误报为已解决。

## 明确不做

- 不 commit、push、创建 tag 或触发 GitHub Release
- 不发布正式 Release
- 不构建 Linux、MSI、移动端产物
- 不配置自动更新
- 不配置 Apple notarization 或 Windows 正式签名
- 不添加 LICENSE、贡献指南、Issue 模板或 PR 模板
- 不修改与发布准备无关的产品功能
