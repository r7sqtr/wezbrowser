//! WebViewPane - A pane that renders web content using wry
//!
//! This module implements a Pane that can display web content
//! within the Wezterm terminal, enabling integrated browsing.

use crate::domain::DomainId;
use crate::pane::{
    alloc_pane_id, CachePolicy, CloseReason, ForEachPaneLogicalLine, LogicalLine, Pane, PaneId,
    Pattern, PerformAssignmentResult, SearchResult, WithPaneLines,
};
use crate::renderable::{RenderableDimensions, StableCursorPosition};
use async_trait::async_trait;
use config::keyassignment::KeyAssignment;
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use rangeset::RangeSet;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::ops::Range;
use std::sync::Arc;
use termwiz::surface::{CursorShape, CursorVisibility, Line, SequenceNo, SEQ_ZERO};
use url::Url;
use wezterm_term::color::ColorPalette;
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent, StableRowIndex, TerminalSize};

/// WebView state and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebViewConfig {
    /// Initial URL to load
    pub url: String,
    /// Whether JavaScript is enabled
    pub javascript_enabled: bool,
    /// User agent string (optional)
    pub user_agent: Option<String>,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            url: "about:blank".to_string(),
            javascript_enabled: true,
            user_agent: None,
        }
    }
}

/// Internal state for the WebView
struct WebViewState {
    /// Current URL being displayed
    current_url: String,
    /// Page title
    title: String,
    /// Whether the page is loading
    is_loading: bool,
    /// Current scroll position (for future use)
    scroll_y: f64,
    /// Dimensions in terminal cells
    size: TerminalSize,
    /// Sequence number for change tracking
    seqno: SequenceNo,
}

impl WebViewState {
    fn new(url: &str, size: TerminalSize) -> Self {
        Self {
            current_url: url.to_string(),
            title: "WebView".to_string(),
            is_loading: true,
            scroll_y: 0.0,
            size,
            seqno: SEQ_ZERO,
        }
    }
}

/// A Pane that displays web content using wry WebView
pub struct WebViewPane {
    pane_id: PaneId,
    domain_id: DomainId,
    config: WebViewConfig,
    state: Mutex<WebViewState>,
    /// Placeholder writer for the Pane trait
    writer: Mutex<Vec<u8>>,
}

impl WebViewPane {
    /// Create a new WebViewPane with the given configuration
    pub fn new(domain_id: DomainId, config: WebViewConfig, size: TerminalSize) -> Arc<Self> {
        let pane_id = alloc_pane_id();
        let state = WebViewState::new(&config.url, size);

        Arc::new(Self {
            pane_id,
            domain_id,
            config,
            state: Mutex::new(state),
            writer: Mutex::new(Vec::new()),
        })
    }

    /// Create a WebViewPane with a URL
    pub fn with_url(domain_id: DomainId, url: &str, size: TerminalSize) -> Arc<Self> {
        let config = WebViewConfig {
            url: url.to_string(),
            ..Default::default()
        };
        Self::new(domain_id, config, size)
    }

    /// Navigate to a new URL
    pub fn navigate(&self, url: &str) {
        let mut state = self.state.lock();
        state.current_url = url.to_string();
        state.is_loading = true;
        state.seqno += 1;
    }

    /// Get the current URL
    pub fn current_url(&self) -> String {
        self.state.lock().current_url.clone()
    }

    /// Check if this pane is a WebView pane
    pub fn is_webview_pane(&self) -> bool {
        true
    }
}

/// A dummy writer that captures output
struct WebViewWriter<'a> {
    inner: MutexGuard<'a, Vec<u8>>,
}

impl<'a> Write for WebViewWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // WebView panes don't accept terminal input in the traditional sense
        // but we might use this for IPC or command injection later
        self.inner.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[async_trait(?Send)]
impl Pane for WebViewPane {
    fn pane_id(&self) -> PaneId {
        self.pane_id
    }

    fn get_cursor_position(&self) -> StableCursorPosition {
        // WebView doesn't have a traditional cursor
        StableCursorPosition {
            x: 0,
            y: 0,
            shape: CursorShape::Default,
            visibility: CursorVisibility::Hidden,
        }
    }

    fn get_current_seqno(&self) -> SequenceNo {
        self.state.lock().seqno
    }

