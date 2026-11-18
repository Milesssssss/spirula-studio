# Studio workspace redesign

This iteration extends the training workspace introduced in `0dbec1c` across
the native Dear ImGui application. Home becomes a useful record of recent work;
each workspace makes its context, actions, content, and diagnostics legible.
The existing files, recent-item kinds, workers, and compute backends remain the
product model.

## Problems and decisions

The large Home banner consumed space before users reached their work. Commands
and recent items competed for attention, and each tool arranged navigation,
long paths, controls, and logs differently. Preparation and batch actions could
follow long scrolling forms. Fixed horizontal toolbars also ran out of room
with larger UI scale or longer translations.

The reading order is now **context, actions, workspace, diagnostics**. Run
controls belong outside the scrolling content they act on. The main content
area holds a scene, editable settings, or a queue according to the current
task. Logs use a shared height budget. Responsive composition stays explicit;
there is no dockspace or additional panel-persistence system to recover.

## Visual system and global context

`apply_style` in `Layout.cpp` defines all 61 color slots in the pinned ImGui version.
The palette uses graphite surfaces, neutral ordinary buttons, and restrained
blue for focus, selection, and primary actions. Status colors retain their
meaning. Thin borders distinguish panels without large accent-colored fills.

| Role | Token |
| --- | --- |
| Window / panel / popup surfaces | `#11161C` / `#171E26` / `#1D2731` |
| Input / ordinary button surfaces | `#25313D` / `#293745` |
| Main / secondary text | `#E8EDF2` / `#B0BFCC` |
| Primary action / selected action | `#225B86` / `#263F55` |
| Focus / progress fill | `#8CC4F3` / `#2E6D9D` |

The existing embedded font chain, thirteen languages, UI-scale preference, and
translated widget IDs remain. The File, View, language, device, and Help menus
continue to provide global commands and layout recovery.

`workspace_header` in `StudioUi.{h,cpp}` places Home, the current task title, and its path
in a compact group. A path is reduced to one line with middle elision and a full
tooltip; it moves below the title when necessary. The helper reports a Home
click without performing navigation. Its optional help remains available on a
disabled Home button, including the training-leave explanation.

## Home: recent work and commands

At an available width of at least `1040 * scale`, the recent-work list takes
the main column and a `340 * scale` command column sits beside it. Commands use
measured, wrapping title-and-description rows. In compact windows, ordinary
buttons wrap above the recent-work list and expose their descriptions on hover.
The oversized artwork banner is replaced by a compact product heading.

One filter selects all recent work, datasets, reconstructions, models, runs,
or render projects. Existing record ordering, type-specific open behavior,
asynchronous availability checks, and context-menu actions remain. A Run still
opens its output model; Resume Training remains a separate command offered
when full-checkpoint retention is enabled. When a runner is present, Home offers
the existing return-to-trainer action and its dataset context. Empty lists
explain the selected category and file dropping.

## Workspace changes

| Workspace | Structure and behavior |
| --- | --- |
| Data preparation | A fixed action area holds Create/Update, review Continue, Cancel, Log, stage feedback, and applicable memory status. Settings and Preview share the responsive workspace; compact mode starts with Settings. Completion links stay with the job status. |
| Training | The existing fixed run area and independent Settings/Metrics pages remain. The common header and button roles join the wider Studio system. Preview mode, screenshots, and mask correction stay with the preview. Ready retains its start hint without an empty progress bar. |
| Mesh | A fixed action area holds Start, Cancel, or the completed-result handoff and Log. At `960 * scale`, setup becomes two columns for source/preset options and output/color choices; narrower windows stack them. Results retain linked splat/mesh comparison and individual output visibility controls. |
| Batch | Input-add controls, run/list-management actions, queue progress, and stop controls sit above a scrolling queue. Validation issues, command details, and the execution plan remain with that queue. Starting is deferred until its row editors have been drawn. |
| Viewer / Edit / Render | The header names the current viewing or editing task and uses the active model path. Open, Render, screenshot, Log, Add Model, linked-view, and wipe controls share measured wrapping rows. The existing adjustable Edit/Render sidebar and comparison panes remain. Render transport wraps, and its measured row count contributes to the timeline height. |

Viewer keeps its own sidebar geometry; it does not use the preparation and
training compact tabs. The timeline reserves transport widths for Play/Pause
and the time/frame readout so normal playback does not continually repack rows.

When the active Render model pane is shorter than `360 * scale`, display
controls and ordinary idle/selection help move into its existing `...`
placement popup. Live flight, transformation, and picking instructions remain
inline as a single-line summary with full hover text. The model heading reserves
the measured widths of its Edit/Render actions and object counts before eliding
the title. Ordinary Viewer, Edit, and training viewports keep inline controls
by default; this compact treatment applies only to the active Render pane.

