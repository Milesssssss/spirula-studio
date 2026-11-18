---
name: Spirula Studio
description: A native reconstruction workbench with graphite surfaces, measured controls, and a clear scene focus.
colors:
  text: '#E8EDF2'
  text-disabled: '#B0BFCC'
  window-bg: '#11161C'
  child-bg: '#171E26'
  popup-bg: '#1D2731'
  border: '#334352'
  border-shadow: rgba(0, 0, 0, 0)
  frame-bg: '#25313D'
  frame-bg-hovered: '#304253'
  frame-bg-active: '#2D4358'
  title-bg: '#151C23'
  title-bg-active: '#1D2832'
  title-bg-collapsed: '#151C23'
  menu-bar-bg: '#151C23'
  scrollbar-bg: '#11161C'
  scrollbar-grab: '#3A4B5C'
  scrollbar-grab-hovered: '#50677D'
  scrollbar-grab-active: '#65829D'
  check-mark: '#87BDEC'
  checkbox-selected-bg: '#264C6B'
  slider-grab: '#35638A'
  slider-grab-active: '#2E6D9D'
  button: '#293745'
  button-hovered: '#304253'
  button-active: '#34485B'
  header: '#222D38'
  header-hovered: '#2D4154'
  header-active: '#2C4357'
  separator: '#2A3947'
  separator-hovered: '#659FD0'
  separator-active: '#8CC4F3'
  resize-grip: rgba(80, 103, 125, 0.25)
  resize-grip-hovered: rgba(101, 159, 208, 0.65)
  resize-grip-active: '#8CC4F3'
  input-text-cursor: '#CDE2F5'
  tab-hovered: '#2A3D50'
  tab: '#1B242D'
  tab-selected: '#263F55'
  tab-selected-overline: '#77B9EC'
  tab-dimmed: '#171E26'
  tab-dimmed-selected: '#22313F'
  tab-dimmed-selected-overline: '#47657D'
  plot-lines: '#88BCE6'
  plot-lines-hovered: '#C1DDF5'
  plot-histogram: '#2E6D9D'
  plot-histogram-hovered: '#66A7D8'
  table-header-bg: '#202C37'
  table-border-strong: '#334352'
  table-border-light: '#26333F'
  table-row-bg: rgba(23, 30, 38, 0)
  table-row-bg-alt: rgba(48, 66, 83, 0.18)
  text-link: '#8DC7F4'
  text-selected-bg: rgba(53, 109, 158, 0.65)
  tree-lines: '#405367'
  drag-drop-target: '#89C9F8'
  drag-drop-target-bg: rgba(53, 109, 158, 0.24)
  unsaved-marker: '#E6B765'
  nav-cursor: '#8CC4F3'
  nav-windowing-highlight: rgba(220, 233, 245, 0.75)
  nav-windowing-dim-bg: rgba(12, 18, 25, 0.68)
  modal-window-dim-bg: rgba(8, 13, 19, 0.72)
  action-primary: '#225B86'
  action-primary-hovered: '#2B6A99'
  action-primary-active: '#225075'
  action-selected: '#263F55'
  action-selected-hovered: '#304E67'
  action-selected-active: '#29445A'
  action-danger: '#773F47'
  action-danger-hovered: '#8D4C54'
  action-danger-active: '#65363E'
  command-primary-detail: '#D7E8F7'
  status-ok: rgb(35% 85% 45%)
  status-error: rgb(100% 42% 42%)
  status-warning: rgb(95% 75% 30%)
  status-dim: rgb(65% 69% 73%)
  render-status-error: rgb(100% 45% 45%)
  render-status-dim: rgb(62% 64% 68%)
  shortcut-hint: rgba(255, 255, 255, 0.35294117647058826)
typography:
  display:
    fontFamily: Spirula UI
    fontSize: 25.6px
  headline:
    fontFamily: Spirula UI
    fontSize: 19.2px
  title:
    fontFamily: Spirula UI
    fontSize: 18.4px
  body:
    fontFamily: Spirula UI
    fontSize: 16px
  label:
    fontFamily: Spirula UI
    fontSize: 16px
rounded:
  window: 6px
  popup: 6px
  child: 4px
  frame: 4px
  scrollbar: 4px
  grab: 3px
  tab: 4px
spacing:
  window-padding: 10px 12px
  frame-padding: 5px 8px
  item-x: 8px
  item-y: 6px
  inner-x: 6px
  inner-y: 4px
  cell-padding: 5px 8px
  indent: 20px
  separator-text-padding: 6px 8px
  command-padding: 10px 12px
  command-gap: 4px
  grab-minimum: 12px
