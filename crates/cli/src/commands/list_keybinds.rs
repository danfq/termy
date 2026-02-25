use termy_config_core::{AppConfig, KeybindConfigLine, config_path};

struct DefaultKeybind {
    trigger: &'static str,
    action: &'static str,
    #[allow(dead_code)]
    platform: Platform,
}

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
enum Platform {
    All,
    MacOs,
    Linux,
}

const DEFAULT_KEYBINDS: &[DefaultKeybind] = &[
    DefaultKeybind {
        trigger: "secondary-q",
        action: "quit",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-,",
        action: "open_config",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-p",
        action: "toggle_command_palette",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-t",
        action: "new_tab",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-w",
        action: "close_tab",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-m",
        action: "minimize_window",
        platform: Platform::MacOs,
    },
    DefaultKeybind {
        trigger: "secondary-=",
        action: "zoom_in",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-+",
        action: "zoom_in",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary--",
        action: "zoom_out",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-0",
        action: "zoom_reset",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-f",
        action: "open_search",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-g",
        action: "search_next",
        platform: Platform::All,
    },
    DefaultKeybind {
        trigger: "secondary-shift-g",
        action: "search_previous",
        platform: Platform::All,
    },
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    DefaultKeybind {
        trigger: "secondary-c",
        action: "copy",
        platform: Platform::All,
    },
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    DefaultKeybind {
        trigger: "secondary-v",
        action: "paste",
        platform: Platform::All,
    },
    #[cfg(target_os = "linux")]
    DefaultKeybind {
        trigger: "ctrl-shift-c",
        action: "copy",
        platform: Platform::Linux,
    },
    #[cfg(target_os = "linux")]
    DefaultKeybind {
        trigger: "ctrl-shift-v",
        action: "paste",
        platform: Platform::Linux,
    },
];

pub fn run() {
    let mut keybinds: Vec<(String, String)> = Vec::new();

    // Start with defaults
    for kb in DEFAULT_KEYBINDS {
        #[cfg(target_os = "macos")]
        let is_current_platform = kb.platform == Platform::All || kb.platform == Platform::MacOs;
        #[cfg(target_os = "linux")]
        let is_current_platform = kb.platform == Platform::All || kb.platform == Platform::Linux;
        #[cfg(target_os = "windows")]
        let is_current_platform = kb.platform == Platform::All;

        if is_current_platform {
            keybinds.push((kb.trigger.to_string(), kb.action.to_string()));
        }
    }

    // Apply user config overrides
    if let Some(path) = config_path() {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let config = AppConfig::from_contents(&contents);
            let directives = parse_keybind_lines(&config.keybind_lines);
            for directive in directives {
                match directive {
                    KeybindDirective::Clear => keybinds.clear(),
                    KeybindDirective::Bind { trigger, action } => {
                        // Remove existing binding for this trigger
                        keybinds.retain(|(t, _)| t != &trigger);
                        keybinds.push((trigger, action));
                    }
                    KeybindDirective::Unbind { trigger } => {
                        keybinds.retain(|(t, _)| t != &trigger);
                    }
                }
            }
        }
    }

    // Print all keybindings
    for (trigger, action) in &keybinds {
        println!("{} = {}", trigger, action);
    }
}

pub enum KeybindDirective {
    Clear,
    Bind { trigger: String, action: String },
    Unbind { trigger: String },
}

pub fn parse_keybind_lines(lines: &[KeybindConfigLine]) -> Vec<KeybindDirective> {
    let mut directives = Vec::new();

    for line in lines {
        let value = line.value.trim();
        if value.eq_ignore_ascii_case("clear") {
            directives.push(KeybindDirective::Clear);
            continue;
        }

        if let Some((trigger, action)) = value.split_once('=') {
            let trigger = trigger.trim().to_string();
            let action = action.trim().to_string();

            if trigger.is_empty() || action.is_empty() {
                continue;
            }

            if action.eq_ignore_ascii_case("unbind") {
                directives.push(KeybindDirective::Unbind { trigger });
            } else {
                directives.push(KeybindDirective::Bind { trigger, action });
            }
        }
    }

    directives
}
