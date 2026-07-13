# Project Harness Page Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the Project Harness page to simplify the header into a single line, place action buttons inline, and lock overall page height to provide an independent 3-column scrollable workspace, preventing page-level scrollbars.

**Architecture:** Conditional viewport-height locking using CSS `:has()` selector. Combine metadata and files/editor UI into a single viewport-height filling `Card` container, displaying a 3-column split view (Files list, Editor, Metadata Sidebar).

**Tech Stack:** React, TypeScript, Tailwind/Vanilla CSS, Lucide icons.

## Global Constraints
- Avoid global vertical scrollbar on the right of the Project Harness page.
- Merge the metadata card and the files/editor card into a unified 3-column structure.
- Clean up text clutter (delete "PROJECT HARNESS" eyebrow and place title/description side-by-side).

---

### Task 1: CSS Styles for Page Height Locking & 3-Column Split View

**Files:**
- Modify: `src/features/projects/pages/project-harness.css`

**Interfaces:**
- Consumes: Existing styles in `project-harness.css`
- Produces: CSS layout rules for `.project-harness-page-container`, `.project-harness-header`, and the 3-column layout classes.

- [ ] **Step 1: Append Height Locking and Layout Grid CSS**

Add the following CSS declarations to the end of `src/features/projects/pages/project-harness.css`:

```css
/* Height Locking for Project Harness Page */
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

/* Container & Header Redesign */
.project-harness-page-container {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  overflow: hidden;
}

.project-harness-header {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  margin-bottom: var(--space-1);
}

.project-harness-header-title {
  display: flex;
  align-items: baseline;
  gap: var(--space-2);
}

.project-harness-header-title h2 {
  margin: 0;
  font-size: 1.45rem;
  font-weight: 700;
}

.project-harness-header-desc {
  color: var(--color-muted);
  font-size: 0.85rem;
}

.project-harness-header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

/* 3-Column Split View for Editor */
.project-harness-main-card {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  padding: var(--space-3) !important;
}

.project-harness-editor-layout-new {
  display: grid;
  grid-template-columns: 220px minmax(0, 1fr) 240px;
  gap: var(--space-3);
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

/* Column 1: Files List */
.project-harness-file-list-new {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--color-outline);
  padding-right: var(--space-2);
  overflow: hidden;
}

.project-harness-file-list-new h3 {
  margin: 0 0 var(--space-2) 0;
  font-size: 0.95rem;
  font-weight: 700;
}

.project-harness-file-list-scroll {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  padding-right: 4px;
}

.project-harness-file-list-scroll button {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 0.45rem;
  border: 0;
  background: transparent;
  color: var(--color-ink);
  text-align: left;
  padding: 0.55rem 0.5rem;
  border-radius: 5px;
  cursor: pointer;
  transition: background 0.15s;
}

.project-harness-file-list-scroll button:hover {
  background: var(--color-surface-soft);
}

.project-harness-file-list-scroll button.is-active {
  background: color-mix(in srgb, var(--color-primary) 13%, var(--color-surface));
  color: var(--color-primary-ink);
  font-weight: 700;
}

.project-harness-file-list-scroll button span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.83rem;
  flex: 1;
}

.project-harness-file-list-scroll button i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #c66a00;
  margin-left: auto;
  flex-shrink: 0;
}

/* Column 2: Code Editor */
.project-harness-editor-new {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.project-harness-editor-new .project-harness-empty-editor {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--color-muted);
  font-size: 0.9rem;
}

.project-harness-editor-textarea-new {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  margin-top: var(--space-2);
}

.project-harness-editor-textarea-new textarea {
  flex: 1;
  width: 100%;
  height: 100%;
  resize: none;
  box-sizing: border-box;
  border: 1px solid var(--color-outline);
  border-radius: 5px;
  padding: 0.85rem;
  background: var(--color-canvas);
  color: var(--color-ink);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.86rem;
  line-height: 1.55;
  outline: none;
}

.project-harness-editor-textarea-new textarea:focus {
  border-color: var(--color-primary);
}

/* Column 3: Sidebar Properties */
.project-harness-meta-sidebar {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  border-left: 1px solid var(--color-outline);
  padding-left: var(--space-3);
  overflow-y: auto;
}

.project-harness-sidebar-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.project-harness-sidebar-section h4 {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-muted);
  letter-spacing: 0.05em;
}

.project-harness-sidebar-kv {
  display: flex;
  flex-direction: column;
  gap: 0.65rem;
}

.project-harness-sidebar-kv-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.82rem;
}

.project-harness-sidebar-kv-label {
  color: var(--color-muted);
}

.project-harness-sidebar-kv-value {
  color: var(--color-ink);
  font-weight: 700;
}

.project-harness-sidebar-warnings {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}
```

