# 設計: 便 18 — 相談窓口の正本 `design-intent/intake.yaml` を `folio check` の床に入れる（正本 5 file → 6 file）

- 要件: FR1（相談窓口・質問は 5 問以下・回答から文書の集合を写像）/ FR5（3 値・確かめられなかった検査を合格にしない）
- 条: P-6.1（各文書の正本は YAML 1 file）/ P-5.1（型の一覧・閾値は型付きデータに置く）/ P-4.1・P-4.2（実行できなかった検査を異常なしにしない・まだ分からないを表に出す）/ P-10.1（独立した凍結 anchor）
- 判断の記録: ADR-6（決定 (1) 質問・推奨回答・写像表・支度表の欄の決まりは新しい正本 intake.yaml に置く・床が見る正本を 6 file へ広げる変更は別の便で運び凍結 fixture の期待は変えない／決定 (2) 質問 5 つ・行き先は入口の棚の文書の id と注入の 5 つに閉じる／決定 (5) 順序 = 発効 → 本便 → 命令の便）。便 17（f2-648.28）の後に直列で置く。
- 裁定: 持ち主 2026-09-18「承認するし質問も全て推奨で承認する」（f2-648 notes・置き場・質問 5 つ・支度表の形）。ADR-6 の発効の承認（逐語）を受けてから受付する（決定 (5)）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 s が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`）。

## 1. 目的と中身

相談窓口（intake）の質問・推奨回答・回答から文書の集合への写像・支度表の欄の決まりは、ADR-6 決定 (1) により新しい正本 `design-intent/intake.yaml`（v0.1・planner が PR #71 で main に置いた・本便の write-set の外・作業者は 1 字も変えない）が持つ。本便は `folio check` がこの 6 本目の正本を読み、形を数えるようにする。便 12（入口の正本 index.yaml を 5 本目として床に入れた便）と同じ作りで運ぶ。命令 `folio intake` と支度表の生成は後続の便で、本便は検査だけを運ぶ。

planner の実測（2026-09-18・main a4778c8）: intake.yaml の最上位の節は 5 つ（meta・answers・targets・questions・sheet）。meta は id・title・version・status・generated・approval。answers は values（はい・いいえ の 2 つ）と default（値は recommend）。targets は 5 行（constitution〔with = vocabulary・rules〕・srs・adr・design-note・inject・with は他は空・note は 3 行にある）。questions は 5 行（id は q1〜q5・ask・recommend〔全部 はい〕・why・yes〔各 1 つ・順に constitution・srs・adr・design-note・inject〕・no〔全部空〕）。sheet は file・title・explain・sections 5 行（meta・documents・recommended・answers・approval・各 id と text）。入口の正本 index.yaml の棚の documents の id は constitution・srs・design-note・adr の 4 つ、annexes の id は vocabulary・rules の 2 つ。既存の歯の fixture のうち `folio check` を通る設計文書の置き場の形をした組は 17（check 4・refs 3・vocab 2・adr 3・link 3・anchor 2）で、どれも index.yaml を持ち intake.yaml を持たない（`tests/fixtures/face/` と inject の 5 組は `folio check` を通らない）。verify の filter 語 intake を名に含む `#[test]` の関数は base に 0 本（`face_index.rs` の intake と intake_line は歯ではない）。

(a) 読み: `crates/folio/src/check.rs` の正本の一覧の定数を 5 本から 6 本（憲法・rules・語彙・要件書・入口・相談窓口の順）にし、intake.yaml を他の 5 本と同じ読み手（同じ関数）で読む。したがって intake.yaml が symlink・無い・file でない・読めない・parse できない・最上位が欄の表でない はどれも他の 5 本と同じ文言の形で「まだ分からない」（終了 2）になり、無いときの文言は「intake.yaml: 正本が無い」。重複キーは他と同じ種別「重複キー」の違反。6 本のどれかが読めなければ、今と同じく後段の検査には進まない。今の読みは一覧の末尾から取り出して割り当てているので、末尾に足した相談窓口を最初に取り出し、5 本の割り当てはずらさない。

