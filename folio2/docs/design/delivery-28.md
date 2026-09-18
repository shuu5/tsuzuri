# 設計: 便 28 — 設計ノートの面を正本から `folio face --face note --id <文書 id>` で生成する（5 面目・見本 v1・要件書 FR9 / AC7）

- 要件: FR9（設計ノートを 1 つの型で生成し、節は閉じた一覧の型だけ・一覧に無い型の節を持つ正本は生成せずに落とす）/ FR10（契約表の欄は器の導出 file から読む）/ NFR2（部品目録に無い class は rules 行 R-3 の値 0）
- 条: P-2.1・P-2.4（人が読むページは 1 つの生成器から・部品は閉じた一覧）/ P-5.3（文書の種類の違いは節の型のデータで表す）/ P-6.1・P-6.3・P-6.4（逐語で生成・欄の一覧を二重に持たない）/ P-4.2（判定できないものは「まだ分からない」）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-3（決定 (1) 設計ノートは 1 つの型・節の型は閉じた一覧／決定 (2) 契約表の欄の正本は器の導出 file）。ADR-5 決定 (3)（手書き・外部の部品を足さない）。ADR-5 / ADR-7 の撤退条件 ①（面の型が増える便）は持ち主が 2026-09-18「推奨でよい」= 手書きのまま と裁定（f2-648 notes・数 = 面の型 5・見た目だけの便 1）。便 27（f2-648.42）の後に直列で置く。入口の棚の導線と build の出力への追加は便 29（別の行）。
- 裁定: 持ち主 2026-09-18「推奨でよい」（手書きのまま）。見本 v1（配信先 mock/note-example.html・repo 外）の方向の裁定は本便の受付の前置（f2-648 notes に逐語で記帳してから受付を頼む）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 ac が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

設計ノートの正本（`design-intent/design-note/<文書 id>.yaml`・欄の決まりは同じ dir の `schema.yaml`・便 23 で床に入った）1 本から、人が読むページ 1 枚（`note-<文書 id>.html`）を便 25 の判断の記録の面と同じ規律で生成する。生成物の文字列は (α) 正本の欄の値を escape して逐語で差し込んだもの／(β) 生成器が閉じた表として持つ名札／(γ) 正本から数えた数 の 3 種だけ。設計ノートだけの決まり: 節の型（閉じた一覧 6 つ）ごとに見せ方を変え、一覧に無い型の節が 1 つでもあれば面を導出せず 2（FR9・AC7 の形）。契約表の節の行は、欄の一覧を生成器に持たず、器（scribe2）の導出 file `contracts/schema.toml`（正本の置き場の親 dir・便 23 の note.rs と同じ置き場）の field の順に出す（FR10・P-6.4）。本便は面 1 枚の生成器・部品目録の faces の追加・凍結 fixture・歯だけで、入口の棚の導線と build / serve の出力への追加（便 29）・図の生成（FR15・別の便）・css の変更（無し）は含まない。

planner の実測（2026-09-18・main 96346bb）: 設計ノートの正本は `design-intent/design-note/example.yaml` の 1 本（status example・節 6 = prose・parts-table・ports-table・fields-table・teeth-table・contract-table が 1 つずつ・図 1・sources 空）。欄の決まりの節の型は 6（schema.section.type_enum）で、型ごとの行の欄は schema.section.by_type（parts = id・name・role + ref・note／ports = id・name・input・output・refuses + ref・note・断らない口の印は「なし」／fields = id・name・need・shape + enum・note・need の値域 required / optional・shape の値域 text / list / number / bool / table／teeth = id・name・red_when・fixture + ref・note／contract = 器の導出 file の field）。文書の状態の値域は draft / effective / retired / example。器の導出 file の field は 14（id・title・req・section・touches・surfaces・write-set・creates・tests・also・verify・size・done・depends・need と shape 付き）。face_adr.rs は 692 行（正規化・余地 808）・face.rs は 880（余地 620）・parts.rs は 548・main.rs は 388・tests/parts.rs は 353・note.rs は 1,349（余地 151・本便は触らない）。部品目録の部品は 30 で、便 25 で adr の面に足した 7 部品（freshness-stamp・font-size-control・doc-cover-band・approval-block・chapter-deck-band・section-lead-callout・item-row）が本便の面でも足りる。folio.css の便 27 の規則（sub-title・intro・items・basis・retreat）は設計ノートの面でも使う（css は変えない）。

