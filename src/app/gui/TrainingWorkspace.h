#pragma once

#include <functional>

namespace gui {

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
