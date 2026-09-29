# 設計: 便 35 — 要件書の面の章 09 の図番号を自前の図 1〜3 の続きにし、face_srs.rs を割って余地を作る（FR4 / FR15 / NFR2・ADR-5 決定 (3)）

- 要件: FR4（3 枚を 1 つの生成器で出す・要件書の面）/ FR15（図は型付き記述から生成し、通らない図は出さない・章 09 の図）/ NFR2（部品目録に無い class は 0・出力の部品は変えない）
- 条: P-2.1（生成器は 1 つ）/ P-6.1（人が読むページは正本から逐語で生成）/ P-4.1（導出できなければ書かない・据え置き）
- 判断の記録: ADR-5 決定 (3)（要件書の面は手書きの生成器）・ADR-5 撤退条件 ②（見た目だけの直しの数え・要件書の面は本便が 1 本目・上限 R-7 = 2）。便 34（f2-648.49・章 09）の後に直列で置く。
- 裁定: 持ち主 2026-09-19「図はこれでよい。」（実の判断の記録と要件書の図の対の walk 承認・f2-648 notes）。番号の重なりは planner が walk の実測で見つけた点で、図の中身は変えない。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 aj が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規 1 file・縮む 1 file）。

## 1. 目的と中身

要件書の面（folio face --face srs）は、folio が自前で描く帯の型の図（図 1 = 誰が使い何が出るか・図 2 = 7 段の rail・図 3 = 検査の答え〔verdicts の節が在るときだけ〕）と、便 34 で足した章 09「図」（図の道具で描く図）の 2 種の図を持つ。章 09 の図の番号は 1 から数えるので、実の要件書（図の節 2 枚・main 733fe34）では「図 1」「図 2」が同じ面に 2 度出る（fig-title の fn と figcaption の ver）。本便は章 09 の番号を自前の図の続き（自前の図の数 + 1 から）にする。自前の図の数は 2 + 〔verdicts の節が在れば 1〕で、実の要件書と凍結 fixture（tests/fixtures/face/srs.yaml・verdicts 3 行）はどちらも 3 なので、章 09 は 4 から数える。

planner の実測（2026-09-19・main 733fe34）: crates/folio/src/face_srs.rs は正規化 1,494 行で余地 6。器の受付は src の余地（1,500 − 正規化行数）を size の見積（S = 100）と比べるので、face_srs.rs を中身を変える既存 file として載せると受付が止まる（便 34 の事前読みと同じ機構）。よって本便は face_srs.rs を縮む file（write-set で - の形）として宣言し、章 07（要件と根拠の対応・関数 rtm_chapter・行 1151〜1241）と章 08（用語集・関数 glossary_chapter・行 1242〜1266）の生成を新しい file crates/folio/src/face_srs_rtm.rs へ移す（この 2 関数で正規化 118 行）。番号のずらしは章 09 の関数 figures_chapter（行 1267〜1291）の figure_panel の呼び出しの引数 i + 1 を 自前の図の数 + i + 1 にする 3 行の変更（自前の図の数は Ctx の verdicts の有無から数える定数の関数）。差し引きで face_srs.rs は 110 行以上縮む（縮んだ後の余地は 120 以上）。crates/folio/src/main.rs（正規化 434・余地 1,066）に mod 行を 1 つ足す。共有の図の枠（crates/folio/src/face.rs の figure_panel・便 34）と部品目録は触らない。tests/site.rs（正規化 476）は srs.html の期待に同じ凍結 file tests/fixtures/face/expected-srs.html を使う（行 148）ので、本文不変のまま再凍結に追従する。

(a) 分割（crates/folio/src/face_srs.rs → crates/folio/src/face_srs_rtm.rs・新規）。rtm_chapter と glossary_chapter の 2 関数を新しい file へ移し、face_srs.rs の derive からは移した先を呼ぶ。2 関数が使う Ctx・Item・band・xref・article_link・chapter_h2・hint 等の可視性は必要な分だけ pub(crate) にし、字面（出力の HTML）は 1 byte も変えない。他の関数は動かさない。新しい file の先頭の注釈は face_srs.rs の注釈と同じ書き方で「章 07・08 の生成（便 35 で face_srs.rs から分けた・出力は不変）」と書く。