(a) 命令の口。`crates/folio/src/main.rs` の Face の doc に note を足し、`--id` の doc を「面 adr と note に付く」に。`crates/folio/src/face.rs` の関数 run は面の名 note を受け（「どれでもない」の文言を index・constitution・srs・adr・note の 5 つの列挙に）、note でも id が要る（無ければ 2・「--id が無い」）、adr と note 以外に id が付いていれば 2（文言を「--id は面 adr と note にだけ付く」に）。面 note の生成器は `crates/folio/src/face_note.rs` の関数 derive（引数は dir と id）。`--out` の扱いと --write / --check の 3 値は既存の面と同じ。

(b) 正本の読み。face_note.rs は face.rs の関数 load で `design-note/<id>.yaml` を読む（id は欄の決まりの id_pattern（英小文字で始まり英小文字・数字・ハイフン）でなければ 2・file が無ければ 2・file の meta.id と `--id` が違えば 2）。同じ dir の `constitution.yaml`・`rules.yaml`・`srs.yaml` も load で読み、参照 id（ref・req・refs）の行き先の解決に使う（便 25 の resolve と同じ 4 形 + 契約 id）。契約表の節が 1 つでもあれば、正本の置き場の親 dir の `contracts/schema.toml` を行走査で読む（便 23 の note.rs の読み方と同じ形 = 先頭が schema = 1・[[field]] の区間・name / need / shape の引用符付きの組・読めなければ 2・「器の導出 file が読めない」）。face_note.rs は note.rs の非公開の関数（load_external・quoted_pair）を呼ばず、同じ字面の読みを自前に持つ（note.rs は触らない・余地 151）。読む欄は欄の決まりの doc（meta・sections + figures・sources）・doc_meta（id・title・version・status・generated・profile + approval・supersedes・superseded_by・note）・section（n・type・title + body・rows・note）。無い欄・型違いは X の文言つきの Err = 2。本便は欄の決まりの規則（n の昇順・型ごとの行の欄の過不足・need / shape の値域・承認欄の要否）を数えない（床の領分・二重に持たない P-6.3）＝面は読んだ値をそのまま出し、名札の表に無い値だけ 2 にする。

(c) 名札の表（β・表に無い値は 2）:
- 節の型 → 章の名札: prose →「説明」・parts-table →「部品」・ports-table →「口」・fields-table →「欄」・teeth-table →「検査」・contract-table →「契約表」。一覧に無い型の節が 1 つでもあれば 2（「節の型「<型>」は閉じた一覧に無い」・FR9）。
- 文書の状態 → 名札と状態の行: example →「見本」／「見本・拘束力なし」・draft →「下書き」／「未承認・拘束力なし → 持ち主の承認で発効」・effective →「発効」／「発効・拘束力あり（承認 <approval.date>）」（approval が無ければ 2）・retired →「廃止」／「廃止 → 後継 <superseded_by へのリンク note-<id>.html>」（superseded_by が無ければ 2・後継のリンクは実在を確かめず `note-<superseded_by>.html` にする＝後継の実在は床が数える・便 25 の判断の記録の面が id_link で実在を見るのとは違う点）。
- 欄の要否（need）→ pill: required →「必須」・optional →「任意」。欄の形（shape）→ pill: text →「文字」・list →「一覧」・number →「数」・bool →「真偽」・table →「表」。
- 参照 id の行き先: 便 25 の 4 形（条・rules 行・要件・判断の記録）+ 契約 id `<文書 id>#<行 id>`（同じ文書の契約表の行なら `#<節の anchor>-<行 id>`・他の文書なら「（まだ分からない）」）。在れば a（class xref）、無ければ id の直後に「（まだ分からない）」。
- 章の帯: 章 i（1 から）は band-<((i − 1) mod 6) + 1>・kicker の絵記号は要件書の面の BANDS の 6 つを同じ順で回す。

