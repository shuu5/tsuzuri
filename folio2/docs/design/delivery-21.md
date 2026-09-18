# 設計: 便 21 — 気づかせる 1 行 `folio hello`（未整備のあいだ・止める設定は 1 つ・プロジェクト 1 つにつき 1 回）

- 要件: FR3（design-intent が未整備であるあいだ、セッション開始時に 1 行だけ案内を出す。止める設定は 1 つ、プロジェクト 1 つにつき 1 回のみ）
- 条: P-1.1（知らせるところまで・判断を代行しない）/ P-4.1（実行できなかったことを異常なしにしない）/ N-1.1（管理下の対象を消さない＝印は repo の外・repo には書かない）/ N-3.1（例外機構を足さない＝止める設定は要件が定める 1 つだけ）
- 判断の記録: ADR-6（決定 (5) 順序 = 床 → 命令 → 入口の節 → 気づかせる 1 行）。便 20（f2-648.34）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「承認するし質問も全て推奨で承認する」と ADR-6 の発効「承認する」（f2-648.31 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 v が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

AI のセッションが始まったときに 1 行だけ「設計文書がまだ無い・相談は folio intake から」と知らせる命令 `folio hello` を置く。出すのは design-intent が未整備のあいだだけ（FR3 の型 state）。うるさければ止める設定 1 つで切れ、同じプロジェクトには 1 回しか出さない。セッション開始の仕掛け（hook）にこの命令を結ぶのは各プロジェクトの設定（folio の外・本便の write-set の外）で、本便は命令と歯だけを運ぶ。folio2 自身は整備済みなので folio2 では何も出ない。

planner の実測（2026-09-18・main e505f56）: 命令の一覧は check・inject・render・parts・face・build・serve の 7 つ（`main.rs` の Command）。sha256 の手書き実装は `sha256.rs` に在る（便 7・外部 crate なし）。folio2 の design-intent は constitution.yaml を持つ（整備済み）。verify の filter 語 hello を名に含む `#[test]` の関数は base に 0 本。

(a) 命令の形: `folio hello [--dir <design-intent の置き場・既定 design-intent>] [--state <印の置き場・既定は環境変数 XDG_STATE_HOME の下の folio・無ければ HOME の下の .local/state/folio>]`。判定の順:
1. 止める設定 = `--dir` の親 dir（プロジェクトの根）の直下に `.folio-quiet` という名の file が在る → 何も出さず終了 0（唯一の止める設定・中身は読まない）。
2. 整備済み = `--dir` が dir として在り、その直下に `constitution.yaml` が在る → 何も出さず終了 0。
3. 出した印 = `--state` の下の `greeted/<プロジェクトの根の絶対 path の sha256 の 16 進>` が在る → 何も出さず終了 0（プロジェクト 1 つにつき 1 回）。
4. それ以外（未整備）= 標準出力に 1 行「folio: この project には設計文書（design-intent）がまだ無い。AI に「folio intake」と頼むと相談が始まる（止めるには .folio-quiet を置く）」を書き、印の file を作って（dir が無ければ作る・中身は空）終了 0。印を書けないときは 1 行は出したまま標準エラーに「folio hello: まだ分からない: 印を書けない: <理由>」で終了 2（次回も出る＝黙って 1 回きりにしない・P-4.1）。
- 出すのは標準出力の 1 行だけ（改行 1 つ）。判定は上の順で最初に当たったもの。`--dir` の親が無い（`--dir` が根）なら止める設定は `--dir` の直下を見る。
- repo には何も書かない（印は `--state` の下だけ・N-1）。

(b) 実装 `crates/folio/src/hello.rs`（新規・判定と 1 行・印の書き・sha256.rs を使う）。`crates/folio/src/main.rs` に `mod hello;` と Command の variant（Hello・dir・state）とその分岐。正規表現は使わない。外部 crate は足さない。

(c) 歯 `crates/folio/tests/hello.rs`（binary 経由・関数名はすべて hello を含める＝verify の filter 語・`--state` は必ず一時 dir）。歯の置き場のため既存の歯 `crates/folio/tests/vocab.rs` を write-set に載せるが本文も期待も変えない。
- 3 状態の行数（FR3 の確かめ方）: 未整備（design-intent が無い一時 dir）= 標準出力 1 行 ∧ 終了 0 ∧ 印が出来る／整備済み（constitution.yaml だけ置く）= 0 行／止める設定（`.folio-quiet` を根に置く）= 0 行（未整備でも）。
- 1 回きり: 未整備で 2 回撃つ → 1 回目 1 行・2 回目 0 行・印は 1 つ。別の根（別の一時 dir）では改めて 1 行。
- 印を書けない: `--state` を file にして撃つ → 1 行は出る ∧ 終了 2 ∧ 標準エラーに「印を書けない」。
- 1 行の中身: 「folio intake」と「.folio-quiet」を含む。
- unit（`src/hello.rs` の中・名に hello を含む）: 判定の順（止める設定 → 整備済み → 印 → 出す）・印の file 名（根の絶対 path の sha256）。

(d) 便 20 までの形との接続: 新規は `crates/folio/src/hello.rs`・歯 `crates/folio/tests/hello.rs`。`crates/folio/src/main.rs` は `mod hello;` と variant と分岐だけ。`sha256.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`design-intent/`・`.github/workflows/` は触らない。語彙の識別子への命令名の追加は正本を触るので planner の別 PR（本便の外）。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`main.rs` の増分は variant と分岐で 40 行未満）。

## 2. 範囲

- 入れる: 命令 `folio hello`（3 状態の判定・1 行・止める設定・1 回きりの印）・歯。
- 入れない: セッション開始の仕掛け（hook）への結線（各プロジェクトの設定）・整備済みの判定の拡張（constitution.yaml の有無だけ）・語彙への命令名の追加（planner の別 PR）・印の掃除の命令。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| judge | 判定 | 止める設定 → 整備済み → 印 → 出す の順 |
| line | 1 行 | 固定の案内文を標準出力へ |
| mark | 印 | 根の絶対 path の sha256 の file を state の下に |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 20 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "v"
title = "気づかせる 1 行 folio hello（未整備のあいだ・止める設定 1 つ・プロジェクト 1 つにつき 1 回）"
req = ["FR3"]
section = "1"
write-set = ["+crates/folio/src/hello.rs", "crates/folio/src/main.rs", "+crates/folio/tests/hello.rs", "crates/folio/tests/vocab.rs"]
verify = ["cargo nextest run -p folio --test hello hello", "cargo nextest run -p folio --test vocab vocab", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "hello の歯（3 状態の行数・1 回きり・印を書けない・1 行の中身・unit）が緑、vocab の歯が期待不変で緑（置き場として write-set に在るだけ）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
