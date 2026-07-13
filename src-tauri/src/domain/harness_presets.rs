use super::harness::{CodeWorkModule, HarnessPreset, HarnessPresetFile};

fn markdown(path: &str, label: &str, content: &str) -> HarnessPresetFile {
    HarnessPresetFile {
        path: path.into(),
        kind: "markdown".into(),
        label: label.into(),
        content: content.into(),
    }
}

fn json(path: &str, label: &str, content: &str) -> HarnessPresetFile {
    HarnessPresetFile {
        path: path.into(),
        kind: "json".into(),
        label: label.into(),
        content: content.into(),
    }
}

fn shared_code_files() -> Vec<HarnessPresetFile> {
    vec![
        markdown(
            "docs/task-status.md",
            "Verified task status",
            "# Task Status\n\n## Current Verified State\n- Repository state:\n- Verification state:\n- Active work item:\n- Current blocker:\n\n## Session Log\n\n### Session 001\n- Goal:\n- Completed:\n- Verification evidence:\n- Risks:\n- Next step:\n\n## Decisions\n- Decision:\n  - Context:\n  - Consequence:\n",
        ),
        markdown(
            "docs/session-handoff.md",
            "Session handoff",
            "# Session Handoff\n\n## Current Objective\n- Goal:\n- Current status:\n- Active work item:\n\n## Verified Evidence\n- Check:\n- Result:\n\n## Blockers and Risks\n-\n\n## Next Session\n1. Read `AGENTS.md`.\n2. Read the current status and verification files.\n3. Continue only the active work item.\n",
        ),
        markdown(
            "docs/verification.md",
            "Verification and completion evidence",
            "# Verification\n\n## Definition of Done\n- The requested behavior is implemented.\n- Required checks have actually run.\n- Evidence is recorded in the state files.\n- The repository remains restartable.\n\n## Verification Commands\n- Full verification: `[replace with project command]`\n\n## Evidence\nRecord the command, result, and relevant output for every completed item.\n",
        ),
        markdown(
            "docs/risk-rules.md",
            "Risk and scope rules",
            "# Risk Rules\n\n## Scope Rules\n- Work on one active item at a time.\n- Do not broaden scope without recording the reason.\n\n## Risky Operations\nList operations that need explicit approval.\n\n## Blocked Actions\n- Do not claim completion without verification evidence.\n- Do not hide destructive behavior in scripts.\n",
        ),
    ]
}

fn feature_list() -> HarnessPresetFile {
    json(
        "docs/feature_list.json",
        "Machine-readable feature list",
        "{\n  \"features\": [\n    {\n      \"id\": \"feat-001\",\n      \"name\": \"Replace with the first concrete feature\",\n      \"description\": \"Define behavior and acceptance evidence\",\n      \"dependencies\": [],\n      \"status\": \"not-started\",\n      \"evidence\": \"\"\n    }\n  ]\n}\n",
    )
}

pub fn code_work_shared_files() -> Vec<HarnessPresetFile> {
    shared_code_files()
}

pub fn code_work_shared_files_for_language(language: &str) -> Vec<HarnessPresetFile> {
    code_work_shared_files()
        .into_iter()
        .map(|file| localize_file(file, language))
        .collect()
}