(d) 面の組み立て（face_note.rs）。骨格は判断の記録の面と同じ順で、face.rs の共有の口（Frame の関数 head・toc・band・approval_band・foot・dc と関数 card・hint・esc・anchor）を使う。Frame は const でなく関数 derive の中で組む（章の数が正本ごとに違うため）: name「設計ノート」・source「design-note/<id>.yaml」の字面そのまま「design-note/<文書 id>.yaml」（静的な字・foot の 1 行）・favicon は判断の記録の面の FRAME の favicon の字面・current は NAV の要素数（3・どの nav にも aria-current を付けない）・first 1・bands は 12 本の静的な表（band-1〜6 を 2 回）の先頭から章の数だけの切り出し（節の数 + 図の章の有無・13 章以上は 2「章が多すぎる（上限 12）」）・prev は（srs.html・要件書）・next は（index.html・入口）・parts は 7 部品。
- head: title「folio2 — 設計ノート <id>（<状態の名札>）」・generated は meta.generated・version は「<id> <version>」・status は状態の名札。here の行は Frame が出す「設計ノート ▸ 全 <章の数 + 1> 章 — 目次へ」。
- cover（doc-cover-band・便 27 の判断の記録の面の関数 cover と同じ字面の組）: eyebrow「設計ノート (DESIGN NOTE)」+「folio2 — <id>」・h1「設計ノート <id>」・p（class sub-title）に title の逐語・meta.note が在れば summary-card（ic「注」・lab「注」・txt は note）・cover-meta は 状態（名札）・版「<version> / <generated>」・節「<数> 節（<型の名札> <数>・…・0 の型は出さない・型の一覧の順）」・図「<数> 枚」・契約表「<数> 行」（contract-table の節の行の合計・節が無ければ 0）・cover-status は (c) の状態の行 +「（承認欄へ）」。
- toc: 章 i ごとに n は「<i の 2 桁>」・k は「§<節の n> <節の title>」・t は型の名札。図の章（figures が 1 つ以上）は k「図」・t「<数> 枚」。最後に承認欄。
- 節の章: 帯の h2 は「§<n> <title>」・kicker は型の名札・chapbody の中身は型ごとに次のとおり。節の note が在れば chapbody の先頭に p（class intro）。
  - prose: body を空行で段落に分け、段落の中の改行は空白 1 つに（Markdown の段落と同じ）。各段落に便 27 の列挙の分割（文の頭の (k) が 1 から続くとき ol.items）を掛ける（規則は delivery-27.md (b) と同じ字面）。
  - parts-table: 行ごとに item-row（判断の記録の面の案の item-row と同じ属性の組・id は `s<i>-<行 id>`・rid は 行 id・rt は name・norm は role・meta-chips は 根拠の hint（ref の各 id のリンクを「・」で）と 注の hint（note）・無い欄の chip は出さない）。
  - ports-table: item-row・norm は span（class ew）の札「入力」の後に input、同じ札「出力」の後に output（要件書の面の when の札と同じ字面）・plain（pk「断る」）は refuses（値が「なし」なら「断らない」）・meta-chips は根拠と注。
  - fields-table: item-row・badges は need の pill と shape の pill・norm は enum が在れば「値域: <各値を「・」で>」（無ければ norm を出さない）・meta-chips は注。
  - teeth-table: item-row・norm は red_when・plain（pk「固定の材料」）は fixture を code で・meta-chips は根拠と注。
  - contract-table: item-row・rid は 行 id・rt は title・badges は size の pill・norm は done・plain（pk「節」）は section の値を「§<n>」の字で同じ文書の該当の節の章へのリンク（節が無ければ「§<n>（まだ分からない）」）・meta-chips は 要件の hint（req の各 id のリンク）・検証の hint（verify の各値を code・改行で）・それ以外の欄は器の導出 file の field の順に、行に在る欄だけを field の name をそのまま label にした hint（一覧は各値を code・「・」で・文字はそのまま）。値が空の一覧の欄は hint を出さない（実の example.yaml の行 a の depends = [] がこの形・admin の事前読み 2026-09-18）。行の欄のうち id・title・req・section・verify・size・done は固定の置き場、器の導出 file に無い欄が行に在れば 2（「契約表の欄「<欄>」は器の導出 file に無い」）。
- 図の章（figures が 1 つ以上のとき、節の章の後）: 帯の h2「図 <数> 枚」・kicker「図」・chapbody に section-lead-callout（style は `--band-n:2`）の中に図ごとに card（class「card accent warn」・cid「<id> · <type>」・ct は caption・cd は「図の生成はまだ無い（要件書 FR15 の便で足す）＝まだ分からない。根拠: <refs の各 id のリンク>」・refs が無ければ「根拠:」以降を出さない）。spec の中身は出さない（図の生成は FR15 の便）。
- 承認欄: Frame の approval_band（h2「承認」・lead は状態の名札）→ chapbody → approval-block。approval が在れば sign 1 つ（role「承認」・who・when は date・「逐語「<verbatim>」」・stamp は ruling）。無ければ approval-block の中に p: example →「見本（拘束力なし）は承認欄を持たない。」・それ以外 →「未（持ち主の逐語と日付が入ると発効）」。
- foot: Frame の foot（version は「<id> <version>」・generated は meta.generated・機械のための面の dl は id / status / version / profile / sections（節の数）/ figures（図の数））。字下げ無し・部品ごとに改行 1 つ・末尾に改行 1 つ・escape と id の規則は便 14 と同じ。

