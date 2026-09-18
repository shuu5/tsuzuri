# 設計: 便 27 — 判断の記録の面の見た目の直し（見本 v2）: 見出し・列挙の分割・根拠の 4 群と題・撤退条件の幅

- 要件: FR16（判断の記録の面を正本から逐語で生成する・根拠の id は行き先へ跳ぶ導線）/ NFR2（部品目録に無い class は rules 行 R-3 の値 0・見た目の材料は 1 か所から）
- 条: P-2.3（design token は 1 か所）/ P-2.4（部品は閉じた一覧）/ P-6.1（逐語で生成）/ P-3.2・P-4.2（読みやすさは天井・判定できないものは「まだ分からない」）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-7（決定 (3) 中身は逐語・採用と却下は同じ形・行き先の無い id は「まだ分からない」の印／決定 (4) 受入 = 名札の一致 + 持ち主が開いて読んで承認）。ADR-5 の撤退条件 ②（見た目だけを直す便が同じ部品について rules 行 R-7 の上限を超えたらテンプレートの案を問う）: 本便は判断の記録の面の見た目だけを直す便の 1 本目（R-7 = 2 回）。便 26（f2-648.41）の後に直列で置く。
- 裁定: 持ち主 2026-09-18 の walk の逐語「見たがすべてひどすぎて全く意味わからん。ぐちゃぐちゃだ。見直せ」（f2-648 notes・承認は不成立）。見本 v2（配信先の mock/adr-7.html・adr-3.html・adr-1.html・repo 外）の方向の裁定は本便の受付の前置（f2-648 notes に逐語で記帳してから受付を頼む）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ab が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形）。

## 1. 目的と中身

便 25・26 で生成した判断の記録の面（`adr-<数>.html`）を持ち主が開いて読み、読めないと退けた。planner が同じ面を開いて確かめた欠陥は 4 つ: (1) 表紙の h1 が文の長さの title（60〜171 字）で 4 行になる。(2) 章 01（問題）と 02（決定）が 900〜2,200 字の 1 段落で、本文の列挙 (1)〜(n) が改行されない。(3) 章 04（根拠）が id だけの縦一列（P-2・P-4・…）で何を指すか読めない。(4) 撤退条件の card が section-lead-callout の 4 列の格子の 1 列に潰れる。本便は `crates/folio/src/face_adr.rs` の組み立てを見本 v2 のとおり直し、`design-intent/preview/folio.css` に判断の記録の面の規則を足す。中身は正本の欄の逐語のまま（文を削らない・言い換えない・P-6.1）。部品は足さない（部品目録の 30 は不変・data-component の値域は変えない）。図の型は増やさない。

planner の実測（2026-09-18・main eb239cb）: 判断の記録の正本 7 本の title は 60〜171 字・ADR-3 と ADR-7 だけ「 — 」を含む。context / decision の中の列挙の印「(k) 」の並び（後述 (b) の規則で分かれる数）= ADR-3 決定 9・ADR-4 決定 7・ADR-5 問題 5 と決定 5・ADR-6 問題 4 と決定 5・ADR-7 問題 5 と決定 6。ADR-1・ADR-2 は分かれない（ADR-2 の (1)(2)(3) は読点で続く 1 文の中の列挙で、規則 (b) は文の頭の印だけを分ける）。basis の id の種類は 4 種（条 P-/A-/N-・rules 行 R-/D-・要件 FR/NFR/AC/CON/GOAL・判断の記録 ADR-）で、7 本の basis に枝番付きの id は 0。face_adr.rs は正規化 547 行（余地 953）・歯 face_adr.rs は 514 行（余地 986）。folio.css に判断の記録の面だけの class は無い。section-lead-callout の css は `--band-n` 列の格子（folio.css 237 行）で、要件書の面のゴールの章がその中に card を 4 枚並べる形（cid + ct + cd）を持つ。

