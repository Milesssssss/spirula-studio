#include "app/gui/StudioUi.h"

#include "app/gui/Layout.h"
#include "app/gui/Ui.h"
#include "i18n/catalog/Gui.h"

#include <algorithm>

namespace gui {
namespace {

ImVec4 color(unsigned rgb) {
    return ImVec4((float)((rgb >> 16) & 255) / 255.0f,
                  (float)((rgb >> 8) & 255) / 255.0f,
                  (float)(rgb & 255) / 255.0f, 1.0f);
}

}  // namespace

ButtonStyle::ButtonStyle(ButtonRole role) {
    unsigned normal = 0x4772B3, hovered = 0x4067A0, active = 0x345581;
    if (role == ButtonRole::Selected) {
        normal = 0x3D639C;
        hovered = 0x4772B3;
        active = 0x345581;
    } else if (role == ButtonRole::Danger) {
        normal = 0x8B444A;
        hovered = 0x9B4C52;
        active = 0x71363B;
    }
    ImGui::PushStyleColor(ImGuiCol_Button, color(normal));
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, color(hovered));
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, color(active));
    ImGui::PushStyleColor(ImGuiCol_Text, color(0xFFFFFF));
}

ButtonStyle::~ButtonStyle() { ImGui::PopStyleColor(4); }

bool workspace_header(const spirula::i18n::Msg& title,
                      const std::string& context, bool home_enabled,
                      const spirula::i18n::Msg* home_help) {
    namespace msg = spirula::i18n::msg::gui;
    ImGui::PushStyleColor(ImGuiCol_ChildBg, ImGui::GetStyleColorVec4(ImGuiCol_TitleBg));
    ImGui::PushStyleVar(ImGuiStyleVar_WindowPadding, ImVec2(px(6.0f), px(4.0f)));
    ImGui::PushStyleVar(ImGuiStyleVar_ChildRounding, 0.0f);
    const bool visible = ImGui::BeginChild("##workspace_header", ImVec2(0, 0),
        ImGuiChildFlags_AutoResizeY | ImGuiChildFlags_AlwaysAutoResize |
            ImGuiChildFlags_AlwaysUseWindowPadding,
        ImGuiWindowFlags_NoScrollbar | ImGuiWindowFlags_NoScrollWithMouse);
    if (!visible) {
        ImGui::EndChild();
        ImGui::PopStyleVar(2);
        ImGui::PopStyleColor();
        return false;
    }
    const ImGuiStyle& style = ImGui::GetStyle();
    const float left = ImGui::GetCursorPosX();
    const float width = std::max(1.0f, ImGui::GetContentRegionAvail().x);
    const float right = left + width;
    const float gap = style.ItemSpacing.x;
    const float home_width = std::min(width,
        ImGui::CalcTextSize(msg::back_home.get()).x + 2.0f * style.FramePadding.x);

    ImGui::BeginGroup();
    ImGui::BeginDisabled(!home_enabled);
    const bool home = ui::Button(msg::back_home, ImVec2(home_width, 0));
    if (home_help) ui::help_on_hover_disabled(*home_help);
    ImGui::EndDisabled();

    const float title_width = ImGui::CalcTextSize(title.get()).x;
    const bool same_line = home_width + gap + title_width <= width;
    const float title_left = same_line ? left + home_width + gap : left;
    if (same_line) {
        ImGui::SameLine();
        ImGui::SetCursorPosX(title_left);
    }
    ImGui::AlignTextToFramePadding();
    ImGui::PushTextWrapPos(right);
    ui::Text(title);
    ImGui::PopTextWrapPos();

    if (!context.empty()) {
        const float next = title_left + title_width + gap;
        const bool beside = next + px(160.0f) <= right;
        const float context_left = beside ? next : title_left;
        if (beside) ImGui::SameLine();
        ImGui::SetCursorPosX(context_left);
        if (beside) ImGui::AlignTextToFramePadding();
        std::string line = context;
        std::replace(line.begin(), line.end(), '\n', ' ');
        std::replace(line.begin(), line.end(), '\r', ' ');
        ui::TextDisabledRaw(elide_middle(line, std::max(1.0f, right - context_left)));
        ui::help_on_hover_raw(context.c_str());
    }
    ImGui::EndGroup();
    ImGui::EndChild();
    ImGui::PopStyleVar(2);
    ImGui::PopStyleColor();
    return home;
}

bool command_row(const spirula::i18n::Msg& title,
                 const spirula::i18n::Msg& description, bool primary) {
    const float width = std::max(1.0f, ImGui::GetContentRegionAvail().x);
    const ImVec2 pad(px(8.0f), px(4.0f));
    const float text_width = std::max(1.0f, width - 2.0f * pad.x);
    const char* name = title.get();
    const ImVec2 title_size = ImGui::CalcTextSize(name, nullptr, false, text_width);
    const float height = std::max(ImGui::GetFrameHeight(),
        2.0f * pad.y + title_size.y);
    const auto draw = [&] {
        return ui::ButtonRaw(ui::detail::label("", title.id), ImVec2(width, height));
    };
    bool clicked = false;
    if (primary) {
        ButtonStyle button(ButtonRole::Primary);
        clicked = draw();
    } else {
        clicked = draw();
    }
    ui::help_on_hover_disabled(description);
    if (!ImGui::IsItemVisible()) return clicked;

    const ImVec2 lo = ImGui::GetItemRectMin(), hi = ImGui::GetItemRectMax();
    const ImVec2 text_at(lo.x + pad.x, lo.y + pad.y);
    ImDrawList* dl = ImGui::GetWindowDrawList();
    dl->PushClipRect(text_at, ImVec2(hi.x - pad.x, hi.y - pad.y), true);
    dl->AddText(ImGui::GetFont(), ImGui::GetFontSize(), text_at,
                ImGui::GetColorU32(primary ? color(0xFFFFFF)
                                          : ImGui::GetStyleColorVec4(ImGuiCol_Text)),
                name, nullptr, text_width);
    dl->PopClipRect();
    return clicked;
}

}  // namespace gui