    fn get_changed_since(
        &self,
        _lines: Range<StableRowIndex>,
        _seqno: SequenceNo,
    ) -> RangeSet<StableRowIndex> {
        // WebView rendering is handled separately, not line-by-line
        RangeSet::new()
    }

    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        // Return empty lines - WebView content is rendered as a texture overlay
        let state = self.state.lock();
        let num_lines = (lines.end - lines.start) as usize;
        let empty_lines: Vec<Line> = (0..num_lines)
            .map(|_| Line::with_width(state.size.cols as usize, SEQ_ZERO))
            .collect();
        (lines.start, empty_lines)
    }

    fn with_lines_mut(&self, lines: Range<StableRowIndex>, with_lines: &mut dyn WithPaneLines) {
        let (first, mut lines_vec) = self.get_lines(lines);
        let mut refs: Vec<&mut Line> = lines_vec.iter_mut().collect();
        with_lines.with_lines_mut(first, &mut refs);
    }

    fn for_each_logical_line_in_stable_range_mut(
        &self,
        lines: Range<StableRowIndex>,
        for_line: &mut dyn ForEachPaneLogicalLine,
    ) {
        crate::pane::impl_for_each_logical_line_via_get_logical_lines(self, lines, for_line);
    }

    fn get_logical_lines(&self, lines: Range<StableRowIndex>) -> Vec<LogicalLine> {
        crate::pane::impl_get_logical_lines_via_get_lines(self, lines)
    }

    fn get_dimensions(&self) -> RenderableDimensions {
        let state = self.state.lock();
        RenderableDimensions {
            cols: state.size.cols,
            viewport_rows: state.size.rows,
            scrollback_rows: 0,
            physical_top: 0,
            scrollback_top: 0,
            dpi: state.size.dpi,
            pixel_width: state.size.pixel_width,
            pixel_height: state.size.pixel_height,
            reverse_video: false,
        }
    }

    fn get_title(&self) -> String {
        let state = self.state.lock();
        if state.title.is_empty() {
            format!("WebView: {}", state.current_url)
        } else {
            state.title.clone()
        }
    }

    fn send_paste(&self, _text: &str) -> anyhow::Result<()> {
        // TODO: Forward paste to WebView via JavaScript injection
        Ok(())
    }

    fn reader(&self) -> anyhow::Result<Option<Box<dyn std::io::Read + Send>>> {
        // WebView doesn't produce readable output in the traditional sense
        Ok(None)
    }

    fn writer(&self) -> MappedMutexGuard<'_, dyn std::io::Write> {
        MutexGuard::map(self.writer.lock(), |w| w as &mut dyn std::io::Write)
    }

    fn resize(&self, size: TerminalSize) -> anyhow::Result<()> {
        let mut state = self.state.lock();
        state.size = size;
        state.seqno += 1;
        // TODO: Resize the actual WebView
        Ok(())
    }

    fn key_down(&self, key: KeyCode, mods: KeyModifiers) -> anyhow::Result<()> {
        // TODO: Forward key events to WebView
        log::trace!("WebViewPane key_down: {:?} mods={:?}", key, mods);
        Ok(())
    }

    fn key_up(&self, key: KeyCode, mods: KeyModifiers) -> anyhow::Result<()> {
        log::trace!("WebViewPane key_up: {:?} mods={:?}", key, mods);
        Ok(())
    }

    fn perform_assignment(&self, assignment: &KeyAssignment) -> PerformAssignmentResult {
        // Handle WebView-specific key assignments
        match assignment {
            // TODO: Add WebView-specific actions like navigate back/forward
            _ => PerformAssignmentResult::Unhandled,
        }
    }

    fn mouse_event(&self, event: MouseEvent) -> anyhow::Result<()> {
        // TODO: Forward mouse events to WebView
        log::trace!("WebViewPane mouse_event: {:?}", event);
        Ok(())
    }

    fn is_dead(&self) -> bool {
        false
    }

    fn palette(&self) -> ColorPalette {
        ColorPalette::default()
    }

    fn domain_id(&self) -> DomainId {
        self.domain_id
    }

    fn is_mouse_grabbed(&self) -> bool {
        // WebView always captures mouse for interaction
        true
    }

    fn is_alt_screen_active(&self) -> bool {
        // WebView is always in "alt screen" mode conceptually
        true
    }

    fn get_current_working_dir(&self, _policy: CachePolicy) -> Option<Url> {
        Url::parse(&self.state.lock().current_url).ok()
    }

    fn can_close_without_prompting(&self, _reason: CloseReason) -> bool {
        // WebView can always be closed without prompting
        true
    }

    async fn search(
        &self,
        pattern: Pattern,
        _range: Range<StableRowIndex>,
        _limit: Option<u32>,
    ) -> anyhow::Result<Vec<SearchResult>> {
        // TODO: Implement in-page search via WebView's find API
        log::debug!("WebViewPane search: {:?}", pattern);
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_pane_creation() {
        let size = TerminalSize {
            rows: 24,
            cols: 80,
            pixel_width: 800,
            pixel_height: 600,
            dpi: 96,
        };
        let pane = WebViewPane::with_url(0, "https://example.com", size);
        assert!(pane.is_webview_pane());
        assert_eq!(pane.current_url(), "https://example.com");
    }

    #[test]
    fn test_webview_navigate() {
        let size = TerminalSize {
            rows: 24,
            cols: 80,
            pixel_width: 800,
            pixel_height: 600,
            dpi: 96,
        };
        let pane = WebViewPane::with_url(0, "https://example.com", size);
        pane.navigate("https://rust-lang.org");
        assert_eq!(pane.current_url(), "https://rust-lang.org");
    }
}
