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

## インストール

### 前提条件

- macOS 10.12以上
- Rust（ビルドする場合）

### リリースビルドのインストール

```bash
# リポジトリのクローン
git clone https://github.com/user/wezbrowser.git
cd wezbrowser/wezterm

# ビルド
cargo build --release -p wezterm-gui

# アプリケーションを起動
open target/release/WezBrowser.app
```

### DMGパッケージの作成

```bash
./scripts/build-release.sh 0.1.0
# 出力: target/release/WezBrowser-0.1.0-macos.dmg
```

### 初回起動時の注意

署名なしアプリのため、初回起動時は以下の手順が必要です：

1. FinderでWezBrowser.appを右クリック
2. 「開く」を選択
3. 確認ダイアログで「開く」をクリック

## 使い方

### WebViewを開く

| キー | 説明 |
|------|------|
| `Option+Shift+O` | URL入力 → 右に分割表示 |
| `Option+Shift+U` | URL入力 → 下に分割表示 |
| `Option+Shift+G` | Googleを右分割で開く |
| `Option+Shift+H` | GitHubを下分割で開く |

### ナビゲーション

| キー | 説明 |
|------|------|
| `Option+[` または `Option+←` | 戻る |
| `Option+]` または `Option+→` | 進む |
| `Option+R` | リロード |
| `Option+W` | ペインを閉じる |

### ペイン操作

WezTermの標準ペイン操作がそのまま使えます：

| キー | 説明 |
|------|------|
| `Cmd+Shift+←/→/↑/↓` | ペイン間の移動 |
| `Cmd+Shift+Z` | ペインのズーム切り替え |

## 設定

`~/.config/wezterm/wezterm.lua` でカスタマイズ可能。

### 基本的な設定例

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

### 分割サイズの指定

```lua
-- ペインを30%のサイズで分割
{
  key = "d",
  mods = "ALT|SHIFT",
  action = act.SplitWebView({
    url = "https://docs.example.com",
    direction = "Right",
    size = wezterm.SplitSize.Percent(30),
  }),
}

-- セル数で指定（80セル幅）
{
  key = "e",
  mods = "ALT|SHIFT",
  action = act.SplitWebView({
    url = "https://example.com",
    direction = "Down",
    size = wezterm.SplitSize.Cells(80),
  }),
}
```

### URLプロンプトの設定

```lua
-- URLを入力して開くプロンプト
{
  key = "o",
  mods = "ALT|SHIFT",
  action = act.PromptInputLine({
    description = "Enter URL to open",
    action = wezterm.action_callback(function(window, pane, url)
      if url and url ~= "" then
        window:perform_action(
          act.SplitWebView({ url = url, direction = "Right" }),
          pane
        )
      end
    end),
  }),
}
```

## 利用可能なアクション

| アクション | 説明 | パラメータ |
|-----------|------|----------|
| `SplitWebView` | WebViewを分割表示 | `url`, `direction`, `size`(任意) |
| `WebViewGoBack` | 履歴を戻る | なし |
| `WebViewGoForward` | 履歴を進む | なし |
| `WebViewReload` | リロード | なし |
| `WebViewNavigate` | 指定URLに移動 | `url` |

### SplitWebViewのパラメータ

| パラメータ | 型 | 説明 |
|-----------|-----|------|
| `url` | String | 開くURL |
| `direction` | String | 分割方向: `"Up"`, `"Down"`, `"Left"`, `"Right"` |
| `size` | SplitSize | (任意) ペインサイズ: `Percent(n)` または `Cells(n)` |

## 開発・デバッグ

### デバッグビルド

```bash
cargo build -p wezterm-gui
```

### ログ付きで起動

```bash
WEZTERM_LOG=info ./target/release/WezBrowser.app/Contents/MacOS/wezterm-gui
```

### 詳細なログ

```bash
WEZTERM_LOG=debug ./target/release/WezBrowser.app/Contents/MacOS/wezterm-gui 2>&1 | tee debug.log
```

### DevTools

WebViewには開発者ツール（DevTools）が自動的に有効化されています。
WebView上で右クリック → 「Inspect Element」で開きます。

## 技術仕様

- **WebViewエンジン**: wry v0.50 (macOSではWebKit使用)
- **レンダリング**: GPU加速対応
- **プラットフォーム**: macOS専用

## 制限事項

- **macOS専用**: WebView機能はmacOSでのみ動作します（Windows/Linuxは未対応）
- **HTTPS推奨**: HTTPのlocalhostはApp Transport Securityにより制限される場合があります
- **署名なしアプリ**: 初回起動時は右クリック→「開く」が必要

## クレジット

[WezTerm](https://github.com/wez/wezterm) をベースにしています。

## ライセンス

MIT License - 詳細は [LICENSE.md](LICENSE.md) を参照
