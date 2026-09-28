# 設計: 便 196 — 器の導出 file の need / shape で folio2 が意味を知る値と、id の一覧の anchor の形を生成区間へ写す（台帳 f2-648.169・f2-648.126・M）

- 要件: FR19（欄の決まりの生成区間は床の定数から決定的に導出する）・FR10（契約表の欄は器の導出 file から読む）。規範文・確かめ方・受入基準は変えない。
- 条: P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ導出する）・P-6.3・P-7.1（id の一覧の anchor が比較元）。
- 出所: 規則の表の行 D-11。台帳 **f2-648.169**（便 120 の検証の申し送り＝値域の定数 EXTERNAL_NEED / SHAPE の写しが無く、注が実装と食い違う）と **f2-648.126**（便 88 の後続＝id の一覧の anchor の種別と要約値の欄の写しが無い）。
- 承認: 判定の値も式も変えず、既に効いている定数の写しを足し、実装と食い違う注の字を実装に合わせるだけ。行 D-17 の「実装の定数を変えて生成区間の規則を変える」には当たらないと読み、席の裁定で受け付ける（読みが割れたら持ち主の承認の 1 回に載せる・字は変わらない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾。審査の材料は行 `gq` が指す §1 だけ。write-set 13 本（src 3・歯の file 4〔本文不変 1〕・設計文書 2・fixture 4）・新しい file も dir も無い。
- 門: **0（通す）**。本流 44c14ac の組み立てに write-set 13 本を渡した `folio ceiling --gate --dir design-intent --write-set …` の答え。
- 前の便: **base = 本流 44c14ac**（便 185・192〜194 の着地の後）。数は base の写しの実測（参考値・行 D-13）。組み直す手順は控え `~/.local/share/folio2/handoff-2026-09-28/copies-scripts/d196/`。
- 見本: origin の枝 `impl/d196`（**1a7c4c7**・本流 44c14ac を merge した commit）。`git diff 44c14ac 1a7c4c7` が便の全体の差分。作業者は write-set の file をこの commit の中身にしてよい。

## 1. 設計

### (a) いま起きていること（参考値・base 44c14ac）

1. **導出 file の値の写しが無く、注が実装と食い違う（.169）。** 設計ノートの床（`note.rs`）は器の導出 file（contracts/schema.toml）の need と shape を、`floor_note.rs` の閉じた一覧 EXTERNAL_NEED（required・optional・conditional）と EXTERNAL_SHAPE（text・list）で見て、無い値は「まだ分からない」にする。ところが欄の決まり `design-intent/design-note/schema.yaml` の生成区間は `value_domains: from-file` だけを持ち、注 external_schema_note は「need / shape の値域は file の値をそのまま受ける」「欄の追加・値域の変更は器の版上げで足り」と書く。器が値域を広げると folio2 の便が要ること（便 120 がその実例）が、写しからは読めない。
2. **id の一覧の anchor の形の写しが無い（.126）。** 条 P-7.1 の再利用と改番の比較元（`anchors/ids-<要件書の版>.yaml`）の種別の値 `ids-anchor` と、節ごとに要約値を取る欄（requirements と nonfunctional は shall か title の先に在る方・acceptance と判断の記録は title）は、`ids.rs` の IDS_KIND と SECTIONS にしか無い。
3. **base の歯。** workspace の nextest 1070 / 1070・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`f196_` と行 id `gq` は 0 件。

### (b) 直す先

