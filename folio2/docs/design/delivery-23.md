# 設計: 便 23 — 設計ノートの正本（design-intent/design-note/*.yaml）を `folio check` の床に入れる（M1 の 1 便目・形と契約表の欄）

- 要件: FR9（設計ノートは 1 つの型・節は閉じた一覧の型だけ・正本の形は構造の床で数える）/ FR10（契約表の欄は器の導出 file から読む・読めなければ「まだ分からない」）/ FR5（3 値・確かめられなかった検査を合格にしない）/ NFR3（参照はつながる）
- 条: P-5.1（型の一覧は型付きデータ）/ P-2.4（閉じた一覧）/ P-4.1・P-4.2（黙って飛ばさない）/ P-6.3・P-6.4（欄の一覧を 2 面で人が書かない＝床の定数と欄の決まりの写しは 1 字違えば落とす）/ N-2.1（散文にしか無い規則は規則でない）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-3（決定 (1) 1 つの型と閉じた節の型／決定 (2) 契約表の欄の正本は器の型から導出した file／決定 (3) folio2 が持つのは正本の形・導出物の差分 0・自分の id 空間の解決だけ）。欄の決まりの正本 = design-intent/design-note/schema.yaml（f2-648.7・持ち主の承認 2026-09-17「承認する」）。便 22（f2-648.36）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「進んで良い」（f2-648 notes・M1 の最初の便 = 設計ノートの床）。要件書 CON8 の順序（契約表は今の形で着地させ M1 で YAML 正本へ移す）のとおり、本便は床だけを運び、導出物（FR11）と散文の門（FR12）は後続の便（散文の門 = 便 24・導出物 = 器の s2-07l.473 の着地後）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 x が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

設計ノートの正本（`design-intent/design-note/` の下の `.yaml`・欄の決まりは同じ dir の `schema.yaml`）を `folio check` が読み、形を数えるようにする。数えるのは 欄の決まりの schema 節が定める形（文書と meta の欄・節の番号と型・型ごとの行の欄と値域・承認欄の要否）と、契約表の節の欄（器 scribe2 の導出 file `contracts/schema.toml` から読む・欄の一覧を自分の型にも散文にも持たない・FR10）と、参照 id の解決（folio2 が所有する id 空間 = 条・規範文・rules 行・要件書の id・判断の記録の id・同じ文書の行 id）である。散文の門（FR12・rules 行 R-16）と導出物（FR11）は本便に入れない。判断の記録の欄の決まり（便 5・`adr.rs` の床の定数 FLOOR と `adr/schema.yaml` の schema 節の 1 字一致）と同じ作りで運ぶ。

planner の実測（2026-09-18・main 935bcc5）: `design-intent/design-note/` には `schema.yaml`（欄の決まり・schema 節 + meta・mapping・plain）と `example.yaml`（見本・status example・節 6 つ = prose・parts-table・ports-table・fields-table・teeth-table・contract-table・figures 1 つ・sources 空）の 2 file。schema 節の閉じた一覧: doc の required = meta・sections／optional = figures・sources／doc_meta の required = id・title・version・status・generated・profile／status_enum = draft・effective・retired・example／effective_status = effective・retired／profile_enum = design-note／approval の required = who・date・ruling・verbatim・surface（surface_enum = R-8）／section の required = n・type・title・optional = body・rows・note／type_enum = prose・parts-table・ports-table・fields-table・teeth-table・contract-table／by_type の required と forbid と row の欄（prose = body 必須・rows 禁止／parts-table の行 = id・name・role + ref・note／ports-table の行 = id・name・input・output・refuses + ref・note・refuses_none_marker = なし／fields-table の行 = id・name・need・shape + enum・note・need_enum = required・optional・shape_enum = text・list・number・bool・table／teeth-table の行 = id・name・red_when・fixture + ref・note／contract-table = rows 必須・section_ref_type = prose）／contract_table.external_schema = owner scribe2・path contracts/schema.toml・format toml・reader_expects の head「schema = 1」・rows_key field・row_fields name・need・shape／row_id の pattern `^[a-z][a-z0-9-]*$`／figures の entry = id・type・caption・spec 必須 + refs・note・type_enum_ref = parts.json の figure_type_enum（8 型）。器の導出 file `contracts/schema.toml` は scribe2 の tracked な生成物（2026-09-15・`scribe2 contracts schema` の出力と一致）で、欄 16（id・title・req・section・touches・surfaces・write-set・creates・tests・also・verify・size・done・depends・classes・opens・need は required 7〔id・title・req・section・verify・size・done〕/ optional 9・shape は text 4 / list 12）。planner が本便と同じ PR でその写しを `contracts/schema.toml`（folio2 の根・版管理）に置いた（本便の write-set の外・作業者は 1 字も変えない・写しの取り直しは planner の手番）。既存の歯の fixture 17 組（`folio check` を通る設計文書の置き場の形）は `design-note/` を持たない＝設計ノート 0 本として通り、期待は変わらない。verify の filter 語 note を名に含む `#[test]` の関数は base に 0 本（render の歯 1 本〔status_note の oracle〕は filter 語 note を含むが `--test note` の scope の外）。参照 id の既知の集合を作る関数 `refs.rs` の known_ids は private（式は変えず可視性だけ crate の中から呼べる形に広げる・便 12 の check.rs の 5 関数と同じ扱い）。

