---
name: Spirula Studio
description: A native reconstruction editor with Blender-referenced neutral surfaces, compact controls, and a dominant
  scene.
colors:
  text: '#E6E6E6'
  text-disabled: '#B0B0B0'
  window-bg: '#282828'
  child-bg: '#303030'
  popup-bg: '#262626'
  border: '#161616'
  border-shadow: rgba(0, 0, 0, 0)
  frame-bg: '#1D1D1D'
  frame-bg-hovered: '#242424'
  frame-bg-active: '#181818'
  title-bg: '#242424'
  title-bg-active: '#303030'
  title-bg-collapsed: '#242424'
  menu-bar-bg: '#1D1D1D'
  scrollbar-bg: '#282828'
  scrollbar-grab: '#5C5C5C'
  scrollbar-grab-hovered: '#737373'
  scrollbar-grab-active: '#8C8C8C'
  check-mark: '#FFFFFF'
  checkbox-selected-bg: '#4772B3'
  slider-grab: '#3D639C'
  slider-grab-active: '#35578A'
  button: '#545454'
  button-hovered: '#606060'
  button-active: '#3D639C'
  header: '#3D3D3D'
  header-hovered: '#484848'
  header-active: '#3D639C'
  separator: '#161616'
  separator-hovered: '#777777'
  separator-active: '#FFA028'
  resize-grip: rgba(118, 118, 118, 0.25)
  resize-grip-hovered: rgba(160, 160, 160, 0.65)
  resize-grip-active: '#FFA028'
  input-text-cursor: '#71A8FF'
  tab-hovered: '#3D3D3D'
  tab: '#1D1D1D'
  tab-selected: '#303030'
  tab-selected-overline: '#FFA028'
  tab-dimmed: '#242424'
  tab-dimmed-selected: '#303030'
  tab-dimmed-selected-overline: '#707070'
  plot-lines: '#71A8FF'
  plot-lines-hovered: '#ACC8FF'
  plot-histogram: '#4772B3'
  plot-histogram-hovered: '#71A8FF'
  table-header-bg: '#3D3D3D'
  table-border-strong: '#161616'
  table-border-light: '#282828'
  table-row-bg: rgba(48, 48, 48, 0)
  table-row-bg-alt: rgba(255, 255, 255, 0.025)
  text-link: '#71A8FF'
  text-selected-bg: '#3D639C'
  tree-lines: '#656565'
  drag-drop-target: '#FFA028'
  drag-drop-target-bg: rgba(255, 160, 40, 0.12)
  unsaved-marker: '#FFA028'
  nav-cursor: '#FFA028'
  nav-windowing-highlight: rgba(230, 230, 230, 0.75)
  nav-windowing-dim-bg: rgba(17, 17, 17, 0.65)
  modal-window-dim-bg: rgba(17, 17, 17, 0.72)
  action-primary: '#4772B3'
  action-primary-hovered: '#4067A0'
  action-primary-active: '#345581'
  action-selected: '#3D639C'
  action-selected-hovered: '#4772B3'
  action-selected-active: '#345581'
  action-danger: '#8B444A'
  action-danger-hovered: '#9B4C52'
  action-danger-active: '#71363B'
  action-text: '#FFFFFF'
  status-ok: rgb(35% 85% 45%)
  status-error: rgb(100% 42% 42%)
  status-warning: rgb(95% 75% 30%)
  status-dim: rgb(65% 69% 73%)
  render-status-error: rgb(100% 45% 45%)
  render-status-dim: rgb(62% 64% 68%)
  shortcut-hint: rgba(255, 255, 255, 0.35294117647058826)
typography:
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
  window: 3px
  popup: 3px
  child: 2px
  frame: 3px
  scrollbar: 4px
  grab: 3px
  tab: 3px
  workspace-header: 0px
spacing:
  window-padding: 6px 8px
  frame-padding: 3px 6px
  item-x: 6px
  item-y: 4px
  inner-x: 4px
  inner-y: 3px
  cell-padding: 3px 6px
  indent: 20px
  separator-text-padding: 4px 6px
  command-padding: 4px 8px
  workspace-header-padding: 4px 6px
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
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-primary-hovered:
    backgroundColor: '{colors.action-primary-hovered}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-primary-active:
    backgroundColor: '{colors.action-primary-active}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected:
    backgroundColor: '{colors.action-selected}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected-hovered:
    backgroundColor: '{colors.action-selected-hovered}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-selected-active:
    backgroundColor: '{colors.action-selected-active}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger:
    backgroundColor: '{colors.action-danger}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger-hovered:
    backgroundColor: '{colors.action-danger-hovered}'
    textColor: '{colors.action-text}'
    typography: '{typography.label}'
    rounded: '{rounded.frame}'
    padding: '{spacing.frame-padding}'
  button-danger-active:
    backgroundColor: '{colors.action-danger-active}'
    textColor: '{colors.action-text}'
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
  workspace-header:
    backgroundColor: '{colors.title-bg}'
    textColor: '{colors.text}'
    typography: '{typography.body}'
    rounded: '{rounded.workspace-header}'
    padding: '{spacing.workspace-header-padding}'
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
    textColor: '{colors.action-text}'
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
---