## Shared geometry and components

`TrainingWorkspace.{h,cpp}` now exposes `draw_workspace_actions` and
`draw_workspace_panels`. The former sizes a persistent action band to its
content. The latter draws an inspector beside Preview at `960 * scale` and
above, or full-width tabs below that threshold. Metrics is optional, and each
visible page owns its scrolling area. Preparation chooses Settings first;
training chooses Preview first.

The wide inspector starts at a scaled minimum of 320 pixels. Its maximum is
the smaller of 45% of the available width and the width left after reserving
480 scaled pixels for Preview and the splitter. Saved widths remain unscaled;
resizing clamps their rendered size without replacing the saved preference.

`workspace_log_limit` in `Layout.cpp` caps logs at 30% of the remaining height while
reserving 240 scaled pixels for the main body, splitter extent, and spacing.
Preparation, training, Mesh, Batch, and Viewer apply that cap to both display
and dragging. If the result cannot hold one log line with spacing, top and
bottom window padding, and a horizontal scrollbar, the limit becomes zero.
The saved visibility choice remains unchanged. Preparation also budgets for
its compact tab bar.

Render computes its timeline height from the actual right-hand pane width
before allocating the log, so the transport and ruler do not consume the space
reserved for the image. Both this estimate and the sidebar layout use
`viewer_sidebar_limits`. The width calculation uses `splitter_width`, which
contains only the grab width; `splitter_extent` keeps its existing spacing for
vertical height budgets. `RenderSession::timeline_height(available_width)`
supports this advance measurement and still defaults to the current content
width when drawing the timeline.

`ViewportPanel::show_controls` and `draw_options` let the active Render pane
move its existing controls into the popup. Hidden inline controls contribute
zero measured control height and no alignment padding; rendering and navigation
continue through the same `draw` path. `RenderSession::draw_status(compact)`
keeps live interaction instructions visible while moving general help out of
that short pane. Its default remains the normal inline status presentation.

`FlowRow` packs controls using explicit measured widths, including localized
labels. Hovering a tooltip cannot change the next control's placement.
`ButtonStyle` applies Primary, Selected, or Danger roles with balanced
style pushes. `command_row` keeps standard button navigation and disabled
behavior while drawing wrapped catalog text under a stable message ID.
`KeyButton` measures the action label and shortcut together. When both fit,
the label uses the left side and the hint the right; otherwise the shortcut
appears in the tooltip. Button IDs, click results, and key bindings stay intact.

## State and navigation boundaries

Only visible workspace callbacks run. Preparation therefore restores workspace
settings, polls progress, follows frame-review transitions, and records the
current preview stage outside those callbacks. Hiding Preview pauses source
playback; it does not stop job-state advancement. Model, mask, partition,
rebuild, and related dialogs also remain outside the tab-dependent form.

Preparation records a requested Start/Continue/Cancel action, then applies it
after the form and job update so edits committed by that click are included.
Mesh start and batch start similarly follow their editable content. Training
continues to resolve configuration macros in `start_training`, independently
of whether Settings was drawn.

Header helpers do not own workers or documents. Callers retain the existing
busy checks, `request_go_home` stop/engine-release path, and Viewer discard
confirmation. Dataset Home remains disabled while native work owns the screen.
The Mesh result preview continues to disallow the Viewer's render workspace.
This change does not introduce a new navigation or job-execution state machine.

## Validation

The complete Vulkan GUI was compiled and linked through `build_develop.bash`,
with GUI enabled and SfM, SAM, and patented features disabled. The build used
the pinned Slang compiler and a verified cache from identical shader sources.
The interface changes add no runtime dependency.

The actual GLFW application ran on Linux with Mesa software OpenGL and
Lavapipe Vulkan. All inputs were generated: three cameras, three test images,
and 192 colored points. This allowed the real Viewer, Edit, and Render screens
to open through the existing device-selection path.

| Native capture | Interface checks |
| --- | ---: |
| 1600 x 950, English, 100% UI scale | 45 / 45 |
| 800 x 600, English, 100% UI scale | 50 / 50 |
| 1600 x 950, Simplified Chinese, 200% UI scale | 50 / 50 |

These 145 checks cover navigation and layout rather than compute workloads.
Home filtering, opening a recent dataset, and returning to the current Ready
session were exercised with actual GUI input. In the compact window, Dataset,
Train, Mesh, and Batch content scrolled by 690, 383, 229, and 203 pixels;
their corresponding action rectangles stayed unchanged. Switching training
pages and hiding the inspector also preserved run access.

Render checks cover transport wrapping, non-overlapping controls, complete
track bounds, and opening/closing the short-pane display-options popup.
An additional fixture-specific framebuffer check confirms that the scene is
actually visible: the final Render canvas has 367, 235, and 220 visible pixels
of height in the three cases, with camera geometry present in each. This uses
the known synthetic fixture's clear color and frustum pixels; it does not
claim general visual recognition or rendering-quality coverage.