pub fn built_in_code_work_modules() -> Vec<CodeWorkModule> {
    vec![
        CodeWorkModule {
            id: "technical-design".into(),
            name: "Technical Design".into(),
            description: "Architecture-first design work with explicit alternatives, constraints, and verification.".into(),
            files: vec![
                markdown(
                    "docs/architecture.md",
                    "Architecture boundaries",
                    "# Architecture Boundaries\n\n## System Overview\nDescribe the system and its major boundaries.\n\n## Layer Rules\n- Keep dependencies flowing in the documented direction.\n- Put decisions at the owning boundary.\n\n## Invariants\n- Record rules that must remain true.\n",
                ),
                markdown(
                    "docs/decision-record.md",
                    "Technical decision record",
                    "# Decision Record\n\n## Decision\nState the chosen approach.\n\n## Context\nWhat problem and constraints led to this decision?\n\n## Alternatives\n- Option:\n  - Benefits:\n  - Costs:\n\n## Consequences\n- Positive:\n- Negative:\n- Follow-up:\n",
                ),
            ],
            agent_instructions: "Focus on architecture-first design. Explore alternatives, specify constraints, and define verification plans before implementation.".into(),
        },
        CodeWorkModule {
            id: "feature-development".into(),
            name: "Feature Development".into(),
            description: "Verified, one-feature-at-a-time implementation for long-running coding work.".into(),
            files: vec![
                feature_list(),
            ],
            agent_instructions: "Implement features one at a time using test-driven development. Verify each feature is fully working before moving to the next.".into(),
        },
        CodeWorkModule {
            id: "code-review".into(),
            name: "Code Review".into(),
            description: "Independent, evidence-based review for code, plans, and technical changes.".into(),
            files: vec![
                markdown(
                    "docs/review-rubric.md",
                    "Evidence-based review rubric",
                    "# Review Rubric\n\n| Dimension | Pass condition | Evidence |\n| --- | --- | --- |\n| Correctness | Requested behavior is covered | |\n| Architecture | Boundaries and invariants hold | |\n| Verification | Required checks pass | |\n| Scope | No unrelated changes | |\n\n## Verdict\n- Accept\n- Revise\n- Block\n",
                ),
                markdown(
                    "docs/review-findings.md",
                    "Review findings",
                    "# Review Findings\n\n## Finding 001\n- Severity: high / medium / low\n- Location:\n- Evidence:\n- Why it matters:\n- Required follow-up:\n- Status: open / fixed / accepted\n\n## Review Summary\n- Blocking findings:\n- Verification run:\n- Final verdict:\n",
                ),
            ],
            agent_instructions: "Conduct independent, evidence-based code reviews. Assess correctness, architecture alignment, scope, and verification evidence.".into(),
        },
    ]
}

pub fn built_in_code_work_modules_for_language(language: &str) -> Vec<CodeWorkModule> {
    built_in_code_work_modules()
        .into_iter()
        .map(|mut module| {
            if language == "zh-CN" {
                let (name, description, instructions) = match module.id.as_str() {
                    "technical-design" => (
                        "技术设计",
                        "在实现前明确架构边界、备选方案、约束和验证条件。",
                        "先完成架构设计：分析备选方案，明确约束、决策记录与验证计划，再进入实现。",
                    ),
                    "feature-development" => (
                        "功能开发",
                        "面向长期编码工作的逐功能实现、测试与证据记录。",
                        "一次只实现一个功能，采用测试驱动方式，并在进入下一项前记录真实验证证据。",
                    ),
                    "code-review" => (
                        "代码审查",
                        "基于证据审查代码、方案和技术变更。",
                        "独立审查正确性、架构一致性、范围和验证证据；记录发现项与必要的后续动作。",
                    ),
                    _ => ("", "", ""),
                };
                module.name = name.into();
                module.description = description.into();
                module.agent_instructions = instructions.into();
            }
            module.files = module
                .files
                .into_iter()
                .map(|file| localize_file(file, language))
                .collect();
            module
        })
        .collect()
}

pub fn find_code_work_module(id: &str) -> Option<CodeWorkModule> {
    built_in_code_work_modules()
        .into_iter()
        .find(|module| module.id == id)
}

pub fn find_code_work_module_for_language(id: &str, language: &str) -> Option<CodeWorkModule> {
    built_in_code_work_modules_for_language(language)
        .into_iter()
        .find(|module| module.id == id)
}