- [ ] **Step 2: Commit CSS Changes**

Run:
```bash
git add src/features/projects/pages/project-harness.css
git commit -m "style: add height locking and 3-column classes for project harness"
```

---

### Task 2: Redesign the HarnessPage React Component

**Files:**
- Modify: `src/features/projects/pages/HarnessPage.tsx`

**Interfaces:**
- Consumes: API queries and hooks defined in `HarnessPage.tsx`
- Produces: Updated single-line header rendering and restructured `ManagedHarnessEditor` component.

- [ ] **Step 1: Update page wrapper and header in `HarnessPage.tsx`**

Replace the root wrapper and header section (around lines 160-164) with the new title layout and action buttons.

Existing code in `HarnessPage.tsx`:
```tsx
  return (
    <div className="page-stack project-harness-page">
      <div className="project-harness-header">
        <div><p className="eyebrow">PROJECT HARNESS</p><h2>项目 Harness</h2><p className="muted-copy">从模板创建项目副本，之后由项目独立维护。</p></div>
        <StatusBadge tone={statusQuery.data.state === 'managed' ? 'success' : statusQuery.data.state === 'invalid' ? 'danger' : 'neutral'}>{statusLabel}</StatusBadge>
      </div>
```

New code replacing it:
```tsx
  return (
    <div className="project-harness-page-container">
      <div className="project-harness-header">
        <div className="project-harness-header-title">
          <h2>项目 Harness</h2>
          <span className="project-harness-header-desc">从模板创建项目副本，之后由项目独立维护。</span>
        </div>
        <div className="project-harness-header-actions">
          {(statusQuery.data.state === 'managed' || statusQuery.data.state === 'invalid') && (
            <>
              <button
                type="button"
                className="button button--secondary"
                onClick={() => void statusQuery.refetch()}
              >
                <RefreshCw size={15} /> 刷新磁盘状态
              </button>
              <button
                type="button"
                className="button button--secondary"
                onClick={() => unmanageMutation.mutate()}
                disabled={unmanageMutation.isPending}
              >
                解除纳管
              </button>
            </>
          )}
          <StatusBadge tone={statusQuery.data.state === 'managed' ? 'success' : statusQuery.data.state === 'invalid' ? 'danger' : 'neutral'}>{statusLabel}</StatusBadge>
        </div>
      </div>
```

- [ ] **Step 2: Restructure `ManagedHarnessEditor` component**

Replace the entire `ManagedHarnessEditor` function and component definition (lines 205-231) to implement the 3-column viewport-locked editor.

Existing code:
```tsx
interface ManagedHarnessEditorProps {
...
}

function ManagedHarnessEditor(...) {
  return <>
    <Card>...</Card>
    <Card>...</Card>
  </>;
}
```