(e) 部品目録。`design-intent/preview/parts.json` の 7 部品（freshness-stamp・font-size-control・doc-cover-band・approval-block・chapter-deck-band・section-lead-callout・item-row）の faces に note を足す。凍結目録 `tests/fixtures/floor/parts-catalog.json` の同じ 7 部品の faces にも note を足す（それ以外の byte は変えない・便 25 と同じ）。部品は足さない（30 のまま）。`crates/folio/src/parts.rs` の FACES を 5 つ（index・constitution・srs・adr・note）にし、「どれでもない」の文言を 5 つの列挙に。build.rs は触らない。

(f) 凍結 fixture（最小の手書き）: `tests/fixtures/face/design-note/full.yaml`（status draft・approval 無し・節 6 = 各型 1 つずつ（prose は段落 2 つで 2 つ目が「前置き。(1) あ。(2) い。」・parts 2 行（1 行は ref = [P-1, FR1]）・ports 2 行（1 行は refuses なし）・fields 2 行（1 行は enum あり）・teeth 1 行・contract 1 行（id a・req [FR1]・section 1・write-set 1 つ・verify 1 つ・size S・done・depends 無し））・図 1（refs [FR1]）・本文は日本語だけ）と、その期待 `tests/fixtures/face/expected-note.html`。face の fixture の憲法・rules・要件書（P-1・A-1・R-1・FR1・AC1）はそのまま。歯の写しには `contracts/schema.toml`（実の repo の写し・便 23 の歯と同じ）を親 dir に置く。

(g) 歯 `crates/folio/tests/face_note.rs`（関数名はすべて face_note を含める・`--test face_note` の scope）。写しの作り: fixture の 4 file + `design-note/full.yaml` を一時 dir の `src/` へ、`contracts/schema.toml` を一時 dir の直下へ写す。
1. 写しで `--face note --id full --write` = 0 ∧ 出力が expected-note.html と byte 一致。
2. title に `<b>` を入れた写し = 0 ∧ 出力に `&lt;b&gt;` が在り `<b>` が無い。
3. 実の正本（example）を `--write` = 0 ∧ `folio parts --check --dir design-intent` に note の面を --page で渡して = 0 ∧ stdout に「違反 0」（NFR2）。
4. 実の正本の census: h1「設計ノート example」・副題が title の逐語・章の帯が 7（節 6 + 図 1）・item-row の数が 5 つの表の節の行の合計（4 + 4 + 4 + 2 + 1 = 15）・契約表の行の hint に write-set が無く（example の行は write-set を持たない）depends の hint も無く（値が空の一覧）verify が在る・図の card 1・「（まだ分からない）」は図の cd の 1 か所だけ。
5. `--check` の 3 値: 面が無い 2・1 byte 変えた面 1・一致 0。
6. `--face note` に --id 無し = 2 ∧「--id が無い」／`--id nope`（無い）= 2／`--face srs --id full` = 2 ∧「--id は面 adr と note にだけ付く」／`--id Full`（形でない）= 2。
7. 写しの節の型を「recipe」に = 2 ∧「閉じた一覧に無い」（FR9・AC7）。
8. 写しの status を「final」に = 2 ∧「状態」。
9. 写しの fields の行の need を「maybe」に = 2 ∧「要否」。
10. 写しの contract の行に欄「budget」を足す = 2 ∧「器の導出 file に無い」。
11. 写しの親 dir の `contracts/schema.toml` を消す = 2 ∧「器の導出 file が読めない」。
12. 写しの parts の行の ref に FR9（fixture に無い）を足す = 0 ∧ 根拠の hint に「FR9（まだ分からない）」。
13. 写しの status を effective にし approval（who 持ち主・date・ruling・verbatim・surface R-8）を足す = 0 ∧ cover-status「発効・拘束力あり（承認 <date>）」∧ approval-block に sign 1 つ／status を retired にし superseded_by full2 を足す = 0 ∧「廃止」∧ `note-full2.html` へのリンク。
14. 写しの prose の 2 段落目が ol.items の li 2 と p.intro に分かれ、1 段落目は p 1 つ。
15. 写しの節を 13 に増やす（型 prose の節を足す）= 2 ∧「上限 12」。
16. `--write` と `--check` を両方付ける = 2。
- 既存の歯: `crates/folio/tests/parts.rs` は write-set に在るが本文は変えない（凍結目録の fixture だけ (e) のとおり）。`crates/folio/tests/face_adr.rs` は「--id は面 adr にだけ付く」の文言を見る歯 1 本（face_adr_id_is_required_and_only_on_the_adr_face）の期待の字面を「--id は面 adr と note にだけ付く」に直す（この 1 本だけ・他は不変）。歯 face.rs・note.rs は触らない。unit は置かない。