fn document_professional_report() -> HarnessPreset {
    let mut files = vec![
        markdown("docs/document-brief.md", "Document brief", "# Document Brief\n\n## Audience\n\n## Purpose and decision\n\n## Constraints\n\n## Required deliverable\n"),
        markdown("docs/outline.md", "Report outline", "# Outline\n\n## Executive conclusion\n\n## Supporting sections\n\n## Open questions\n"),
        markdown("docs/research-notes.md", "Research notes", "# Research Notes\n\n## Source 001\n- Source:\n- Relevant observation:\n- Reliability:\n- Used for:\n"),
        markdown("docs/evidence-matrix.md", "Evidence matrix", "# Evidence Matrix\n\n| Claim | Evidence | Source | Confidence | Open question |\n| --- | --- | --- | --- | --- |\n"),
        markdown("docs/quality-rubric.md", "Document quality rubric", "# Quality Rubric\n\n| Dimension | Standard | Evidence |\n| --- | --- | --- |\n| Accuracy | Claims are supported | |\n| Structure | Reader can follow the argument | |\n| Completeness | Required questions are answered | |\n| Clarity | Language matches the audience | |\n"),
    ];
    files.extend(shared_document_lifecycle_files());
    HarnessPreset {
        id: "document-professional-report".into(),
        work_type: "document".into(),
        name: "Professional Report".into(),
        description: "Evidence-based reports for decisions, briefings, and long-form professional communication.".into(),
        files,
    }
}

fn document_academic_paper() -> HarnessPreset {
    let mut files = vec![
        markdown("docs/research-question.md", "Research question", "# Research Question\n\n## Question\n\n## Scope and exclusions\n\n## Contribution\n\n## Method or evidence standard\n"),
        markdown("docs/paper-outline.md", "Paper outline", "# Paper Outline\n\n## Abstract\n\n## Introduction\n\n## Related work\n\n## Method\n\n## Results\n\n## Discussion\n\n## Conclusion\n"),
        markdown("docs/literature-review.md", "Literature review", "# Literature Review\n\n## Themes\n\n## Agreement and disagreement\n\n## Gap\n\n## Sources to verify\n"),
        markdown("docs/evidence-matrix.md", "Evidence matrix", "# Evidence Matrix\n\n| Claim | Evidence | Source | Confidence | Limitation |\n| --- | --- | --- | --- | --- |\n"),
        markdown("docs/citation-register.md", "Citation register", "# Citation Register\n\n| Key | Full citation | Used in | Verified |\n| --- | --- | --- | --- |\n"),
        markdown("docs/quality-rubric.md", "Academic quality rubric", "# Academic Quality Rubric\n\n| Dimension | Standard | Evidence |\n| --- | --- | --- |\n| Question | Research question is precise | |\n| Evidence | Claims match cited evidence | |\n| Method | Method and limits are explicit | |\n| Citation | Sources are complete and consistent | |\n"),
    ];
    files.extend(shared_document_lifecycle_files());
    HarnessPreset {
        id: "document-academic-paper".into(),
        work_type: "document".into(),
        name: "Academic Paper".into(),
        description:
            "Long-running research and paper writing with traceable evidence and citation control."
                .into(),
        files,
    }
}

fn shared_document_lifecycle_files() -> Vec<HarnessPresetFile> {
    vec![
        markdown("docs/task-status.md", "Verified task status", "# Task Status\n\n## Current Verified State\n- Active section or task:\n- Verified evidence:\n- Current blocker:\n\n## Next step\n"),
        markdown("docs/session-handoff.md", "Session handoff", "# Session Handoff\n\n## Current objective\n\n## Completed and verified\n\n## Open questions\n\n## Next session\n"),
        markdown("docs/verification.md", "Delivery verification", "# Verification\n\n## Completion criteria\n- All required claims and sections are present.\n- Sources and citations are checked.\n- Quality rubric is satisfied.\n\n## Evidence\nRecord checks and results here.\n"),
    ]
}

