# 設計: 便 6 — `folio check` に判断の記録と正本の突き合わせ（参照・双方向・本文の英字語）を足す

- 要件: FR5（3 値・実行できなかった検査を合格にしない）/ NFR3（参照は必ずつながる・正本の内部の整合）
- 条: A-2.1（条文の改訂は判断の記録と承認を伴う）/ N-4.1（判断の記録・裁定 id・承認の逐語を欠く改訂を拒む）/ P-5.2（文書と台帳からは規則を id で参照する）/ P-12.1（承認は rules 行 R-8 が指す対話面を通る）
- rules 行: R-4（参照 id の未解決 0・行と母集団は変えない）/ R-9（本文の英字語・行と母集団は変えない）。判断の記録の側は欄の決まり（adr/schema.yaml の basis_note・prose_note）の規則として同じ式を課す。
- 判断の記録: ADR-1（欄の決まり・双方向の来歴）。便 5（f2-648.15）の後に直列で置く。
- 裁定: 持ち主 2026-09-17「進めて良い」と前もっての確認（f2-648 notes・毎便問わない）。
- 置き場: この文書は folio2 の設計ノート（M1 で YAML 正本へ移す）。契約表は末尾の区間。審査の材料は行 g が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は `+`・新規 dir は file を 1 本ずつ列挙）。fixture は最小の手書き（正本の写しは使わない）。

## 1. 目的と中身

便 5 の `folio check`（正本 4 file・参照 id・逆参照・件数・語彙 R-9・判断の記録の欄の決まり・3 値・終了コード 合格 0 / 違反 1 / 読めない 2）に、判断の記録（adr/）と正本 4 file・凍結 anchor の列を突き合わせる検査を足す。day-1 の床 `scripts/check_draft.py` の adr・refs・vocab の節のうち便 5 が残した突き合わせを同じ式で写し、この便では床の script を触らない。凍結 anchor の列そのもの（digest の検算・索引・版管理との照合・現行の写しとの一致）は便 7。

床の定数の所在: 便 5 の (b) の床の定数は `crates/folio/src/adr.rs` の中の非公開の定数（FLOOR・同じ file の型 Floor と Keys で表す・外へ出ている関数は check_adr の 1 本）。便 6 は同じ file で可視性を crate の中へ広げるか読み口の関数を足して `link.rs` から読む（adr.rs は write-set に在る・他の file は要らない・定数の値は変えない）。

(a) 撤退条件の種類の一致。床の定数の enums.retreat_kind と、憲法の schema 節の enums.retreat_kind を比べ、一覧の長さか要素の字面か順が違えば 1 違反（種別 adr・憲法が正）。憲法の schema.enums.retreat_kind が無いか一覧でなければ「まだ分からない」。

(b) 対話面の行。床の定数の enums.surface の各 id（R-8）が rules の thresholds か discipline の行の id に無ければ 1 違反ずつ（種別 adr）。

(c) 改訂の範囲。憲法の schema.amendment_scope（無ければ空の一覧）が床の定数の anchor.scope_minimum（schema・precedence・articles）を含まなければ 1 違反（種別 anchor）。憲法の条 id が amendment_scope の節名と同じなら 1 違反ずつ（種別 schema）。

(d) 判断の記録の id の参照。id の形 = ADR- の後に 1〜9 で始まる数字列が続き、その直前が英字・数字・「-」でなく、直後が英字・数字でないもの（4 桁の ADR-0047 は 1〜9 で始まらないので形に当たらず、外部の参照として数えない）。母集団と種別: 正本 4 file は便 1 の参照 id と同じ母集団（憲法は schema 節を除き meta の changes_from で始まる欄を除く・rules は schema 節を除く・語彙と要件書は全欄）で種別 A-2、判断の記録は各 file の全欄（欄の道つき）と adr/schema.yaml の plain で種別 adr。形に当たる語が adr/ の判断の記録の id に無ければ 1 違反ずつ（文言に欄の道と id）。加えて判断の記録の全欄と adr/schema.yaml の plain に出る内部 3 空間の id（便 1 の refs と同じ形 = P・A・N + 「-」+ 数字列 + 任意で「.」+ 数字列／FR・NFR・AC・CON・GOAL + 数字列／R・D + 「-」+ 数字列・前後が英字・数字・「-」でない）が、便 1 の解決先（要件書の id・条 id と規範文 id・rules 行 id）にも凍結 anchor の列に在った id（次の (e)）にも無ければ 1 違反ずつ（種別 adr）。

