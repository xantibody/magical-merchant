# Dioxus移行 + UI刷新プラン

## Context

現在のTauri + React + TypeScriptフロントエンドを、Dioxus desktopに全面移行する。
目的はパフォーマンス改善と技術スタックの統一（全Rust化）。同時にUIをミニマルデザインに刷新する。

**現状**: `core/`(Rust) → Tauri IPC bridge → React + Tailwind (TypeScript)
**移行後**: `core/`(Rust) → Dioxus desktop (Rust RSX) + Open Props CSS

`tauri-app/`は移行完了まで残し、`dioxus-app/`を並行して構築する。

## 技術選定

| 項目 | 選定 |
|---|---|
| フレームワーク | Dioxus 0.7 desktop |
| スタイリング | Open Props (CSS custom properties) |
| アイコン | Phosphor Icons (SVG) |
| Markdown変換 | pulldown-cmark |
| コードハイライト | syntect（後回し可） |

## UI設計

```
┌──────────────────────────┐
│ [≡]              [⚡]    │  ← トグル(メニュー開閉) + 現在のモードアイコン
├──────────────────────────┤
│                          │
│                          │
│   メモ領域（全画面）       │  ← 選択中モードの入力/表示エリア
│   インラインMarkdown変換   │
│                          │
│                          │
│              [送信][保存] │  ← フリック/ホバーで固有アクション表示
└──────────────────────────┘
```

- **ヘッダー**: トグルボタン（メニュー開閉）+ 現在モードのアイコン。最小限。
- **トグルメニュー**: 3モード選択（Timeline / Notes / Tasks）
- **メモ領域**: 画面全体を占有。各モードに応じた入力UI。
- **固有アクション**: PC=ホバー、モバイル=フリックで表示。モードごとに2〜3個。
- **編集中**: インラインMarkdown即時変換（Typora風）
- **保存済み閲覧**: 別issue（#5）

## ステップ

### 00 - ブランチ作成 + プロジェクト設定

**Goal**: 開発ブランチを切り、CLAUDE.md・UIコンポーネントスキルを配置

- `git checkout -b feat/dioxus-migration` でブランチ作成
- `CLAUDE.md`（リポジトリルート）作成 — デザイン優先順位（Simple > Lightweight > Stylish）、技術スタック、UI方針を記載
- `.claude/skills/ui-component/SKILL.md` 作成 — Open Props変数の使い方、アクションバー、トグルメニュー、テキスト入力、Markdownプレビューのパターン定義

**Verify**: ブランチが `feat/dioxus-migration` であること。`CLAUDE.md` と `.claude/skills/ui-component/SKILL.md` が存在すること。

### 01 - Dioxusクレート雛形

**Goal**: 最小のDioxusデスクトップアプリをワークスペースに追加

- `dioxus-app/Cargo.toml` 作成（deps: `dioxus = { version = "0.7", features = ["desktop", "router"] }`, `magical-merchant-core = { path = "../core" }`, `dirs = "6"`, `chrono = "0.4"`)
- `dioxus-app/src/main.rs` 作成（`dioxus::launch(App)` + stub App）
- `Cargo.toml`（workspace root）に `"dioxus-app"` を追加
- `dioxus-app/Dioxus.toml` 作成（アプリ設定）

**Verify**: `cd dioxus-app && dx serve --platform desktop` でウィンドウ表示

### 02 - データディレクトリ解決

**Goal**: Tauriと同じデータディレクトリを使うモジュール作成

- `dioxus-app/src/data_dir.rs` 作成
- Tauri identifier `com.magical-merchant.app` に基づくパス解決
- macOS: `~/Library/Application Support/com.magical-merchant.app/`

**Verify**: ユニットテスト。既存のTauriアプリで作ったデータが見えること。

**参照ファイル**:
- `tauri-app/src-tauri/tauri.conf.json` (identifier: `com.magical-merchant.app`)

### 03 - ルーティングとレイアウト骨格

**Goal**: 3ビュー + ヘッダー（トグル+モードアイコン）の骨格

- `dioxus-app/src/views/mod.rs` 作成
- `dioxus-app/src/views/timeline.rs` - stub
- `dioxus-app/src/views/notes.rs` - stub
- `dioxus-app/src/views/tasks.rs` - stub
- `dioxus-app/src/main.rs` に `Route` enum定義、`Layout`コンポーネント（ヘッダー + Outlet）

```rust
#[derive(Routable, Clone, Debug, PartialEq)]
enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Timeline {},
    #[route("/notes")]
    Notes {},
    #[route("/tasks")]
    Tasks {},
}
```

- `AppLayout`: ヘッダー（トグルボタン + モードアイコン）+ メニュー開閉状態

**Verify**: アプリ起動、メニューからモード切替が動作

### 04 - Open Props + 基本スタイル

**Goal**: CSSデザイントークンとアプリの基本スタイル適用

- `dioxus-app/assets/open-props.min.css` ダウンロード配置
- `dioxus-app/assets/style.css` 作成（レイアウト、ヘッダー、メニュー、メモ領域のスタイル）
- `main.rs`の`AppLayout`に`document::Stylesheet`追加

**Verify**: スタイルが適用され、ミニマルな見た目になる

