---
version: 1
slug: "src-app-gui-guiapp-cpp"
primary_target: "src/app/gui/GuiApp.cpp"
related_targets: ["src/app/gui/Layout.cpp","src/app/gui/TrainingWorkspace.cpp","src/app/gui/CompareView.cpp"]
---

# Studio workspaces

Mode: Operate. Scope: the native application shell, Home, dataset preparation,
training, model workspaces, meshing, and batch processing.

The user expanded the existing design PR to an overall UI redesign. Preserve
existing tasks, localized copy, engine ownership, and document confirmations.
This implementation proceeds directly in native code, with real screenshots
and interaction evidence; it sets no permanent workflow preference.

## Direction contract

THESIS: A Blender-referenced reconstruction editor. The scene owns the main
region; compact headers identify the current document and controls configure
it without competing with the content.

OWN-WORLD: Achromatic #303030 editor surfaces, #3D3D3D panel headers,
#545454 ordinary controls, dark input wells, blue value/action states and
orange selection/focus cues. Small corners, narrow boundaries, compact rows;
Source Sans with the embedded CJK chain stays readable at every UI scale.

STORY: Resume a recent capture, adjust properties, inspect the reconstruction,
then edit or render it. Execution state remains visible outside scrolling
properties; navigation retains existing worker and unsaved-document guards.

FIRST VIEWPORT: A compact product strip over recent files and a command list.
Each workspace starts with a narrow context header. Wide dataset/training
and model editors put the scene on the left and properties on the right;
render transport and timeline stay beneath the scene. Compact preparation
and training use existing tabs, retaining full-size controls and run access.

FORM: User-pinned Blender interface, superseding candidate 7 from the prior
seed 96a6b1d5. No new direction choice or generated comp is required by the
user's explicit reference and authorization to update the existing PR.
Authority: Blender's official window-system manual and v4.5.0 default theme.
The native implementation and captured screenshots are the review surface.

Signature interaction: property edits immediately affect the adjacent scene;
the right-hand divider resizes the inspector while the scene and timeline
retain their usable area. Motion is immediate native input feedback.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