(h) 便 27 までの形との接続: 新規は `crates/folio/src/face_note.rs`（見積 700〜900 行）・歯 `crates/folio/tests/face_note.rs`・fixture 2 本。`crates/folio/src/face.rs` は run の腕と文言（数行）・`crates/folio/src/main.rs` は doc 2 行と `mod face_note;` の 1 行（module の宣言は main.rs・lib.rs は無い）・`crates/folio/src/parts.rs` は FACES の 1 語と文言 1 行・`design-intent/preview/parts.json` と凍結目録は 7 部品の faces・`crates/folio/tests/face_adr.rs` は文言 1 か所。`face_adr.rs`・`face_index.rs`・`face_srs.rs`・`face_constitution.rs`・`site.rs`・`note.rs`・`build.rs`・`folio.css`・`.github/workflows/` は触らない。face_adr.rs の非公開の関数（cover・item_marks・prose_chapter・resolve・id_link）は呼ばず、同じ字面を face_note.rs に自前で持つ。外部 crate は増えない。正規表現は使わない。size は M = 中身を変える既存の file 1 本あたりの増分は小さい（face.rs 数行・main.rs 3 行・parts.rs 2 行・parts.json 7 行・face_adr.rs の歯 1 行）が、新規の face_note.rs が大きい。

## 2. 範囲

- 入れる: 面 note の生成器・--id の口・部品目録の faces・凍結 fixture・歯 16 本。
- 入れない: 入口の棚の導線と build / serve の出力（便 29）・図の生成（FR15）・欄の決まりの規則の二重化・css の変更・docs/design の便の文書を YAML へ移すこと（CON8 の裁定の後）・walk 承認（便 29 の後）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| gate | 命令の口 | --face note と --id・3 値 |
| read | 読み | design-note/<id>.yaml + 憲法・rules・要件書 + 器の導出 file |
| label | 名札の表 | 節の型・状態・要否・形・行き先 |
| page | 組み立て | cover・節の章（型ごと）・図・承認欄・foot |
| catalog | 部品目録 | 7 部品の faces に note |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 27 の歯は期待不変で全部回る・face_adr の 1 本は文言だけ）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。器の導出 file の写し `contracts/schema.toml` は便 23 のまま。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ac"
title = "設計ノートの面を正本から folio face --face note --id <文書 id> で生成する（5 面目・節の型ごとの見せ方・契約表の欄は器の導出 file から）"
req = ["FR9", "FR10", "NFR2"]
section = "1"
write-set = ["+crates/folio/src/face_note.rs", "+crates/folio/tests/face_note.rs", "crates/folio/src/face.rs", "crates/folio/src/main.rs", "crates/folio/src/parts.rs", "crates/folio/tests/parts.rs", "crates/folio/tests/face_adr.rs", "design-intent/preview/parts.json", "tests/fixtures/floor/parts-catalog.json", "+tests/fixtures/face/design-note/full.yaml", "+tests/fixtures/face/expected-note.html"]
verify = ["cargo nextest run -p folio --test face_note face_note", "cargo nextest run -p folio --test face_adr face_adr", "cargo nextest run -p folio --test parts parts", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "face_note の歯 16 本（凍結 fixture の byte 一致・escape・実の正本で parts 合格・census・check の 3 値・id の口 4 形・型の一覧外・状態と要否の表外・導出 file に無い欄・導出 file 不在・未解決の参照・effective と retired・列挙の分割・章の上限・mode）が緑、face_adr の歯は文言 1 か所を直して緑、parts の歯は本文不変・凍結目録に note を足して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