(a) 読み: `folio check --dir <正本の置き場>` は、正本 6 file の後に `<dir>/design-note/` を読む。dir が無い = 設計ノート 0 本（違反でも「まだ分からない」でもない）。dir の直下で名が `.yaml` で終わる file のうち `schema.yaml` 以外を名の昇順に設計ノートの正本として読む（他の 6 file と同じ読み手 = symlink・読めない・parse できない・最上位が欄の表でない は「まだ分からない」・重複キーは種別「重複キー」の違反）。`schema.yaml` は欄の決まりの写しとして読み、schema 節のうち名が `_note` で終わらない欄の型と値が床の定数（(b) の FLOOR）と 1 字も違わないことを確かめる（違えば種別「note」の違反「design-note/schema.yaml: 床の定数と違う: <欄の道>」・`adr.rs` の便 5 と同じ式）。器の導出 file は `<dir>` の親 dir の `contracts/schema.toml`（欄の決まり contract_table.external_schema.path・path_base = repo-root）で、契約表の節を持つ設計ノートが 1 本以上あるときだけ読む。無い・読めない・先頭が「schema = 1」でない・`[[field]]` の行に name・need・shape が揃わない・need が required / optional でない・shape が text / list でない = 「まだ分からない」（文言「contracts/schema.toml: 器の導出 file が読めない: <理由>」・AC8）。読み手は行走査（`[[field]]` の見出しと、名 = 引用符で囲んだ値 の形の行だけ・正規表現は使わない・外部 crate を足さない）。

(b) 床の定数（`crates/folio/src/note.rs` の FLOOR・`adr.rs` と同じ木の型）= 欄の決まり schema.yaml の schema 節の `_note` でない全欄（path_base・date_format・doc・doc_meta・section・by_type・contract_table・derived・landing・index・figures・guards）。値は schema.yaml の字面そのまま（数は数・一覧は一覧・表は表）。derived・landing・index・guards の欄は本便では検査に使わず写しの一致だけを確かめる（後続の便が使う）。

