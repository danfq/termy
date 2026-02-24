use super::*;

impl TerminalView {
    pub(super) fn truncate_tab_title(title: &str) -> String {
        // Keep titles single-line so shell-provided newlines do not break tab layout.
        let normalized = title.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.chars().count() > MAX_TAB_TITLE_CHARS {
            return normalized.chars().take(MAX_TAB_TITLE_CHARS).collect();
        }
        normalized
    }

    fn is_path_like_tab_title(title: &str) -> bool {
        title.contains('/') || title.contains('\\')
    }

    fn squeezed_path_tab_label_for_preserved_chars(
        chars: &[char],
        basename_len: usize,
        preserved_chars: usize,
    ) -> String {
        if chars.is_empty() {
            return String::new();
        }

        if preserved_chars == 0 {
            return "...".to_string();
        }

        let (head_chars, tail_chars) = if preserved_chars == 1 {
            (0, 1)
        } else {
            let max_tail_chars = preserved_chars - 1;
            let min_tail_chars = preserved_chars / 2;
            let preferred_tail_chars = (basename_len + 1).min(max_tail_chars);
            let tail_chars = preferred_tail_chars.max(min_tail_chars).min(max_tail_chars);
            (preserved_chars.saturating_sub(tail_chars), tail_chars)
        };

        let mut formatted = String::with_capacity(head_chars + 3 + tail_chars);
        for ch in chars.iter().take(head_chars) {
            formatted.push(*ch);
        }
        formatted.push_str("...");
        for ch in chars
            .iter()
            .skip(chars.len().saturating_sub(tail_chars))
            .take(tail_chars)
        {
            formatted.push(*ch);
        }

        formatted
    }

    fn fitting_dots_for_width<F>(available_text_px: f32, measure_text_px: &mut F) -> String
    where
        F: FnMut(&str) -> f32,
    {
        if available_text_px <= f32::EPSILON {
            return String::new();
        }

        for dots in ["...", "..", "."] {
            if measure_text_px(dots) <= available_text_px {
                return dots.to_string();
            }
        }

        String::new()
    }

    pub(super) fn format_tab_label_for_render_measured<F>(
        title: &str,
        available_text_px: f32,
        mut measure_text_px: F,
    ) -> String
    where
        F: FnMut(&str) -> f32,
    {
        let available_text_px = if available_text_px.is_finite() {
            available_text_px.max(0.0)
        } else {
            0.0
        };
        if title.is_empty() || available_text_px <= f32::EPSILON {
            return String::new();
        }

        if measure_text_px(title) <= available_text_px {
            return title.to_string();
        }

        if !Self::is_path_like_tab_title(title) {
            // Non-path titles keep end-truncation behavior through render-level text ellipsis.
            return title.to_string();
        }

        let chars: Vec<char> = title.chars().collect();
        if chars.is_empty() {
            return String::new();
        }
        let basename_len = chars
            .iter()
            .rposition(|ch| *ch == '/' || *ch == '\\')
            .map_or(chars.len(), |index| chars.len().saturating_sub(index + 1));
        let candidate_for = |preserved_chars: usize| {
            Self::squeezed_path_tab_label_for_preserved_chars(&chars, basename_len, preserved_chars)
        };

        let mut low = 0usize;
        let mut high = chars.len();
        while low < high {
            let mid = (low + high + 1) / 2;
            let candidate = candidate_for(mid);
            if measure_text_px(candidate.as_str()) <= available_text_px {
                low = mid;
            } else {
                high = mid.saturating_sub(1);
            }
        }

        let fitted = candidate_for(low);
        if measure_text_px(fitted.as_str()) <= available_text_px {
            fitted
        } else {
            Self::fitting_dots_for_width(available_text_px, &mut measure_text_px)
        }
    }

    pub(super) fn fallback_title(&self) -> &str {
        let fallback = self.tab_title.fallback.trim();
        if fallback.is_empty() {
            DEFAULT_TAB_TITLE
        } else {
            fallback
        }
    }

    pub(super) fn resolve_template(
        template: &str,
        cwd: Option<&str>,
        command: Option<&str>,
    ) -> String {
        template
            .replace("{cwd}", cwd.unwrap_or(""))
            .replace("{command}", command.unwrap_or(""))
    }

    pub(super) fn should_seed_predicted_prompt_title(tab_title: &TabTitleConfig) -> bool {
        tab_title
            .priority
            .iter()
            .any(|source| *source == TabTitleSource::Explicit)
    }