New code:
```tsx
interface ManagedHarnessEditorProps {
  status: ProjectHarnessStatus;
  files: ProjectHarnessFile[];
  selectedFile: string;
  activeFile?: ProjectHarnessFile;
  draft: string;
  newFilePath: string;
  setNewFilePath: (value: string) => void;
  onOpenFile: (path: string) => void;
  onDraftChange: (value: string) => void;
  onSave: () => void;
  onDelete: (path: string) => void;
  onCreate: () => void;
  savePending: boolean;
  deletePending: boolean;
  createPending: boolean;
}

function ManagedHarnessEditor({
  status,
  files,
  selectedFile,
  activeFile,
  draft,
  newFilePath,
  setNewFilePath,
  onOpenFile,
  onDraftChange,
  onSave,
  onDelete,
  onCreate,
  savePending,
  deletePending,
  createPending,
}: ManagedHarnessEditorProps) {
  return (
    <Card className="project-harness-main-card">
      <div className="project-harness-editor-layout-new">
        {/* Column 1: Harness File List */}
        <div className="project-harness-file-list-new">
          <h3>Harness 文件</h3>
          <div className="project-harness-file-list-scroll">
            {files.map((file) => (
              <button
                type="button"
                key={file.path}
                className={selectedFile === file.path ? 'is-active' : ''}
                onClick={() => onOpenFile(file.path)}
              >
                <FileText size={15} />
                <span>{file.path}</span>
                {file.changedSinceApply && <i title="已修改" />}
              </button>
            ))}
          </div>
          <div className="project-harness-create-file">
            <input
              value={newFilePath}
              onChange={(event) => setNewFilePath(event.target.value)}
              aria-label="新 Harness 文件路径"
            />
            <button
              type="button"
              className="button button--secondary"
              onClick={onCreate}
              disabled={!newFilePath.trim() || createPending}
            >
              新增文件
            </button>
          </div>
        </div>

        {/* Column 2: Editor */}
        <div className="project-harness-editor-new">
          {activeFile ? (
            <>
              <div className="project-harness-editor-toolbar" style={{ padding: '0 0 var(--space-2) 0', borderBottom: '1px solid var(--color-outline)' }}>
                <code>{activeFile.path}</code>
                <div className="project-harness-actions">
                  <button
                    type="button"
                    className="button button--primary"
                    onClick={onSave}
                    disabled={savePending}
                  >
                    <Save size={15} /> 保存
                  </button>
                  <button
                    type="button"
                    className="button button--secondary"
                    disabled={deletePending}
                    onClick={() => onDelete(activeFile.path)}
                  >
                    删除
                  </button>
                </div>
              </div>
              <div className="project-harness-editor-textarea-new">
                <textarea
                  value={draft}
                  onChange={(event) => onDraftChange(event.target.value)}
                  spellCheck={false}
                  aria-label={`编辑 ${activeFile.path}`}
                />
              </div>
            </>
          ) : (
            <div className="project-harness-empty-editor">选择一个 Harness 文件开始编辑。</div>
          )}
        </div>

        {/* Column 3: Sidebar properties */}
        <div className="project-harness-meta-sidebar">
          <div className="project-harness-sidebar-section">
            <h4>Harness 属性</h4>
            <div className="project-harness-sidebar-kv">
              <div className="project-harness-sidebar-kv-item">
                <span className="project-harness-sidebar-kv-label">来源模板</span>
                <strong className="project-harness-sidebar-kv-value">{status.sourceTemplateId || '未知'}</strong>
              </div>
              <div className="project-harness-sidebar-kv-item">
                <span className="project-harness-sidebar-kv-label">来源状态</span>
                <strong className="project-harness-sidebar-kv-value">
                  {status.sourceStatus === 'changed'
                    ? '模板已有变化'
                    : status.sourceStatus === 'deleted'
                    ? '原模板已删除'
                    : '独立项目副本'}
                </strong>
              </div>
              <div className="project-harness-sidebar-kv-item" style={{ flexDirection: 'column', alignItems: 'flex-start', gap: '0.25rem' }}>
                <span className="project-harness-sidebar-kv-label">应用时间</span>
                <strong className="project-harness-sidebar-kv-value" style={{ wordBreak: 'break-all', fontSize: '0.78rem', marginTop: '0.1rem' }}>
                  {status.appliedAt || '未知'}
                </strong>
              </div>
            </div>
          </div>

          {status.warnings.length > 0 && (
            <div className="project-harness-sidebar-section">
              <h4>警告信息</h4>
              <div className="project-harness-sidebar-warnings">
                {status.warnings.map((warning: string) => (
                  <p
                    key={warning}
                    className="project-harness-warning"
                    style={{ display: 'flex', alignItems: 'flex-start', gap: '0.35rem', margin: '0.2rem 0', color: '#a15c00', fontSize: '0.78rem', lineHeight: '1.4' }}
                  >
                    <AlertCircle size={14} style={{ flexShrink: 0, marginTop: '0.15rem' }} />
                    <span>{warning}</span>
                  </p>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </Card>
  );
}
```