1. **`floor_note.rs` の FLOOR（設計ノートの欄の決まり）。** contract_table.external_schema の value_domains の次に `known_values: {need: EXTERNAL_NEED, shape: EXTERNAL_SHAPE}` を足す（値は定数を引く）。value_domains = from-file は「欄の一覧と値域の正本は器の file」の意味で残す。external_schema_note を実装に合わせる: folio2 が判定に使えるのは known_values の値だけで、file の値がその外なら まだ分からない、器が値域を広げたら folio2 の便で known_values に足す（判断の記録は要らない）。欄の追加は器の版上げで足りる、は今の実装どおりで残す。
2. **`check.rs` の SRS_FLOOR（要件書の欄の決まり）。** 末尾に `ids_anchor: {kind: IDS_KIND, sections: {requirements: [shall, title], nonfunctional: [shall, title], acceptance: [title], adr: [title]}}` と注 ids_anchor_note（anchor の置き場・比較元の意味・先に在る方の欄を取る・正本は `ids.rs` の定数でこの節はその写し）を足す。`ids.rs` の 2 本は `pub(crate)` にして引く（check と ids は同じ層 2）。file 名の形は行 D-11 の範囲の外（名）なので型付きの欄にせず、注の字にだけ書く。
3. **設計文書と fixture。** `design-intent/design-note/schema.yaml` と `design-intent/srs.yaml` の生成区間は便の binary の `folio schema --dir design-intent --write` で書き直す（区間の外は 1 byte も変えない）。凍結 anchor `tests/fixtures/schema/note-region.txt`・`srs-region.txt` は導出を使わない script（控えの anchors-196.py）で組み、`tests/schema.rs`・`tests/schema_docs.rs` の行数・byte 数・要約値を写す。床は設計ノートの欄の決まりの注でない欄を比べるので、土台の写し `tests/fixtures/floor_base/design-intent/design-note/schema.yaml` に known_values の行を字面で足す。その 1 行で節点の要約値の anchor `tests/fixtures/schema/node-digest-anchor.txt` の残差が動くので、便 99 の独立の script（node-digest.py）で組み直し、`tests/graph.rs` の F99 の値を写す。
4. **変えないもの。** 判定（違反・まだ分からない）と違反の字・EXTERNAL_NEED / SHAPE と IDS_KIND / SECTIONS の値・`note.rs` と `ids.rs` の判定の式・要件書の規範文・folio2 の床 4 本の答え・`folio build` の出力。`tests/schema.rs` は歯 f89 の上限（700 行）に収めるため、頭の来歴の 1 行に便 185 と本便を並べる（行数は増やさない）。

### (c) 歯（f196_・base で 0 件・どれも src の単体）

1. **floor_note::tests::f196_the_note_region_copies_the_known_values。** 実の design-note/schema.yaml の external_schema で value_domains の次が known_values、鍵が need・shape の順、値が定数と同じ、注に古い字「値域の変更は器の版上げで足り」が無く known_values を名指す。**base では欄が無い＝RED。**
2. **ids::tests::f196_the_srs_region_copies_the_ids_anchor。** 実の srs.yaml の schema.ids_anchor の鍵が kind・sections の順、kind = IDS_KIND、sections が SECTIONS の順と字のまま、注が `crates/folio/src/ids.rs` と P-5.6 を名指す。**base では欄が無い＝RED。**
3. **ids::tests::f196_only_the_owner_spells_the_copied_constants（1 枚の歯）。** src の各 file の最初の `#[cfg(test)]` より前で、引用符付きの ids-anchor の字は ids.rs に 1 回・shall と title の一覧の字は ids.rs に 2 回・need と shape の一覧の字は floor_note.rs に 1 回ずつだけ在る（床の木が字で写すと落ちる）。守りの歯＝base でも緑。
- 層の歯に掛からないよう、単体の歯は層 1 の `crate::yaml` だけで file 全体を読む。歯だけを base に当てると単体 168 本のうち 1・2 だけが落ち、歯の data（anchor と値）も当てると anchor に繋がる 11 本が落ちる。

### (d) 採らなかった形

1. **注だけを実装に合わせ、値は写さない（.169 の候補 (b)）。** 行 D-11 は「まだ分からない」を出す判定に使う値域を写しの範囲に入れる＝写しが要る。
2. **id の一覧の anchor の写しを判断の記録の欄の決まり adr/schema.yaml の anchor の節に置く。** 写しの fixture 17 本を動かし、未着地の便 162 と検査の信頼のレーン（便 190）と同じ生成区間に当たる。要件書の区間は床が fixture と比べない。
3. **器の file の値をそのまま受ける形に実装を変える。** need の意味（必須か）を知らない値では床が判定できない＝判定が変わる。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は無い**（anchor の値の歯は (b) の 3 のとおり同じ便で直す）。便の全部を当てた写しで workspace の nextest 1073 / 1073（+3 = f196_）・clippy 0 警告・床 4 本 rc 0・`folio build` 37 file は base と byte で同じ。
2. **突然変異 16 通り・生き残り 0**（見本の src だけを 1 通りずつ変え、verify の歯の束を撃った・base 44c14ac の見本 1a7c4c7 で撃ち直し・落ちる歯の数は前の base と同じ）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 known_values を外す / M2 need と shape の順を入れ替え / M3 need に shape の定数 / M4 known_values を unknown_value の後ろへ / M5 注を古い字に戻す | 1・設計ノートの区間の anchor と値（25〜26 本） |
| M7 EXTERNAL_NEED から conditional を落とす | 1・2・anchor（28 本） |
| M6 shape を字で書く / M9 kind を字で書く / M12 requirements の欄を字で書く | 3 |
| M8 ids_anchor を外す / M10 sections の adr を落とす / M11 sections の 1・2 行目を入れ替え / M13 注から「P-5.6・行 D-11」を落とす / M16 kind と sections の順を入れ替え | 要件書の区間の anchor と値（15 本） |
| M14 IDS_KIND の字を変える / M15 SECTIONS の nonfunctional の欄を変える | 2・anchor（16〜18 本） |

