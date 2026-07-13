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

pub fn find_code_work_module(id: &str) -> Option<CodeWorkModule> {
    built_in_code_work_modules()
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

pub fn find_harness_preset(id: &str) -> Option<HarnessPreset> {
    built_in_harness_presets()
        .into_iter()
        .find(|preset| preset.id == id)
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
}