    pub(super) fn predicted_prompt_seed_title(
        tab_title: &TabTitleConfig,
        cwd: Option<&str>,
    ) -> Option<String> {
        if !Self::should_seed_predicted_prompt_title(tab_title) {
            return None;
        }

        let resolved = Self::resolve_template(&tab_title.prompt_format, cwd, None);
        let resolved = resolved.trim();
        if resolved.is_empty() {
            return None;
        }

        Some(Self::truncate_tab_title(resolved))
    }

    pub(super) fn parse_explicit_title(&self, title: &str) -> Option<ExplicitTitlePayload> {
        let prefix = self.tab_title.explicit_prefix.trim();
        if prefix.is_empty() {
            return None;
        }

        let payload = title.strip_prefix(prefix)?.trim();
        if payload.is_empty() {
            return None;
        }

        if let Some(prompt) = payload.strip_prefix("prompt:") {
            let prompt = prompt.trim();
            if prompt.is_empty() {
                return None;
            }
            return Some(ExplicitTitlePayload::Prompt(Self::resolve_template(
                &self.tab_title.prompt_format,
                Some(prompt),
                None,
            )));
        }

        if let Some(command) = payload.strip_prefix("command:") {
            let command = command.trim();
            if command.is_empty() {
                return None;
            }
            return Some(ExplicitTitlePayload::Command(Self::resolve_template(
                &self.tab_title.command_format,
                None,
                Some(command),
            )));
        }

        let explicit = payload.strip_prefix("title:").unwrap_or(payload).trim();
        if explicit.is_empty() {
            return None;
        }

        Some(ExplicitTitlePayload::Title(explicit.to_string()))
    }

    pub(super) fn resolved_tab_title(&self, index: usize) -> String {
        let tab = &self.tabs[index];

        for source in &self.tab_title.priority {
            let candidate = match source {
                TabTitleSource::Manual => tab.manual_title.as_deref(),
                TabTitleSource::Explicit => tab.explicit_title.as_deref(),
                TabTitleSource::Shell => tab.shell_title.as_deref(),
                TabTitleSource::Fallback => Some(self.fallback_title()),
            };

            if let Some(candidate) = candidate.map(str::trim).filter(|value| !value.is_empty()) {
                return Self::truncate_tab_title(candidate);
            }
        }

        Self::truncate_tab_title(self.fallback_title())
    }

    pub(super) fn refresh_tab_title(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        let next = self.resolved_tab_title(index);
        if self.tabs[index].title == next {
            return false;
        }

        self.tabs[index].title = next;
        // Keep title-width behavior uniform across manual, shell, explicit, and fallback sources.
        self.tabs[index].sticky_title_width = 0.0;
        self.tabs[index].title_text_width = 0.0;
        self.mark_tab_strip_layout_dirty();
        true
    }

    pub(super) fn cancel_pending_command_title(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }

        let tab = &mut self.tabs[index];
        tab.pending_command_token = tab.pending_command_token.wrapping_add(1);
        tab.pending_command_title = None;
    }

    pub(super) fn set_explicit_title(&mut self, index: usize, explicit_title: String) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        let explicit_title = Self::truncate_tab_title(&explicit_title);
        if self.tabs[index].explicit_title.as_deref() == Some(explicit_title.as_str()) {
            return false;
        }

        self.tabs[index].explicit_title = Some(explicit_title);
        self.refresh_tab_title(index)
    }

    pub(super) fn schedule_delayed_command_title(
        &mut self,
        index: usize,
        command_title: String,
        delay_ms: u64,
        cx: &mut Context<Self>,
    ) {
        if index >= self.tabs.len() {
            return;
        }

        let tab = &mut self.tabs[index];
        tab.pending_command_token = tab.pending_command_token.wrapping_add(1);
        tab.pending_command_title = Some(Self::truncate_tab_title(&command_title));
        let token = tab.pending_command_token;

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            smol::Timer::after(Duration::from_millis(delay_ms)).await;
            let _ = cx.update(|cx| {
                this.update(cx, |view, cx| {
                    if view.activate_pending_command_title(index, token) {
                        cx.notify();
                    }
                })
            });
        })
        .detach();
    }

    pub(super) fn activate_pending_command_title(&mut self, index: usize, token: u64) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        let tab = &mut self.tabs[index];
        if tab.pending_command_token != token {
            return false;
        }

        let Some(command_title) = tab.pending_command_title.take() else {
            return false;
        };

        if tab.explicit_title.as_deref() == Some(command_title.as_str()) {
            return false;
        }

        tab.explicit_title = Some(command_title);
        self.refresh_tab_title(index)
    }

    pub(super) fn apply_terminal_title(
        &mut self,
        index: usize,
        title: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        let title = title.trim();
        if title.is_empty() || index >= self.tabs.len() {
            return false;
        }

        if let Some(explicit_payload) = self.parse_explicit_title(title) {
            return match explicit_payload {
                ExplicitTitlePayload::Prompt(prompt_title) => {
                    self.tabs[index].running_process = false;
                    self.cancel_pending_command_title(index);
                    self.set_explicit_title(index, prompt_title)
                }
                ExplicitTitlePayload::Title(prompt_title) => {
                    self.cancel_pending_command_title(index);
                    self.set_explicit_title(index, prompt_title)
                }
                ExplicitTitlePayload::Command(command_title) => {
                    self.tabs[index].running_process = true;
                    self.schedule_delayed_command_title(
                        index,
                        command_title,
                        COMMAND_TITLE_DELAY_MS,
                        cx,
                    );
                    false
                }
            };
        }

        let shell_title = Self::truncate_tab_title(title);
        if self.tabs[index].shell_title.as_deref() == Some(shell_title.as_str()) {
            return false;
        }

        self.tabs[index].shell_title = Some(shell_title);
        self.refresh_tab_title(index)
    }

    pub(super) fn clear_terminal_titles(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }

        self.cancel_pending_command_title(index);
        let tab = &mut self.tabs[index];
        tab.running_process = false;
        let had_shell = tab.shell_title.take().is_some();
        let had_explicit = tab.explicit_title.take().is_some();
        if !had_shell && !had_explicit {
            return false;
        }

        self.refresh_tab_title(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{TabTitleConfig, TabTitleSource};

    fn synthetic_text_width(text: &str) -> f32 {
        text.chars()
            .map(|ch| match ch {
                '/' | '\\' => 5.0,
                '.' => 3.5,
                'i' | 'l' | '1' => 4.5,
                'W' | 'M' => 9.0,
                _ => 7.0,
            })
            .sum()
    }

    #[test]
    fn predicted_prompt_seed_title_uses_cwd_template_when_explicit_is_enabled() {
        let config = TabTitleConfig::default();
        let title = TerminalView::predicted_prompt_seed_title(&config, Some("~/projects/termy"));
        assert_eq!(title.as_deref(), Some("~/projects/termy"));
    }

    #[test]
    fn predicted_prompt_seed_title_skips_static_only_priority() {
        let mut config = TabTitleConfig::default();
        config.priority = vec![TabTitleSource::Manual, TabTitleSource::Fallback];

        let title = TerminalView::predicted_prompt_seed_title(&config, Some("~/projects/termy"));
        assert!(title.is_none());
    }

    #[test]
    fn predicted_prompt_seed_title_ignores_empty_resolved_output() {
        let mut config = TabTitleConfig::default();
        config.prompt_format = "{cwd}".to_string();

        let title = TerminalView::predicted_prompt_seed_title(&config, None);
        assert!(title.is_none());
    }

    #[test]
    fn measured_tab_title_fit_keeps_exact_fit_path_untruncated() {
        let title = "~/Desktop";
        let width = synthetic_text_width(title);

        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(title, width, synthetic_text_width),
            title
        );
    }

    #[test]
    fn measured_tab_title_fit_middle_squeezes_path_titles() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs";
        let available = synthetic_text_width("~/Desktop/.../docs");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            synthetic_text_width,
        );

        assert!(formatted.contains("..."));
        assert!(formatted.starts_with("~/"));
        assert!(formatted.ends_with("/docs"));
        assert!(synthetic_text_width(&formatted) <= available);
    }

    #[test]
    fn measured_tab_title_fit_returns_dots_for_tiny_widths() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs";
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width("..."),
                synthetic_text_width,
            ),
            "..."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width(".."),
                synthetic_text_width,
            ),
            ".."
        );
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(title, 0.0, synthetic_text_width),
            ""
        );
    }

    #[test]
    fn measured_tab_title_fit_leaves_non_path_titles_for_end_truncation() {
        let title = "cargo test --workspace --all-features";
        assert_eq!(
            TerminalView::format_tab_label_for_render_measured(
                title,
                synthetic_text_width("cargo test"),
                synthetic_text_width,
            ),
            "cargo test --workspace --all-features"
        );
    }

    #[test]
    fn measured_tab_title_fit_never_overflows_available_width() {
        let title = "~/Desktop/claudeCode/claude-code-provider-proxy/docs/test2/test4/test4";
        let available = synthetic_text_width("~/Desktop/.../test4");
        let formatted = TerminalView::format_tab_label_for_render_measured(
            title,
            available,
            synthetic_text_width,
        );

        assert!(synthetic_text_width(&formatted) <= available);
    }
}
