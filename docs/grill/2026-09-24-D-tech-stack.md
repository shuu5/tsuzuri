# grill 論点 D — 技術スタック（討論の記録・正本は design-intent/ の YAML）

- 出自: docs/handoff/2026-09-24-v3-kickoff.md §1（持ち主の要望・2026-09-24）・§2 論点 D（持ち主が最重要と名指し）
- 状態: 討論中（裁定なし）。裁定が出たら ADR へ書き、この file は経緯だけを残す
- 逐語は台帳へ（この file には写さない）

## 1. 確認した事実（verified・2026-09-24）

| 事実 | 出所 |
|---|---|
| scribe2 core crate の runtime 依存は 0 本（dev = insta / proptest のみ） | scribe2/crates/scribe2/Cargo.toml |
| scribe2 憲法 C12 = test は Rust の 1 framework・bash/bats 全廃。C13 = 依存は許可リスト + 予算（R-C13-1: 直接依存 12 本以下・1 PR 1 本）。C13.3 = core で async を使わない | scribe2/design-intent/spec/constitution.html |
| folio2 の runtime 依存 = clap + yaml-rust2 の 2 本。`folio serve` は std::net の手書き HTTP（tokio / axum 無し） | folio2/Cargo.toml・crates/folio/src/serve.rs |
| folio2 の面は生成 HTML + 手書き folio-ui.js（build 無し） | crates/folio/src/face.rs |
| folio2 は既に Node.js 18+ を図の道具 archify のために持つ。vendor/archify/ に写し 43 file・sha256 で凍結・ADR-4 と A-3.1 の承認 | folio2/design-intent/rules.yaml R-15 |
| host に node / npm / bun / cargo が在る。pnpm はコンテナ側（ホストで pnpm install 禁止 = 共通設定） | which・~/.claude 相当の共通 CLAUDE.md |
| s3 の design-intent は違反 0・まだ分からない 2（凍結 anchor と id baseline） | `folio check --dir design-intent` |

## 2. 要件の読み（持ち主の要望 3 点を面の要件に翻訳・deduced）

1. **裁定面（GUI）**: 口座・pipeline の一覧と操作、project ごとの台帳（beads）の提示と持ち主とのすり合わせ、裁定の受付 → 裁定 id の機械発行と台帳への直接記帳。
2. **表示面（session が制御する画面）**: menu 無しの browser 面。session が「この URL を出せ・viewport をこう・reload・screenshot」を命令。remote（tmux・tailnet）でも local でも同じ。スマホの模擬（viewport / DPR / UA）→ 必要なら本物の emulator 配信。
3. 共通: 1 つの表示 server + 2 種の client（持ち主の browser・session の headless の目）。app ごとに表示の仕組みを作らない。

## 3. 候補と評価

| 案 | 中身 | 利点 | 代償 |
|---|---|---|---|
| (a) Rust server + build 無し HTML/JS | 同期 HTTP（std::net か tiny_http 級）+ SSE/WebSocket + 手書き JS（必要なら vendor + sha256 で 1〜2 本の小さな lib） | toolchain 1 本・依存 0〜2 本・C12/C13 と整合・folio2 の面と同じ作り・R-15 の先例で vendor 化の門が既に在る | UI の表現力に上限（複雑な編集 UI・仮想スクロール・図の対話は手書きが重い）。型は Rust 側の契約（JSON schema 生成）で担保し、JS 側は薄く保つ規律が要る |
| (b) Rust backend + TypeScript frontend（Vite + Svelte/React） | SPA を build して Rust server が配る | 表現力・生態系・型 | Node/pnpm の toolchain・npm の supply chain（cargo deny 相当の門を別に持つ）・C12「1 framework」と C13 予算の改憲が要る・host/container 規律（pnpm はコンテナ）で開発の往復が増える |
| (c) Tauri | Rust + webview の desktop 殻 | 軽い・Rust と相性良 | remote（tmux 越し）の要件を満たさない。(a)/(b) の面を後から殻に入れる任意の追加としてのみ意味がある |
| (d) Electron | Node runtime の二重持ち | — | remote 不可・依存が最大。不採用 |
| (e) Rust→WASM（Leptos / Dioxus / Yew） | 全 Rust の SPA | 言語 1 本 | compile 予算（C13）に最も重い・生態系が薄い・表示面（= iframe の器）には過剰 |

## 4. s3 席の推奨（提案・裁定は持ち主）

**(a)** を採る。core は Rust のまま（裁定 0.1 で確定済み）。面は Rust の同期 server + build 無し HTML/JS。JS の lib が要るときは folio2 R-15 と同じ門（vendor + sha256 + ADR + A-3 相当の承認）で 1 本ずつ入れる。TypeScript は runtime にも build にも入れない（型は Rust 側の契約から JSON schema を生成して JS が守る形。JSDoc + tsc --noEmit の検出線は後の判断）。Tauri は「後から任意の殻」として射程外に置く。Electron は不採用。

根拠: 要件 3 点はどれも「一覧・表・フォーム・iframe と命令の中継」であり、SPA の表現力を要しない。(b) は改憲と別の supply chain 門を同時に持ち込み、v3 = 合流の世代（書き直しでない）という裁定 0.1 と釣り合わない。

撤退条件（案）: 裁定面の 1 画面（台帳のすり合わせ）を (a) で spike して、手書き JS が閾値（行数か画面数・rules 行で凍結）を超えたら (b) を ADR で再審議。

## 5. 持ち主への問い（1 問）

面を (a) で進めてよいか。前提 = 面に求める複雑さが §2 の 3 点（一覧・裁定の入力・表示面の制御）の範囲に収まる。この前提が違う（自由な編集・図の対話・リッチな差分表示が要る）なら推奨は (b) に変わる。

## 6. 経緯

- 2026-09-24: s3 席が事実を確認し、§3〜§5 を持ち主へ提示（答え待ち）

## 7. 論点 A への持ち越し（2026-09-24・s3 席の実測）

- scribe3 は `.vessel` marker と `.vessel.toml` を欠いたまま席が立っていた（state dir・git 設定・登録 row は手作業で再現済み）。marker が無いので plugin の SessionStart hook が黙り、orchestrator の指示文（決定はしご = scribe2 C17 の行を含む）が席に入っていなかった。doctor はこの欠落を出さない。
- 根: 器に「新しい置き場を載せる口」が無く、立ち上げは runbook（管理席の cache）と手作業に頼っている（consumer-sync.md は N2 で散文の手順を持たないと掲げる）。
- 持ち越し: 論点 A で「器の口として init を 1 本持つ（marker・宣言・state dir・登録を 1 発で・doctor が欠落を名指す）」を要件候補に挙げる。
- 直しは scribe2 席へ Claude Code の session 間メッセージで依頼（2026-09-24・持ち主の指示「討論の前に修正させる」）。