fn presentation_briefing() -> HarnessPreset {
    let mut files = vec![
        markdown("docs/presentation-brief.md", "Presentation brief", "# Presentation Brief\n\n## Audience\n\n## Desired decision or action\n\n## Duration\n\n## Constraints\n"),
        markdown("docs/narrative-outline.md", "Narrative outline", "# Narrative Outline\n\n## Conclusion first\n\n## Why it matters\n\n## Evidence and turning points\n\n## Call to action\n"),
        markdown("docs/slide-plan.md", "Slide plan", "# Slide Plan\n\n| Slide | Purpose | Core message | Evidence or asset | Status |\n| --- | --- | --- | --- | --- |\n"),
        markdown("docs/speaker-notes.md", "Speaker notes", "# Speaker Notes\n\n## Slide 1\n- Message:\n- Talk track:\n- Transition:\n"),
        markdown("docs/evidence-matrix.md", "Presentation evidence", "# Evidence Matrix\n\n| Claim | Evidence | Source | Confidence | Slide |\n| --- | --- | --- | --- | --- |\n"),
        markdown("docs/visual-direction.md", "Visual direction", "# Visual Direction\n\n## Audience and tone\n\n## Visual principles\n\n## Brand constraints\n\n## Prohibited treatments\n"),
        markdown("docs/quality-rubric.md", "Presentation quality rubric", "# Presentation Quality Rubric\n\n| Dimension | Standard | Evidence |\n| --- | --- | --- |\n| Narrative | The story supports the desired decision | |\n| Content | Claims are accurate and relevant | |\n| Slides | Each slide has one clear job | |\n| Delivery | Notes and timing are usable | |\n"),
    ];
    files.extend(shared_document_lifecycle_files());
    HarnessPreset {
        id: "presentation-briefing".into(),
        work_type: "presentation".into(),
        name: "Presentation Briefing".into(),
        description: "Decision-oriented presentation planning with narrative, slide, speaker, and evidence artifacts.".into(),
        files,
    }
}

pub fn built_in_harness_presets() -> Vec<HarnessPreset> {
    vec![
        document_professional_report(),
        document_academic_paper(),
        presentation_briefing(),
    ]
}

pub fn built_in_harness_presets_for_language(language: &str) -> Vec<HarnessPreset> {
    built_in_harness_presets()
        .into_iter()
        .map(|mut preset| {
            if language == "zh-CN" {
                let (name, description) = match preset.id.as_str() {
                    "document-professional-report" => {
                        ("专业报告", "为决策、汇报和长篇专业沟通准备的证据型报告。")
                    }
                    "document-academic-paper" => {
                        ("学术论文", "为长期研究和论文写作提供可追溯证据与引文管理。")
                    }
                    "presentation-briefing" => (
                        "汇报演示",
                        "面向决策的演示规划，包含叙事、页面、讲稿和证据材料。",
                    ),
                    _ => ("", ""),
                };
                preset.name = name.into();
                preset.description = description.into();
            }
            preset.files = preset
                .files
                .into_iter()
                .map(|file| localize_file(file, language))
                .collect();
            preset
        })
        .collect()
}

pub fn find_harness_preset(id: &str) -> Option<HarnessPreset> {
    built_in_harness_presets()
        .into_iter()
        .find(|preset| preset.id == id)
}

pub fn find_harness_preset_for_language(id: &str, language: &str) -> Option<HarnessPreset> {
    built_in_harness_presets_for_language(language)
        .into_iter()
        .find(|preset| preset.id == id)
}