The build and static checks passed translation completeness (4,794 messages),
font coverage (10,996 characters across five fonts), UI-copy length, comment
length, Windows macro, file-macro, SS-prefix, and whitespace checks. Independent
source review covered navigation guards, ImGui stacks and IDs, deferred form
actions, hidden-preview state progression, and timeline/pane geometry.

An independent agent applied the Impeccable finish-review protocol to 37 native
captures and identified one label/shortcut collision in Edit. The follow-up
verdict reviewed the recaptured set plus two scrolled Selection states, scored
that material fix **resolved**, and returned **ship** for the listed fix. The
shared button's other visible labels were checked for new collisions. The
reference catalog was unavailable, so no external quality-board comparison
was made; this was an independent generic-agent review using the protocol.

[Machine-readable interface results](studio-ui-validation.json) and
[screenshot origins](images/studio-redesign/provenance.json) record the evidence
and its UI-source fingerprint. Every screenshot also embeds its origin.

No physical-GPU training, in-training pause/resume, meshing job, batch execution,
or render export was run. SfM/SAM providers were absent in this validation
build. Those workloads and active-worker transitions still need hardware
validation; source review is not a substitute for that coverage.

## Visual evidence

The following are native application captures. The before images are from the
published training-layout iteration `0dbec1c`; the after images include the
overall Studio redesign. They use the same feature flags and generic fixture.
The earlier master-to-training comparison remains in
[the historical training note](training-workspace-layout.md).

### Home

The same empty state, at 1600 x 950 and 100% scale:

| Before: large banner and equal-weight launch commands | After: compact identity, recent-work ledger and command column |
| --- | --- |
| ![Home before](images/studio-redesign/before-wide-en-home.png) | ![Home after](images/studio-redesign/wide-en-home.png) |

The populated state follows a real dataset open and return to Home. It shows
the current-session action and a filtered recent record:

![Home with recent work](images/studio-redesign/wide-en-home-recent.png)

<details>
<summary>Compact and Chinese Home</summary>

![Home at 800 x 600](images/studio-redesign/compact-en-home-recent.png)

![Home in Chinese at 200%](images/studio-redesign/large-zh-home-recent.png)

</details>

### Preparation and training

Preparation keeps execution and completed-result handoffs above the inspector
and input preview:

![Data preparation](images/studio-redesign/wide-en-dataset-photos.png)

Training uses the same context and action hierarchy with its Settings/Metrics
inspector and scene:

![Training](images/studio-redesign/wide-en-train-ready.png)

<details>
<summary>Compact tabs and large-text previews</summary>

![Compact training preview](images/studio-redesign/compact-en-train-ready.png)

![Compact training settings](images/studio-redesign/compact-en-train-settings.png)

![Chinese training at 200%](images/studio-redesign/large-zh-train-ready.png)

![Chinese preparation Preview at 200%](images/studio-redesign/large-zh-dataset-preview.png)

</details>

### Mesh and batch

The mesh form separates source and output decisions on wide windows, with the
execution area above both:

| Before | After |
| --- | --- |
| ![Mesh before](images/studio-redesign/before-wide-en-mesh.png) | ![Mesh after](images/studio-redesign/wide-en-mesh.png) |

The batch queue scrolls independently from its input, execution, and list
controls:

![Batch workspace](images/studio-redesign/wide-en-batch.png)

### Viewer, Edit and Render

![Viewer workspace](images/studio-redesign/wide-en-viewer.png)

![Edit workspace](images/studio-redesign/wide-en-edit.png)

![Render workspace](images/studio-redesign/wide-en-render.png)

At 800 x 600, the scene and timeline remain visible together. Display settings
are available from the pane's existing options button:

![Compact Render](images/studio-redesign/compact-en-render.png)

<details>
<summary>Display-options popup and Chinese Render at 200%</summary>

![Compact Render display settings](images/studio-redesign/compact-en-render-display-settings.png)

![Chinese Render at 200%](images/studio-redesign/large-zh-render.png)

</details>

## Manual review focus

1. Check Home with empty and populated recent lists, every filter, and long paths.
2. Resize across the Home and workspace breakpoints; repeat with large UI scale
   and longer translations. Check wrapping, scrolling, and log recovery.
3. Switch preparation Settings/Preview during progress and frame review; ensure
   actions and dialogs remain reachable and returning to Preview shows current state.
4. Exercise run controls and queue-specific stopping, then verify navigation
   guards with active workers and unsaved edit/render documents.
5. Inspect Mesh results, multi-model comparison, Edit, and Render; resize the
   timeline and check transport, screenshots, and bounded logs together.