Wait, also make sure we remove `onRefresh`, `onUnmanage` and `unmanagePending` references from `<ManagedHarnessEditor ... />` instantiation inside `HarnessPage` (around line 200).

Line 200 inside `HarnessPage`:
```tsx
      {(statusQuery.data.state === 'managed' || statusQuery.data.state === 'invalid') && <ManagedHarnessEditor status={statusQuery.data} files={files} selectedFile={selectedFile} activeFile={activeFile} draft={draft} newFilePath={newFilePath} setNewFilePath={setNewFilePath} onOpenFile={(path) => void openFile(path)} onDraftChange={setDraft} onSave={() => saveMutation.mutate()} onDelete={(path) => { if (window.confirm(`确定删除 ${path} 吗？`)) deleteFileMutation.mutate(path); }} onCreate={() => createFileMutation.mutate()} onRefresh={() => void statusQuery.refetch()} onUnmanage={() => unmanageMutation.mutate()} savePending={saveMutation.isPending} deletePending={deleteFileMutation.isPending} createPending={createFileMutation.isPending} unmanagePending={unmanageMutation.isPending} />}
```

Should be changed to:
```tsx
      {(statusQuery.data.state === 'managed' || statusQuery.data.state === 'invalid') && (
        <ManagedHarnessEditor
          status={statusQuery.data}
          files={files}
          selectedFile={selectedFile}
          activeFile={activeFile}
          draft={draft}
          newFilePath={newFilePath}
          setNewFilePath={setNewFilePath}
          onOpenFile={(path) => void openFile(path)}
          onDraftChange={setDraft}
          onSave={() => saveMutation.mutate()}
          onDelete={(path) => {
            if (window.confirm(`确定删除 ${path} 吗？`)) deleteFileMutation.mutate(path);
          }}
          onCreate={() => createFileMutation.mutate()}
          savePending={saveMutation.isPending}
          deletePending={deleteFileMutation.isPending}
          createPending={createFileMutation.isPending}
        />
      )}
```

- [ ] **Step 3: Commit component updates**

Run:
```bash
git add src/features/projects/pages/HarnessPage.tsx
git commit -m "feat: redesign project harness page layout and ManagedHarnessEditor"
```

---

### Task 3: Verify and Build Layout Redesign

**Files:**
- None (testing commands only)

**Interfaces:**
- Consumes: All updated pages and styles.
- Produces: Correct type compilation and layout rendering.

- [ ] **Step 1: Check TypeScript type correctness**

Run the build script to compile TypeScript.
Run: `npm run build`
Expected output: No compilation errors.

- [ ] **Step 2: Verify in Vite development mode**

Run: `npm run dev`
Expected: Dev server runs successfully at http://127.0.0.1:1420.
Verify in browser devtools:
- Navigate to the "Project Harness" page.
- Check that the page header fits onto a single line.
- Verify that the layout splits into exactly 3 columns (Files tree, Code editor, Metadata sidebar).
- Confirm that there are no page-level scrollbars, and that each of the three columns scroll independently.
