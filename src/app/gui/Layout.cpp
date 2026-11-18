// Layout.cpp -- see Layout.h.

#include "app/gui/Layout.h"

#include "app/gui/Ui.h"

#include <algorithm>
#include <cmath>

namespace gui {

namespace {

// The window the sizes in this GUI were chosen against. Below it the auto
// scale shrinks so the viewport keeps its share of a small screen; above it
// it grows, up to the point where growing stops buying legibility.
constexpr float kRefW = 1600.0f;
constexpr float kRefH = 950.0f;
constexpr float kAutoMin = 0.80f;
constexpr float kAutoMax = 1.25f;

ImVec4 color(unsigned rgb, float alpha = 1.0f) {
    return ImVec4((float)((rgb >> 16) & 255) / 255.0f,
                  (float)((rgb >> 8) & 255) / 255.0f,
                  (float)(rgb & 255) / 255.0f, alpha);
}

}  // namespace

void apply_style(float scale) {
    ImGuiStyle& style = ImGui::GetStyle();
    style = ImGuiStyle();
    style.WindowPadding = ImVec2(12, 10);
    style.WindowRounding = 6.0f;
    style.WindowBorderSize = 1.0f;
    style.ChildRounding = 4.0f;
    style.ChildBorderSize = 1.0f;
    style.PopupRounding = 6.0f;
    style.PopupBorderSize = 1.0f;
    style.FramePadding = ImVec2(8, 5);
    style.FrameRounding = 4.0f;
    style.FrameBorderSize = 0.0f;
    style.ItemSpacing = ImVec2(8, 6);
    style.ItemInnerSpacing = ImVec2(6, 4);
    style.CellPadding = ImVec2(8, 5);
    style.IndentSpacing = 20.0f;
    style.ScrollbarSize = 12.0f;
    style.ScrollbarRounding = 4.0f;
    style.GrabMinSize = 12.0f;
    style.GrabRounding = 3.0f;
    style.TabRounding = 4.0f;
    style.TabBorderSize = 0.0f;
    style.TabBarBorderSize = 1.0f;
    style.TabBarOverlineSize = 2.0f;
    style.SeparatorTextBorderSize = 1.0f;
    style.SeparatorTextPadding = ImVec2(8, 6);

    ImVec4* c = style.Colors;
    c[ImGuiCol_Text]                      = color(0xE8EDF2);
    c[ImGuiCol_TextDisabled]              = color(0xB0BFCC);
    c[ImGuiCol_WindowBg]                  = color(0x11161C);
    c[ImGuiCol_ChildBg]                   = color(0x171E26);
    c[ImGuiCol_PopupBg]                   = color(0x1D2731);
    c[ImGuiCol_Border]                    = color(0x334352);
    c[ImGuiCol_BorderShadow]              = color(0x000000, 0.0f);
    c[ImGuiCol_FrameBg]                   = color(0x25313D);
    c[ImGuiCol_FrameBgHovered]            = color(0x304253);
    c[ImGuiCol_FrameBgActive]             = color(0x2D4358);
    c[ImGuiCol_TitleBg]                   = color(0x151C23);
    c[ImGuiCol_TitleBgActive]             = color(0x1D2832);
    c[ImGuiCol_TitleBgCollapsed]          = color(0x151C23);
    c[ImGuiCol_MenuBarBg]                 = color(0x151C23);
    c[ImGuiCol_ScrollbarBg]               = color(0x11161C);
    c[ImGuiCol_ScrollbarGrab]             = color(0x3A4B5C);
    c[ImGuiCol_ScrollbarGrabHovered]      = color(0x50677D);
    c[ImGuiCol_ScrollbarGrabActive]       = color(0x65829D);
    c[ImGuiCol_CheckMark]                 = color(0x87BDEC);
    c[ImGuiCol_CheckboxSelectedBg]        = color(0x264C6B);
    c[ImGuiCol_SliderGrab]                = color(0x35638A);
    c[ImGuiCol_SliderGrabActive]          = color(0x2E6D9D);
    c[ImGuiCol_Button]                    = color(0x293745);
    c[ImGuiCol_ButtonHovered]             = color(0x304253);
    c[ImGuiCol_ButtonActive]              = color(0x34485B);
    c[ImGuiCol_Header]                    = color(0x222D38);
    c[ImGuiCol_HeaderHovered]             = color(0x2D4154);
    c[ImGuiCol_HeaderActive]              = color(0x2C4357);
    c[ImGuiCol_Separator]                 = color(0x2A3947);
    c[ImGuiCol_SeparatorHovered]          = color(0x659FD0);
    c[ImGuiCol_SeparatorActive]           = color(0x8CC4F3);
    c[ImGuiCol_ResizeGrip]                = color(0x50677D, 0.25f);
    c[ImGuiCol_ResizeGripHovered]         = color(0x659FD0, 0.65f);
    c[ImGuiCol_ResizeGripActive]          = color(0x8CC4F3);
    c[ImGuiCol_InputTextCursor]           = color(0xCDE2F5);
    c[ImGuiCol_TabHovered]                = color(0x2A3D50);
    c[ImGuiCol_Tab]                       = color(0x1B242D);
    c[ImGuiCol_TabSelected]               = color(0x263F55);
    c[ImGuiCol_TabSelectedOverline]       = color(0x77B9EC);
    c[ImGuiCol_TabDimmed]                 = color(0x171E26);
    c[ImGuiCol_TabDimmedSelected]         = color(0x22313F);
    c[ImGuiCol_TabDimmedSelectedOverline] = color(0x47657D);
    c[ImGuiCol_PlotLines]                 = color(0x88BCE6);
    c[ImGuiCol_PlotLinesHovered]          = color(0xC1DDF5);
    c[ImGuiCol_PlotHistogram]             = color(0x2E6D9D);
    c[ImGuiCol_PlotHistogramHovered]      = color(0x66A7D8);
    c[ImGuiCol_TableHeaderBg]             = color(0x202C37);
    c[ImGuiCol_TableBorderStrong]         = color(0x334352);
    c[ImGuiCol_TableBorderLight]          = color(0x26333F);
    c[ImGuiCol_TableRowBg]                = color(0x171E26, 0.0f);
    c[ImGuiCol_TableRowBgAlt]             = color(0x304253, 0.18f);
    c[ImGuiCol_TextLink]                  = color(0x8DC7F4);
    c[ImGuiCol_TextSelectedBg]            = color(0x356D9E, 0.65f);
    c[ImGuiCol_TreeLines]                 = color(0x405367);
    c[ImGuiCol_DragDropTarget]            = color(0x89C9F8);
    c[ImGuiCol_DragDropTargetBg]          = color(0x356D9E, 0.24f);
    c[ImGuiCol_UnsavedMarker]             = color(0xE6B765);
    c[ImGuiCol_NavCursor]                 = color(0x8CC4F3);
    c[ImGuiCol_NavWindowingHighlight]     = color(0xDCE9F5, 0.75f);
    c[ImGuiCol_NavWindowingDimBg]         = color(0x0C1219, 0.68f);
    c[ImGuiCol_ModalWindowDimBg]          = color(0x080D13, 0.72f);
    style.ScaleAllSizes(scale);
    style.FontScaleMain = scale;
}

void UiScale::update(ImVec2 display_size) {
    float f = _user;
    if (f <= 0.0f) {
        const float w = display_size.x / _dpi, h = display_size.y / _dpi;
        f = std::min(w / kRefW, h / kRefH);
        f = std::clamp(f, kAutoMin, kAutoMax);
        // Snapped, so that dragging a window edge steps the interface size a
        // few times rather than reflowing it on every pixel.
        f = std::round(f * 20.0f) / 20.0f;
    }
    const float s = _dpi * f;
    if (std::fabs(s - _applied) < 1e-3f) return;
    _applied = s;
    apply_style(s);
}

// ---------------------------------------------------------------------------
// Splitters
// ---------------------------------------------------------------------------

namespace {

float grab_thickness() { return std::max(4.0f, px(6.0f)); }

void draw_grab(ImVec2 p, ImVec2 size, bool hot) {
    const ImU32 col = ImGui::GetColorU32(hot ? ImGuiCol_SeparatorActive
                                            : ImGuiCol_Separator);
    const float t = 1.0f + px(0.5f);
    ImVec2 a, b;
    if (size.x < size.y) {
        const float cx = p.x + size.x * 0.5f;
        a = ImVec2(cx - t, p.y);
        b = ImVec2(cx + t, p.y + size.y);
    } else {
        const float cy = p.y + size.y * 0.5f;
        a = ImVec2(p.x, cy - t);
        b = ImVec2(p.x + size.x, cy + t);
    }
    ImGui::GetWindowDrawList()->AddRectFilled(a, b, col);
}

}  // namespace

float splitter_extent() {
    return grab_thickness() + ImGui::GetStyle().ItemSpacing.y;
}

float splitter_width() {
    return grab_thickness();
}

SidebarWidthLimits viewer_sidebar_limits(float available) {
    const float minimum = std::min(px(320.0f), available * 0.45f);
    const float maximum = std::max(minimum,
        std::min(available * 0.45f, available - px(320.0f) - splitter_extent()));
    return {minimum, maximum};
}

float workspace_log_limit(float available) {
    const ImGuiStyle& style = ImGui::GetStyle();
    const float limit = std::max(0.0f, std::min(available * 0.30f,
        available - px(240.0f) - splitter_extent() - style.ItemSpacing.y));
    const float minimum = ImGui::GetTextLineHeightWithSpacing() +
                          2.0f * style.WindowPadding.y + style.ScrollbarSize;
    return limit < minimum ? 0.0f : limit;
}

void FlowRow::place(float width) {
    const float gap = ImGui::GetStyle().ItemSpacing.x;
    if (_used > 0.0f && _used + gap + width <= _width) {
        ImGui::SameLine();
        _used += gap + width;
    } else {
        _used = width;
    }
}

void FlowRow::group(float width) {
    if (_used > 0.0f && _used + ImGui::GetStyle().ItemSpacing.x + width > _width)
        _used = 0.0f;
}

bool splitter_v(const char* id, float* extent, float lo, float hi, float span) {
    hi = std::max(hi, lo);   // a window too narrow for the minimum: pin it
    const float t = grab_thickness();
    ImGui::SameLine(0.0f, 0.0f);
    const ImVec2 p = ImGui::GetCursorScreenPos();
    ui::InvisibleButtonRaw(id, ImVec2(t, span));
    const bool hot = ImGui::IsItemActive() || ImGui::IsItemHovered();
    if (hot) ImGui::SetMouseCursor(ImGuiMouseCursor_ResizeEW);
    draw_grab(p, ImVec2(t, span), hot);
    ImGui::SameLine(0.0f, 0.0f);
    if (!ImGui::IsItemActive()) return false;
    const float was = *extent;
    *extent = std::clamp(*extent + ImGui::GetIO().MouseDelta.x, lo, hi);
    return *extent != was;
}

bool splitter_h(const char* id, float* extent, float lo, float hi, float span) {
    hi = std::max(hi, lo);
    const float t = grab_thickness();
    const ImVec2 p = ImGui::GetCursorScreenPos();
    ui::InvisibleButtonRaw(id, ImVec2(span, t));
    const bool hot = ImGui::IsItemActive() || ImGui::IsItemHovered();
    if (hot) ImGui::SetMouseCursor(ImGuiMouseCursor_ResizeNS);
    draw_grab(p, ImVec2(span, t), hot);
    if (!ImGui::IsItemActive()) return false;
    const float was = *extent;
    // The handle sits above the pane it sizes, so dragging up makes it taller.
    *extent = std::clamp(*extent - ImGui::GetIO().MouseDelta.y, lo, hi);
    return *extent != was;
}

// ---------------------------------------------------------------------------

std::string elide_middle(const std::string& s, float max_w) {
    if (max_w <= 0.0f || ImGui::CalcTextSize(s.c_str()).x <= max_w) return s;
    const char* kDots = "...";
    // Bytes kept from each end, grown together until the result stops fitting.
    size_t head = 0, tail = 0;
    std::string best = kDots;
    auto step_fwd = [&](size_t i) {
        i++;
        while (i < s.size() && (s[i] & 0xC0) == 0x80) i++;
        return i;
    };
    auto step_back = [&](size_t i) {
        i--;
        while (i > 0 && (s[i] & 0xC0) == 0x80) i--;
        return i;
    };
    while (true) {
        const size_t nh = step_fwd(head);
        const size_t nt = s.size() - step_back(s.size() - tail);
        if (nh + nt >= s.size()) return s;
        std::string cand = s.substr(0, nh) + kDots + s.substr(s.size() - nt);
        if (ImGui::CalcTextSize(cand.c_str()).x > max_w) return best;
        head = nh;
        tail = nt;
        best.swap(cand);
    }
}

}  // namespace gui
