# 設計: 便 5 — `folio check` に判断の記録（adr/）の欄の決まりの検査を足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（参照は必ずつながる・正本の内部の整合）
- 条: P-8.1（取り消しにくい判断には撤退条件を同時に書く）/ N-4.1（条文の改訂が判断の記録・裁定 id・承認の逐語を欠くなら拒む）/ P-12.2（承認は逐語と日付を添えて承認欄に記帳する）/ N-3.1（規則の例外機構を足さない＝欄の決まりは床の定数の写し）
- 判断の記録: ADR-1（判断の記録の正本は 1 判断 = YAML 1 file・欄の決まりは adr/schema.yaml・床の定数の写し）。ADR-1 の帰結「M0 で folio が判断の記録の型を実装の側で持つ」の 1 歩目。便 4（f2-648.14）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes・毎便問わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 f が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`・新規 dir は file を 1 本ずつ列挙）。fixture は最小の手書き（正本の写しは使わない）。

## 1. 目的と中身

便 4 の `folio check`（正本 4 file の形・参照 id・逆参照・件数・語彙 R-9・3 値・終了コード 合格 0 / 違反 1 / 読めない 2）に、判断の記録（`design-intent/adr/`）の欄の決まりの検査を足す。day-1 の床 `scripts/check_draft.py` の adr の節のうち **判断の記録の file と欄の決まりの file だけで閉じる検査**を同じ式で写し、この便では床の script を触らない。憲法・rules・anchor と突き合わせる検査（amends の対象の実在・amended_by との双方向・対話面 R-8 の行の実在・撤退条件の種類の値域と憲法の enums の一致・判断の記録の id の参照の解決・判断の記録の本文の英字語・凍結 anchor の列）は便 6 以降（母集団は広げない）。

(a) 置き場と欄の決まりの file。`<dir>/adr/` が無い・symlink・dir でない → 「まだ分からない」。`adr/schema.yaml` が無い・読めない・節が meta / schema / plain 以外を持つ・schema 節が表でない → 「まだ分からない」（欄の決まりが読めなければ判断の記録は測れない）。schema 節から名前が `_note` で終わる欄を（入れ子の表の中も含めて）落としたものが、床の定数（次の (b)）と 1 字も違わないこと。違いは欄の道ごとに 1 違反（種別 adr・未知の欄／欠落／値の違い）。meta の decided_by が空か無い → 1 違反、その各要素が adr/ の判断の記録の id に実在しなければ 1 違反ずつ。

(b) 床の定数（Rust の中の型付きの定数・adr/schema.yaml の schema 節と同じ形・値は day-1 の床の FLOOR と同じ）: id_pattern = `^ADR-[1-9][0-9]*$`／date_format = `^\d{4}-\d{2}-\d{2}$`／ruling_pattern = `[a-z]\d-[0-9a-z]+(\.\d+)?`／owner = 持ち主／required = id・title・status・date・context・decision・options・basis・retreat・plain／optional = amends・grill・approval・consequences・supersedes・superseded_by・note／non_empty = title・context・decision・plain／enums = status: proposed・accepted・retired／verdict: adopted・rejected／retreat_kind: spike・measure・ruling／approver: 持ち主・planner 席／surface: R-8／effective_status = accepted・retired／option = required: id・name・text・verdict・reason, optional: 空／options_rule = min: 2, adopted: 1／retreat = required: kind・condition, optional: 空／amends_entry = required: target・field・version・previous_text・new_text, optional: 空, new_article_marker: （新設）, deleted_marker: （削除）, empty_marker: （空）／grill = required: when・who・where・summary, optional: 空／approval = required: who・date・ruling・verbatim・surface, optional: 空／amended_by_entry = required: adr・date・approved_by・ruling・previous_text・rationale, optional: 空／anchor = dir: anchors, file_name: constitution-<version>.yaml, index_file: index.yaml, first_version: v1.0, root_digest: acb52acd04b5d3a1feaf9ad5f0138f7614ce31964144b46ead914bde86e866ed, version_pattern: `^v[0-9]+\.[0-9]+$`, digest_algo: sha256-json-1, file_keys: kind・digest_algo・version・previous・projection・meta_approval・approvals・content・digest, projection_article_fields: id・title・tier・binds・statements, statement_fields: id・text・pattern・strength, scope_minimum: schema・precedence・articles。定数の欄の順は adr/schema.yaml の schema 節の順と同じにし、比べ方は 表 = 欄の集合が同じで各欄を再帰／一覧 = 長さが同じで各要素を順に／値 = yaml の値の字面（引用符を除く）が同じ、で、表と一覧と値の種類が違えば違い（2.0 と 2・true と 1 は字面が違うので違い・空の一覧は空の一覧とだけ同じ）。パターンの文字列は定数として字面で持つだけで、判定は次の (d) の走査で行う（正規表現は使わない）。

(c) 判断の記録の file。`adr/` の直下の `.yaml` を名前順に読み、`schema.yaml` は除く。symlink か、実体が `<dir>` の外を指す file は 1 違反（種別 adr）で読まない。読めない・表でない → 1 違反（種別 adr・判断の記録が欄の表でない）で次へ。id が (b) の id_pattern の形（ADR- の後に 1〜9 で始まる数字列だけ）でなければ 1 違反。file 名の拡張子を除いた部分が id と違えば 1 違反。同じ id が 2 file 以上に在れば 2 つ目以降は 1 違反で読まない。

(d) 各判断の記録の欄（違反は 1 件ずつ・種別は括弧内）: required の欄が無い（adr・欠けた欄の一覧）／required にも optional にも無い欄（adr・未知の欄）／status が enums.status の外（adr）／date が 年 4 桁-月 2 桁-日 2 桁 の形でない（adr）／non_empty の各欄が空（前後の空白を落として空・adr）／options が一覧でない（adr）・案の数が options_rule.min 未満（adr）・各案の欄が option の required と一致しない（adr）・各案の verdict が enums.verdict の外（adr）・各案の name / text / reason が空（adr）・adopted の案の数が options_rule.adopted と違う（adr）／retreat の欄が retreat の required と一致しない（P-8）・kind が enums.retreat_kind の外（P-8）・condition が空（P-8）／basis が空でない一覧でない（adr）・各要素が id の形（P・A・N + 「-」+ 数字列 + 任意で「.」+ 数字列／FR・NFR・AC・CON・GOAL + 数字列／R・D + 「-」+ 数字列／ADR- + 1〜9 で始まる数字列 の全体一致）でない（adr）／amends が在って一覧でない（A-2）・各項の欄が amends_entry の required と一致しない（A-2）・各項の field / version / previous_text / new_text が空（A-2）／status が effective_status に在るのに approval が無い（N-4）／approval が在るとき: 欄が approval の required と一致しない（N-4）・ruling / verbatim が空（N-4）・date の形（N-4）・who が enums.approver の外（N-4）・ruling の中に 小文字の英字 1 字 + 数字 1 字 + 「-」+ 小文字の英字か数字 1 字以上 の並びが無い（N-4・台帳 id）・surface が enums.surface の外（N-4）／grill が在るとき: 欄が grill の required と一致しない（A-2）・when の日付の形（A-2）・who / where / summary が空（A-2）。

(e) 判断の記録どうし。発効した判断 = status が effective_status に在り approval が表であるもの。発効した判断が amends に 1 項以上を持つとき、approval.who が owner と違えば 1 違反（N-4）・grill が表でなければ 1 違反（A-2）。supersedes / superseded_by が在ってその id の判断の記録が無ければ 1 違反ずつ（adr）。status が retired で superseded_by が無い → 1 違反（adr）。superseded_by が在って status が retired でない → 1 違反（adr）。superseded_by の先の supersedes がこの id でない → 1 違反（adr）。supersedes の先の superseded_by がこの id でない → 1 違反（adr）。retired の後継の列（superseded_by をたどる）: 同じ id に戻れば 1 違反（輪・adr）、accepted に着けば良し、retired でも accepted でもない status に着けば 1 違反（adr）、実在しない後継は (e) の先頭で数えてあるので列は止める。

(f) 判定は便 4 までと同じ 3 値（読めない・測れないが 1 つでも在れば「まだ分からない」が先・違反が在れば 不合格・どちらも無ければ 合格）。正本（main）の判断の記録 4 本（ADR-1〜4・すべて accepted・承認欄あり・amends 空）と adr/schema.yaml では違反 0（day-1 の床の実測・2026-09-17）。

便 0・便 1・便 4 の fixture のうち 4 file が読める 8 組（`tests/fixtures/check/` の dup-key・empty-field・unknown-section、`tests/fixtures/refs/` の bad-counts・dangling-id・orphan-rule、`tests/fixtures/vocab/` の unknown-word・exemptions）は `adr/` を持たないので、(a) を通すと「まだ分からない」が立ち、各歯の期待（終了コード 1・違反 1 件）が崩れる。便 5 はこの 8 組に `adr/schema.yaml`（meta = id・version・date・decided_by: ADR-1 の 4 欄と schema = (b) の定数の写し・`_note` の欄は持たない・plain 1 行）と `adr/ADR-1.yaml`（status proposed・案 2 つで adopted 1 つ・basis P-1・retreat kind ruling・approval なし・amends なし＝(d)(e) で違反 0）の最小の 2 file を足し、fixture を便 5 の正本の形に追随させる。`tests/fixtures/check/missing-file/` は constitution.yaml を欠き 4 file の読みで「まだ分からない」になって判断の記録の検査に届かないので足さない。便 0 の歯 `crates/folio/tests/check.rs`・便 1 の歯 `crates/folio/tests/refs.rs`・便 4 の歯 `crates/folio/tests/vocab.rs` は verify で回すために write-set に在るが触らない（本文も期待も変えない）。

`folio check` 自身の歯（`crates/folio/tests/adr.rs`・binary 経由・`tests/refs.rs` と同じ形で種別と文言を名指して 1 違反を見る）の fixture は `tests/fixtures/adr/` の 3 組。各組は上の 8 組と同じ最小の 4 file（違反 0・識別子は folio と opus）+ `adr/schema.yaml` + `adr/ADR-1.yaml` の 6 file で、変異を 1 つ当てる: `schema-drift/` = adr/schema.yaml の options_rule.min を 3 にする → 1（種別 adr・文言に options_rule.min）／`two-adopted/` = ADR-1 の案 2 つを両方 adopted にする → 1（種別 adr・文言に 採用の案）／`effective-no-approval/` = ADR-1 の status を accepted にし approval を持たない → 1（種別 N-4・文言に approval）。parity の入力にはしない。

突き合わせの歯（crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役）・便 4 の 13 入力に足す・写し全部 + git 1 commit + 変異 1 つ・床と folio の終了コードの一致）: (14) `adr/ADR-1.yaml` の retreat の condition を空にする → 1／(15) `adr/ADR-1.yaml` の rejected の案 1 つを adopted にする → 1／(16) `adr/schema.yaml` の options_rule の min を 3 にする → 1／(17) `adr/ADR-1.yaml` の approval の行を消す（accepted のまま）→ 1。既存の 13 入力は変えない。

便 4 が置いた形との接続: `crates/folio/src/main.rs` に `mod adr;` を足し、実装は新規 `crates/folio/src/adr.rs` に置き、`check.rs` の `check_dir` から `vocab` の次に呼ぶ（`<dir>` の path と Report を渡す・4 file の Node は渡さない）。YAML は既存の `crates/folio/src/yaml.rs` の読み手を使う（欄の取り出しの補助を足してよい）。3 値は既存の `crates/folio/src/verdict.rs` の型を使い、変えない（違反の種別は文字列で渡す）。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: `crates/folio/src/adr.rs`（(a)〜(e)）と `check.rs` からの呼び出し・歯 `adr.rs`・parity の 4 入力・fixture 3 組・既存 fixture 8 組への adr/ 2 file。
- 入れない: amends の対象の実在・amended_by との双方向・対話面 R-8 の行の実在・retreat_kind と憲法 enums の一致・判断の記録の id の参照の解決（4 file と判断の記録の本文）・判断の記録の本文の英字語（R-9 の欄の決まり側）・凍結 anchor の列（digest・索引・版管理との照合）・`--freeze-anchor`・`--emit-amends`・`scripts/check_draft.py` の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の定数 | 欄の決まりの閾値・値域・置き場を型付きの定数で持ち、schema.yaml の写しと比べる |
| record | 判断の記録の読み | adr/ の各 file を読み、id・file 名・重複・symlink を見る |
| fields | 欄の検査 | required / optional / non_empty / enums / options / retreat / basis / amends / approval / grill |
| chain | 判断どうし | 発効の条件・supersedes の双方向・retired の後継の列 |

## 4. 検査（歯）

§1 のとおり（adr の歯 3 組・便 0 / 便 1 / 便 4 の歯そのまま・parity 17 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "f"
title = "folio check に判断の記録（adr/）の欄の決まりの検査を足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/adr.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/src/yaml.rs", "+crates/folio/tests/adr.rs", "crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "+tests/fixtures/check/dup-key/adr/schema.yaml", "+tests/fixtures/check/dup-key/adr/ADR-1.yaml", "+tests/fixtures/check/empty-field/adr/schema.yaml", "+tests/fixtures/check/empty-field/adr/ADR-1.yaml", "+tests/fixtures/check/unknown-section/adr/schema.yaml", "+tests/fixtures/check/unknown-section/adr/ADR-1.yaml", "+tests/fixtures/refs/bad-counts/adr/schema.yaml", "+tests/fixtures/refs/bad-counts/adr/ADR-1.yaml", "+tests/fixtures/refs/dangling-id/adr/schema.yaml", "+tests/fixtures/refs/dangling-id/adr/ADR-1.yaml", "+tests/fixtures/refs/orphan-rule/adr/schema.yaml", "+tests/fixtures/refs/orphan-rule/adr/ADR-1.yaml", "+tests/fixtures/vocab/unknown-word/adr/schema.yaml", "+tests/fixtures/vocab/unknown-word/adr/ADR-1.yaml", "+tests/fixtures/vocab/exemptions/adr/schema.yaml", "+tests/fixtures/vocab/exemptions/adr/ADR-1.yaml", "+tests/fixtures/adr/schema-drift/constitution.yaml", "+tests/fixtures/adr/schema-drift/rules.yaml", "+tests/fixtures/adr/schema-drift/vocabulary.yaml", "+tests/fixtures/adr/schema-drift/srs.yaml", "+tests/fixtures/adr/schema-drift/adr/schema.yaml", "+tests/fixtures/adr/schema-drift/adr/ADR-1.yaml", "+tests/fixtures/adr/two-adopted/constitution.yaml", "+tests/fixtures/adr/two-adopted/rules.yaml", "+tests/fixtures/adr/two-adopted/vocabulary.yaml", "+tests/fixtures/adr/two-adopted/srs.yaml", "+tests/fixtures/adr/two-adopted/adr/schema.yaml", "+tests/fixtures/adr/two-adopted/adr/ADR-1.yaml", "+tests/fixtures/adr/effective-no-approval/constitution.yaml", "+tests/fixtures/adr/effective-no-approval/rules.yaml", "+tests/fixtures/adr/effective-no-approval/vocabulary.yaml", "+tests/fixtures/adr/effective-no-approval/srs.yaml", "+tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "+tests/fixtures/adr/effective-no-approval/adr/ADR-1.yaml"]
verify = ["cargo nextest run -p folio adr", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "adr の歯が fixture 3 組で §1 の 1 違反を名指して緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、parity の 17 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