# Design System: Spirula Studio

## Overview

**Creative North Star: "Blender-referenced reconstruction editor"**

Spirula Studio uses Blender's neutral dark editor grammar within its native
Dear ImGui/GLFW application. Compact controls, narrow context headers, fine
boundaries, and a large scene area support repeated reconstruction work.
Blue identifies action and editable-value states; orange identifies focus
and manipulation feedback. The product retains its own task names, workflows,
and embedded Source Sans-derived face with regional CJK glyphs.

Context, execution, content, and diagnostics have distinct places. Shared
controls measure translated text; properties and previews own their scrolling
areas, while execution controls remain reachable. Wide scene workspaces put
properties on the right and apply their changes to the scene in the same
frame. Feedback is immediate native input feedback. This document records the
built system; task composition stays in the surface brief.

**Key Characteristics:**

- Achromatic editor surfaces, dark fields, and fine tonal boundaries.
- Neutral ordinary controls, blue action/value states, and orange focus cues.
- One embedded Regular font chain with restrained size emphasis.
- Measured wrapping, scaled geometry, and independently scrolling work areas.

The frontmatter is normative for shared token values. Sizes are unscaled
logical pixels; padding strings use CSS order (vertical, horizontal). Native
code remains authoritative for metrics, interaction, and rendering. Sidecar
HTML/CSS swatches are explanatory scale-1 equivalents, not application code or
pixel-equivalent captures. Their eight-step OKLCH tonal ramps are synthesized
illustrations; they add no runtime colors. The
[reference and adaptation note](docs/notes/blender-ui-style.md) records the
Blender source and native validation scope; this system makes no claim of
pixel-perfect Blender replication.

## Colors

The palette is predominantly neutral gray. Frontmatter keys map all 61
`ImGuiCol_*` assignments in `Layout.cpp`, followed by `StudioUi.cpp`'s explicit
action colors and the retained job-status and shortcut colors.

### Primary

- **Action blue:** `action-primary` and its hovered/active variants identify
  a primary operation. `ButtonStyle` pairs all three explicit action roles
  with `action-text`, including the Selected and Danger variants.
- **Value and selection blue:** checked values use `checkbox-selected-bg`;
  slider fills, ordinary pressed controls, active group headers, and text
  selection use their own native roles. The darker shared blue supports
  ordinary `text`; the explicit Selected button uses its scoped white text.
- **Information blue:** `text-link`, `input-text-cursor`, and the plot roles
  carry their existing link, editing, and data meanings. Progress uses the
  native `plot-histogram` role.

### Secondary

- **Focus orange:** `nav-cursor`, active separators and resize grips, drag/drop
  targets, unsaved markers, and the selected tab overline share the focus
  accent. An overline appears only when native tab-bar flags request it.
- **Destructive action:** `action-danger` and its variants remain distinct
  from primary and selected operations.
- **Job status:** `status-ok`, `status-error`, `status-warning`, and `status-dim`
  preserve existing job feedback. Render retains its local error and dim
  variants and the shared success value. These local colors do not expand
  the general editor palette.

### Neutral

`window-bg` frames the application; `child-bg` supplies editor content.
`header` and `table-header-bg` group properties; `title-bg` supplies the narrow
workspace header. `popup-bg` distinguishes transient surfaces. Fields use
`frame-bg`, ordinary buttons use `button`, and selected native tabs use the
editor's neutral surface rather than an action fill.

`text` carries ordinary labels and content; `text-disabled` carries secondary
context and native disabled text. Borders and separators provide narrow dark
boundaries. Alpha-bearing row, drag, resize, and dimming tokens preserve their
source alpha instead of assuming a background. `border-shadow` is transparent.

**The Meaning Before Accent Rule.** Use action blue for action and value states, orange for the existing focus and manipulation roles, and neutral fills for ordinary controls.

## Typography

The shipped `SpirulaUI-Regular.ttf` is a renamed subset of Source Sans 3.
`FontSet::rebuild()` installs it first, then an available full regional CJK face,
then embedded `SpirulaCJK-{JP,SC,TC,KR}` subsets with the current region first.
These are merged native glyph sources, not a fixed CSS fallback order.

| Token | Implemented role | Source relationship |
| --- | --- | --- |
| `body` / `label` | Fields, buttons, paths, descriptions, and workspace titles | Base rasterization size in `Fonts.cpp` |
| `title` | Home product name and existing license/device heading emphasis | Base size multiplied by 1.15 |