(a) 表紙（doc-cover-band・face_adr.rs の関数 cover）。h1 の字を「判断の記録 <id>」にし、title は h1 の直後に p（class sub-title・新しい class・(d)）として逐語で出す。eyebrow・summary-card（やさしく言うと = plain）・cover-meta・cover-status は変えない。head の title「folio2 — 判断の記録 <id>（<状態の名札>）」も変えない。

(b) 章 01・02（関数 prose_chapter）。本文（context / decision の escape 済みの字）を次の規則で分ける。列挙の印 = 半角の開き括弧 + ASCII の数字 1 つ以上 + 半角の閉じ括弧 + 半角空白 1 つ（例「(3) 」）。印を「文の頭の印」と数えるのは、印の直前（末尾の空白を除く）が本文の先頭か句点「。」で、かつ番号が 1 から 1 ずつ増えて続くときだけ（前の印の番号 + 1 に等しい印だけ拾い、それ以外の印は本文の字のまま）。文の頭の印が 2 つ以上あれば、先頭から最初の印の手前までを p（class intro・空なら出さない）、各印から次の印の手前までを ol（class items）の li（印の字を除き前後の空白を落とす）にする。1 つ以下なら今のまま p 1 つ。文の中の列挙（読点で続く「(1) …、(2) …」）は分けない。番号が飛ぶ列挙（(1) の次が (3)）は (1) までで止まり 1 つなので分けない。

(c) 章 04（関数 basis_chapter）。basis の各 id を 4 群に振り分ける: 憲法の条（P-・A-・N-・枝番付きも）・数値の表（R-・D-）・要件書（FR・NFR・AC・CON・GOAL）・判断の記録（ADR-）。群の順はこの順・群の中は basis の順。非空の群だけを section-lead-callout（style は `--band-n:<非空の群の数>`）の中に card 1 枚ずつ出す: cid は「<群の名>（<数>）」（群の名は 憲法の条・数値の表・要件書・判断の記録）・中は ul（class basis）で li 1 つ = 根拠のリンク（今の id_link の字面・在れば a の xref・無ければ「id（まだ分からない）」）+ 題の span。題 = 条なら憲法の該当の条の title（枝番付きの規範文 id も条の title）・rules 行なら rules の行の what・要件なら要件書の該当の行の title・判断の記録なら `adr/<id>.yaml` を face.rs の関数 load で読んだ title。行き先が無ければ題の span は出さない。題は行き先の欄が読めるときだけ出す＝判断の記録は file が在っても title の欄が無い・空・読めないときは題の span を出さず、面を 2 にしない（face_adr の歯の写しの 2 行の stub ADR-1.yaml がこの形・admin の事前読み 2026-09-18）。関数 context の Ctx に id と題の組を持たせる（articles・rules・reqs を id の一覧から id と題の組の一覧に広げる・statements は今のまま）。撤退条件は callout の外に card 1 枚（class「card retreat」・cid「撤退条件（<種類の名札>）」・p に condition・今と同じ字）。

(d) folio.css に足す規則（design token だけを使い色リテラルを書かない・min-width 0・nowrap を足さない・style 属性は足さない）: `.doc-cover-band .sub-title`（副題・1.0625rem・行間 1.65・太さ 600・最大幅 60rem）／`.chapbody p.intro`（最大幅 57.5rem）／`.chapbody ol.items` と `.chapbody ol.items li`（最大幅 57.5rem・左の余白 1.6em・li の上下 8px・行間 1.75）／`ul.basis` と `ul.basis li` と `ul.basis li .xref`（一覧の点なし・li は横並び・下線 var(--line-soft)・字 .85rem・xref は幅 5.5rem の等幅）／`.chapbody>.card.retreat`（上の余白 16px・最大幅なし）。`tests/fixtures/face/folio.css`（fixture の写し）は触らない（fixture の写しの歯は byte 一致だけを見て parts の検査は実の正本の css で回す）。