(b) 相談窓口の正本の検査は新しい module `crates/folio/src/intake.rs` の関数 1 つに置き、check.rs の検査の並びの中で入口の検査の直後に呼ぶ（引数は相談窓口の正本・入口の正本・語彙の正本・報告）。便 12 の `entrance.rs` と同じ部品（check.rs の non_empty・duplicate_ids・unknown_sections・rows・row_id・便 4 の `vocab.rs` の既知の語の集合と免除されない語を並べる関数）を使い、式・文言・種別の形を揃える。数えるもの:
- 節の閉じた一覧（床の定数・入口と同じ持ち方）= meta・answers・targets・questions・sheet の 5 つ。これに無い最上位の節は種別「未知の節」の違反（文言「intake.yaml: 未知の節「名」」）。
- 欄の非空（既存の非空の関数・同じ種別と文言の形）: meta の id・title・version・status／answers の values・default／targets の各行の id・type／questions の各行の id・ask・recommend・why／sheet の file・title・explain／sheet.sections の各行の id・text。targets の with と note・questions の yes と no は空でよい（数えない）。meta の generated・approval は数えない（発効の形は ADR-6 の発効の便で決める）。
- 行 id の重複（既存の重複の関数・種別「重複キー」）: targets の中・questions の中・sheet.sections の中。
- 行き先の解決（種別「intake」の違反・文言「intake.yaml: 場所: 行き先「値」が一覧に無い」）: targets の各行の id は 入口の正本の shelf.documents の id か、注入を表す固定の id（inject・床の定数）のどれか／targets の各行の with の各要素は 入口の正本の shelf.annexes の id のどれか／questions の各行の yes と no の各要素は targets の id のどれか／questions の各行の recommend は answers.values のどれか／answers.default は answers.values のどれかか固定の値 recommend（床の定数）。
- 質問の数: questions の行数が FR1 の上限（5・床の定数・要件書 FR1 の規範文の数）を超えたら種別「intake」の違反（文言「intake.yaml: questions が FR1 の上限を超える」）。
- 語彙（相談窓口の文の中の英字の語）: meta の title／targets の type・note／questions の ask・why／answers の values の各要素／sheet の title・explain／sheet.sections の text を、便 4 の語彙の検査と同じ関数に通し、免除されない語 1 つごとに種別「intake」の違反（文言「intake.yaml 場所: 語彙に無い英字の語「語」」）。rules 行 R-9 の母集団（憲法・rules・要件書）は変えない（入口・判断の記録と同じ持ち方）。
- 型が違う（節が欄の表でない・行の一覧が表の一覧でない・values や with や yes や no が一覧でない）は「まだ分からない」。
正本が新しい検査を 0 件で通ることの実測（planner・2026-09-18・main a4778c8 の intake.yaml v0.1 に上の検査を手で当てた）: 最上位の節は 5 つだけ／非空の欄に空は 0／行 id の重複は 3 つの空間とも 0／targets の id 5 つは棚の 4 つと inject に解け、with の vocabulary・rules は annexes に解け、yes の 5 つは targets に解け、no は全部空、recommend は全部 はい で values に在り、default は recommend／questions は 5 行で上限と同じ／文を持つ欄の英字の語は AI・CLAUDE.md（識別子）だけで語彙に無い語 0。admin の受付時の独立の実測を添える。

(c) 既存の歯の fixture: `tests/fixtures/` の下の 17 組に、同じ中身の最小の intake.yaml を 1 本ずつ足す（新しい dir は作らない）。足さないと (a) により全部に「まだ分からない」が 1 件増えて期待が変わる。足した後、既存の歯の期待（終了コード・件数・文言）は 1 つも変えない＝既存の歯の file は 1 字も変えない。`tests/fixtures/check/missing-file/` にも足す（その組が「まだ分からない」になる理由を憲法が無いことだけに保つ）。最小の intake.yaml の中身（17 本とも同じ・各組の index.yaml の棚は constitution だけなので行き先はそれと inject だけ）:

```
meta: {id: fixture-intake, title: 相談窓口, version: v0.1, status: draft}
answers: {values: [はい, いいえ], default: recommend}
targets:
  - {id: constitution, type: 憲法, with: []}
  - {id: inject, type: 注入, with: []}
questions:
  - {id: q1, ask: 守る線を決めますか, recommend: はい, why: 基準になる。, yes: [constitution], no: []}
sheet:
  file: intake-sheet.yaml
  title: 支度表
  explain: 何を持つかの表。
  sections:
    - {id: documents, text: 持つ文書}
```

凍結 fixture `tests/floor_cases.yaml`（134 case）と、`design-intent/` を丸ごと写す歯（entrance・face・site・serve・floor_cases・render・freeze）は、写しに intake.yaml が自然に入る＝fixture も歯も期待も変えない。

(d) 歯 `crates/folio/tests/intake.rs`（binary 経由・歯の関数名はすべて intake を含める＝verify の filter 語）。verify の各行は歯の file を `--test <name>` で名指し filter 語を末尾に置く形（器の scope の旗）で、歯の置き場の解決を write-set の歯の file 6 本（intake・check・refs・vocab・adr・link・anchor）に狭める（filter 語が fn 名の一致で他の file へ広がるのを避ける・admin の実測 2026-09-18）。src の unit の歯は契約の verify の外だが common-verify（workspace 全体）で回る。入力は便 12 の歯と同じ作り（`design-intent/` を丸ごと一時 dir へ写し git の 1 commit にしてから intake.yaml に変異を 1 つ当てて `folio check --dir` を回す）。
- 写しそのまま = 終了 0。
- intake.yaml を消す = 終了 2 ∧ 出力に「intake.yaml: 正本が無い」。
- 最上位に未知の節を足す = 終了 1 ∧「未知の節」。
- questions の 1 つ目の ask を空にする = 終了 1。
- questions の 1 つ目の yes を targets に無い id にする = 終了 1 ∧「一覧に無い」。
- targets の 1 つ目の id を棚に無い id にする = 終了 1 ∧「一覧に無い」。
- questions の 1 つ目の recommend を values に無い値にする = 終了 1 ∧「一覧に無い」。
- questions に 6 つ目の行を足す = 終了 1 ∧「上限」。
- questions に同じ id の行をもう 1 つ足す = 終了 1 ∧「重複」（上限にも触れるので違反 2 = この歯だけ件数 2）。
- questions の 1 つ目の why に語彙に無い英字の語を 1 つ足す = 終了 1 ∧「語彙に無い英字の語」。語彙に在る語（folio）を足しても終了 0。
- questions を欄の一覧でない値にする = 終了 2。
- 同じ表に同じキーを 2 度書く = 終了 1 ∧「重複キー」。
違反の歯は、変異が 1 つなら違反の件数が 1 であること（出力の件数の表示）も確かめる（重複 id の歯だけ 2）。unit の歯（`src/intake.rs` の中・名に intake を含む）: 上限の定数が 5・行き先の集合の作り方（棚の documents に inject を足す）。