### (f) 大きさ・余地・verify と done

1. **write-set 13 本**（印なし）: src 3・歯の file 4（`tests/floor_cases.rs` は verify の scope で本文不変）・設計文書 2・fixture 4。
2. **余地（参考値）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: |
| check.rs | 1113 | 1136 | 364 |
| floor_note.rs | 578 | 617 | 883 |
| ids.rs | 369 | 431 | 1069 |

3. **size は M**（最小の余地は check.rs の 364 ≥ 300）。
4. **verify は 6 行**で、done の塊と 1 対 1: `--bin folio f196_`（単体の歯 3 本）・`--test schema`（2 区間の anchor と byte の値）・`--test schema_docs`（区間の行数と要約値・歯の file の上限）・`--test graph f99_`（節点の要約値の anchor）・`--test floor_cases`（土台の写しの上の床の case）・clippy。base では f196_ の行が 0 件で終了コード 4、ほかは緑。

### (g) 受付・並行の便

- **重なり**（本流 44c14ac の上の見本どうしの `git merge-tree` の実測・2026-09-28 20:0x・読むだけ）: 便 190（検査の信頼・持ち主の承認待ち・83f049d）と `tests/schema.rs` を両方が書く。190 は判断の記録の区間の値 2 行、本便は設計ノートの区間の来歴 1 行（120 字に畳んだ）と値 3 行を書き換えるだけで、どちらも行を足さない＝どちらが先に着地しても字の衝突は無く、自動で合わさった後は器の式で 700 行（歯 f89 の上限 700 ちょうど・余り 0・下の 2 つの順の各段で実測）。越えないので直しは要らない。ただし余りが 0 なので、後の便が `tests/schema.rs` に行を足すなら、足す便が来歴の行を畳むか歯を別の file へ移す。便 187（床の穴・先・impl/d187 430c9c7）とは重ならない。便 198（編集時の止め・後・15b235a）と `check.rs`（衝突なし・重ねた後の余地 356）。便 192〜194 は base に入った（194 は `tests/schema.rs` の本文を変えず 700 行のまま・`floor.rs` の外の置き場の字の落とし方が変わっても、本便の 2 つの注の外の置き場の字は 194 の前後で同じ）。
- **同じレーン**: 便 195・197 とは `tests/schema_docs.rs` の頭の来歴の行が隣り合う所だけが当たる（後に着地する側が両方の行を残す）。便 195 とは `tests/graph.rs` も共有する（別の所）。3 本を本流 44c14ac の上に 195 → 196 → 197 の順で重ねた写し（控え copies-scripts/stack44/・commit を作らない git apply -3）では、字の衝突はその頭の注の 2 塊だけ（両方の行を残して解いた）で、9 本の生成区間は重ねたまま schema --check が一致し（--write は何も書き直さない）、workspace の nextest 1083 / 1083（base 1070 + 5 + 3 + 5）・clippy 0 警告・床 4 本 rc 0・`folio build` は本流と byte で同じ・3 本の verify 17 行が全部 rc 0、`tests/schema_docs.rs` は器の式で 1177 行（上限 1200）・`tests/schema.rs` は 700 行（上限 700）。着地の順は 187 → 190 → 195 → 196 → 197 を既定とし、190 は持ち主の承認待ちなので 195〜197 が先に着地してもよい（187 → 190 → 195 → 196 → 197 と 187 → 195 → 196 → 197 → 190 の 2 つの順で 44c14ac の上に重ねた写しは、最後の木が byte で同じで、workspace の nextest 1107 / 1107・clippy 0 警告・床 4 本 rc 0・歯 f89 が緑）。
- **受付の時点の main が 44c14ac と違えば**、check.rs の余地・2 区間・anchor 4 本・`tests/schema.rs` の 700 行の上限を数え直してから運ぶ（便 190 の着地の後は必ず）。