(c) 設計ノート 1 本の検査（種別「note」・文言は「design-note/<file>: <場所>: <理由>」の形）:
- 最上位の節: doc.required（meta・sections）が無い = 違反「節「sections」が無い」・doc.required + optional 以外の節 = 「未知の節」（種別「未知の節」・文言の形は他の正本と同じ）。
- meta: required の欄の非空（既存の非空の関数）・id は file 名の stem と一致し pattern に合う・version は version_pattern・status は status_enum・generated は date_format・profile は profile_enum・status が effective_status のとき approval が 1 行以上（各行 who・date・ruling・verbatim・surface 非空・surface は surface_enum・date は date_format）・status が example のとき approval 無し（在れば違反）・supersedes / superseded_by は他の設計ノートの id（無ければ違反）・retired は superseded_by 必須。
- sections: 一覧でなければ「まだ分からない」。各節は required（n・type・title）非空・n は 1 以上の整数で前の節より大きい（同じ・戻る = 違反・飛びは可）・type は type_enum に無ければ違反「節の型「<値>」が一覧に無い」（AC7）・by_type の required の欄が無い／forbid の欄が在る = 違反・rows は一覧でなければ「まだ分からない」・各行は行の required 非空・required + optional 以外の欄は違反「行の欄「<名>」が欄の決まりに無い」・行 id は文書内の同じ節で一意（種別「重複キー」）。
- 型ごと: fields-table の need は need_enum・shape は shape_enum／ports-table の refuses は非空（断らない口は「なし」の字面）／teeth-table の fixture は非空（path の実在は数えない）／ref（parts・ports・teeth）と figures の refs の各要素は既知の id（下の id 空間）に解ける。
- contract-table: rows の各行の欄は器の導出 file の field の name の集合に閉じる（無い欄 = 違反「契約表の欄「<名>」が器の導出 file に無い」・AC13 = 導出 file に欄を 1 つ足すとその欄を持つ行が通る）・need = required の欄は非空・shape = text は文字列・list は文字列の一覧（空の一覧は可）・id は row_id の pattern・同じ節で一意・section は同じ文書の prose の節の n（無い・prose でない = 違反）・req の各要素は要件書の id・depends の各要素は同じ節の行 id。
- figures: 一覧でなければ「まだ分からない」・各 entry は required 非空・type は parts.json の figure_type_enum（便 13 の build 時の一覧）に無ければ違反・refs は既知の id・spec は表（中身は数えない・図の生成は FR15 の便）。
- sources: 参照 id の母集団に入れない（読まない）。
- id 空間（既知の id）= 憲法の条 id と規範文 id・rules 行の id・要件書の 7 節の id・判断の記録の id（`adr/` の file 名）・同じ設計ノートの契約表の行 id（`<doc id>#<row id>` の形でも裸の row id でも解ける）。`refs.rs` の known_ids（条・rules・要件書）と `link.rs` の判断の記録の id の読みを crate の中から呼ぶ（式は変えない）。
- 実の正本で 0 件で通ることの実測（planner・2026-09-18・example.yaml v0.1 に上の検査を手で当てた）: 節 6 つの型は全部一覧内・行の欄は全部 required + optional の中・契約表の行 a の欄 8 つ（id・title・req・section・verify・size・done・depends）は導出 file の 16 欄の中で required 7 つが非空・section 1 は prose・req の FR15 と ref の ADR-4・R-15・R-3・P-10・R-14・FR14・AC12・FR15 は全部解ける・figures の type archify-architecture は 8 型の中・status example で approval 無し・id は stem と一致（admin の事前読みで meta.id が example-figure-pipeline だった食い違いを planner が example に直した・PR 受付前）。admin の受付時の独立の実測を添える。

(d) 歯 `crates/folio/tests/note.rs`（binary 経由・関数名はすべて note を含める＝verify の filter 語）。歯の置き場のため既存の歯 `crates/folio/tests/check.rs` を write-set に載せるが本文も期待も変えない（verify は `--test check check` の scope）。入力は `design-intent/` を丸ごと一時 dir の `design-intent/` へ写し、`contracts/schema.toml` を一時 dir の根の `contracts/` へ写し、git の 1 commit にしてから設計ノートの file を置く・変異を当てて `folio check --dir` を回す。
- 写しそのまま（example.yaml と schema.toml）= 終了 0。
- AC7（凍結 fixture `tests/fixtures/design-note/section-types/`）: `ok.yaml`（一覧内の型だけ・最小の 2 節 = prose と parts-table）を置く = 0／`unknown-type.yaml`（一覧に無い型 `mystery` の節 1 つ）を置く = 1 ∧「節の型「mystery」が一覧に無い」∧ 違反 1。
- AC8（`tests/fixtures/design-note/schema-missing/note.yaml` = 契約表の節を持つ最小の設計ノート）: 写しの `contracts/schema.toml` を消して = 2 ∧「器の導出 file が読めない」∧ 合格にならない／契約表の節を持たない写し（example.yaml も外す）では消しても 0（読まない）。
- AC13（`tests/fixtures/design-note/schema-plus-one/schema.toml` = 実の導出 file に optional・list の欄 `extra` を 1 つ足したもの／`note.yaml` = 契約表の行に `extra` を持つ）: 写しの導出 file を fixture のものに替えて = 0／実の導出 file のままでは = 1 ∧「契約表の欄「extra」が器の導出 file に無い」。
- 形の違反（example.yaml の写しに変異 1 つずつ・各 1 で違反 1）: meta の status を `nope`／id を stem と違う値／節の n を前と同じ／prose の節に rows を足す／parts-table の行から role を消す／fields-table の need を `maybe`／契約表の行の section を prose でない節の n／req に無い要件 id／ref に無い id `ADR-99`／figures の type を `mystery`。
- 「まだ分からない」（各 2）: sections を表に／導出 file の先頭を「schema = 2」に／設計ノートの file を symlink に。
- 承認欄: status を effective にして approval 無し = 1／approval に 5 欄の行を足す = 0／status example のまま approval を足す = 1。
- 欄の決まりの写し: schema.yaml の schema 節の `type_enum` に 1 つ足す = 1 ∧「床の定数と違う」。
- 既存の fixture 17 組と便 0〜22 の歯は期待不変（design-note/ を持たない = 0 本）。
- unit は置かない（`--test note` の scope で測れる binary の歯だけ）。

