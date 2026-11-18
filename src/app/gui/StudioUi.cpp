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
    unsigned normal = 0x225B86, hovered = 0x2B6A99, active = 0x225075;
    if (role == ButtonRole::Selected) {
        normal = 0x263F55;
        hovered = 0x304E67;
        active = 0x29445A;
    } else if (role == ButtonRole::Danger) {
        normal = 0x773F47;
        hovered = 0x8D4C54;
        active = 0x65363E;
    }
    ImGui::PushStyleColor(ImGuiCol_Button, color(normal));
    ImGui::PushStyleColor(ImGuiCol_ButtonHovered, color(hovered));
    ImGui::PushStyleColor(ImGuiCol_ButtonActive, color(active));
}

ButtonStyle::~ButtonStyle() { ImGui::PopStyleColor(3); }

bool workspace_header(const spirula::i18n::Msg& title,
                      const std::string& context, bool home_enabled,
                      const spirula::i18n::Msg* home_help) {
    namespace msg = spirula::i18n::msg::gui;
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

    ImGui::PushFont(nullptr, style.FontSizeBase * 1.2f);
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
    ImGui::PopFont();

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
    return home;
}

bool command_row(const spirula::i18n::Msg& title,
                 const spirula::i18n::Msg& description, bool primary) {
    const float width = std::max(1.0f, ImGui::GetContentRegionAvail().x);
    const ImVec2 pad(px(12.0f), px(10.0f));
    const float text_width = std::max(1.0f, width - 2.0f * pad.x);
    const char* name = title.get();
    const char* detail = description.get();
    const ImVec2 title_size = ImGui::CalcTextSize(name, nullptr, false, text_width);
    const float detail_height = *detail
        ? ImGui::CalcTextSize(detail, nullptr, false, text_width).y : 0.0f;
    const float gap = detail_height > 0.0f ? px(4.0f) : 0.0f;
    const float height = std::max(ImGui::GetFrameHeight(),
        2.0f * pad.y + title_size.y + gap + detail_height);
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
    if (!ImGui::IsItemVisible()) return clicked;

    const ImVec2 lo = ImGui::GetItemRectMin(), hi = ImGui::GetItemRectMax();
    const ImVec2 text_at(lo.x + pad.x, lo.y + pad.y);
    ImDrawList* dl = ImGui::GetWindowDrawList();
    dl->PushClipRect(text_at, ImVec2(hi.x - pad.x, hi.y - pad.y), true);
    dl->AddText(ImGui::GetFont(), ImGui::GetFontSize(), text_at,
                ImGui::GetColorU32(ImGuiCol_Text), name, nullptr, text_width);
    if (detail_height > 0.0f) {
        const ImU32 muted = primary ? ImGui::GetColorU32(color(0xD7E8F7))
                                   : ImGui::GetColorU32(ImGuiCol_TextDisabled);
        dl->AddText(ImGui::GetFont(), ImGui::GetFontSize(),
                    ImVec2(text_at.x, text_at.y + title_size.y + gap), muted,
                    detail, nullptr, text_width);
    }
    dl->PopClipRect();
    return clicked;
}

}  // namespace gui