components:
  button-neutral:
    backgroundColor: '{colors.button}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-neutral-hovered:
    backgroundColor: '{colors.button-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-neutral-active:
    backgroundColor: '{colors.button-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-primary:
    backgroundColor: '{colors.action-primary}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-primary-hovered:
    backgroundColor: '{colors.action-primary-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-primary-active:
    backgroundColor: '{colors.action-primary-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected:
    backgroundColor: '{colors.action-selected}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected-hovered:
    backgroundColor: '{colors.action-selected-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected-active:
    backgroundColor: '{colors.action-selected-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger:
    backgroundColor: '{colors.action-danger}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger-hovered:
    backgroundColor: '{colors.action-danger-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger-active:
    backgroundColor: '{colors.action-danger-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-key:
    backgroundColor: '{colors.button}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-key-on:
    backgroundColor: '{colors.button-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-toggle:
    backgroundColor: '{colors.frame-bg}'
    textColor: '{colors.text-disabled}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-toggle-on:
    backgroundColor: '{colors.button-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  input:
    backgroundColor: '{colors.frame-bg}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  tab:
    backgroundColor: '{colors.tab}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.tab}'
    padding: '{spacing.frame-padding}'
  tab-hovered:
    backgroundColor: '{colors.tab-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.tab}'
    padding: '{spacing.frame-padding}'
  tab-selected:
    backgroundColor: '{colors.tab-selected}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.tab}'
    padding: '{spacing.frame-padding}'
  workspace-actions:
    backgroundColor: '{colors.child-bg}'
    textColor: '{colors.text}'
    rounded: '{rounded.child}'
    padding: '{spacing.window-padding}'
  popup:
    backgroundColor: '{colors.popup-bg}'
    textColor: '{colors.text}'
    rounded: '{rounded.popup}'
    padding: '{spacing.window-padding}'
  command-row:
    backgroundColor: '{colors.button}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.command-padding}'
  command-row-primary:
    backgroundColor: '{colors.action-primary}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.command-padding}'
  scrollbar:
    backgroundColor: '{colors.scrollbar-bg}'
    rounded: '{rounded.scrollbar}'
    width: 12px
  slider-grab:
    backgroundColor: '{colors.slider-grab}'
    rounded: '{rounded.grab}'
  field-hovered:
    backgroundColor: '{colors.frame-bg-hovered}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  field-active:
    backgroundColor: '{colors.frame-bg-active}'
    textColor: '{colors.text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
---

# Design System: Spirula Studio

## Overview

**Creative North Star: "Reconstruction laboratory workbench"**

Spirula Studio is a native Dear ImGui/GLFW workspace. Graphite surfaces, neutral controls, restrained blue state cues, and fine boundaries support dense reconstruction work. The embedded Source Sans-derived face and regional CJK chain provide one continuous typographic voice across the thirteen interface languages.

Context, execution, content, and diagnostics have distinct places. Reusable controls measure translated text and adapt to the available pane; persistent actions stay reachable beside independently scrolling content. Input feedback is immediate, without decorative entrance animation. This contract records the implementation; Home and Render composition remain in the surface brief.

**Key Characteristics:**

- Tonal surface layers and fine boundaries.
- Neutral ordinary controls; explicit primary, selected, and danger roles.
- One embedded Regular font chain with a small size hierarchy.
- Measured wrapping, scaled geometry, and independently scrolling work areas.

The frontmatter is normative for shared token values. All size values are
unscaled logical pixels; padding strings use CSS order (vertical, horizontal).
Native code remains authoritative for metrics, interaction, and rendering.
The sidecar's HTML/CSS swatches explain these roles at scale 1; they are not
runtime components or pixel-equivalent native captures.

## Colors

The palette moves from deep graphite to blue-gray controls, with pale text and
restrained blue interaction cues. Frontmatter keys are the kebab-case names of
all 61 `ImGuiCol_*` assignments in `Layout.cpp`, followed by the explicit
`StudioUi.cpp` action roles and existing status/shortcut colors.

### Primary

- **Action blue:** `action-primary` and its hover/active variants identify
  the primary operation. `command-primary-detail` carries supporting text
  inside a primary command row.
- **Selection blue:** `action-selected` belongs to `ButtonRole::Selected`;
  `tab-selected` belongs to native tabs. Pressed key/toggle buttons use
  `button-active`, as their helpers specify.
- **Focus and progress:** `nav-cursor`, `separator-active`, and
  `resize-grip-active` mark interaction; `plot-histogram` supplies the shared
  progress fill. Roles remain separate even where source values coincide.

### Secondary

- **Destructive action:** `action-danger` has its own hovered and active fills.
- **Job status:** `status-ok`, `status-error`, `status-warning`, and `status-dim`
  preserve `GuiApp.cpp`'s existing status text. Render uses the same success
  color but carries the separately recorded `render-status-error` and
  `render-status-dim`; these local differences are not a new global palette.
