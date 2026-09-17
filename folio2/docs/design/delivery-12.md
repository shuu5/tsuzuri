# 設計: 便 12 — 入口の面の正本 `design-intent/index.yaml` を `folio check` の床に入れる（正本 4 file → 5 file）

- 要件: FR4（入口・憲法・要件書の 3 面を 1 つの生成器から出す＝入口の面にも正本が要る）/ FR5（3 値・確かめられなかった検査を合格にしない）
- 条: P-6.1（各文書の正本は YAML 1 file）/ P-6.3（同じ内容を 2 面が持つとき一方を正本とし他方は導出する）/ P-5.1（型の一覧は型付きデータに置く）/ P-4.1・P-4.2（実行できなかった検査を異常なしにしない・まだ分からないを表に出す）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-5（決定 (2) 入口の面は自分の正本を持つ・帰結 1「床が見る正本を 5 file へ広げる・凍結 fixture の期待は変えない」）。便 11（f2-648.21）の後に直列で置く。
- 裁定: 持ち主 2026-09-17（f2-648 notes・入口の案内文の置き場 = 新しい正本）と ADR-5 の発効の承認（f2-648.22 notes）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 m が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

入口の面（設計文書の入口のページ）の案内文は、手書きの見本の中にしか無かった。ADR-5 決定 (2) により、入口の面は自分の正本 `design-intent/index.yaml` を持つ。正本そのものは planner がこの設計文書と同じ取り込みで main に置いた（本便の write-set の外・作業者は 1 字も変えない）。本便は `folio check` がこの 5 本目の正本を読み、形を数えるようにする。入口の面の生成（HTML）は後続の便で、本便は検査だけを運ぶ。

(a) 読み: `crates/folio/src/check.rs` の正本の一覧の定数を 4 本から 5 本（憲法・rules・語彙・要件書・入口の順）にし、入口の正本 index.yaml を他の 4 本と同じ読み手（同じ関数）で読む。したがって index.yaml が symlink・無い・file でない・読めない・parse できない・最上位が欄の表でない はどれも他の 4 本と同じ文言の形で「まだ分からない」（終了 2）になり、無いときの文言は「index.yaml: 正本が無い」。重複キーは他の 4 本と同じ種別「重複キー」の違反。5 本のどれかが読めなければ、今と同じく後段の検査には進まない。

(b) 入口の正本の検査は新しい module `crates/folio/src/entrance.rs` の関数 1 つに置き、check.rs の検査の並びの中で要件書の形の検査の直後に呼ぶ（引数は入口の正本・語彙の正本・報告）。数えるもの:
- 節の閉じた一覧（床の定数・要件書と同じ持ち方）= meta・audience・shelf・lanes・intake の 5 つ。これに無い最上位の節は種別「未知の節」の違反（文言の形は他の正本と同じ「index.yaml: 未知の節「名」」）。
- 欄の非空（既存の非空の関数と同じ判定・同じ種別と文言の形）: meta の id・title・version・status／audience の label・short・text／shelf の title・explain／shelf.legend の各行の id・text／shelf.documents の各行の id・type・use／shelf.annexes の各行の id・type・inside／shelf.relations の各行の id・from・to・label・hint／lanes の title・lead／lanes.rows の各行の id・mark・who・why・minutes・stops／各 stops の行の doc・label／intake の title・lead・heading・text・steps。shelf.documents の行の欄 absent は null でよい（数えない）。intake の note は任意。
- 行 id の重複（既存の重複の関数と同じ種別「重複キー」）: shelf.legend の中・shelf.documents と shelf.annexes を合わせた 1 つの空間の中・shelf.relations の中・lanes.rows の中。
- 行き先の解決（種別「index」の違反・文言は「index.yaml: 場所: 行き先「値」が棚に無い」）: shelf.relations の from と to は shelf.documents か shelf.annexes の id のどれか／shelf.annexes の inside は shelf.documents の id のどれか／lanes.rows の各 stops の doc は shelf.documents の id のどれか。
- minutes は 1 以上の整数。文字列・小数・0 以下は種別「index」の違反（文言「index.yaml: lanes の行 id: minutes が 1 以上の整数でない」）。
- 語彙（入口の文の中の英字の語）: 上の非空の欄のうち文を持つ欄（id・from・to・inside・doc・version・status・minutes を除く全部と、intake の note と steps の各要素、shelf.documents の absent が文字列のとき）を、便 4 の語彙の検査と同じ関数（`vocab.rs` の既知の語の集合と、免除されない語を並べる関数・どちらも crate の中から呼べる）に通し、免除されない語 1 つごとに種別「index」の違反（文言「index.yaml 場所: 語彙に無い英字の語「語」」）。rules 行 R-9 の母集団（憲法・rules・要件書）は変えない＝種別 R-9 では数えない（判断の記録の本文を種別 adr で数えているのと同じ持ち方）。
- 型が違う（節が欄の表でない・行の一覧が表の一覧でない・stops や steps が一覧でない）は「まだ分からない」。
meta の approval・generated と、発効の状態の値域は本便では数えない（入口の面の発効の形は、生成した面の承認の便で決める）。
既存の関数の所在: 非空の関数 non_empty・行 id の重複の関数 duplicate_ids・未知の節の関数 unknown_sections・行の一覧を読む関数 rows・行 id を読む関数 row_id は、どれも check.rs の中の private の関数である（check.rs は本便が中身を変える file＝閉包の中）。entrance.rs から呼ぶために、要るものだけ crate の中から呼べる可視性に広げる（式・文言・種別は 1 字も変えない）。yaml.rs には無いので yaml.rs の中身は変えない。
正本が新しい検査を 0 件で通ることの実測（admin 席・2026-09-17・main ec87254 の index.yaml v0.1 に上の検査を当てた）: 最上位の節は 5 つだけで閉じた一覧と一致／上に挙げた非空の欄に空は 0（shelf.documents の absent は null と文字列の両方が在り、null は数えない）／行 id の重複は 4 つの空間とも 0／行き先（relations の from と to・annexes の inside・stops の doc）は全部棚の id に解けて違反 0／minutes は全行 1 以上の整数で違反 0／文を持つ欄 62 か所の英字の語は、便 4 と同じ式（既知 = 語彙の見出し語・原語・識別子／免除 = id の形・日本語（原語）の括弧の中・1 字・旗の形）で語彙に無い語 0。したがって (d) の「写しそのまま = 終了 0」と、`design-intent/` の丸ごとの写しを入力にする歯（parity・凍結 fixture の 134 case・render）の期待が変わらないことの前提は成り立つ。