The shared workspace header uses normal UI type. There is no separate large
workspace headline or display face. The Regular face supplies the hierarchy;
the source defines no separate bold or monospaced face, tracking rule, or CSS
line-height scale. Measure line and frame heights with ImGui. A frame's height
is the current font height plus twice the scaled vertical frame padding.

The embedded chain covers the interface languages, not arbitrary writing
systems. The [font reference](assets/fonts/README.md) records shaping, bidi,
other-script, and combining-mark limits; this contract adds no coverage claim.

**The Measure the Language Rule.** Measure rendered localized text with the current ImGui font and scale before placing controls, paths, or shortcut hints.

## Layout

Let `s = ui_scale()`. `px(x)` returns `x * s`. `apply_style(s)` starts from a fresh
`ImGuiStyle`, assigns the base tokens, calls `ScaleAllSizes(s)`, and sets
`FontScaleMain = s`. The pinned ImGui implementation truncates scaled style
sizes to integer pixels; `px()` itself does not. Text is rasterized at the
scaled size. Saved panel extents stay unscaled and convert at their use site.

A fixed user multiplier combines with monitor DPI. Auto scale uses the smaller
of the DPI-normalized width divided by 1600 and height divided by 950, clamps
it to 0.80–1.25, then snaps in steps of 0.05 before multiplying by DPI.

The compact spacing rhythm comes from window, frame, item, and inner-spacing
tokens. `FlowRow` measures each complete control or group against available
content width and wraps before an item that does not fit. Tooltip drawing does
not change the next placement. These are native pane rules, not web-device
categories.

| Reused native helper | Implemented geometry |
| --- | --- |
| `workspace_header` | Automatic height, square dark strip, explicit header padding, and normal UI type |
| `draw_workspace_actions` | Bordered child with measured automatic height, separate from scrolling content |
| `draw_workspace_panels` | Below available width `960 * s`, full-width tabs; otherwise Preview left and inspector right |
| Shared wide inspector | Minimum `320 * s`; maximum `max(320 * s, min(45% of width, width - 480 * s - splitter_extent()))` |
| Viewer sidebar | Minimum `min(320 * s, 45% of width)`; maximum `max(minimum, min(45% of width, width - 320 * s - splitter_extent()))` |
| Splitter | Hit width `max(4, 6 * s)`; `splitter_extent()` also includes vertical item spacing |
| Workspace log | Maximum `max(0, min(30% of available height, available height - 240 * s - splitter_extent() - style.ItemSpacing.y))` |

Preparation/training settings and model Edit/Render properties appear to the
right of the scene. Their callbacks still execute before the scene, keeping
edits visible in the same frame. Dragging the divider left widens properties;
dragging right widens the scene. Only an actual drag updates the unscaled
saved width. A window-size clamp does not replace a saved preference.

The log limit becomes zero when it cannot contain one native text line with
spacing, both window-padding edges, and the horizontal scrollbar. Render
reserves the measured transport/timeline height before assigning the log
budget; the timeline occupies the scene column below the viewports.

