# Skill Pack Detail Layout Redesign Spec

## Goal

Improve the visual hierarchy, layout, and usability of the Skill Pack detail modal (`SkillDetailModal`). Currently, the list of child skills is mixed into the right configuration sidebar, making it easily overlooked or crowded. The new design moves the list of child skills (members) into the main content panel as a rich grid of cards, while keeping the right sidebar purely for configuration and action buttons.

## User Experience and Layout Changes

### 1. Left Main Content Area (`.modal-markdown-area`)

The left-hand content pane will have two states:

#### State A: Default Package View (No sub-skill selected)
- **Top Header**: Displays the skill package name `<h1>{skill.metadata.name}</h1>`, and potentially other top-level package metadata (version, author, etc.).
- **Overview Content**: If the skill package contains a top-level README (`skill.html_content` is present), it will render this HTML content in a `.markdown-body` container.
- **Subskills Section**:
  - Below the README (or at the top if no README exists), a clean section heading: `<h3>所含子技能 ({skill.members.length})</h3>`.
  - A responsive CSS Grid (`.pack-members-grid`) displaying the child skills as cards.
  - **Child Skill Card (`.pack-member-card`)**:
    - **Header**: Contains a small folder/package icon and the sub-skill name (`member.metadata.name`).
    - **Body**: The description (`member.metadata.description`), clamped to a maximum of 3 lines for alignment.
    - **Footer**: Displays the child skill's version (if any) and a call-to-action text: `查看详情 →`.
    - **Interactions**: Clicking the card sets the selected sub-skill state (`selectedMember`), transitioning the pane to State B. Hovering over a card triggers a smooth transition:
      - Border color changes to the primary brand color (`var(--color-primary)`).
      - Card translates slightly upwards (`translateY(-4px)`).
      - Subtle box-shadow glow.

#### State B: Sub-skill Detail View (Sub-skill selected)
- **Navigation Header**: Displays a back link `← 返回 {skill.metadata.name}`. Clicking this returns to the Default Package View (State A).
- **Sub-skill Content**: Displays the selected sub-skill name as an `<h1>` followed by its specific documentation (`member.html_content`) rendered inside `.markdown-body`.

---

### 2. Right Configuration Sidebar (`.modal-meta-editor`)

- **Sub-skill Navigation Removal**: The `.pack-members` list is completely removed from the right sidebar. This eliminates duplication and frees up vertical space.
- **Content**:
  - Package status actions (e.g., "Install Update", "Trust version").
  - Warning messages (e.g., security warnings).
  - "Set Category" dropdown field.
  - "User Notes and Remarks" textarea (with flexible size).
  - Save / Cancel actions footer.

---

## Styling Specifications

### CSS Classes and Rules to Add/Modify in `skills.css`

1. **`.pack-members-grid`**:
   - `display: grid;`
   - `grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));`
   - `gap: var(--space-2);`
   - `margin-top: var(--space-2);`

2. **`.pack-member-card`**:
   - `border: 1px solid var(--color-outline);`
   - `border-radius: var(--radius-md);`
   - `background: var(--color-surface);`
   - `padding: var(--space-2);`
   - `display: flex;`
   - `flex-direction: column;`
   - `cursor: pointer;`
   - `transition: transform 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;`

3. **`.pack-member-card:hover`**:
   - `transform: translateY(-3px);`
   - `border-color: var(--color-primary);`
   - `box-shadow: 0 4px 12px color-mix(in srgb, var(--color-primary) 12%, transparent);`

4. **`.pack-member-card__header`**:
   - `display: flex;`
   - `align-items: center;`
   - `gap: 8px;`
   - `margin-bottom: 6px;`
   - `color: var(--color-ink);`

5. **`.pack-member-card__header h4`**:
   - `margin: 0;`
   - `font-size: 0.95rem;`
   - `font-weight: 600;`

6. **`.pack-member-card__desc`**:
   - `font-size: 0.8rem;`
   - `color: var(--color-muted);`
   - `margin: 0 0 12px 0;`
   - `line-height: 1.4;`
   - `display: -webkit-box;`
   - `-webkit-line-clamp: 3;`
   - `-webkit-box-orient: vertical;`
   - `overflow: hidden;`
   - `height: 3.36rem;`

7. **`.pack-member-card__footer`**:
   - `margin-top: auto;`
   - `display: flex;`
   - `justify-content: space-between;`
   - `align-items: center;`
   - `font-size: 0.72rem;`

8. **`.pack-member-card__version`**:
   - `color: var(--color-muted);`

9. **`.pack-member-card__action`**:
   - `color: var(--color-primary-ink);`
   - `font-weight: 500;`

10. **`.member-back`**:
    - Update spacing and font styles to align perfectly with the page grid:
    - `display: inline-flex;`
    - `align-items: center;`
    - `gap: 4px;`
    - `margin-bottom: var(--space-2);`
    - `font-size: 0.85rem;`
    - `font-weight: 500;`
    - `color: var(--color-primary-ink);`

---

## Verification Plan

### Automated Tests
- Verification of page build and compilation: `npm run build`
- Unit tests: Run `npm run test:run` to ensure all existing tests pass.

### Manual Verification
- Launch the application: `npm run tauri:dev` or Vite dev server `npm run dev` (we will check where the UI displays).
- Open the Skill catalog, click on a Skill Pack (e.g., a pack containing multiple skills like `taste-skill` or standard packs).
- Confirm that:
  - The main area renders the package title, README content (if present), and a grid of child skill cards.
  - Hovering over a card shows visual transformation and active shadow.
  - Clicking a card correctly loads the child skill documentation.
  - A back button is rendered above the child skill document and works as expected.
  - The right sidebar is clean, not crowded, and does not show the duplicated sub-skill buttons list.
