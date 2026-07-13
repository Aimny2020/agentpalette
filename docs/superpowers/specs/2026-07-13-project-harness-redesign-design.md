# Project Harness Page Redesign Specification

This design document outlines the UI/UX changes for the **Project Harness Page** (rendered by `HarnessPage.tsx` at `/projects/harness`). The goal is to simplify the layout, display the page header on a single line, place action buttons inline, and prevent page-level vertical scrolling by implementing a viewport-height locked 3-column split layout when in the managed/invalid states.

## Proposed Layout Structure

### 1. Header (Single Row)
- Remove the eyebrow tag `PROJECT HARNESS`.
- Place the page title `项目 Harness` and description `从模板创建项目副本，之后由项目独立维护。` on the same line, with the title bold and the description muted.
- Place the "刷新磁盘状态" (Refresh Status) and "解除纳管" (Unmanage) buttons inline on the right side of the header.
- Place the `StatusBadge` next to the actions.

### 2. Main Workspace (3-Column Layout)
The metadata Card on top and the Editor Card on the bottom will be merged into a single viewport-height-locked container divided into three columns:
- **Left Column (width: ~220px)**: Harness File Tree
  - Displays the "Harness 文件" title.
  - Scrollable list of files in the project harness.
  - Fixed new file input and button at the bottom.
- **Center Column (flexible)**: Code Editor
  - Header toolbar with current filename, and "保存" (Save) / "删除" (Delete) buttons.
  - Textarea editor area that scrolls internally.
- **Right Column (width: ~240px)**: Instance Metadata & Warnings
  - Displays the "来源模板" (Source Template), "来源状态" (Source Status), and "应用时间" (Applied At) properties.
  - Displays warnings alerts from `status.warnings` if any are present.
  - Scrollable internally to prevent pushing the page down.

## CSS Height Locking Strategy
To prevent global scrollbars, when `.project-harness-page-container` is present, the parent wrappers will have height constraints:
```css
body:has(.project-harness-page-container) {
  overflow: hidden !important;
}

.app-shell:has(.project-harness-page-container) {
  height: 100vh;
  overflow: hidden !important;
}

.shell-body:has(.project-harness-page-container) {
  height: calc(100vh - 4rem);
  overflow: hidden !important;
  box-sizing: border-box;
}

.workspace-main:has(.project-harness-page-container) {
  height: 100%;
  overflow: hidden !important;
  display: flex;
  flex-direction: column;
}

.workspace-main:has(.project-harness-page-container) > .page-stack {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  gap: var(--space-2);
}
```

## Verification Plan

### Manual Verification
- Run the Tauri desktop app (`npm run tauri:dev` or `npm run dev` with browser testing).
- Check that the Project Harness page fits exactly within the viewport without displaying a right scrollbar.
- Verify that resizing the page adjusts the internal heights of the file list, text editor, and right metadata panel, and that each panel scrolls independently if its content overflows.
- Check that the title, description, actions, and status badge align neatly on a single row.
- Ensure the template choose view (`absent` state) is also responsive and fits nicely in the height-locked container.