fn localize_file(mut file: HarnessPresetFile, language: &str) -> HarnessPresetFile {
    if language != "zh-CN" {
        return file;
    }

    let (label, content) = match file.path.as_str() {
        "docs/architecture.md" => ("架构边界", "# 架构边界\n\n## 系统概览\n描述系统及其主要边界。\n\n## 分层规则\n- 依赖应遵循既定方向。\n- 决策应记录在所属边界。\n\n## 不变量\n- 记录必须始终成立的规则。\n"),
        "docs/decision-record.md" => ("技术决策记录", "# 技术决策记录\n\n## 决策\n说明选择的方案。\n\n## 背景\n问题与约束是什么？\n\n## 备选方案\n- 方案：\n  - 收益：\n  - 成本：\n\n## 后果\n- 正面：\n- 负面：\n- 后续动作：\n"),
        "docs/feature_list.json" => ("机器可读功能列表", "{\n  \"features\": [\n    {\n      \"id\": \"feat-001\",\n      \"name\": \"替换为第一个具体功能\",\n      \"description\": \"定义行为和验收证据\",\n      \"dependencies\": [],\n      \"status\": \"not-started\",\n      \"evidence\": \"\"\n    }\n  ]\n}\n"),
        "docs/review-rubric.md" => ("基于证据的审查准则", "# 审查准则\n\n| 维度 | 通过条件 | 证据 |\n| --- | --- | --- |\n| 正确性 | 覆盖需求行为 | |\n| 架构 | 边界和不变量成立 | |\n| 验证 | 必要检查通过 | |\n| 范围 | 没有无关改动 | |\n\n## 结论\n- 接受\n- 修改\n- 阻断\n"),
        "docs/review-findings.md" => ("审查发现", "# 审查发现\n\n## 发现 001\n- 严重程度：高 / 中 / 低\n- 位置：\n- 证据：\n- 影响：\n- 必要后续动作：\n- 状态：待处理 / 已修复 / 已接受\n\n## 审查总结\n- 阻断项：\n- 已运行验证：\n- 最终结论：\n"),
        "docs/task-status.md" => ("已验证任务状态", "# 任务状态\n\n## 当前已验证状态\n- 仓库状态：\n- 验证状态：\n- 当前工作项：\n- 当前阻塞：\n\n## 会话记录\n\n### 会话 001\n- 目标：\n- 已完成：\n- 验证证据：\n- 风险：\n- 下一步：\n"),
        "docs/session-handoff.md" => ("会话交接", "# 会话交接\n\n## 当前目标\n- 目标：\n- 当前状态：\n- 当前工作项：\n\n## 已验证证据\n- 检查：\n- 结果：\n\n## 阻塞与风险\n-\n\n## 下次会话\n1. 阅读 `AGENTS.md`。\n2. 阅读当前状态和验证文件。\n3. 只继续当前工作项。\n"),
        "docs/verification.md" => ("验证与完成证据", "# 验证\n\n## 完成定义\n- 已实现请求行为。\n- 已实际运行必要检查。\n- 已在状态文件中记录证据。\n- 仓库可继续工作。\n\n## 验证命令\n- 完整验证：`[替换为项目命令]`\n\n## 证据\n记录每个已完成项目的命令、结果和关键输出。\n"),
        "docs/risk-rules.md" => ("风险与范围规则", "# 风险规则\n\n## 范围规则\n- 一次只处理一个工作项。\n- 扩大范围前必须记录原因。\n\n## 高风险操作\n列出需要明确批准的操作。\n\n## 禁止行为\n- 没有验证证据不得声明完成。\n- 不得在脚本中隐藏破坏性行为。\n"),
        "docs/document-brief.md" => ("文档简报", "# 文档简报\n\n## 受众\n\n## 目的与决策\n\n## 约束\n\n## 交付物\n"),
        "docs/outline.md" => ("报告大纲", "# 大纲\n\n## 执行结论\n\n## 支撑章节\n\n## 开放问题\n"),
        "docs/research-notes.md" => ("研究笔记", "# 研究笔记\n\n## 来源 001\n- 来源：\n- 关键观察：\n- 可靠性：\n- 用途：\n"),
        "docs/evidence-matrix.md" => ("证据矩阵", "# 证据矩阵\n\n| 主张 | 证据 | 来源 | 置信度 | 开放问题 |\n| --- | --- | --- | --- | --- |\n"),
        "docs/quality-rubric.md" => ("质量准则", "# 质量准则\n\n| 维度 | 标准 | 证据 |\n| --- | --- | --- |\n| 准确性 | 主张有证据支撑 | |\n| 结构 | 读者能够理解论证 | |\n| 完整性 | 回答必要问题 | |\n| 清晰度 | 语言适合受众 | |\n"),
        "docs/research-question.md" => ("研究问题", "# 研究问题\n\n## 问题\n\n## 范围与排除项\n\n## 贡献\n\n## 方法或证据标准\n"),
        "docs/paper-outline.md" => ("论文大纲", "# 论文大纲\n\n## 摘要\n\n## 引言\n\n## 相关工作\n\n## 方法\n\n## 结果\n\n## 讨论\n\n## 结论\n"),
        "docs/literature-review.md" => ("文献综述", "# 文献综述\n\n## 主题\n\n## 共识与分歧\n\n## 缺口\n\n## 待核实来源\n"),
        "docs/citation-register.md" => ("引文登记表", "# 引文登记表\n\n| 标识 | 完整引文 | 使用位置 | 已核实 |\n| --- | --- | --- | --- |\n"),
        "docs/presentation-brief.md" => ("演示简报", "# 演示简报\n\n## 受众\n\n## 期望决策或行动\n\n## 时长\n\n## 约束\n"),
        "docs/narrative-outline.md" => ("叙事大纲", "# 叙事大纲\n\n## 先给结论\n\n## 为什么重要\n\n## 证据与转折点\n\n## 行动号召\n"),
        "docs/slide-plan.md" => ("页面计划", "# 页面计划\n\n| 页面 | 目的 | 核心信息 | 证据或素材 | 状态 |\n| --- | --- | --- | --- | --- |\n"),
        "docs/speaker-notes.md" => ("讲稿备注", "# 讲稿备注\n\n## 第 1 页\n- 信息：\n- 讲述要点：\n- 过渡：\n"),
        "docs/visual-direction.md" => ("视觉方向", "# 视觉方向\n\n## 受众与语气\n\n## 视觉原则\n\n## 品牌约束\n\n## 禁止处理方式\n"),
        _ => return file,
    };
    file.label = label.into();
    file.content = content.into();
    file
}