Home's command column and its width threshold, recent-list composition, and
short Render-pane control placement are surface decisions recorded in the
[surface brief](https://github.com/Milesssssss/spirula-studio/blob/design/studio-workspace-layout/.impeccable/surfaces/src-app-gui-guiapp-cpp.md) and
[implementation note](docs/notes/blender-ui-style.md).

## Elevation & Depth

Shared surfaces use tonal layering and native borders. There is no decorative
cast-shadow token; `border-shadow` remains transparent. Popups and modal
backdrops provide contextual separation. Hover, pressed, focus, and selection
states change immediately. Playback and progress communicate product state,
not a decorative motion vocabulary.

**The Tonal Boundaries Rule.** Separate shared surfaces with their existing fill and border roles; the shared theme has no cast-shadow layer.

## Shapes

Controls, windows, popups, tabs, and slider grabs use small corners. Bordered
children are slightly squarer; scrollbars retain their own corner token.
The workspace context strip is square. `grab-minimum` is a minimum size, not a
fixed slider length. Native tabs round their upper corners.

Window, child, popup, tab-bar, separator-text, and optional tab-overline
boundaries are one logical pixel before style scaling. Ordinary frames and
tabs have no border. Splitter lines are `max(1, s)` pixels thick inside their
larger drag area; hover or active dragging uses the orange separator role and
the native resize cursor.

Content clips within native items and panes. Shared containers are editor
regions, with no separate pill or shadowed-card abstraction.

## Components

### Buttons

Compact, neutral controls carry routine commands. `ButtonStyle` scopes
Primary, Selected, and Danger fills and white foreground with balanced style
pushes. Default labels are centered; geometry comes from measured content and
frame padding. Hover and pressed states use named variants, keyboard
navigation uses `nav-cursor`, and disabled controls retain native behavior.

`KeyButton` measures label, shortcut, inner gap, and both horizontal padding
edges. When they fit, it left-aligns the label and places the shortcut at the
right. Otherwise the label stays centered and the shortcut moves to hover
help. The translated ID and keyboard handlers remain intact. Enabled key and
toggle states use `button-active` with ordinary text, including hover; toggles
use the dark field fill and secondary text when off.

### Inputs / Fields

Dark inset fields sit below the surrounding editor tone. Text fields retain
`frame-bg` during hover and editing; numeric drags and sliders use hovered and
active frame roles. They share frame radius and padding. Text selection and
cursor use their native colors. Checked values use the blue selected fill and
white check mark. Editing and focus remain native, with no added animation or
new validation behavior.

### Cards / Containers

The recurring container is a bordered native child for action bands,
inspectors, and model panes. It uses `child-bg`, child radius, and native window
padding where requested. Action bands measure their content height; settings,
metrics, and preview pages keep their own scrolling regions. This is the
editor's shared container, not a separate dashboard-card component.

### Navigation

`workspace_header` combines Home, task title, and optional document context in
a narrow `title-bg` strip. It requests `AlwaysUseWindowPadding` with its
explicit header padding. The title wraps when needed; context shares the row
when enough room remains and otherwise moves down. UTF-8-safe middle elision
keeps both path ends visible, with the full value in hover help. The helper
reports Home; callers retain worker and unsaved-document checks.

Workspace tabs keep dark inactive backgrounds and neutral selected surfaces
continuous with the editor. `ButtonRole::Selected` also supports filter
choices as ordinary native buttons. Menus and popups retain their native roles.

### Command rows

`command_row` is a compact native button with a wrapped title and stable
translated message ID. Height is the larger of the native frame height and
the wrapped title plus command padding. Descriptions live in hover help,
including for disabled commands. Drawn title text clips to the padded bounds;
the primary variant uses the primary fills and white foreground. Keyboard
navigation and disabled behavior remain native.

### Viewports and transport

`ViewportPanel` and `CompareView` retain shared pane boundaries and measured
control rows. Render transport reserves the larger Play/Pause label and a
stable time/frame readout width. Timeline height includes the measured number
of control rows, a scaled 66-pixel ruler/track area, and item spacing. Timeline,
camera gizmo, recent-item category, and scene-overlay colors remain local to
those visualizations; they do not become general editor accents.

## Do's and Don'ts

### Do:

- **Do** use ButtonStyle for Primary, Selected, and Danger roles with its scoped white foreground and native behavior.
- **Do** scale handwritten geometry with px(), keep saved panel sizes unscaled, and rebuild style from its base before scaling.
- **Do** measure localized labels, wrap complete controls with FlowRow, and retain full context in hover help when paths are elided.
- **Do** keep execution controls outside scrolling pages and keep job polling and dialogs outside visibility-dependent callbacks.
- **Do** evaluate properties before their scene preview even when the properties are placed on the right.
- **Do** use the embedded regional font chain and preserve translated message IDs when customizing a control.

### Don't:

- **Don't** introduce an unimplemented display face, weight hierarchy, letter spacing, shadow, or decorative animation as a system rule.
- **Don't** substitute the bright primary fill behind ordinary-text selected controls; preserve their darker native selection role.
- **Don't** reserve shortcut space by guesswork or draw a hint over its label; use KeyButton's measured fit and tooltip fallback.
- **Don't** treat saved panel width as rendered pixels or overwrite the preference merely because a smaller window clamps it.
- **Don't** promote Home, Render, timeline, recent-category, or gizmo-specific composition and color into universal rules.

Extraction sources: [Layout.cpp](src/app/gui/Layout.cpp) and
[Layout.h](src/app/gui/Layout.h); [StudioUi.cpp](src/app/gui/StudioUi.cpp);
[Fonts.cpp](src/app/gui/Fonts.cpp) and the [font reference](assets/fonts/README.md);
[TrainingWorkspace.cpp](src/app/gui/TrainingWorkspace.cpp);
[Ui.h](src/app/gui/Ui.h); [GuiApp.cpp](src/app/gui/GuiApp.cpp);
[CompareView.cpp](src/app/gui/CompareView.cpp);
[ViewportPanel.cpp](src/app/gui/ViewportPanel.cpp);
[RenderTimeline.cpp](src/app/gui/render/RenderTimeline.cpp) and
[RenderPanel.cpp](src/app/gui/render/RenderPanel.cpp).
The [current native gallery](docs/notes/blender-ui-style.md#native-screenshots)
and [capture provenance](docs/notes/images/blender-ui/provenance.json) ground
the rendered examples. Validation scope remains in the implementation note.
