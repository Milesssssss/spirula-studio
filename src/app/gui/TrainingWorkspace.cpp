#include "app/gui/TrainingWorkspace.h"

#include "app/gui/Layout.h"
#include "app/gui/Ui.h"
#include "i18n/catalog/Dataset.h"
#include "i18n/catalog/Gui.h"

#include <algorithm>

namespace gui {
namespace {

namespace msg = spirula::i18n::msg::gui;
namespace dmsg = spirula::i18n::msg::dataset;

void page(const char* id, const std::function<void()>& draw) {
    if (ImGui::BeginChild(id, ImVec2(0, 0))) draw();
    ImGui::EndChild();
}

void inspector(const TrainingWorkspacePanels& panels) {
    if (!ImGui::BeginTabBar("##inspector_tabs")) return;
    if (ui::BeginTabItem(dmsg::section_settings)) {
        page("##settings", panels.settings);
        ImGui::EndTabItem();
    }
    if (ui::BeginTabItem(msg::section_metrics)) {
        page("##metrics", panels.metrics);
        ImGui::EndTabItem();
    }
    ImGui::EndTabBar();
}

}  // namespace

bool draw_training_workspace(float& panel_width, bool& show_settings,
                             const TrainingWorkspacePanels& panels) {
    if (ImGui::BeginChild("##training_actions", ImVec2(0, 0),
                         ImGuiChildFlags_Borders | ImGuiChildFlags_AutoResizeY |
                         ImGuiChildFlags_AlwaysAutoResize))
        panels.controls();
    ImGui::EndChild();

    const ImVec2 available = ImGui::GetContentRegionAvail();
    if (!show_settings) {
        page("##training_preview", panels.preview);
        return false;
    }

    // A readable inspector and an operable preview must both fit. A tab keeps
    // either task full width when a small window or larger UI scale prevents it.
    if (available.x < px(960.0f)) {
        if (ImGui::BeginTabBar("##training_pages")) {
            if (ui::BeginTabItem(msg::workspace_preview)) {
                page("##training_preview", panels.preview);
                ImGui::EndTabItem();
            }
            if (ui::BeginTabItem(dmsg::section_settings)) {
                page("##settings", panels.settings);
                ImGui::EndTabItem();
            }
            if (ui::BeginTabItem(msg::section_metrics)) {
                page("##metrics", panels.metrics);
                ImGui::EndTabItem();
            }
            ImGui::EndTabBar();
        }
        return false;
    }

    const float min_width = px(320.0f);
    const float max_width = std::max(min_width,
        std::min(available.x * 0.45f, available.x - px(480.0f) - splitter_extent()));
    float width = std::clamp(panel_width * ui_scale(), min_width, max_width);
    if (ImGui::BeginChild("##training_inspector", ImVec2(width, 0),
                         ImGuiChildFlags_Borders))
        inspector(panels);
    ImGui::EndChild();
    const bool changed = splitter_v("##panelsplit", &width, min_width,
                                    max_width, available.y);
    if (changed) panel_width = width / ui_scale();
    page("##training_preview", panels.preview);
    return changed;
}

}  // namespace gui
