# WezBrowser

WezTermをベースにしたWebView統合ターミナルエミュレータ。ターミナルとブラウザを同一ウィンドウ内で分割表示できます。

## ビルド

```bash
cd /Users/vvsaito/Development/wezbrowser/wezterm
cargo build --release -p wezterm-gui
```

## 起動方法

### アプリバンドルとして起動（推奨）

```bash
open /Users/vvsaito/Development/wezbrowser/wezterm/target/release/WezBrowser.app
```

### バイナリを直接起動

```bash
/Users/vvsaito/Development/wezbrowser/wezterm/target/release/WezBrowser.app/Contents/MacOS/wezterm-gui
```

### デバッグログ付きで起動

```bash
WEZTERM_LOG=info /Users/vvsaito/Development/wezbrowser/wezterm/target/release/WezBrowser.app/Contents/MacOS/wezterm-gui
```

## キーバインド

### WebView操作

| キー | 説明 |
|------|------|
| `Option+Shift+O` | URL入力プロンプト（右分割） |
| `Option+Shift+U` | URL入力プロンプト（下分割） |
| `Option+Shift+G` | Googleを開く（右分割） |
| `Option+Shift+H` | GitHubを開く（下分割） |

### ナビゲーション

| キー | 説明 |
|------|------|
| `Option+[` | 戻る |
| `Option+]` | 進む |
| `Option+R` | リロード |

### ペイン操作

| キー | 説明 |
|------|------|
| `Option+W` | 現在のペインを閉じる（確認なし） |
| `Cmd+W` | 現在のペインを閉じる（確認あり） |

## 使用例

### 1. URLを入力してWebページを開く

1. `Option+Shift+O` を押す
2. URLを入力（例: `google.com`）
3. Enter を押す
4. 右側にWebViewペインが開く

### 2. 開発中のローカルサーバーを表示

1. ターミナルでローカルサーバーを起動
   ```bash
   npm run dev  # localhost:3000 など
   ```
2. `Option+Shift+U` を押す
3. `localhost:3000` と入力
4. 下部にWebViewが表示される

### 3. ドキュメントを参照しながらコーディング

1. `Option+Shift+O` でドキュメントサイトを開く
2. ターミナルペインでコードを編集
3. `Option+[` / `Option+]` でドキュメント内を移動

## 設定ファイル

WezBrowserの設定は `~/.config/wezterm/` に配置されます。

### ファイル構成

```
~/.config/wezterm/
├── wezterm.lua           # メイン設定
└── config/
    ├── ui.lua
    ├── fonts.lua
    ├── keymap.lua
    ├── colorscheme.lua
    └── webview.lua       # WebView設定
```

### webview.lua の内容

```lua
local wezterm = require("wezterm")
local act = wezterm.action

local webview = {}

function webview.setup(config)
    local webview_keys = {
        -- URL入力プロンプト
        {
            key = "o",
            mods = "ALT|SHIFT",
            action = act.PromptInputLine({
                description = "Enter URL to open in WebView (right split):",
                action = wezterm.action_callback(function(window, pane, line)
                    if line and line ~= "" then
                        local url = line
                        if not url:match("^https?://") then
                            url = "https://" .. url
                        end
                        window:perform_action(
                            act.SplitWebView({
                                url = url,
                                direction = "Right",
                            }),
                            pane
                        )
                    end
                end),
            }),
        },
        -- 他のキーバインド...
    }

    config.keys = config.keys or {}
    for _, key in ipairs(webview_keys) do
        table.insert(config.keys, key)
    end
end

return webview
```

### カスタムショートカットの追加

`webview.lua` に新しいショートカットを追加できます：

```lua
-- YouTube を Option+Shift+Y で開く
{
    key = "y",
    mods = "ALT|SHIFT",
    action = act.SplitWebView({
        url = "https://youtube.com",
        direction = "Right",
    }),
},
```

## 利用可能なアクション

### SplitWebView

WebViewペインを分割して開きます。

```lua
act.SplitWebView({
    url = "https://example.com",
    direction = "Right",  -- "Right", "Left", "Up", "Down"
})
```

### WebViewGoBack

WebViewの履歴を戻ります。

```lua
act.WebViewGoBack
```

### WebViewGoForward

WebViewの履歴を進みます。

```lua
act.WebViewGoForward
```

### WebViewReload

WebViewをリロードします。

```lua
act.WebViewReload
```

## 制限事項

- **macOS専用**: WebView機能はmacOSでのみ動作します
- **localhost**: HTTP（非HTTPS）のlocalhostアクセスは制限される場合があります
- **開発者ツール**: WebView上で右クリックすると開発者ツールにアクセスできます

## トラブルシューティング

### WebViewが表示されない

1. アプリバンドル（`.app`）として起動しているか確認
2. ログを確認: `WEZTERM_LOG=info` で起動

### localhostが読み込めない

macOSのApp Transport Securityの制限により、HTTPのlocalhostがブロックされる場合があります。HTTPSを使用するか、開発サーバーをHTTPSで起動してください。

### ペインを閉じてもWebViewが残る

通常は自動的にクリーンアップされますが、残る場合はウィンドウを閉じて再起動してください。

## 配布

### リリースビルドの作成

```bash
./scripts/build-release.sh 0.1.0
```

これにより以下が生成されます：
- `target/release/WezBrowser.app` - アプリバンドル
- `target/release/WezBrowser-0.1.0-macos.dmg` - 配布用DMG

### GitHub Releasesでの配布

1. GitHubリポジトリを作成
2. DMGファイルをReleasesにアップロード
3. ユーザーはDMGをダウンロードしてインストール

**注意**: 署名なしアプリのため、ユーザーは初回起動時に以下の手順が必要：
1. アプリを右クリック
2. 「開く」を選択
3. 警告ダイアログで「開く」をクリック

### Apple Developer IDで署名する場合（オプション）

Apple Developer Program（年間$99）に加入すると、警告なしで配布できます：

```bash
# Developer ID で署名
codesign --force --deep --sign "Developer ID Application: Your Name (TEAM_ID)" WezBrowser.app

# Apple に公証を申請
xcrun notarytool submit WezBrowser.dmg --apple-id "your@email.com" --team-id "TEAM_ID" --password "app-specific-password" --wait

# 公証完了後、ステープル
xcrun stapler staple WezBrowser.dmg
```

## 技術情報

- **ベース**: WezTerm
- **WebViewエンジン**: wry (WebKit on macOS)
- **言語**: Rust + Lua (設定)
