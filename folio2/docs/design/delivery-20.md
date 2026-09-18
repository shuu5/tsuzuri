# 設計: 便 20 — 入口の面に「支度表」の節を出す（`folio face --face index`・4 面目を足さない）

- 要件: FR1（支度表 1 枚を生成する＝人が読む面も要る）/ FR4（3 面を 1 つの生成器から出す）/ NFR2（部品目録に無い class は 0）
- 条: P-2.1・P-2.4（人が読むページは 1 つの生成器から・部品は閉じた一覧）/ P-4.2（判定できないものは「まだ分からない」＝支度表が無いときは「まだ無い」と表に出す）/ P-6.3（同じ内容は一方を正本に他方を導出）
- 判断の記録: ADR-6（決定 (3) 支度表の人が読む面は 4 面目を足さず入口の面の節として出す／帰結「入口の面に支度表の節を足す便は、部品目録に無い部品を足さず（AC2）、支度表が無いときは節を「まだ無い」の形で出す」）。ADR-5（決定 (3) 部品目録を型で閉じる・撤退条件に触れない＝面の型は 3 のまま）。便 19（f2-648.33）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「承認するし質問も全て推奨で承認する」と ADR-6 の発効「承認する」（f2-648.31 notes・main f54efaa）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 u が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

入口の面の生成器（`crates/folio/src/face_index.rs`・便 16）に、相談窓口の結果である支度表を人が読む節「支度表」を足す。正本は相談窓口の正本 `intake.yaml`（節の見出しと説明・支度表の file 名）と、在れば支度表 `<dir>/<sheet.file>`（便 19 の生成物・無ければ「まだ無い」の形）。部品は入口の面が既に使う 12 種の中から chapter-deck-band（slim の帯）と status-line（行の並び）だけを使い、部品目録に部品も class も足さない（AC2・ADR-6 帰結）。面の型は 3 のまま（4 面目を足さない・ADR-5 の撤退条件に触れない）。

planner の実測（2026-09-18・main 67da80f）: 入口の面の節の並びは head → cover → 棚 → status-line → intake-line → 章 01 読む順番（s1・band-5）→ 章 02 相談窓口（s2・band-3）→ foot。slim の帯の関数 slim_band（番号・帯の class・kicker・title と lead を持つ欄）と status-line の字面（section data-component status-line・p.st の中に span.mark・span.k・span.v・色の class は ok と next）は既に在る。様式の定義 folio.css の帯の色の class は band-1〜band-6。読む正本の一覧 SOURCES は 6（index・constitution・srs・vocabulary・rules・adr/）。intake.yaml v0.1 は発効済み（main f54efaa）で sheet の節に file（intake-sheet.yaml）・title・explain・sections を持つ。支度表の形は便 19 の (a) のとおり（meta・documents〔id・type・with・from〕・recommended〔q・ask・recommend〕・answers〔q・value・source〕・approval〔空か承認の行〕）。凍結 fixture `tests/fixtures/face/` は intake.yaml を持たない。fixture の写しを作る歯は `tests/face.rs`（index_fixture_copy）・`tests/site.rs`（fixture_copy の 5 file の一覧）・`tests/serve.rs`（同じ一覧）の 3 か所で、どれも file 名を列挙して写す。

(a) 読み: `face_index.rs` の derive は intake.yaml を他の正本と同じ読み手（face.rs の load）で読み（無い・読めない・欄が無いは導出できない = 2・便 14 と同じ形）、`<dir>/<sheet.file>` は在れば読み、無ければ「まだ無い」（導出できないではない）。SOURCES は 7（末尾に intake.yaml・foot の sources の字面もそれに従う）。支度表は生成物なので sources には載せない（節の中で file 名を出す）。

(b) 節「支度表」: 章 02 の直後・foot の前に、slim の帯（section id s3・帯の class band-4・kicker「支度表」・h2 = intake.yaml の sheet.title・lead = sheet.explain）と chapbody を出す。chapbody の中は status-line の部品 1 つ（section data-component status-line・aria-label「支度表」）で、行は次のとおり:
- 支度表が在るとき: p.st.ok = mark「●」・k「持つ文書」・v = documents の各行の「<type>」を「・」で繋ぐ（with が空でない行は「<type>（付録の <with の各 id を targets から type に写したもの・「・」で>）」）・0 行なら「なし」／p.st = mark「○」・k「推奨で進めた項目」・v = recommended の各行の「<ask>（おすすめ: <recommend>）」を「／」で繋ぐ・0 行なら「なし（全部に答えた）」／p.st = mark「○」・k「承認」・v = approval が空なら「まだ（対話面で承認したら台帳と承認欄に記帳する）」・空でなければ最初の行の「<when> <verbatim>」／末尾に p.st = mark「·」・k「正本」・v「<sheet.file>（folio intake の生成物・手で直さない）」。
- 支度表が無いとき: p.st = mark「○」・k「まだ無い」・v = 「支度表はまだ無い。「<intake.yaml の相談の命令＝入口の正本 intake.command>」と AI に頼むと作られる」の 1 行だけ（P-4.2 の形）。
- 数と日付は正本から数えたものだけ（支度表の行数）。文言（k の名札・「なし」・「まだ」）は本便の名札の表（β）として生成器が持つ。escape は便 14 と同じ。
- 導出できない（2・出力先に 1 byte も書かない）: intake.yaml の sheet の file・title・explain が無い・空／支度表が在るのに読めない・最上位が表でない／documents の行に id・type が無い／documents の id が intake.yaml の targets に無い／with の id が targets に無い／recommended の行に ask・recommend が無い／approval が一覧でない。