(e) 便 17 までの形との接続: 新規は `crates/folio/src/intake.rs`・歯 `crates/folio/tests/intake.rs`・fixture の intake.yaml 17 本。`crates/folio/src/check.rs` で変えるのは 4 つだけ: 正本の一覧の定数（5 本 → 6 本）／読んだ正本を持つ型（相談窓口を足す）／読みの取り出し（(a) のとおり 6 本に合わせる）／検査の並びへの 1 行（相談窓口の検査を呼ぶ）。`crates/folio/src/main.rs` は `mod intake;` の 1 行だけ。`entrance.rs`・`vocab.rs`・`yaml.rs`・他の src・`build.rs`・`Cargo.toml`・`Cargo.lock`・`scripts/`・`.github/workflows/`・`design-intent/` は触らない。既存の歯の file（check・refs・vocab・adr・link・anchor）は write-set に在るが本文も期待も変えない（fixture に intake.yaml を足しても期待不変であることを verify で回す）。外部 crate は増えない。正規表現は使わない。size は S = 中身を変える既存の file 1 本あたりの増分の見積（`check.rs` の増分は定数 1 つ・型の欄 1 つ・取り出し 1 行・呼び出し 1 行で 20 行未満・`main.rs` は 1 行・新規と fixture は増分に数えない）。

## 2. 範囲

- 入れる: `folio check` の 6 本目の正本（読み・節の一覧・非空・行 id の重複・行き先の解決・上限・相談窓口の文の語彙）・歯・既存の fixture 17 組への最小の intake.yaml。
- 入れない: 命令 `folio intake`（質問の提示・回答の受け取り・支度表の生成・後続の便）・支度表の正本 intake-sheet.yaml の形の検査（支度表の便で）・入口の面の支度表の節・intake.yaml の中身の変更・発効の形（承認欄の検査）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| load6 | 読み | 正本の一覧を 6 本にし、同じ読み手で intake.yaml を読む |
| shape | 形の検査 | 節の一覧・非空・行 id の重複・質問の上限 |
| resolve | 行き先の解決 | targets を棚と inject に・with を annexes に・yes / no を targets に・recommend と default を values に解く |
| words | 相談窓口の文の語彙 | 便 4 の関数で英字の語を数える（種別 intake） |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は `.vessel.toml` の common-verify（便 0〜便 17 の歯は期待不変で全部回る）。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "s"
title = "相談窓口の正本 intake.yaml を folio check の床に入れる（正本 6 file）"
req = ["FR1", "FR5"]
section = "1"
write-set = ["+crates/folio/src/intake.rs", "+crates/folio/tests/intake.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/anchor.rs", "+tests/fixtures/check/dup-key/intake.yaml", "+tests/fixtures/check/empty-field/intake.yaml", "+tests/fixtures/check/missing-file/intake.yaml", "+tests/fixtures/check/unknown-section/intake.yaml", "+tests/fixtures/refs/bad-counts/intake.yaml", "+tests/fixtures/refs/dangling-id/intake.yaml", "+tests/fixtures/refs/orphan-rule/intake.yaml", "+tests/fixtures/vocab/exemptions/intake.yaml", "+tests/fixtures/vocab/unknown-word/intake.yaml", "+tests/fixtures/adr/effective-no-approval/intake.yaml", "+tests/fixtures/adr/schema-drift/intake.yaml", "+tests/fixtures/adr/two-adopted/intake.yaml", "+tests/fixtures/link/adr-id-missing/intake.yaml", "+tests/fixtures/link/amended-by-orphan/intake.yaml", "+tests/fixtures/link/retreat-kind-drift/intake.yaml", "+tests/fixtures/anchor/no-anchor/intake.yaml", "+tests/fixtures/anchor/root-digest-drift/intake.yaml"]
verify = ["cargo nextest run -p folio --test intake intake", "cargo nextest run -p folio --test check check", "cargo nextest run -p folio --test refs refs", "cargo nextest run -p folio --test vocab vocab", "cargo nextest run -p folio --test adr adr", "cargo nextest run -p folio --test link link", "cargo nextest run -p folio --test anchor anchor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "intake の歯（写しそのまま・intake.yaml 無し・未知の節・空の欄・行き先 4 つ・上限・行 id の重複・語彙 2 つ・型違い・重複キー・unit）が緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、便 6 の歯 link が期待不変で緑、便 7 の歯 anchor が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