(e) 凍結 anchor の列に在った id。`<dir>/anchors/` が dir なら、その直下の `constitution-` で始まる `.yaml` を名前順に読み、読めるものの content.articles の各条の id と各規範文の id、projection.scope の各節名を集める（読めない file は飛ばす・列の真偽は便 7）。anchors/ が無ければ空。amends の対象の集合 = 現行の条 id ∪ amendment_scope の節名 ∪ 列に在った条 id（「.」を含まない）∪ 列に在った節名。各判断の記録の amends の各項の target がこの集合に無ければ 1 違反（種別 A-2・文言に判断の記録の id と target）。

(f) 判断の記録の本文の英字語。便 4 の語彙の検査と同じ切り出し・既知の集合・免除 5 形を、判断の記録の本文の対に掛ける: 各判断の記録の title・context・decision・plain（場所 = id + 欄名）・retreat.condition（id + retreat）・options の各案の name・text・reason を空白 1 つで繋いだもの（id + option + 案の id）・consequences の各要素（id + consequences）・adr/schema.yaml の plain（adr/schema.yaml plain）。免除されない語 1 つにつき (小文字の語, 場所) で 1 違反（種別 adr・文言に 語彙に無い英字の語 と「日本語（原語）の形で書く」）。便 4 の `crates/folio/src/vocab.rs` の切り出し・既知の集合・免除の関数を crate の中で共有し（可視性を広げるだけ・式は変えない）、R-9 の行の母集団（正本 4 file）は広げない。

(g) 改訂来歴の双方向。憲法の各条の amended_by（無ければ 0 件・一覧でなければ 1 違反 A-2）の各項について: 欄が床の定数の amended_by_entry の required（adr・date・approved_by・ruling・previous_text・rationale）と一致しない（A-2）／approved_by・ruling・previous_text・rationale が空（N-4）／date が年 4 桁-月 2 桁-日 2 桁でない（N-4）／ruling に台帳 id の並び（便 5 の (d) と同じ）が無い（N-4）／adr の判断の記録が無い（N-4・次の照合は飛ばす）／adr が発効した判断（status が accepted か retired で approval が表）でない（N-4・飛ばす）／approved_by がその判断の approval.who と違う（N-4）／previous_text が、その判断の amends のうち target がこの条である項の previous_text のどれとも一致しない（A-2）。逆向き: 発効した各判断の amends の target のうち現行の条 id であるものについて、その条の amended_by にその判断の id を adr に持つ項が無ければ 1 違反（A-2）。

(h) 違反の記録型は便 4・便 5 と同じ: `crates/folio/src/verdict.rs` の Report の関数 violation は種別を文字列で受け、既存の呼び手（check・refs・vocab・adr）に種別の閉じた型も網羅の分岐も無い。便 6 が使う種別（adr・A-2・N-4・anchor・schema）はすべて文字列 1 つで閉じ、`verdict.rs` は触らない。判定は便 5 までと同じ 3 値。正本（main）では違反 0（day-1 の床の実測・2026-09-17: 判断の記録 4 本はすべて発効・amends 空・amended_by 0 件・anchor は v1.0 の 1 本）。