(e) 便 22 までの形との接続: 新規は `crates/folio/src/note.rs`・歯 `crates/folio/tests/note.rs`・fixture 6 本（新規 dir `tests/fixtures/design-note/` と 3 つの下位 dir = 要件書 AC7・AC8・AC13 の red_test が名指す path）。`crates/folio/src/check.rs` は検査の並びへの 1 行（入口・相談窓口の検査の後に設計ノートの検査を呼ぶ・引数は dir・憲法・rules・要件書・判断の記録の id・報告）だけ。`crates/folio/src/main.rs` は `mod note;` の 1 行。`crates/folio/src/refs.rs` は known_ids の可視性だけ（式・文言は変えない）。`crates/folio/src/link.rs` は判断の記録の id の集合を返す読み口が無ければ 1 関数を足す（既存の式を切り出すだけ）。`adr.rs`・`vocab.rs`・`yaml.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`design-intent/`・`contracts/schema.toml`・`.github/workflows/` は触らない。design-intent の写しを一時 dir に作って folio check を回す既存の歯 `crates/folio/tests/entrance.rs`・`crates/folio/tests/intake.rs`・`crates/folio/tests/freeze.rs`・`crates/folio/tests/floor_cases.rs`（134 case の runner）は、写しの根に `contracts/schema.toml` も写す（見本 example.yaml の契約表の節が器の導出 file を要るため。4 file は write-set に在るが期待は変えない＝写しの手順に導出 file を足すだけで、歯の数と判定は変えない。worker の問い 2026-09-18 への planner の回答。写しで check を回す歯の全数は copy_tree と check の引数の組で grep して確かめた＝この 4 本で全部。render.rs・face.rs・sheet.rs の写しは render / face / intake を回し、check.rs は実の置き場を写さずに回す）。外部 crate は増えない（TOML の導出 file は行走査で読む）。正規表現は使わない。size は M = 中身を変える既存の file 1 本あたりの増分は小さい（check.rs 数行・main.rs 1 行・refs.rs 1 語・link.rs 1 関数）が、新規の `note.rs` が床の定数の写しと型ごとの検査で大きい（見積 600〜900 行）。

## 2. 範囲

- 入れる: 設計ノートの読み・欄の決まりの写しの一致・形と型ごとの検査・契約表の欄を器の導出 file から・参照 id の解決・承認欄の要否・fixture（AC7・AC8・AC13）・歯。
- 入れない: 散文の門（FR12・R-16・便 24）・導出物の生成と差分 0（FR11・器の s2-07l.473 の後）・着地の印（FR13）・索引（FR14）・図の生成（FR15）・設計ノートの面（HTML・ADR-5 の撤退条件に触れるので別に問う）・既存の docs/design の .md を YAML へ移すこと（CON8 の範囲の裁定の後）・語彙への語の追加。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の定数 | schema.yaml の schema 節の写し・1 字一致 |
| read | 読み | design-note/ の列挙・6 file と同じ読み手 |
| shape | 形の検査 | meta・節の番号と型・型ごとの行の欄と値域・承認欄 |
| external | 器の欄 | contracts/schema.toml を行走査で読み契約表の欄を登録 |
| resolve | 参照の解決 | 条・rules・要件・判断の記録・同じ文書の行 id |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 22 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。器の導出 file の写し `contracts/schema.toml` は scribe2 の生成物の逐語（取り直しは planner）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "x"
title = "設計ノートの正本を folio check の床に入れる（形・型ごとの行・契約表の欄は器の導出 file から・参照の解決）"
req = ["FR9", "FR10", "FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/note.rs", "+crates/folio/tests/note.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/src/refs.rs", "crates/folio/src/link.rs", "crates/folio/tests/check.rs", "crates/folio/tests/entrance.rs", "crates/folio/tests/intake.rs", "crates/folio/tests/freeze.rs", "crates/folio/tests/floor_cases.rs", "+tests/fixtures/design-note/section-types/ok.yaml", "+tests/fixtures/design-note/section-types/unknown-type.yaml", "+tests/fixtures/design-note/schema-missing/note.yaml", "+tests/fixtures/design-note/schema-plus-one/schema.toml", "+tests/fixtures/design-note/schema-plus-one/note.yaml"]
verify = ["cargo nextest run -p folio --test note note", "cargo nextest run -p folio --test check check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "note の歯（写しそのまま 0・AC7 の 2 本・AC8 の 2 形・AC13 の 2 形・形の違反 10・まだ分からない 3・承認欄 3・欄の決まりの写し 1）が緑、便 0 の歯 check が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
