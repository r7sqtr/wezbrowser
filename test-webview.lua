-- WezBrowser Test Configuration
-- WebView-enabled WezTerm configuration

local wezterm = require 'wezterm'
local act = wezterm.action
local config = wezterm.config_builder()

-- Basic configuration
config.color_scheme = 'Dracula'
config.font_size = 14.0

-- Key bindings for WebView
config.keys = {
  -- Open URL prompt (Option+Shift+O)
  {
    key = 'o',
    mods = 'ALT|SHIFT',
    action = act.PromptInputLine {
      description = 'Enter URL to open in WebView (right split):',
      action = wezterm.action_callback(function(window, pane, line)
        if line and line ~= '' then
          -- Add https:// if no protocol specified
          local url = line
          if not url:match('^https?://') then
            url = 'https://' .. url
          end
          window:perform_action(
            act.SplitWebView {
              url = url,
              direction = 'Right',
            },
            pane
          )
        end
      end),
    },
  },
  -- Open URL in bottom split (Option+Shift+U)
  {
    key = 'u',
    mods = 'ALT|SHIFT',
    action = act.PromptInputLine {
      description = 'Enter URL to open in WebView (bottom split):',
      action = wezterm.action_callback(function(window, pane, line)
        if line and line ~= '' then
          local url = line
          if not url:match('^https?://') then
            url = 'https://' .. url
          end
          window:perform_action(
            act.SplitWebView {
              url = url,
              direction = 'Down',
            },
            pane
          )
        end
      end),
    },
  },

  -- Quick access sites
  -- Split right with Google (Option+Shift+G)
  {
    key = 'g',
    mods = 'ALT|SHIFT',
    action = act.SplitWebView {
      url = 'https://www.google.com',
      direction = 'Right',
    },
  },
  -- Split bottom with GitHub (Option+Shift+H)
  {
    key = 'h',
    mods = 'ALT|SHIFT',
    action = act.SplitWebView {
      url = 'https://github.com',
      direction = 'Down',
    },
  },

  -- WebView Navigation
  -- Go back (Option+Left or Option+[)
  {
    key = 'LeftArrow',
    mods = 'ALT',
    action = act.WebViewGoBack,
  },
  {
    key = '[',
    mods = 'ALT',
    action = act.WebViewGoBack,
  },
  -- Go forward (Option+Right or Option+])
  {
    key = 'RightArrow',
    mods = 'ALT',
    action = act.WebViewGoForward,
  },
  {
    key = ']',
    mods = 'ALT',
    action = act.WebViewGoForward,
  },
  -- Reload (Option+R)
  {
    key = 'r',
    mods = 'ALT',
    action = act.WebViewReload,
  },
}

return config