便 0〜便 5 の fixture のうち 4 file が読める 11 組（`tests/fixtures/check/` の dup-key・empty-field・unknown-section、`tests/fixtures/refs/` の bad-counts・dangling-id・orphan-rule、`tests/fixtures/vocab/` の unknown-word・exemptions、`tests/fixtures/adr/` の schema-drift・two-adopted・effective-no-approval）は、憲法の schema 節に enums と amendment_scope を持たず rules に R-8 の行を持たないので、(a)(b)(c) を通すと「まだ分からない」と違反が立ち、各歯の期待（終了コード 1・違反 1 件）が崩れる。便 6 はこの 11 組の `constitution.yaml` の schema 節に enums: {retreat_kind: [spike, measure, ruling]} と amendment_scope: [schema, precedence, articles] を足し（英字語 spike・measure・ruling・schema・precedence・articles は便 4 の R-9 の母集団に入らない: `crates/folio/src/vocab.rs` の母集団は憲法では各条の title・plain・規範文の text と前文の text・plain だけで schema 節を数えない・admin の実測 2026-09-17 main deb1f2a。判断の記録の欄 ruling・approved_by 等も便 4 の母集団外で、便 6 の (f) が数えるのは本文の欄だけ）、rules 行の article が指す条の relations.rules に R-8 を足し、`rules.yaml` の thresholds に行 {id: R-8, article: <条>, what: 持ち主との対話面, value: 対話面} を足す。条は組ごとに違う（admin の実測 2026-09-17 main 65a0dbe）: `tests/fixtures/check/` の 3 組（dup-key・empty-field・unknown-section）は条が P-1 だけで P-2 が無いので article: P-1 とし P-1 の relations.rules に R-8 を足す。他の 8 組（refs 3 組・vocab 2 組・adr 3 組）は P-1・P-2 を持つので article: P-2 とし P-2 の relations.rules に R-8 を足す（P-2 が無い組に article: P-2 を書くと便 1 の逆参照が「条に無い」を足して check の歯が赤になる）。`refs/orphan-rule/` は P-2 の relations.rules が空で R-3 の孤児が期待の 1 件なので、R-8 を足しても R-3 は孤児のまま期待は 1 件で不変。11 組の thresholds 行の欄集合は id・article・value・what の 4 欄で、便 0 の欄の非空が見るのは id・article・what の 3 欄（同実測）＝R-8 の行は 4 欄を非空の日本語で揃える。便 1 の件数の検査は憲法の meta.counts と条の tier の実数だけを比べるので rules の行を足しても動かない（同実測）。fixture の R-8 の行の what には英字語を含めない（各組の vocabulary.yaml の identifiers は folio と opus だけで planner を持たず、英字語を書くと便 4 の R-9 が 1 件増えて各歯の期待が崩れる・admin の実測 2026-09-17: 12 組の vocabulary.yaml に planner は 0 件）。便 6 の (b) が見るのは行 id R-8 の存在だけで、what の字面は材料にしない。11 組の `adr/ADR-1.yaml` の (d)(e)(g) への影響（同実測）: basis は [P-1] だけで P-1 は 11 組すべてに在る（(d) の解決 0）・amends 無し（(e) 0）・憲法側の amended_by 無し（(g) 0）・retreat.kind は ruling（(a) は enums を足せば 0）・status は proposed が 10 組と accepted が 1 組（effective-no-approval・approval 無し＝便 5 の歯の期待の 1 件そのもの）・supersedes 無し。よって増えるのは (f) の英字語 file の 1 件だけで、日本語に直せば 0。便 5 の fixture `adr/ADR-1.yaml`（11 組）は**すべて**本文（title「判断の記録は 1 判断を 1 file に置く」・decision・options の text）に英字語 file を持ち、各組の語彙（identifiers は folio と opus だけ）に無いので、(f) を通すと各組に違反が 1 件増える（admin の実測 2026-09-17・main d1ed6a0）。便 6 は 11 組すべての `adr/ADR-1.yaml` の file を日本語（ファイル）に直す（他の英字語は無い・ADR-1.yaml 以外の判断の記録の file は無く、two-adopted の「採用 2 つ」も ADR-1.yaml 1 本の中の options）。11 組の `adr/schema.yaml` の plain は「判断の記録の書き方の決まりです。」の 1 文で英字語 0・id 0（同実測）なので (d)(f) の母集団に入れても違反は増えず、write-set の外のまま触らない。各歯の期待は変えない。`tests/fixtures/check/missing-file/` は判断の記録の検査に届かないので触らない。便 0・便 1・便 4・便 5 の歯（`crates/folio/tests/check.rs`・`refs.rs`・`vocab.rs`・`adr.rs`）は verify で回すために write-set に在るが触らない（本文も期待も変えない）。

