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

THESIS: A reconstruction workbench with a readable job record. The user can
resume work, identify the current object, configure it, and inspect results
without navigating a stack of equally weighted launch buttons.

OWN-WORLD: Graphite surfaces, neutral controls, restrained blue primary and
selection states, fine boundaries, Source Sans with the embedded CJK chain.
Headings and paths have distinct roles; dense fields stay aligned.

STORY: Home puts recent work and the current session first. Each workspace
keeps context and execution visible around a calm scene or result area.

FIRST VIEWPORT: A compact product heading above the recent-work ledger; a
narrow command column on wide windows becomes wrapped actions above the list
on small windows. Workspaces share Home, title, and an elided object path,
followed by run controls and settings/preview regions.

FORM: Candidate 7, reconstruction laboratory workbench; seed 96a6b1d5.
The reference catalog was unavailable, so no catalog challengers or boards
were used. The seven considered systems were DCC workstation, optical
metrology bench, photographic contact sheet, CAD drafting room, source-editor
workspace, mission console, and reconstruction laboratory workbench.

Signature interaction: the current record opens into a stable contextual
workspace; resized or scaled panels change structure while run access remains.
Motion: immediate native input feedback; no decorative entrance animations.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
