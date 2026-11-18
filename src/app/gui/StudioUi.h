#pragma once

#include <string>

namespace spirula::i18n { struct Msg; }

namespace gui {

enum class ButtonRole { Primary, Selected, Danger };

class ButtonStyle {
public:
    explicit ButtonStyle(ButtonRole role);
    ~ButtonStyle();
    ButtonStyle(const ButtonStyle&) = delete;
    ButtonStyle& operator=(const ButtonStyle&) = delete;
};

// Reports Home without navigating; the caller owns worker and document guards.
bool workspace_header(const spirula::i18n::Msg& title,
                      const std::string& context = {}, bool home_enabled = true,
                      const spirula::i18n::Msg* home_help = nullptr);

bool command_row(const spirula::i18n::Msg& title,
                 const spirula::i18n::Msg& description, bool primary = false);

}  // namespace gui
