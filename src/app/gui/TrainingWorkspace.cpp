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

void page(const char* id, const std::function<void()>& draw, float width = 0.0f) {
    if (ImGui::BeginChild(id, ImVec2(width, 0))) draw();
    ImGui::EndChild();
}

void inspector(const WorkspacePanels& panels) {
    if (!panels.metrics) {
        panels.settings();
        return;
    }
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

void draw_workspace_actions(const char* id, const std::function<void()>& draw) {
    if (ImGui::BeginChild(id, ImVec2(0, 0),
                         ImGuiChildFlags_Borders | ImGuiChildFlags_AutoResizeY |
                         ImGuiChildFlags_AlwaysAutoResize))
        draw();
    ImGui::EndChild();
}

bool draw_workspace_panels(float& panel_width, bool show_settings,
                           const WorkspacePanels& panels, const WorkspaceLayout& layout) {
    const ImVec2 available = ImGui::GetContentRegionAvail();
    if (!show_settings) {
        page(layout.preview_id, panels.preview);
        return false;
    }

    // Tabs keep the inspector and preview usable when both cannot fit.
    if (available.x < px(960.0f)) {
        if (ImGui::BeginTabBar(layout.pages_id)) {
            auto settings = [&] {
                if (ui::BeginTabItem(dmsg::section_settings)) {
                    page("##settings", panels.settings);
                    ImGui::EndTabItem();
                }
            };
            auto preview = [&] {
                if (ui::BeginTabItem(msg::workspace_preview)) {
                    page(layout.preview_id, panels.preview);
                    ImGui::EndTabItem();
                }
            };
            if (layout.settings_first) { settings(); preview(); }
            else { preview(); settings(); }
            if (panels.metrics && ui::BeginTabItem(msg::section_metrics)) {
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
    const float width = std::clamp(panel_width * ui_scale(), min_width, max_width);
    float preview_width = std::max(1.0f, available.x - width - splitter_width());
    const ImVec2 origin = ImGui::GetCursorPos();
    // Keep settings ahead of preview evaluation when the inspector is on the right.
    ImGui::SetCursorPos(ImVec2(origin.x + preview_width + splitter_width(), origin.y));
    if (ImGui::BeginChild(layout.inspector_id, ImVec2(width, 0),
                         ImGuiChildFlags_Borders))
        inspector(panels);
    ImGui::EndChild();
    ImGui::SetCursorPos(origin);
    page(layout.preview_id, panels.preview, preview_width);
    const bool changed = splitter_v(layout.splitter_id, &preview_width,
        available.x - splitter_width() - max_width,
        available.x - splitter_width() - min_width, available.y);
    if (changed)
        panel_width = (available.x - splitter_width() - preview_width) / ui_scale();
    return changed;
}

bool draw_training_workspace(float& panel_width, bool& show_settings,
                             const TrainingWorkspacePanels& panels) {
    draw_workspace_actions("##training_actions", panels.controls);
    const WorkspacePanels content{panels.settings, panels.metrics, panels.preview};
    const WorkspaceLayout layout{"##training_inspector", "##training_pages",
                                 "##training_preview", "##panelsplit"};
    return draw_workspace_panels(panel_width, show_settings, content, layout);
}

}  // namespace gui