#[cfg(test)]
mod tests {
    use super::{
        built_in_code_work_modules, built_in_harness_presets, find_code_work_module,
        find_harness_preset,
    };

    #[test]
    fn code_work_modules_cover_design_development_and_review() {
        let modules = built_in_code_work_modules();

        assert_eq!(modules.len(), 3);
        assert!(find_code_work_module("technical-design")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/decision-record.md"));
        assert!(find_code_work_module("technical-design")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/architecture.md"));
        assert!(find_code_work_module("feature-development")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/feature_list.json"));
        assert!(find_code_work_module("code-review")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/review-findings.md"));
    }

    #[test]
    fn shared_code_files_do_not_include_technical_design_artifacts() {
        assert!(!super::code_work_shared_files()
            .iter()
            .any(|file| file.path == "docs/architecture.md"));
    }

    #[test]
    fn built_in_presets_cover_the_confirmed_workflows() {
        let presets = built_in_harness_presets();

        assert_eq!(presets.len(), 3);
        assert!(find_harness_preset("document-academic-paper")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/citation-register.md"));
        assert!(find_harness_preset("presentation-briefing")
            .unwrap()
            .files
            .iter()
            .any(|file| file.path == "docs/slide-plan.md"));
    }

    #[test]
    fn every_non_custom_preset_includes_lifecycle_files() {
        for preset in built_in_harness_presets() {
            assert!(preset
                .files
                .iter()
                .any(|file| file.path == "docs/task-status.md"));
            assert!(preset
                .files
                .iter()
                .any(|file| file.path == "docs/session-handoff.md"));
        }
    }

    #[test]
    fn chinese_registry_preserves_paths_and_localizes_content() {
        let english = super::built_in_code_work_modules_for_language("en");
        let chinese = super::built_in_code_work_modules_for_language("zh-CN");

        assert_eq!(
            english.iter().map(|module| &module.id).collect::<Vec<_>>(),
            chinese.iter().map(|module| &module.id).collect::<Vec<_>>()
        );
        assert!(chinese.iter().any(|module| module.name == "技术设计"));
        assert!(chinese
            .iter()
            .flat_map(|module| &module.files)
            .any(|file| file.path == "docs/feature_list.json"
                && file.content.contains("替换为第一个具体功能")));
    }
}