`folio check` 自身の歯（`crates/folio/tests/link.rs`・binary 経由・`tests/refs.rs` と同じ形で種別と文言を名指して 1 違反を見る）の fixture は `tests/fixtures/link/` の 3 組。各組は上の 11 組と同じ最小の 6 file（4 file + adr/schema.yaml + adr/ADR-1.yaml・違反 0）に変異を 1 つ当てる: `retreat-kind-drift/` = 憲法の schema.enums.retreat_kind を [spike, measure] にする → 1（種別 adr・文言に retreat_kind）／`adr-id-missing/` = ADR-1 の basis に ADR-9 を足す → 1（種別 adr・文言に ADR-9）／`amended-by-orphan/` = 憲法の P-1 に amended_by 1 項 {adr: ADR-1, date: 2026-09-17, approved_by: 持ち主, ruling: f2-1, previous_text: 前の文, rationale: 理由}（ADR-1 は proposed のまま）→ 1（種別 N-4・文言に 発効していない）。parity の入力にはしない。

突き合わせの歯（crates/folio/tests/parity.rs（2026-09-18 に scripts/retired/ へ退役）・便 5 の 17 入力に足す・写し全部 + git 1 commit + 変異 1 つ・床と folio の終了コードの一致）: (18) `constitution.yaml` の schema.enums.retreat_kind から ruling を外す → 1／(19) `adr/ADR-1.yaml` の basis に ADR-9 を足す → 1／(20) `adr/ADR-1.yaml` の title の末尾に zzzz を足す → 1／(21) `srs.yaml` の FR1 の plain の末尾に ADR-9 を足す → 1／(22) `adr/ADR-1.yaml` の amends に 1 項 {target: P-99, field: title, version: v1.1, previous_text: 前, new_text: 今} を足す → 1。既存の 17 入力は変えない。

便 5 が置いた形との接続: `crates/folio/src/main.rs` に `mod link;` を足し、実装は新規 `crates/folio/src/link.rs` に置き、`check.rs` の `check_dir` から `adr` の次に呼ぶ（4 file の Node・`<dir>` の path・便 5 の `adr.rs` が読んだ判断の記録（id・status・approval・amends・options 等の Node）を渡す。`adr.rs` は読んだ判断の記録を crate の中へ返す形に広げてよいが、便 5 の検査の式は変えない）。YAML は既存の `crates/folio/src/yaml.rs` の読み手を使う。3 値は既存の `crates/folio/src/verdict.rs` の型を使い、変えない。便 1 の `refs.rs` は触らない（凍結 anchor の列に在った id を 4 file の解決先へ足すのは便 7）。外部 crate は増やさない（clap と yaml-rust2 のまま・`Cargo.toml` と `Cargo.lock` は触らない）。正規表現は使わない。便 2・便 3 の歯と fixture は触らない。

## 2. 範囲

- 入れる: `crates/folio/src/link.rs`（(a)〜(g)）と `check.rs` からの呼び出し・`vocab.rs` の関数の共有・`adr.rs` の読んだ判断の記録の受け渡し・歯 `link.rs`・parity の 5 入力・fixture 3 組・既存 fixture 11 組への enums / amendment_scope / R-8 の追記。
- 入れない: 凍結 anchor の列の真偽（digest・索引・previous の列・版管理との照合・現行の写しと最新 anchor の一致・列の区間の amends の消し込み・改番と番号の再利用）・4 file の参照 id の解決先へ列に在った id を足すこと・`--freeze-anchor`・`--emit-amends`・`scripts/check_draft.py` の変更。すべて便 7。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| enums | 値域の一致 | 床の定数と憲法の enums・rules の対話面の行・改訂の範囲 |
| adr-refs | 判断の記録の参照 | ADR id と内部 3 空間の id を 4 file と判断の記録の本文で解く |
| history-ids | 列に在った id | anchors/ の各 anchor から条 id・規範文 id・節名を集める |
| adr-words | 判断の記録の英字語 | 便 4 の式を判断の記録の本文に掛ける |
| lineage | 改訂来歴 | amended_by ⇔ amends の双方向と承認者・previous_text の一致 |

## 4. 検査（歯）

§1 のとおり（link の歯 3 組・便 0 / 便 1 / 便 4 / 便 5 の歯そのまま・parity 22 入力）。共通の検証は `.vessel.toml` の common-verify。

## 5. 依存