- **Unsaved state:** `unsaved-marker` remains distinct from primary selection.

### Neutral

`window-bg`, `child-bg`, and `popup-bg` establish surface layers. `frame-bg` is
the field surface; `button` is the ordinary command surface. `text` carries
primary content; `text-disabled` carries secondary context and native disabled
text. Borders and separators establish fine boundaries. Alpha-bearing
selection, row, drag, and dimming tokens preserve source alpha instead of
flattening it against an assumed background.

**The Meaning Before Accent Rule.** Use the existing action, selection, focus, and status roles; a blue fill does not make every ordinary command primary.

## Typography

The shipped face is `SpirulaUI-Regular.ttf`, a renamed subset of Source Sans 3.
`FontSet::rebuild()` installs it first, then an available full regional CJK face,
then embedded `SpirulaCJK-{JP,SC,TC,KR}` subsets with the current region first.
These are merged native glyph sources, not a fixed CSS fallback order.

| Token | Implemented role | Source relationship |
| --- | --- | --- |
| `body` / `label` | Fields, buttons, paths, data, descriptions | Base rasterization size in `Fonts.cpp` |
| `display` | Compact product heading | Base size multiplied by 1.6 |
| `headline` | Shared workspace title | Base size multiplied by 1.2 |
| `title` | Existing license/device heading emphasis | Base size multiplied by 1.15 |

The Regular face supplies this hierarchy. The source introduces no separate
bold or monospaced face, tracking rule, or CSS line-height scale. Measure text
and frame heights with ImGui. A frame's height is the current font height plus
twice the scaled vertical frame padding.

The embedded chain covers the interface languages, not arbitrary writing
systems. The font reference records missing shaping/bidi support and limits
for other scripts and combining marks; this contract adds no coverage claim.

**The Measure the Language Rule.** Measure rendered localized text with the current ImGui font and scale before placing controls, paths, or shortcut hints.

## Layout

Let `s = ui_scale()`. `px(x)` returns `x * s`. `apply_style(s)` starts from a fresh
`ImGuiStyle`, assigns base tokens, calls `ScaleAllSizes(s)`, and sets
`FontScaleMain = s`. The pinned ImGui implementation truncates scaled style
sizes to integer pixels; `px()` itself does not. Text is rasterized at the
scaled size. Saved panel extents remain unscaled and convert at their use site.

A fixed user multiplier combines with monitor DPI. Auto scale uses the smaller
of the DPI-normalized width divided by 1600 and height divided by 950, clamps
it to 0.80–1.25, then snaps in steps of 0.05 before multiplying by DPI.

`FlowRow` measures each complete control or group against available content
width, with the scaled `item-x` spacing. It wraps before a control that does not fit;
tooltip drawing does not alter the next placement. These are native pane rules,
not device categories or web-viewport breakpoints.

| Reused native helper | Implemented geometry |
| --- | --- |
| `draw_workspace_actions` | Bordered child with measured automatic height, separate from scrolling content |
| `draw_workspace_panels` | Below available width `960 * s`, full-width pages; otherwise inspector beside Preview |
| Shared wide inspector | Minimum `320 * s`; maximum `max(320 * s, min(45% of width, width - 480 * s - splitter_extent()))` |
| Viewer sidebar | Minimum `min(320 * s, 45% of width)`; maximum `max(minimum, min(45% of width, width - 320 * s - splitter_extent()))` |
| Splitter | Grab width `max(4, 6 * s)`; `splitter_extent()` also includes vertical item spacing |
| Workspace log | Maximum `max(0, min(30% of available height, available height - 240 * s - splitter_extent() - style.ItemSpacing.y))` |

The log limit becomes zero when it cannot contain one native text line with
spacing, both window-padding edges, and the horizontal scrollbar. Clamping
rendered geometry does not replace the saved width or visibility preference.
Viewer retains its own sidebar arrangement. Home column strategy and short
Render-pane control relocation remain in the
[surface brief](.impeccable/surfaces/src-app-gui-guiapp-cpp.md) and
[workspace note](docs/notes/studio-workspace-redesign.md).

## Elevation & Depth

Shared surfaces use tonal layering and native borders, with transparent
`border-shadow`; there is no decorative cast-shadow token. Popups and modal
dimming provide contextual separation. Hover, active, focus, and selection
feedback changes immediately. Playback and progress represent product state,
not an animation vocabulary.

**The Tonal Boundaries Rule.** Separate shared surfaces with their existing fill and border roles; the shared theme has no cast-shadow layer.

## Shapes