(c) 既存の歯の fixture: `tests/fixtures/` の下の、設計文書の置き場の形をした 17 組（check の 4 組・refs の 3 組・vocab の 2 組・adr の 3 組・link の 3 組・anchor の 2 組）に、同じ中身の最小の index.yaml を 1 本ずつ足す（新しい dir は作らない）。これらの組は入口の正本を持たないので、足さないと (a) により全部に「まだ分からない」が 1 件増えて期待が変わる。足した後、既存の歯の期待（終了コード・件数・文言）は 1 つも変えない＝既存の歯の file は 1 字も変えない。`tests/fixtures/check/missing-file/` にも足す（その組が「まだ分からない」になる理由を憲法が無いことだけに保つ）。inject の 5 組と figure の組は `folio check` を通らないので足さない。最小の index.yaml の中身（17 本とも同じ・英字の語は欄の名と id だけ）:

```
meta: {id: fixture-index, title: 入口, version: v0.1, status: draft}
audience: {label: 誰のためか, short: 持ち主, text: 持ち主が読む。}
shelf:
  title: 文書
  explain: 文書の一覧。
  legend:
    - {id: readable, text: 読める}
  documents:
    - {id: constitution, type: 憲法, use: 譲らない線。, absent: null}
  annexes: []
  relations: []
lanes:
  title: 読み方
  lead: 読む順番。
  rows:
    - {id: first, mark: 初, who: はじめて読む人, why: つかむ, minutes: 10, stops: [{doc: constitution, label: はじめに}]}
intake:
  title: 相談窓口
  lead: 相談して決める。
  heading: 頼み方
  text: 質問に答える。
  steps: [質問, 承認]
```

凍結 fixture `tests/floor_cases.yaml`（134 case）と parity の歯は、入力が `design-intent/` の丸ごとの写しなので index.yaml が自然に入る＝fixture も歯も期待も変えない。day-1 の床 `scripts/check_draft.py` は index.yaml を読まない（planner の実測: index.yaml を置いても床は違反 0 のまま）ので、parity の突き合わせの入力に index.yaml の変異は足さない。

(d) 歯 `crates/folio/tests/entrance.rs`（binary 経由・歯の関数名はすべて entrance を含める＝verify の filter 語・base の歯に entrance を含む名は 0 本）。入力は `design-intent/` を丸ごと一時 dir へ写し、git の 1 commit にしてから（版管理の無い写しは別の理由で「まだ分からない」になる）変異を 1 つ当てて `folio check --dir` を回す。
- 写しそのまま = 終了 0。
- index.yaml を消す = 終了 2 ∧ 出力に「index.yaml: 正本が無い」。
- 最上位に未知の節を足す = 終了 1 ∧「未知の節」。
- audience の text を空にする = 終了 1。
- lanes.rows の 1 つ目の stops の doc を棚に無い id にする = 終了 1 ∧「棚に無い」。
- shelf.relations の 1 つ目の to を棚に無い id にする = 終了 1 ∧「棚に無い」。
- shelf.documents に同じ id の行をもう 1 つ足す = 終了 1 ∧「重複」。
- shelf の explain に語彙に無い英字の語を 1 つ足す = 終了 1 ∧「語彙に無い英字の語」。語彙に在る語（folio）を足しても終了 0（免除が効いていることの歯）。
- lanes.rows の 1 つ目の minutes を文字列にする = 終了 1 ∧「minutes」。
- lanes を欄の表でない値にする = 終了 2。
- 同じ表に同じキーを 2 度書く = 終了 1 ∧「重複キー」。
違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる。unit の歯（`src/entrance.rs` の中・名に entrance を含む）: minutes の判定（1・0・負・小数・文字列）と、行き先の集合の作り方（documents と annexes を合わせる）。

