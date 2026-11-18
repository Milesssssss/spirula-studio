#pragma once

#include <functional>

namespace gui {

struct WorkspacePanels {
    std::function<void()> settings;
    std::function<void()> metrics;
    std::function<void()> preview;
};

struct WorkspaceLayout {
    const char* inspector_id;
    const char* pages_id;
    const char* preview_id;
    const char* splitter_id;
    bool settings_first = false;
};

void draw_workspace_actions(const char* id, const std::function<void()>& draw);
// Only visible panel callbacks run; keep polling and dialogs outside them.
bool draw_workspace_panels(float& panel_width, bool show_settings,
                           const WorkspacePanels& panels, const WorkspaceLayout& layout);

struct TrainingWorkspacePanels {
    std::function<void()> controls;
    std::function<void()> settings;
    std::function<void()> metrics;
    std::function<void()> preview;
};

// Controls stay outside every scrolling page. Returns true when the user
// changes the unscaled inspector width.
bool draw_training_workspace(float& panel_width, bool& show_settings,
                             const TrainingWorkspacePanels& panels);

}  // namespace gui
