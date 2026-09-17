# 設計: 便 4 — `folio check` に語彙の検査（R-9・本文の英字の語）を足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（参照は必ずつながる・正本 4 file の内部の整合）
- 条: P-5（規則は型付きデータに置く・語彙の identifiers 節が免除一覧）/ P-3.1（機械で決定的に検査できる項目は床に置く）
- rules 行: R-9（本文に出る専門語のうち語彙に定義が無いものの数 = 0・母集団 = 憲法の見出し・規範文・平易文・前文 / rules の what / 要件書の見出し・規範文・平易文・制約）と R-12（規範文と平易文に出る、日本語の言い換えを伴わない英語キーワードの数 = 0）の機械側。
- 判断の記録: ADR-3 決定 (6)（床の母集団は正本 4 file・広げない）。便 3（f2-648.13）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes・毎便問わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 e が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`・新規 dir は file を 1 本ずつ列挙）。fixture は最小の手書き（正本の写しは使わない）。

## 1. 目的と中身

便 1 の `folio check`（正本 4 file の形の床 + 参照 id の解決 + rules 行の逆参照 + 憲法の件数・3 値・終了コード 合格 0 / 違反 1 / 読めない 2）に、語彙の検査を 1 つ足す。day-1 の床 `scripts/check_draft.py` の vocab の節と同じ式にし、この便では床の script を触らない。判断の記録（adr/）の本文と凍結 anchor は母集団に入れない（便 5 以降・母集団は広げない）。

(a) 母集団 = 本文の対（場所, 文字列）。欄が無いか文字列でなければ空として数えない。
- 憲法: 各条の title（場所 = 条 id + title）・plain（条 id + plain）・各規範文の text（規範文 id）・前文 precedence の text（前文）と plain（前文 plain）。
- rules: thresholds と discipline の各行の what（行 id + what）。
- 要件書: requirements と nonfunctional の各行の title・when・shall・plain（要件書 + 行 id + 欄名）／acceptance の各行の title・plain／constraints の各行の title・text／goals の各行の title と text を空白 1 つで繋いだもの（要件書 + 行 id）。
- 語彙: terms の各語の short と def を空白 1 つで繋いだもの（語彙 + 語 id + def・定義の閉包）。

(b) 英字の語の切り出し（正規表現を使わず文字の走査で）: 文字列を先頭から見て、ASCII の英字で始まる位置から、ASCII の英字・数字・「-」・「.」が続く限り伸ばし、末尾に残った「-」と「.」を落としたものを 1 語とする（落とした後は英字か数字で終わる・英字 1 字だけでも 1 語）。語の先頭にならない位置の数字・記号・日本語は読み飛ばす。全角の英字は英字と扱わない。

(c) 既知の集合 = 語彙の terms と field_terms の各語の term と en（無ければ空）を空白 1 つで繋いだ文字列から (b) で切り出した語を小文字にしたもの + identifiers の各組の words の各要素をそのまま小文字にしたもの（要素は切り出さず丸ごと 1 語・例 claude.md や design-intent）。

(d) 免除（語ごと・(i) と (v) は元の大小文字のまま、(ii)(iii)(iv) は小文字にして見る）:
(i) id の形: 先頭が P・A・N・FR・NFR・AC・CON・GOAL・R・D のどれかで、その直後に「-」が 0 個か 1 個あり、その次の 1 文字が数字（それより後ろは問わない）／ADR- の後に 1〜9 で始まる数字列だけが続いて終わる／ADR-n・R-n・D-n そのもの／台帳 id の形（小文字の英字 1 字 + 数字 1 字 + 「-」+ 小文字の英字か数字の列 + 任意で「.」+ 数字列 で終わる）。
(ii) 小文字にした語が (c) の既知の集合にある。
(iii) 同じ本文の中で「日本語（原語）」の形の括弧の中にある: 日本語の文字（ひらがな U+3040〜U+30FF・漢字 U+4E00〜U+9FFF）の直後から、丸括弧「（」「）」を含まない並びに続く全角の「（」から「）」までで、その中身に「（」も「）」も含まないものだけを括弧の中身とする（中に「（」が入る入れ子は括弧の中身にしない＝床の式と同じ。例: 「表（gloss）」の中身は gloss・「表（a（b）」は中身なし）。中身から (b) で切り出した語の小文字の集合を作り、本文の語の小文字がこの集合にあれば免除。
(iv) 小文字にして英字 1 字。
(v) 同じ本文の中に「--」+ その語（元の大小文字のまま・小文字化しない）の並びがある（命令の旗・床の式と同じ）。

(e) 違反 = 免除されない語 1 つにつき (小文字の語, 場所) の組で 1 件（同じ組は 1 件・種別 R-9・文言は 場所 + 語彙に無い英字の語 + 語）。読めない file・型が違う節は「まだ分からない」（終了コード 2・合格にしない）。正本 4 file（main）では違反 0（day-1 の床の実測・2026-09-17）。

違反の記録型（main 8340692 の実測）: `crates/folio/src/verdict.rs` の Report の関数 violation は 種別を文字列（`&str`）で受け、文言を文字列で受ける（種別の閉じた型は無い）。既存の呼び手は `check.rs` が 未知の節・欄の非空・重複キー、`refs.rs` が 参照 id の文字列を渡すだけで、`refs.rs` に種別の一覧や網羅の分岐は無い。便 4 の `vocab.rs` は Report の関数 violation に種別 R-9 と (e) の文言を渡すだけで、`verdict.rs`・`refs.rs` は触らず、`check.rs` は `check_dir` から `vocab` を呼ぶ 1 か所だけを足す。

便 0・便 1 の fixture 7 組（`tests/fixtures/check/` の dup-key・empty-field・missing-file・unknown-section と `tests/fixtures/refs/` の bad-counts・dangling-id・orphan-rule）は rules の D-1 の what に opus を持ち、語彙の identifiers は folio だけなので、(a)〜(e) を通すと違反 R-9 が 1 つ増え、便 0 の歯 `crates/folio/tests/check.rs`（各 1 違反・終了コード 1・canonical は 0）と便 1 の歯 `crates/folio/tests/refs.rs`（各 1 違反・種別と文言を名指す）が赤になる。実測（planner と admin がそれぞれ day-1 の床の vocab の式を 7 組に当てた・2026-09-17・main 8340692）: empty-field・missing-file・unknown-section・bad-counts・dangling-id・orphan-rule の 6 組は未知語がちょうど 1 件（opus・場所 D-1 what）、dup-key は重複キーで読めないが rules の重複行を除けば同じく opus の 1 件、design-intent 本体は 0 件。よって identifiers に opus を足せば 7 組とも R-9 は 0 になり、便 0 の歯の期待（dup-key・unknown-section・empty-field = 各 1 違反・終了コード 1／missing-file = 2／canonical = 0）と便 1 の歯の期待（3 組 = 各 1 違反・種別と文言）は変わらない。便 4 はこの 7 組の `vocabulary.yaml` の identifiers の words に opus を足し（missing-file は検査に届かないが揃える）、fixture を便 4 の正本の形に追随させる。`crates/folio/tests/check.rs` と `crates/folio/tests/refs.rs` は verify で回すために write-set に在るが触らない（本文も期待も変えない）。

`folio check` 自身の歯（`crates/folio/tests/vocab.rs`・binary 経由・`tests/refs.rs` と同じ形で種別 R-9 と語を名指して 1 違反を見る）の fixture は `tests/fixtures/vocab/` の 2 組・4 file 形・`tests/fixtures/refs/dangling-id/` と同じ最小の手書き（basis の実在しない id は持たず・識別子は folio と opus）。`unknown-word/` = 憲法の P-1 の plain の末尾に語彙に無い英字の語 widget を 1 つ置く → 1（違反の文言に widget と P-1 plain）／`exemptions/` = 憲法の P-1 の plain に免除の 5 形（id の形 FR1・既知の語 folio・「型付きの表（gloss）」の形・英字 1 字の x・旗 --check）と語彙に無い語 widget を並べる → 1（違反は widget の 1 件だけ・5 形は数えない）。parity の入力にはしない。

突き合わせの歯（crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役）・便 1 の 9 入力に足す・写し全部 + git 1 commit + 変異 1 つ・床と folio の終了コードの一致）: (10) `constitution.yaml` の P-1 の plain の末尾に語彙に無い英字の語 zzzz を足す → 1／(11) `srs.yaml` の FR1 の shall の末尾に zzzz を足す → 1／(12) `rules.yaml` の R-2 の what の末尾に zzzz を足す → 1／(13) `constitution.yaml` の P-1 の plain の末尾に「型付きの表（zzzz）」を足す → 0（「日本語（原語）」の形は免除・plain は凍結 anchor の写しの欄ではないので anchor の検査に触れない）。既存の 9 入力は変えない。

実装の下書き: commit e4f8806（run f2-648.14-20260917T005039Z の branch・verify 8 段 rc 0・gate は §1 の字面の食い違いで INCONCLUSIVE）の `vocab.rs`・`tests/vocab.rs`・parity の追加・fixture は写してよい（(iii)(v) の実装は床と同じで、直したのは §1 の字面）。

便 1 が置いた形との接続: `crates/folio/src/main.rs` に `mod vocab;` を足し、実装は新規 `crates/folio/src/vocab.rs` に置き、`check.rs` の `check_dir` から `refs` の次に呼ぶ（正本 4 file の Node を渡す）。YAML は既存の `crates/folio/src/yaml.rs` の読み手を使う。3 値は既存の `crates/folio/src/verdict.rs` の型を使い、変えない。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: `crates/folio/src/vocab.rs`（(a)〜(e)）と `check.rs` からの呼び出し・歯 `vocab.rs`・parity の 4 入力・fixture 2 組・既存 fixture 7 組の identifiers に opus。
- 入れない: 判断の記録（adr/）の本文の英字語・凍結 anchor・日本語の専門語の判定（天井）・R-12 の「日本語の言い換え」の意味の検査（天井）・`scripts/check_draft.py` の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| body | 母集団 | 正本 4 file から本文の対（場所, 文字列）を集める |
| words | 切り出し | 文字列から英字の語を走査で切り出す |
| known | 既知の集合 | 語彙の term・en・identifiers から免除の語を作る |
| exempt | 免除 | id の形・括弧の原語・英字 1 字・旗 を語ごとに判定する |

## 4. 検査（歯）

§1 のとおり（vocab の歯 2 組・便 0 / 便 1 の歯そのまま・parity 13 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "e"
title = "folio check に語彙の検査（R-9・本文の英字の語）を足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/vocab.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "+crates/folio/tests/vocab.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "tests/fixtures/check/dup-key/vocabulary.yaml", "tests/fixtures/check/empty-field/vocabulary.yaml", "tests/fixtures/check/missing-file/vocabulary.yaml", "tests/fixtures/check/unknown-section/vocabulary.yaml", "tests/fixtures/refs/bad-counts/vocabulary.yaml", "tests/fixtures/refs/dangling-id/vocabulary.yaml", "tests/fixtures/refs/orphan-rule/vocabulary.yaml", "+tests/fixtures/vocab/unknown-word/constitution.yaml", "+tests/fixtures/vocab/unknown-word/rules.yaml", "+tests/fixtures/vocab/unknown-word/vocabulary.yaml", "+tests/fixtures/vocab/unknown-word/srs.yaml", "+tests/fixtures/vocab/exemptions/constitution.yaml", "+tests/fixtures/vocab/exemptions/rules.yaml", "+tests/fixtures/vocab/exemptions/vocabulary.yaml", "+tests/fixtures/vocab/exemptions/srs.yaml"]
verify = ["cargo nextest run -p folio vocab", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "vocab の歯が fixture 2 組で §1 の 1 違反を名指して緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、parity の 13 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