### (h) 今の置き場の床が変わらないこと

1. **folio2 自身。** 床 4 本の答えは同じ（schema --check の design-note/schema.yaml と srs.yaml の byte 数の 2 行だけが違う）。`folio build` の出力は byte で同じ。
2. **tsuzuri**（写しへ cp・本物では何も撃たない・写し e65191b・本流 44c14ac の binary と比べた）。本流の binary の `schema --write` の後に、便の binary の `schema --write` がさらに書き直すのは design-note/schema.yaml と srs.yaml の 2 本だけで、床の答えは同じ（check 0・schema --check 0・derive --check 0）。便の binary のまま書き直さないと、設計ノートの床（check）と schema --check が落ちる＝binary の入れ替えと書き直しは同時に要る。

### (i) 連絡・運ばないもの・撤退条件

1. **利用者への連絡**: 便 195 の (i) の 1 と同じ 1 回（本便の分は design-note/schema.yaml と srs.yaml・書き直すまでは素の check も落ちる）。
2. **運ばないもの。** 面の側の同じ字の写し（`face_note.rs` の EXTERNAL_HEAD ほか・NEED / SHAPE の名札の鍵）・判断の記録の欄の決まりの anchor の種別の値（constitution-anchor・便 162 の載せ替えに添える候補）。
3. **撤退条件。** (1) 既存の歯が 1 本でも (b) の 3 の値の直しのほかで落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で、folio2 の床 4 本か `folio build` の出力が (h) の 1 の差のほかで着地の直前と違えば止めて席へ返す。

## 2. 範囲

- 入れる: §1 (b) の 1〜3 と (c) の歯 3 本と anchor の値の直し。
- 入れない: §1 (b) の 4・(i) の 2・命令の旗・新しい file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| U1 | 知っている値の写し | floor_note.rs の known_values と注の直し |
| U2 | id の一覧の anchor の写し | check.rs の ids_anchor と注・ids.rs の定数の公開 |
| U3 | 写しの data | 2 区間・anchor 3 本・土台の写し 1 行・歯の値 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は便 185（base に入る）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .169 と .126 を閉じる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gq"
title = "台帳 f2-648.169 と f2-648.126: 設計ノートの床が器の導出 file の need / shape を判定に使う閉じた一覧（EXTERNAL_NEED・EXTERNAL_SHAPE）を設計ノートの欄の決まりの生成区間へ known_values として写し、値域の変更は器の版上げで足りると書く注を実装に合わせる。id の一覧の凍結 anchor の種別の値と節ごとに要約値を取る欄（ids.rs の IDS_KIND・SECTIONS）を要件書の生成区間へ ids_anchor として写す（行 D-11）。floor_note.rs と check.rs の床の木に欄と注を足して folio schema --write で 2 file を書き、土台の写しと凍結 anchor を直す。判定の答えは変えない。歯は f196_ 3 本。base = main 44c14ac"
req = ["FR19", "FR10"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/src/floor_note.rs", "crates/folio/src/ids.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/floor_cases.rs", "design-intent/design-note/schema.yaml", "design-intent/srs.yaml", "tests/fixtures/schema/note-region.txt", "tests/fixtures/schema/srs-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/floor_base/design-intent/design-note/schema.yaml"]
verify = ["cargo nextest run -p folio --bin folio f196_", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test graph f99_", "cargo nextest run -p folio --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "単体の f196_ の歯 3 本（floor_note.rs と ids.rs）が緑、tests/schema.rs と tests/schema_docs.rs の歯の全部（設計ノートの欄の決まりと要件書の生成区間の凍結 anchor・行数・byte 数・要約値と歯の file の上限）が緑、tests/graph.rs の f99_ の歯（節点の要約値の anchor）が緑、tests/floor_cases.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が 9 file とも一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と byte で同じ"
<!-- contracts:end -->
