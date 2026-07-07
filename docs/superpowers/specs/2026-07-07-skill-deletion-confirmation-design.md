# Skill Deletion Confirmation Spec

## Goal

Improve the skill package deletion flow on the Skills management page. Currently, clicking delete triggers a browser-native `confirm()` prompt, and if a skill is in use by projects, it relies on a nested browser-native confirmation. The new design implements a custom React modal (`ConfirmDeleteModal`) that matches the platform's visual system, prevents silent errors, and handles the "enabled in projects" dependency warning progressively in a single modal flow.

## User Experience and Layout Changes

### 1. Delete Target State
- In `SkillsPage.tsx`, manage the deletion target using state:
  ```typescript
  const [deleteTarget, setDeleteTarget] = useState<Skill | null>(null);
  ```
- Clicking the trash button on a `SkillCard` will call `setDeleteTarget(skill)`.
- If `deleteTarget` is set, render the `ConfirmDeleteModal`.

### 2. Confirm Deletion Modal (`ConfirmDeleteModal`)
The modal has a compact design (`modal-body compact-modal`) with two visual states based on whether there are project dependencies blocking standard deletion.

#### State A: General Deletion Confirmation
- **Header**: Icon (`AlertTriangle` in warning orange) + Title "删除技能" + Close button.
- **Body Content**:
  - Main warning text: "你确定要删除技能 **{skill.metadata.name}** 吗？此操作将永久从磁盘删除源文件，且不可恢复。"
  - Sub-info showing the local path of the skill source directory if available.
- **Footer Buttons**:
  - `取消` (closes modal).
  - `确认删除` (styled as a warning/danger button, e.g., red background).

#### State B: Progressive Dependency Warning (When active in projects)
- If the backend returns the "Skill Pack is enabled in projects: project-A, project-B" error during standard deletion, the modal transitions to State B.
- **Header**: Icon (`AlertTriangle` in danger red) + Title "该技能正在被项目使用".
- **Body Content**:
  - Warning text: "该技能已在以下项目中启用，无法直接删除："
  - List of projects: Renders a list of the occupied project names.
  - Action consequence: "如果继续，系统将自动从以上项目中移除并禁用此技能，然后彻底删除本地源文件。"
- **Footer Buttons**:
  - `取消` (closes modal).
  - `一键移除并彻底删除` (danger red background, triggers `delete_skill_everywhere`).

---

## Styling Specifications

### CSS Classes to Add/Modify in `skills.css`

1. **`.delete-modal-warning`**:
   - Spacing and color for warnings in the delete modal.
   - `display: flex; gap: var(--space-2); margin-bottom: var(--space-2);`

2. **`.delete-modal-warning__icon`**:
   - `color: var(--color-danger);` or `color: var(--color-warning);` depending on state.

3. **`.occupied-projects-list`**:
   - Container for list of project names.
   - `background: var(--color-surface-soft);`
   - `border: 1px solid var(--color-outline);`
   - `border-radius: var(--radius-sm);`
   - `padding: var(--space-2) var(--space-3);`
   - `margin-top: var(--space-1);`
   - `margin-bottom: var(--space-2);`
   - `max-height: 10rem;`
   - `overflow-y: auto;`

4. **`.occupied-projects-list ul`**:
   - `margin: 0; padding-left: 1.2rem;`

5. **`.button--danger`**:
   - Danger red button style matching the brand system.
   - `background-color: var(--color-danger); color: white;`
   - `border: 1px solid transparent;`
   - `transition: background-color 0.2s;`
   - Hover state: `background-color: color-mix(in srgb, var(--color-danger) 85%, black);`

---

## Verification Plan

### Automated Tests
- Create `src/features/skills/components/ConfirmDeleteModal.test.tsx` to verify:
  - Default layout showing warnings and buttons.
  - Transition to Progressive Dependency Warning state when mutation throws the projects error.
  - Clicking "一键移除并彻底删除" calls the confirm handler with `force = true`.
- Run frontend test suite: `npm run test:run`
- Ensure compilation is successful: `npm run build`

### Manual Verification
- Open the application via `npm run dev` or `npm run tauri:dev`.
- Locate a standalone skill or skill pack, click the delete button.
- Check that the custom modal is displayed.
- Test normal deletion on an unused skill.
- Test progressive deletion warning on a skill that is currently enabled in a project (confirm it lists the project and offers the double-confirm force delete action).