(b) 番号（crates/folio/src/face_srs.rs の figures_chapter）。図の番号の起点 = 自前の図の数 + 1。自前の図の数 = 2 + 〔ctx.verdicts が Some なら 1〕（図 1 は scope_chapter が、図 2 は fr_chapter が、図 3 は verdicts の節が在るときだけ fr_chapter が出す・行 719・826・891 の実装の数と同じ）。章 09 の帯の h2「図 <数> 枚」・toc の 09 の t「<数> 枚」・foot の figures の数は章 09 の図の数のままで変えない（数えるのは章 09 の図だけ）。figcaption の ver「図 <番号> · <型の名札> · <図の id> · 根拠: …」の番号も同じ起点に揃う（共有の figure_panel が i から組む・便 34 の (b)）。

(c) 凍結 fixture（tests/fixtures/face/expected-srs.html）。(b) の形で再凍結する。差分は章 09 の fig-title の fn「図 1」→「図 4」と figcaption の ver「図 1 ·」→「図 4 ·」の 2 か所だけ（他は byte 不変・章 09 の h2「図 1 枚」と toc の「1 枚」と foot の figures 1 は不変）。tests/fixtures/face/srs.yaml は変えない。

(d) 歯。crates/folio/tests/face.rs（関数名は face を含める・--test face の scope・正規化 1,077・余地 423）: 既存の章 09 の歯（行 928〜975・凍結 file を読む）の fig-title の期待を「図 4」に、figcaption の期待を「図 4 · 構成図（architecture） · fig-1 · 根拠: …」に変える。足す歯: 写し（tests/fixtures/face/srs.yaml の写し）の verdicts の節を消した面で章 09 の fig-title の fn が「図 3」（自前の図が 2 枚のとき 3 から）∧ figure-panel の数が 2 + 1。実の正本（design-intent/srs.yaml・図の節 2 枚・verdicts あり）の面で fig-title の fn「図 4」と「図 5」が 1 つずつ ∧「図 1」の fig-title は自前の図の 1 つだけ（fn の span の字面「図 1」は面に 1 つ）。図の節を消した写しの面が便 34 までの形と byte で同じ（既存の歯・不変）。crates/folio/tests/site.rs（関数名は site を含める・--test site の scope）: 本文は変えず、再凍結した expected-srs.html に追従して緑。

(e) 便 34 までの形との接続: 新規 file = crates/folio/src/face_srs_rtm.rs（1 本）。crates/folio/src/face_srs.rs は (a)(b) で縮む（-110 以上・write-set では - の形）。crates/folio/src/main.rs は mod 行 +1（余地 1,066）。tests/fixtures/face/expected-srs.html は (c)。crates/folio/tests/face.rs は (d)（+30・余地 423）。crates/folio/tests/site.rs は本文不変（verify の scope）。判断の記録の面・設計ノートの面・入口の面・憲法の面は触らず、それらの凍結 expected-*.html も不変。readable.html（render）と inject の導出物は要件書の節の字面を読むので不変。

## 2. 範囲

- 入れる: 章 09 の番号の起点・face_srs.rs の分割（章 07・08 を新しい file へ）・凍結 fixture の再凍結（2 か所）・歯の追従と追加。
- 入れない: 図 1〜3（帯の型）の道具化・要件書の版上げ（要件は変わらない）・章 09 の見た目の他の直し・他の面。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| split | 分割 | rtm_chapter・glossary_chapter を face_srs_rtm.rs へ |
| number | 番号 | 章 09 の起点 = 自前の図の数 + 1 |
| freeze | 再凍結 | expected-srs.html の 2 か所 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。図の道具の写しと Node.js は受け皿（便 30）のまま。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "aj"
title = "要件書の面の章 09 の図番号を自前の図 1〜3 の続き（自前の図の数 + 1）から数え、章 07・08 の生成を新しい file face_srs_rtm.rs へ移して face_srs.rs を縮める（出力は番号の 2 か所だけ変わる）"
req = ["FR4", "FR15", "NFR2"]
section = "1"
write-set = ["-crates/folio/src/face_srs.rs", "+crates/folio/src/face_srs_rtm.rs", "crates/folio/src/main.rs", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --test face face", "cargo nextest run -p folio --test site site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（章 09 の fn と ver が図 4・verdicts 無しの写しで図 3・実の正本で図 4 と図 5 が 1 つずつ・図なしの写しは不変）が緑、site の歯が再凍結に追従して本文不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