(e) 凍結 fixture の再凍結: `tests/fixtures/face/expected-adr.html`（便 25・face_adr の歯の写し）と `tests/fixtures/face/expected-site-adr-2.html`（便 26・組み立ての歯の写し）を (a)〜(c) の形で凍結し直す（差分は表紙の h1 と副題・章 01/02 の段落か列挙・章 04 の群と撤退条件だけ）。fixture の ADR-2.yaml は触らない（その context / decision は列挙を持たないので p のまま・basis 6 = 憲法 2（P-1・A-1）・数値の表 1（R-1）・要件書 2（FR1・AC1）・判断の記録 1（ADR-1）→ card 4・`--band-n:4`）。

(f) 歯 `crates/folio/tests/face_adr.rs`（関数名はすべて face_adr を含める・`--test face_adr` の scope）: 既存 14 本は (e) の再凍結と (a)(c) の字面に追従（census は h1 が「判断の記録 <id>」・副題が title の逐語・ul.basis の li の数が basis の数・各 li に題の span）。次を足す。
15. 写しの context を「前置き。(1) あ。(2) い。」に = 0 ∧ 章 01 に p.intro 1 と ol.items の li 2（字は「あ。」「い。」）。
16. 写しの context を「(1) あ、(2) い。」に = 0 ∧ 章 01 は p 1 つで ol.items が無い。
17. 写しの context を「前置き。(1) あ。(3) い。」に = 0 ∧ ol.items が無い（1 つ以下）。
18. 写しの面で章 04 に card 4・`--band-n:4`・cid の字「憲法の条（2）」「数値の表（1）」「要件書（2）」「判断の記録（1）」∧ P-1 の li に fixture の憲法の P-1 の title の字 ∧ R-1 の li に rules の R-1 の what の字 ∧ 撤退条件の card が callout の外（callout の閉じの後）∧ 判断の記録の card の ADR-1 の li は xref のリンクあり・題の span なし（stub は title を持たない）。
19. 写しの basis に FR9（無い）を足す = 0 ∧ 要件書の card の li に「FR9（まだ分からない）」∧ 題の span なし。
`crates/folio/tests/site.rs`（`--test site` の scope）は write-set に在るが本文は変えない（凍結 6 本目の期待 file が (e) で変わるだけ）。unit は置かない。

(g) 便 26 までの形との接続: 変えるのは `crates/folio/src/face_adr.rs`（cover・prose_chapter・basis_chapter・context・Ctx・見積 +120 行）・`crates/folio/tests/face_adr.rs`（+80 行）・`design-intent/preview/folio.css`（+8 行）・fixture 2 本。`face.rs`・`face_index.rs`・`site.rs`・`parts.rs`・`parts.json`・`main.rs`・`build.rs`・fixture の `folio.css`・`.github/workflows/` は触らない。外部 crate は増えない。正規表現は使わない（列挙の印は byte の走査で読む）。size は S = 中身を変える既存の file 1 本あたりの増分は小さい。

## 2. 範囲

- 入れる: 表紙の見出しと副題・列挙の分割・根拠の 4 群と題・撤退条件の幅・css の規則・fixture の再凍結・歯 5 本。
- 入れない: 記録の本文の書き換え（正本の文は逐語のまま）・案の章と改訂の章と承認欄の変更・入口の棚の変更・新しい部品・図。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| cover | 表紙 | h1 = 判断の記録 <id>・副題 = title |
| split | 列挙の分割 | 文の頭の (k) を ol に |
| basis | 根拠の群 | 4 群の card と題 |
| retreat | 撤退条件 | callout の外の card |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 26 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ab"
title = "判断の記録の面の見た目の直し（見本 v2）: 見出しと副題・列挙の分割・根拠の 4 群と題・撤退条件の幅"
req = ["FR16", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face_adr.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/site.rs", "design-intent/preview/folio.css", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test face_adr face_adr", "cargo nextest run -p folio --test site site", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_adr の歯 19 本（既存 14 の追従 + 列挙の分割 3・根拠の群 1・未解決の題なし 1）が緑、site の歯（6 本の凍結の 6 本目を再凍結）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