### 05 - Phosphor Iconsの導入

**Goal**: ナビゲーションアイコンをPhosphorに

- `dioxus-app/assets/icons/` に必要なSVGをダウンロード
  - `lightning.svg` (Timeline)
  - `note-pencil.svg` (Notes)
  - `check-square.svg` (Tasks)
  - `list.svg` (トグルメニュー)
- `dioxus-app/src/components/mod.rs` + `icon.rs` 作成
- ヘッダーとメニューにアイコン適用

**Verify**: アイコンが正しく表示される

### 06 - Timelineビュー実装

**Goal**: QuickCapture相当の機能をDioxusで実装

- `dioxus-app/src/views/timeline.rs` 実装
  - textarea + 送信（フリック/ホバーで表示）
  - 今日のエントリ一覧表示（逆順）
  - `use_resource` + `spawn_blocking` で `core::read_timeline` / `core::save_timeline_entry` 呼び出し

**参照ファイル**:
- `tauri-app/src/components/QuickCapture.tsx` (移行元)
- `core/src/save.rs` (`save_timeline_entry`, `read_timeline`)

**Verify**: テキスト入力→送信→リスト表示。再起動後もデータ永続化。

### 07 - Markdownレンダリングモジュール

**Goal**: pulldown-cmarkでMarkdown→HTML変換

- `dioxus-app/Cargo.toml` に `pulldown-cmark = "0.12"` 追加
- `dioxus-app/src/markdown.rs` 作成
  - `pub fn render_markdown(source: &str) -> String`

**Verify**: ユニットテスト（見出し、太字、リスト、コードブロック等）

### 08 - Notesビュー実装（エディタ + 自動保存）

**Goal**: Markdownエディタ + 自動保存（1秒デバウンス）

- `dioxus-app/src/views/notes.rs` 実装
  - textarea（全画面メモ領域）
  - タグ入力（フリック/ホバーで表示）
  - 自動保存: `use_effect` + debounce → `core::create_draft_note` / `core::update_note`
  - 保存ステータス表示
  - "Done"ボタン（フリック/ホバーで表示）

**参照ファイル**:
- `tauri-app/src/components/Document.tsx` (移行元、autosave debounceロジック)
- `core/src/note.rs` (`create_draft_note`, `update_note`, `list_notes`)

**Verify**: 入力→1秒後に自動保存→ステータス表示。ファイルシステムにファイル生成確認。

### 09 - インラインMarkdownプレビュー

**Goal**: 編集中にMarkdownをリアルタイム変換表示（Typora風）

- `dioxus-app/src/components/markdown_editor.rs` 作成
  - textarea入力 → `markdown::render_markdown` → プレビュー領域に`dangerous_inner_html`で表示
  - 初期実装: 入力エリアの下にリアルタイムプレビュー（将来的にインライン変換へ進化）
- Notes / Timeline ビューに統合

**Verify**: 入力中にプレビューがリアルタイム更新される

### 10 - Tasksビュー実装

**Goal**: プロジェクト+タスク管理ビュー

- `dioxus-app/src/views/tasks.rs` 実装
  - プロジェクト選択（ドロップダウンまたはリスト）
  - アクティブタスク一覧
  - タスク作成（タイトル、タグ、本文）
  - タスク完了（フリック/ホバーで表示）

**参照ファイル**:
- `core/src/project.rs` (`create_project`, `list_projects`, `create_task`, `list_active_tasks`, `complete_task`, `update_task`)

**Verify**: プロジェクト選択→タスク一覧→作成→完了の一連の流れが動作

### 11 - アクションボタン（フリック/ホバー）統一

**Goal**: 全ビューのアクションボタンをホバー/フリック表示に統一

- `dioxus-app/src/components/action_bar.rs` 作成
  - CSS: `opacity: 0` → ホバーで `opacity: 1`（transition付き）
  - 各ビューのアクションボタンをこのコンポーネントでラップ
- Timeline: [送信]
- Notes: [保存] [Done] [タグ編集]
- Tasks: [作成] [完了]

**Verify**: 通常時はボタン非表示、ホバーで表示・操作可能

### 12 - ダークモードCSS基盤

**Goal**: システム設定に応じたダークモード

- `dioxus-app/assets/style.css` にCSS custom properties層を追加
  - `--surface`, `--text`, `--accent` 等
  - `@media (prefers-color-scheme: dark)` で変数を再定義

**Verify**: macOSダークモード切替に連動して表示が変わる

### 13 - justfile更新

**Goal**: 開発ワークフローにDioxusコマンド追加

- `justfile` に追加:
  - `dx-dev`: `cd dioxus-app && dx serve --platform desktop`
  - `dx-build`: `cd dioxus-app && dx build --platform desktop --release`

**Verify**: `just dx-dev` でアプリ起動

## 実装しないもの（別issue/将来対応）

- 保存済みメモの閲覧機能（issue #5）
- コードブロックのシンタックスハイライト（syntect、後回し）
- モバイルフリック操作の実装（まずホバーで動作確認）
- Tauriアプリの削除（Dioxusアプリが安定してから）
- Windowsサポート（Dioxus desktopのWindows対応が成熟してから）
