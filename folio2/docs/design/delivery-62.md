# 設計: 便 62 — 命令 folio render を退役する（暫定の読み物 readable.html の退役の締め・NFR2・持ち主の裁定 2026-09-21「退役して」）

- 要件: NFR2（生成物の揃い・床の掛からないページ 0 枚）/ FR4（3 面の単一の生成器）
- 条: N-1.2（退役は可逆な移動としてのみ行う）/ P-6.3 / P-2.1
- 出所: 持ち主の裁定 2026-09-21 08:47 JST（f2-648 notes・逐語「退役して」・天井の 11 周目の報告の問 5・A-1）。器（scribe2）の binary が 2026-09-21 に b3f5f01 世代へ入れ替わり、要件面を要件書の正本（design-intent/srs.yaml）から直に読めるようになった（器の台帳 s2-07l.467）。席の PR で読み物 design-intent/preview/readable.html は design-intent/preview/retired/ へ、読み物の歯 crates/folio/tests/render.rs は crates/folio/retired/render-tests.rs へ可逆に移し、.vessel.toml の requirements を要件書の正本に向けた。残るのは命令 folio render の口（crates/folio/src/main.rs の Command::Render と mod render）。
- 根拠の判断: 判断の記録 ADR-7 の案 c の理由（読み物を面として残さない）と要件書 NFR2 の注（器の入れ替えの後に読み物と folio render を可逆に退役して 0 にする）。新しい判断は無い。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bk が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 命令の口を外す。crates/folio/src/main.rs（正規化 544 行）から、列挙 Command の Render の変種（説明の doc comment・ArgGroup・旗 dir / out / write / check）と、その分岐（render::Mode の選択・render::run の呼び出し・stdout / stderr の出力・終了コード）と、module の宣言 mod render を外す。ほかの命令の分岐・旗・説明は 1 字も変えない。folio --help の一覧から render が消える。

(b) render.rs は触らない。mod render を外すと crates/folio/src/render.rs は compile の対象から外れる（cargo は参照されない file を読まない）。file 自体の可逆な移動（crates/folio/retired/render.rs へ）は席の後続の PR で行う（便は file を消せず動かせない・N-1.2 の移動は席の手番）。render.rs の中の unit test（render_escape_ 等の 4 本）は compile の対象から外れるので nextest の総数は 4 本減る。

(c) 歯。crates/folio/tests/check.rs（正規化 約 530 行・関数名は retired_render_ で始める・今この語で始まる歯は無い）に 2 本足す。binary の呼び方は同じ file の既存の歯と同じ（env! の CARGO_BIN_EXE_folio）。
1. retired_render_subcommand_is_unrecognized: folio render --check を撃つと、終了コードが 0 でも 1 でもなく（clap の使い方の誤りの終了コード 2）、stderr に render の字を含む使い方の誤りが出る（unrecognized subcommand の旨・字面は clap の既定のまま固定しない = 終了コードと render の字だけを見る）。
2. retired_render_help_does_not_list_render: folio --help の標準出力に、行頭の命令の一覧として render が出ない（行を「  render」で始まる行として探し 0 件。check / inject / build 等の他の命令は出ている = 一覧が壊れていないことを 1 つ〔build〕で確かめる）。
3. 回帰（期待不変・verify の 2 行目）: tests/check.rs の既存の歯すべて（filter check_ と r11_ を含む file 全体 = --test check）。

(d) 大きさと接続。新規 file は無い。main.rs（約 -30 行）・tests/check.rs（約 +30 行）。size S。render.rs・face.rs・site.rs・design-intent・.vessel.toml・CI は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: main.rs から Render の口と mod render を外す・歯 2 本。
- 入れない: render.rs の移動（席の PR）・読み物の復活の口・要件書の注（席の PR で済み）・面の生成器の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cmd | 口 | main.rs の Command::Render と分岐と mod render を外す |
| teeth | 歯 | tests/check.rs に retired_render_ の 2 本 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bk"
title = "命令 folio render の口（main.rs の Command::Render の変種・分岐・mod render）を外して退役する（render.rs は compile の対象外になり席の後続の PR で可逆に移す・読み物 readable.html の退役の締め・NFR2・持ち主の裁定 2026-09-21「退役して」）"
req = ["NFR2", "FR4"]
section = "1"
write-set = ["crates/folio/src/main.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test check retired_render_", "cargo nextest run -p folio --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "retired_render_ の歯 2 本（folio render が使い方の誤りで終了 2・folio --help の一覧に render が無く build は在る）が緑、tests/check.rs の既存の歯が全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