Windows and popups use the larger corner tokens; children, frames, scrollbars,
and tabs use compact corners. Slider grabs have a smaller radius; `grab-minimum` is a minimum size, not a fixed slider length. Native tabs
round their upper corners. Window, child, popup, tab-bar, and separator-text
boundaries are one logical pixel before style scaling; ordinary frames and
tabs have no border. The configured tab overline is two logical pixels and
appears only where the native tab-bar flags request it.

Clip content within native items and panes. Resize handles retain a visible
separator and change to the native resize cursor during interaction. There is
no shared pill, ornamental outline, or card-shadow component.

## Components

### Buttons

Ordinary buttons use the neutral palette. `ButtonStyle` scopes the three
explicit action roles with balanced color pushes. Default labels are centered;
geometry comes from measured content and frame padding. Hover/active colors
come from named variants, keyboard navigation uses `nav-cursor`, and disabled
interaction retains native ImGui behavior.

`KeyButton` measures label, shortcut, inner gap, and both horizontal padding
edges. When both texts fit, it left-aligns the label and draws a separate
right-hand hint. Otherwise the label remains centered and the shortcut moves
to hover help. Its translated button ID and keyboard handlers remain intact.
An enabled key/toggle state uses `button-active`, including hover, rather than
borrowing the Selected role.

### Inputs / Fields

Text fields retain `frame-bg` during hover and editing. Numeric drag/slider
fields use the existing hovered/active frame roles. Both share frame radius
and padding; text selection and cursor use their named native colors. Editing
and focus remain native, with no additional validation or focus animation.

### Cards / Containers

The recurring container is a bordered native child for action bands,
inspectors, and model panes. It uses `child-bg`, child radius, and native window
padding where the child requests it. Actions size to content; settings and
preview pages own their scrolling areas. This does not introduce a separate
dashboard-card abstraction.

### Navigation

`workspace_header` combines Home, workspace title, and an optional object path.
It measures the title, moves text to another line as needed, and uses UTF-8-safe
middle elision for a single-line context path with full hover text. The helper
reports Home; callers retain worker/document guards.

Workspace tabs use native tab states. `ButtonRole::Selected` also supports
filter choices without a separate pill shape. Global menus retain standard
native menu and popup roles.

### Command rows

`command_row` is a native button with a stable translated message ID. It sizes
itself from wrapped title/description measurements, uses command padding and
the command gap, and clips drawn text within those bounds. The primary variant
changes the button palette and description color. Native keyboard navigation
and disabled behavior remain available.

### Viewports and transport

`ViewportPanel` and `CompareView` retain shared pane boundaries and measured
control rows. Render transport reserves the larger Play/Pause label and a
stable time/frame readout width; measured row count contributes to timeline
height. Timeline, camera gizmo, and scene overlay colors stay local to those
visualizations rather than becoming general UI accents.

## Do's and Don'ts

### Do:

- **Do** use ButtonStyle for Primary, Selected, and Danger roles and preserve the existing native button behavior.
- **Do** scale handwritten geometry with px(), keep saved panel sizes unscaled, and rebuild style from its base before scaling.
- **Do** measure localized labels, wrap complete controls with FlowRow, and retain full context in hover help when paths are elided.
- **Do** keep execution controls outside scrolling pages and keep job polling and dialogs outside visibility-dependent callbacks.
- **Do** use the embedded regional font chain and preserve translated message IDs when customizing a control.

### Don't:

- **Don't** introduce an unimplemented display face, weight hierarchy, letter spacing, shadow, or decorative animation as a system rule.
- **Don't** reserve shortcut space by guesswork or draw a hint over its label; use KeyButton's measured fit and tooltip fallback.
- **Don't** treat saved panel width as rendered pixels or overwrite the preference merely because a smaller window clamps it.
- **Don't** promote a Home, Render, timeline, or gizmo-specific composition or color into a universal rule.

Extraction sources: [Layout.cpp](src/app/gui/Layout.cpp) and
[Layout.h](src/app/gui/Layout.h); [StudioUi.cpp](src/app/gui/StudioUi.cpp);
[Fonts.cpp](src/app/gui/Fonts.cpp) and the [font reference](assets/fonts/README.md);
[TrainingWorkspace.cpp](src/app/gui/TrainingWorkspace.cpp);
[Ui.h](src/app/gui/Ui.h); [GuiApp.cpp](src/app/gui/GuiApp.cpp);
[CompareView.cpp](src/app/gui/CompareView.cpp);
[ViewportPanel.cpp](src/app/gui/ViewportPanel.cpp);
[RenderTimeline.cpp](src/app/gui/render/RenderTimeline.cpp) and
[RenderPanel.cpp](src/app/gui/render/RenderPanel.cpp).
Build and native-review evidence remain in the implementation note rather
than being asserted by this contract.