(c) 凍結 fixture（`tests/fixtures/face/`・手書き・最小）: `intake.yaml`（新規・meta・answers・targets 2 行〔constitution〈with = vocabulary〉・inject〕・questions 2 行・sheet〔file = intake-sheet.yaml・title・explain・sections 1 行〕）と `intake-sheet.yaml`（新規・documents 1 行 constitution〈with = [vocabulary]・from q1〉・recommended 1 行〈q2〉・answers 2 行・approval 空）。期待の面: `expected-index.html`（支度表なし・本便で更新）と `expected-index-sheet.html`（新規・支度表あり）。どちらも最初の 1 回は生成器の出力を写して置いてよく、置いた後は歯が固定する。憲法面・要件書面の期待は変えない。

(d) 歯 `crates/folio/tests/face.rs`（既存の歯の file に足す・関数名はすべて face を含める）:
- 凍結 fixture との byte 一致 2 本: 写し（intake.yaml を写す・支度表は写さない）で `--face index --write` = 0 ∧ `expected-index.html` と一致／写しに `intake-sheet.yaml` も写して = 0 ∧ `expected-index-sheet.html` と一致。
- 実の正本（AC2）: `--dir design-intent`（支度表なし）で書き `folio parts --check --page index=<一時 file>` = 0 ∧ 3 面まとめて = 0。写しに fixture の支度表を置いた形でも parts --check = 0（支度表ありの行が目録外の class を出さない）。
- census: 出力に section の id が s3 の帯・sheet.title・sheet.explain・「まだ無い」（支度表なし）／支度表ありでは documents の type・recommended の ask・「まだ」（承認）が含まれる。
- 導出できない 4 つ（どれも 2・出力先が出来ていない）: intake.yaml を消す／sheet の title を空に／支度表の documents の id を targets に無い id に／支度表の最上位を一覧に。
- 既存の歯の写しの一覧（index_fixture_copy）に intake.yaml を足す（他の期待は変えない）。`tests/site.rs` と `tests/serve.rs` の fixture_copy の 5 file の一覧に intake.yaml を足す（期待は expected-index.html の更新に追従・site の byte 一致の歯は同じ期待 file を読む・他は変えない）。
- unit（`src/face_index.rs` の中・名に face を含む）: 持つ文書の行の字面（with の有無）・承認の行の字面（空／あり）。

(e) 便 19 までの形との接続: 変えるのは `crates/folio/src/face_index.rs`（SOURCES 7・derive の読み 2 つ・節 s3 の関数 1 つ・名札の表）と歯 3 file（face・site・serve の写しの一覧）と fixture 4 本（新規 3・更新 1）だけ。`face.rs`・`face_constitution.rs`・`face_srs.rs`・`parts.rs`・`site.rs`・`serve.rs`・便 19 の module（sheet.rs）・便 18 の module（intake.rs）・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`design-intent/preview/parts.json`・`folio.css`・`design-intent/*.yaml`・`.github/workflows/` は触らない。部品目録に部品も class も style の性質も足さない。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`face_index.rs` の増分は節の関数と名札で 120 行未満・歯の増分は一覧の 1 行と歯 8 本）。

## 2. 範囲

- 入れる: 入口の面の節「支度表」（在る／無いの 2 形）・intake.yaml の読み・fixture・歯。
- 入れない: 支度表の生成（便 19）・支度表の形の床・承認の記帳（人）・見た目だけの直し（h2「17 つの原則」・lane-chip・用語の小窓）・部品目録の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| read | 読み | intake.yaml（必須）と支度表（任意）を読む |
| section | 節 s3 | slim の帯 + status-line の行（在る／無い） |
| labels | 名札の表 | k の名札・「なし」・「まだ」・正本の行の文言 |
| fixture | 凍結 fixture | intake.yaml・intake-sheet.yaml・期待の面 2 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 19 の歯は期待不変で全部回る・face / site の期待 file の更新を除く）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "u"
title = "入口の面に「支度表」の節を出す（folio face --face index・4 面目を足さない）"
req = ["FR1", "FR4", "NFR2"]
section = "1"
write-set = ["crates/folio/src/face_index.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/serve.rs", "+tests/fixtures/face/intake.yaml", "+tests/fixtures/face/intake-sheet.yaml", "+tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/face/expected-index.html"]
verify = ["cargo nextest run -p folio --test face face", "cargo nextest run -p folio --test site site", "cargo nextest run -p folio --test serve serve", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face の歯（支度表なし／ありの凍結 fixture との byte 一致・実の正本と支度表ありの写しで parts --check 合格・census・導出できない 4 つ・unit）が緑で便 14〜16 の歯は期待不変（期待の面の更新を除く）、site の歯が更新した期待で緑、serve の歯が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
