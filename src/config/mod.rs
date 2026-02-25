mod io;
mod mutate;

use std::fs;

pub use io::{ensure_config_file, open_config_file, subscribe_config_changes};
pub use mutate::{import_colors_from_json, set_config_value, set_theme_in_config};
pub use termy_config_core::{
    AppConfig, CursorStyle, CustomColors, KeybindConfigLine, SHELL_DECIDE_THEME_ID,
    TabCloseVisibility, TabTitleConfig, TabTitleMode, TabTitleSource, TabWidthMode,
    TerminalScrollbarStyle, TerminalScrollbarVisibility, WorkingDirFallback,
};

pub(crate) const DEFAULT_CONFIG: &str = "# Main settings\n\
theme = termy\n\
# TERM value for child shells and terminal apps\n\
term = xterm-256color\n\
# Startup directory for new terminal sessions (~ supported)\n\
# working_dir = ~/Documents\n\
# Warn before quitting when tabs are busy (running command/fullscreen TUI)\n\
# warn_on_quit_with_running_process = true\n\
# Tab title mode. Supported values: smart, shell, explicit, static\n\
# smart = manual rename > explicit title > shell/app title > fallback\n\
tab_title_mode = smart\n\
# Export TERMY_* env vars for optional shell tab-title integration\n\
tab_title_shell_integration = true\n\
# Tab close button visibility: active_hover | hover | always\n\
tab_close_visibility = active_hover\n\
# Tab width behavior: stable | active_grow | active_grow_sticky\n\
tab_width_mode = active_grow_sticky\n\
# Show/hide termy in the macOS titlebar (between traffic lights and tabs)\n\
# show_termy_in_titlebar = true\n\
# Optional: static fallback tab title\n\
# tab_title_fallback = Terminal\n\
# Advanced tab-title options are documented in docs/configuration.md:\n\
# tab_title_priority = manual, explicit, shell, fallback\n\
# tab_title_explicit_prefix = termy:tab:\n\
# tab_title_prompt_format = {cwd}\n\
# tab_title_command_format = {command}\n\
# Startup window size in pixels\n\
window_width = 1280\n\
window_height = 820\n\
# Terminal font family\n\
font_family = JetBrains Mono\n\
# Terminal font size in pixels\n\
font_size = 14\n\
# Cursor style shared by terminal and inline inputs (line|block)\n\
# cursor_style = block\n\
# Enable cursor blink for terminal and inline inputs\n\
# cursor_blink = true\n\
# Terminal background opacity (0.0 = fully transparent, 1.0 = opaque)\n\
# background_opacity = 1.0\n\
# Enable/disable platform blur for transparent backgrounds\n\
# background_blur = false\n\
# Inner terminal padding in pixels\n\
padding_x = 12\n\
padding_y = 8\n\
# Mouse wheel scroll speed multiplier\n\
# mouse_scroll_multiplier = 3\n\
# Terminal scrollbar visibility: always | on_scroll | off\n\
# (while scrolled up in history, scrollbar stays visible in all modes)\n\
# scrollbar_visibility = on_scroll\n\
# Scrollbar style: neutral | muted_theme | theme\n\
# scrollbar_style = neutral\n\
\n\
# Advanced runtime settings (usually leave these as defaults)\n\
# Preferred shell executable path\n\
# shell = /bin/zsh\n\
# Fallback startup directory when working_dir is unset: home or process\n\
# working_dir_fallback = home\n\
# Advertise 24-bit color support to child apps\n\
# colorterm = truecolor\n\
# Scrollback history lines (lower = less memory, max 100000)\n\
# scrollback_history = 2000\n\
# Scrollback for inactive tabs (saves memory with many tabs)\n\
# inactive_tab_scrollback = 500\n\
# Keybindings (Ghostty-style trigger overrides)\n\
# keybind = cmd-p=toggle_command_palette\n\
# keybind = cmd-c=copy\n\
# keybind = cmd-c=unbind\n\
# keybind = clear\n\
# Show/hide shortcut badges in command palette\n\
# command_palette_show_keybinds = true\n";

pub fn load_or_create() -> AppConfig {
    let mut config = AppConfig::default();
    let Some(path) = ensure_config_file() else {
        return config;
    };

    if let Ok(contents) = fs::read_to_string(&path) {
        config = AppConfig::from_contents(&contents);
    }

    config
}
