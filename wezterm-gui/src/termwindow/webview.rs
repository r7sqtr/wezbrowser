//! WebView integration for WezTerm
//!
//! This module handles the integration of WebView panes within the terminal window.
//! On macOS, it uses wry/WebKit to render web content as an overlay on the terminal.

use mux::pane::PaneId;
use mux::webview_pane::WebViewPane;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use raw_window_handle::HasWindowHandle;
#[cfg(target_os = "macos")]
use wry::{Rect as WryRect, WebView, WebViewBuilder};

/// Represents the position and size of a WebView overlay (in physical pixels)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scale_factor: f64,
}

impl WebViewRect {
    pub fn new(x: f64, y: f64, width: f64, height: f64, scale_factor: f64) -> Self {
        Self { x, y, width, height, scale_factor }
    }

    #[cfg(target_os = "macos")]
    fn to_wry_rect(&self) -> WryRect {
        // Convert from physical pixels to logical pixels
        let scale = if self.scale_factor > 0.0 { self.scale_factor } else { 1.0 };
        WryRect {
            position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(
                self.x / scale,
                self.y / scale,
            )),
            size: wry::dpi::Size::Logical(wry::dpi::LogicalSize::new(
                self.width / scale,
                self.height / scale,
            )),
        }
    }
}

/// Manages WebView instances for WebViewPane overlays
pub struct WebViewManager {
    /// Map of pane_id to WebView instances
    #[cfg(target_os = "macos")]
    webviews: HashMap<PaneId, WebView>,

    /// Current URLs for tracking changes
    #[cfg(target_os = "macos")]
    urls: HashMap<PaneId, String>,

    /// Cached bounds to avoid unnecessary updates
    #[cfg(target_os = "macos")]
    cached_bounds: HashMap<PaneId, WebViewRect>,

    /// Fallback for non-macOS platforms
    #[cfg(not(target_os = "macos"))]
    _phantom: std::marker::PhantomData<()>,
}