外部 crate は増やさない（clap / yaml-rust2 のまま）。parity の歯は CI が既に持つ Python 3 と pyyaml を使う。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "g"
title = "folio check に判断の記録と正本の突き合わせ（参照・双方向・本文の英字語）を足す"
req = ["FR5", "NFR3"]
section = "1"
write-set = ["+crates/folio/src/link.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/src/vocab.rs", "crates/folio/src/adr.rs", "+crates/folio/tests/link.rs", "~crates/folio/tests/parity.rs", "crates/folio/tests/check.rs", "crates/folio/tests/refs.rs", "crates/folio/tests/vocab.rs", "crates/folio/tests/adr.rs", "tests/fixtures/check/dup-key/constitution.yaml", "tests/fixtures/check/dup-key/rules.yaml", "tests/fixtures/check/dup-key/adr/ADR-1.yaml", "tests/fixtures/check/empty-field/constitution.yaml", "tests/fixtures/check/empty-field/rules.yaml", "tests/fixtures/check/empty-field/adr/ADR-1.yaml", "tests/fixtures/check/unknown-section/constitution.yaml", "tests/fixtures/check/unknown-section/rules.yaml", "tests/fixtures/check/unknown-section/adr/ADR-1.yaml", "tests/fixtures/refs/bad-counts/constitution.yaml", "tests/fixtures/refs/bad-counts/rules.yaml", "tests/fixtures/refs/bad-counts/adr/ADR-1.yaml", "tests/fixtures/refs/dangling-id/constitution.yaml", "tests/fixtures/refs/dangling-id/rules.yaml", "tests/fixtures/refs/dangling-id/adr/ADR-1.yaml", "tests/fixtures/refs/orphan-rule/constitution.yaml", "tests/fixtures/refs/orphan-rule/rules.yaml", "tests/fixtures/refs/orphan-rule/adr/ADR-1.yaml", "tests/fixtures/vocab/unknown-word/constitution.yaml", "tests/fixtures/vocab/unknown-word/rules.yaml", "tests/fixtures/vocab/unknown-word/adr/ADR-1.yaml", "tests/fixtures/vocab/exemptions/constitution.yaml", "tests/fixtures/vocab/exemptions/rules.yaml", "tests/fixtures/vocab/exemptions/adr/ADR-1.yaml", "tests/fixtures/adr/schema-drift/constitution.yaml", "tests/fixtures/adr/schema-drift/rules.yaml", "tests/fixtures/adr/schema-drift/adr/ADR-1.yaml", "tests/fixtures/adr/two-adopted/constitution.yaml", "tests/fixtures/adr/two-adopted/rules.yaml", "tests/fixtures/adr/two-adopted/adr/ADR-1.yaml", "tests/fixtures/adr/effective-no-approval/constitution.yaml", "tests/fixtures/adr/effective-no-approval/rules.yaml", "tests/fixtures/adr/effective-no-approval/adr/ADR-1.yaml", "+tests/fixtures/link/retreat-kind-drift/constitution.yaml", "+tests/fixtures/link/retreat-kind-drift/rules.yaml", "+tests/fixtures/link/retreat-kind-drift/vocabulary.yaml", "+tests/fixtures/link/retreat-kind-drift/srs.yaml", "+tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "+tests/fixtures/link/retreat-kind-drift/adr/ADR-1.yaml", "+tests/fixtures/link/adr-id-missing/constitution.yaml", "+tests/fixtures/link/adr-id-missing/rules.yaml", "+tests/fixtures/link/adr-id-missing/vocabulary.yaml", "+tests/fixtures/link/adr-id-missing/srs.yaml", "+tests/fixtures/link/adr-id-missing/adr/schema.yaml", "+tests/fixtures/link/adr-id-missing/adr/ADR-1.yaml", "+tests/fixtures/link/amended-by-orphan/constitution.yaml", "+tests/fixtures/link/amended-by-orphan/rules.yaml", "+tests/fixtures/link/amended-by-orphan/vocabulary.yaml", "+tests/fixtures/link/amended-by-orphan/srs.yaml", "+tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "+tests/fixtures/link/amended-by-orphan/adr/ADR-1.yaml"]
verify = ["cargo nextest run -p folio link", "cargo nextest run -p folio check", "cargo nextest run -p folio refs", "cargo nextest run -p folio vocab", "cargo nextest run -p folio adr", "cargo nextest run -p folio parity", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "link の歯が fixture 3 組で §1 の 1 違反を名指して緑、便 0 の歯 check が期待不変で緑、便 1 の歯 refs が期待不変で緑、便 4 の歯 vocab が期待不変で緑、便 5 の歯 adr が期待不変で緑、parity の 22 入力が床と一致して緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