(e) 便 11 までの形との接続: 新規は `crates/folio/src/entrance.rs`・歯 `crates/folio/tests/entrance.rs`・fixture の index.yaml 17 本。`crates/folio/src/check.rs` は 正本の一覧の定数・読んだ正本を持つ型・検査の並びへの 1 行・冒頭の説明文 だけを変える。`crates/folio/src/main.rs` は `mod entrance;` の 1 行だけ。`vocab.rs` の 2 つの関数は既に crate の中から呼べるので可視性の変更は要らない。`refs.rs`・`vocab.rs`・`link.rs`・`anchor.rs`・`render.rs`・他の src は中身を変えない（参照 id の母集団・語彙 R-9 の母集団・読み物の生成に index.yaml を入れない）。(c) で fixture に file を足すので、その fixture を読む既存の歯を verify で名指して期待不変を測る: filter 語 check・refs・vocab・adr・link・anchor の 6 行。これらの filter 語が解ける歯の file（`crates/folio/tests/` の check.rs・refs.rs・vocab.rs・adr.rs・link.rs・anchor.rs・gitcheck.rs・parity.rs・render.rs と、unit の歯を持つ `crates/folio/src/` の gitcheck.rs・link.rs・render.rs・yaml.rs）は、verify で回すために write-set に在るが 1 字も変えない（期待も変えない）。本便が中身を変える既存の file は check.rs と main.rs の 2 本だけである。契約表の size は S = 中身を変える既存の file（check.rs・main.rs）1 本あたりの増分の見積（どちらも 100 行に満たない）で、新規の file（entrance.rs・歯・fixture の index.yaml 17 本）は増分に数えない。残りの歯（freeze・floor_cases・inject・lineage）の回帰は共通の検証（workspace の全歯）が捕まえる。`design-intent/` の下・`scripts/`・`tests/floor_cases.yaml`・`tests/run_floor_cases.py`・`.github/workflows/` は触らない。外部 crate は増やさない（`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。

## 2. 範囲

- 入れる: `folio check` の 5 本目の正本（読み・節の一覧・非空・行 id の重複・行き先の解決・minutes・入口の文の語彙）・歯・既存の fixture 17 組への最小の index.yaml。
- 入れない: 入口の面の生成（HTML）・部品目録の型（閉じた一覧）・読む順番の行き先を面の中の節まで解く検査（面の節の id は憲法面・要件書面の生成の便で決まる）・入口の正本の発効の形（承認欄の検査）・day-1 の床の script の変更・index.yaml の中身の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| load5 | 読み | 正本の一覧を 5 本にし、同じ読み手で index.yaml を読む |
| shape | 形の検査 | 節の一覧・非空・行 id の重複・minutes |
| resolve | 行き先の解決 | relations・annexes・stops の行き先を棚の id に解く |
| words | 入口の文の語彙 | 便 4 の関数で英字の語を数える（種別 index） |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 11 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "m"
title = "入口の面の正本 index.yaml を folio check の床に入れる（正本 5 file）"
req = ["FR4", "FR5"]
section = "1"
write-set = ["+crates/folio/src/entrance.rs", "+crates/folio/tests/entrance.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs", "crates/folio/tests/gitcheck.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/render.rs", "crates/folio/src/gitcheck.rs", "crates/folio/src/link.rs", "crates/folio/src/render.rs", "crates/folio/src/yaml.rs", "+tests/fixtures/check/dup-key/index.yaml", "+tests/fixtures/check/empty-field/index.yaml", "+tests/fixtures/check/missing-file/index.yaml", "+tests/fixtures/check/unknown-section/index.yaml", "+tests/fixtures/refs/bad-counts/index.yaml", "+tests/fixtures/refs/dangling-id/index.yaml", "+tests/fixtures/refs/orphan-rule/index.yaml", "+tests/fixtures/vocab/exemptions/index.yaml", "+tests/fixtures/vocab/unknown-word/index.yaml", "+tests/fixtures/adr/effective-no-approval/index.yaml", "+tests/fixtures/adr/schema-drift/index.yaml", "+tests/fixtures/adr/two-adopted/index.yaml", "+tests/fixtures/link/adr-id-missing/index.yaml", "+tests/fixtures/link/amended-by-orphan/index.yaml", "+tests/fixtures/link/retreat-kind-drift/index.yaml", "+tests/fixtures/anchor/no-anchor/index.yaml", "+tests/fixtures/anchor/root-digest-drift/index.yaml"]
verify = ["cargo nextest run -p folio entrance", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio adr", "cargo nextest run -p folio link", "cargo nextest run -p folio anchor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "entrance の歯（写しそのまま・index.yaml 無し・未知の節・空の欄・行き先 2 つ・行 id の重複・語彙 2 つ・minutes・型違い・重複キー・unit）が緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、便 7 の歯 anchor が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