impl WebViewManager {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            webviews: HashMap::new(),
            #[cfg(target_os = "macos")]
            urls: HashMap::new(),
            #[cfg(target_os = "macos")]
            cached_bounds: HashMap::new(),
            #[cfg(not(target_os = "macos"))]
            _phantom: std::marker::PhantomData,
        }
    }

    /// Check if a WebView exists for the given pane
    #[cfg(target_os = "macos")]
    pub fn has_webview(&self, pane_id: PaneId) -> bool {
        self.webviews.contains_key(&pane_id)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn has_webview(&self, _pane_id: PaneId) -> bool {
        false
    }

    /// Create or update a WebView for the given pane
    #[cfg(target_os = "macos")]
    pub fn create_or_update_webview<W: HasWindowHandle>(
        &mut self,
        pane_id: PaneId,
        url: &str,
        rect: WebViewRect,
        window: &W,
    ) -> anyhow::Result<()> {
        if let Some(webview) = self.webviews.get(&pane_id) {
            // Update existing WebView URL if changed
            let current_url = self.urls.get(&pane_id).map(|s| s.as_str()).unwrap_or("");
            if current_url != url {
                log::info!("Navigating WebView {} from {} to {}", pane_id, current_url, url);
                webview.load_url(url)?;
                self.urls.insert(pane_id, url.to_string());
            }

            // Only update bounds if they have changed (performance optimization)
            let bounds_changed = self.cached_bounds.get(&pane_id)
                .map(|cached| cached != &rect)
                .unwrap_or(true);

            if bounds_changed {
                if let Err(e) = webview.set_bounds(rect.to_wry_rect()) {
                    log::error!("Failed to update WebView bounds: {}", e);
                } else {
                    self.cached_bounds.insert(pane_id, rect);
                }
            }
        } else {
            // Create new WebView
            log::info!("Creating WebView for pane {} with URL: {} at {:?}", pane_id, url, rect);

            let window_handle = window.window_handle()
                .map_err(|e| anyhow::anyhow!("Failed to get window handle: {:?}", e))?;

            // Build WebView as a child of the window
            let webview = WebViewBuilder::new()
                .with_bounds(rect.to_wry_rect())
                .with_url(url)
                .with_transparent(false)
                .with_devtools(true)  // Always enable devtools for debugging
                .with_navigation_handler(|url| {
                    log::info!("WebView navigation request: {}", url);
                    true  // Allow all navigation
                })
                .with_on_page_load_handler(|event, url| {
                    match event {
                        wry::PageLoadEvent::Started => {
                            log::info!("WebView page load started: {}", url);
                        }
                        wry::PageLoadEvent::Finished => {
                            log::info!("WebView page load finished: {}", url);
                        }
                    }
                })
                .build_as_child(&window_handle)?;

            self.webviews.insert(pane_id, webview);
            self.urls.insert(pane_id, url.to_string());
            self.cached_bounds.insert(pane_id, rect);
            log::info!("WebView created successfully for pane {}", pane_id);
        }

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn create_or_update_webview<W>(
        &mut self,
        _pane_id: PaneId,
        _url: &str,
        _rect: WebViewRect,
        _window: &W,
    ) -> anyhow::Result<()> {
        log::warn!("WebView integration is only supported on macOS");
        Ok(())
    }

    /// Remove a WebView for the given pane
    #[cfg(target_os = "macos")]
    pub fn remove_webview(&mut self, pane_id: PaneId) {
        if self.webviews.remove(&pane_id).is_some() {
            self.urls.remove(&pane_id);
            self.cached_bounds.remove(&pane_id);
            log::info!("Removed WebView for pane {}", pane_id);
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn remove_webview(&mut self, _pane_id: PaneId) {}

    /// Update the visibility of a WebView
    #[cfg(target_os = "macos")]
    pub fn set_webview_visible(&mut self, pane_id: PaneId, visible: bool) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::debug!("Setting WebView {} visibility to {}", pane_id, visible);
            if let Err(e) = webview.set_visible(visible) {
                log::error!("Failed to set WebView visibility: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn set_webview_visible(&mut self, _pane_id: PaneId, _visible: bool) {}

    /// Update the position and size of a WebView
    #[cfg(target_os = "macos")]
    pub fn update_webview_rect(&mut self, pane_id: PaneId, rect: WebViewRect) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::debug!("Updating WebView {} rect to {:?}", pane_id, rect);
            if let Err(e) = webview.set_bounds(rect.to_wry_rect()) {
                log::error!("Failed to update WebView bounds: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn update_webview_rect(&mut self, _pane_id: PaneId, _rect: WebViewRect) {}

    /// Navigate a WebView to a new URL
    #[cfg(target_os = "macos")]
    pub fn navigate(&mut self, pane_id: PaneId, url: &str) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::info!("Navigating WebView {} to {}", pane_id, url);
            if let Err(e) = webview.load_url(url) {
                log::error!("Failed to navigate WebView: {}", e);
            } else {
                self.urls.insert(pane_id, url.to_string());
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn navigate(&mut self, _pane_id: PaneId, _url: &str) {}

    /// Go back in WebView history
    #[cfg(target_os = "macos")]
    pub fn go_back(&self, pane_id: PaneId) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::info!("WebView {} going back", pane_id);
            if let Err(e) = webview.evaluate_script("window.history.back()") {
                log::error!("Failed to go back in WebView: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn go_back(&self, _pane_id: PaneId) {}

    /// Go forward in WebView history
    #[cfg(target_os = "macos")]
    pub fn go_forward(&self, pane_id: PaneId) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::info!("WebView {} going forward", pane_id);
            if let Err(e) = webview.evaluate_script("window.history.forward()") {
                log::error!("Failed to go forward in WebView: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn go_forward(&self, _pane_id: PaneId) {}

    /// Reload the WebView
    #[cfg(target_os = "macos")]
    pub fn reload(&self, pane_id: PaneId) {
        if let Some(webview) = self.webviews.get(&pane_id) {
            log::info!("Reloading WebView {}", pane_id);
            if let Err(e) = webview.evaluate_script("window.location.reload()") {
                log::error!("Failed to reload WebView: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn reload(&self, _pane_id: PaneId) {}

    /// Execute JavaScript in a WebView
    #[cfg(target_os = "macos")]
    pub fn eval_script(&self, pane_id: PaneId, script: &str) -> anyhow::Result<()> {
        if let Some(webview) = self.webviews.get(&pane_id) {
            webview.evaluate_script(script)?;
            Ok(())
        } else {
            anyhow::bail!("WebView not found for pane {}", pane_id)
        }
    }

    #[cfg(not(target_os = "macos"))]
    pub fn eval_script(&self, _pane_id: PaneId, _script: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// Get all active WebView pane IDs
    #[cfg(target_os = "macos")]
    pub fn active_pane_ids(&self) -> Vec<PaneId> {
        self.webviews.keys().copied().collect()
    }

    #[cfg(not(target_os = "macos"))]
    pub fn active_pane_ids(&self) -> Vec<PaneId> {
        vec![]
    }
}

impl Default for WebViewManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if a pane is a WebViewPane
pub fn is_webview_pane(pane: &Arc<dyn mux::pane::Pane>) -> bool {
    pane.downcast_ref::<WebViewPane>().is_some()
}

/// Try to get a WebViewPane reference from a generic Pane
#[allow(dead_code)]
pub fn as_webview_pane(_pane: &Arc<dyn mux::pane::Pane>) -> Option<Arc<WebViewPane>> {
    // Note: downcast-rs doesn't support Arc directly
    // We would need to use a different pattern for this
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_manager_creation() {
        let manager = WebViewManager::new();
        assert!(!manager.has_webview(0));
    }

    #[test]
    fn test_webview_rect() {
        let rect = WebViewRect::new(10.0, 20.0, 800.0, 600.0, 2.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 800.0);
        assert_eq!(rect.height, 600.0);
        assert_eq!(rect.scale_factor, 2.0);
    }
}
