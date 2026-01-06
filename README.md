# WezBrowser

WezTermベースのWebView統合ターミナルエミュレータ。
ターミナルとブラウザを同一ウィンドウ内で分割表示できます。

![macOS](https://img.shields.io/badge/platform-macOS-blue)
![Rust](https://img.shields.io/badge/language-Rust-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## 特徴

- ターミナルとWebViewを同一ウィンドウで分割表示
- ローカル開発サーバーを見ながらコーディング
- ドキュメント参照しながらターミナル操作
- WezTermの全機能を継承（GPU加速、Lua設定、マルチプレクサ等）

## 使い方

### WebViewを開く

| キー | 説明 |
|------|------|
| `Option+Shift+O` | URL入力 → 右に分割表示 |
| `Option+Shift+U` | URL入力 → 下に分割表示 |

### ナビゲーション

| キー | 説明 |
|------|------|
| `Option+[` | 戻る |
| `Option+]` | 進む |
| `Option+R` | リロード |
| `Option+W` | ペインを閉じる |

## 設定

`~/.config/wezterm/wezterm.lua` でカスタマイズ可能。

```lua
local wezterm = require("wezterm")
local act = wezterm.action

return {
  keys = {
    -- YouTube を Option+Shift+Y で開く
    {
      key = "y",
      mods = "ALT|SHIFT",
      action = act.SplitWebView({
        url = "https://youtube.com",
        direction = "Right",
      }),
    },
  },
}
```

## 利用可能なアクション

- `SplitWebView({ url, direction })` - WebViewを分割表示
- `WebViewGoBack` - 履歴を戻る
- `WebViewGoForward` - 履歴を進む
- `WebViewReload` - リロード

## 制限事項

- macOS専用（WebKitベース）
- 初回起動時は右クリック→「開く」が必要（署名なしアプリのため）

## クレジット

[WezTerm](https://github.com/wez/wezterm) をベースにしています。

## ライセンス

MIT License - 詳細は [LICENSE.md](LICENSE.md) を参照
