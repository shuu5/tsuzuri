# 設計: 便 177 — 印の節点の表（rest・nodes）と stamp_table を外し、索引の欄の決まりの注から印の 1 文を落とす（判断の記録 ADR-30 決定 (6)）

- 要件: FR20（印の中身）・FR19（欄の決まりの生成区間は床の定数から決定的に導く）。便 175（行 fv）の後半＝便 169 の契約 §1 (g) の 3 の後続の残り。
- 条: P-6.2（生成区間は folio の出力）/ P-10.1（凍結 anchor は前の anchor を直した独立の写し）/ P-4.1（偽の字を配らない）。
- 出所: 便 175 と同じ起草の記録（`~/.local/share/folio2/handoff-2026-09-27/d175-draft.md`）。便 175 の見本（1 便）が差分 124,156 byte で上限 120,000 byte を超え、2 便に割った。要件書・規則の表・憲法・人の書く正本は変えない。台帳の id は席が起こす。
- 置き場: 審査の材料は行 `fx` の §1 だけ。write-set 11 本（src 3・歯の file 5〔中身の変わらない 2 本は verify の --test の scope〕・生成区間 1・凍結 anchor 2）。新しい dir・新しい file・消す file は無い。
- 門: **0（通す）**（本流 efe3e7b の binary・印の 51 周目に支持の 止める は無い）。受付の本流（便 175 の後）で撃ち直す。
- base = 便 175 の見本の後（impl/d175 の commit 501c39a・数はその写しの実測・参考値・行 D-13）。**受付は便 175 の着地の後。**
- 実装の見本: origin の枝 `impl/d177`（commit ef26066・`git diff 501c39a ef26066` が本便の差分）が本便の後の中身（差分 26,714 byte・9 file・+54 −168）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout ef26066 -- <write-set の file>`）。write-set の外は変えない。

## 1. 設計

実装の見本は origin の枝 `impl/d177`（commit ef26066・base は 501c39a = 便 175 の見本）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout ef26066 -- <write-set の file>`）。write-set の外は変えない。受付は便 175 の着地の後。

### (a) いま起きていること（参考値・便 175 の後 = 501c39a）

1. **印は読み手の無い節点の表を書く。** `folio ceiling --stamp`（`stamp.rs`）は印の末尾に残差の要約値 `rest` と節点ごとの要約値の表 `nodes` を書く（組み方は `graph.rs` の `stamp_table`）。便 169 の後の門は印の round・verdict・viewpoints・refutes だけを読み、面の名札（`stamp::marks`）は at・sources と観点の行だけを読む＝rest と nodes の読み手は 0（grep の実測）。門が変わった節点の数を数えていた頃の欄（便 99・ADR-13 決定 (8)）。
2. **生成区間がそれを言う。** `design-intent/graph.yaml` の生成区間の注 `schema.digest_note` の末文「天井の印はこの要約値の表と、節点にも辺の欄にも属さない残りの byte の要約値（残差）を持つ（ADR-13 決定 (8)）」。本便で印が持たなくなるので、同じ便で落とさないと偽になる。tsuzuri の写しと骨格も番号を落とした同じ字。
3. **節点の要約値そのものは生きている。** `folio graph --print` の 4 列目と床の口 `check_index` が同じ Scan を使う（残る）。残差の独立の script `node-digest.py` と凍結 anchor `node-digest-anchor.txt` も `tests/graph.rs` の歯が縛るまま（本便は動かさない）。
4. **base の歯（501c39a の写し）。** nextest 999 / 999・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file（要約 6a4c51d7c0fb66c7）。`git grep -n f177_ -- crates` 0 件・行 id `fx` 0 件。

### (b) 直す先

1. **`stamp.rs`。** 印は rest と nodes を書かない（欄の並び = round・at・verdict・sources・faces・viewpoints・refutes・reads の 8 つ）。`graph` の use を外す。
2. **`graph.rs`。** `stamp_table` と、それだけが使う use（`ceiling_src::Ceiling`・`gate`）を外す。床の木の注 digest_note の末文を落とす（graph.yaml の生成区間は 3,537 byte に）。`check_index` の注と違反の字の「天井の印が組めない」を「folio graph --print が組めない」に（印はもう Scan を使わない）。
3. **`main.rs`。** 注の「索引と天井の印が組めない」を「索引が組めない」に（1 行）。
4. **生成区間。** `design-intent/graph.yaml` を `folio schema --dir design-intent --write` の出力に。
5. **凍結 anchor 2 本（席の裁定 2026-09-27: 案 A・前例どおり測り直す・design-intent/anchors/ の下は動かさない）。** folio の code を呼ばない独立の実装（python）で組み、folio の出力と byte で一致させる（歯 `tests/schema_docs.rs`・`tests/stamp.rs` が突き合わせる）。
   - ① `tests/fixtures/schema/graph-region.txt`: 手順 = 前の anchor の digest_note の行から (a) の 2 の末文（155 byte・前の「。」から）を字の一致で 1 か所だけ落とす（apply-tests-177.py の 1）。変わる範囲 = 行 35 の 1 行の中のその字だけ・3,692 → 3,537 byte。定数 F95_GRAPH_*（行数 35 のまま）。
   - ② `tests/fixtures/ceiling/findings/stamp-expected.yaml`: 手順 = 前の anchor の末尾の `nodes:` の見出しから終わりまで（24 行）を落とす（同 2）。変わる範囲 = 行 11〜34 だけ・前の 10 行は同じ・1,343 → 560 byte。
   - **外す字の分だけ動くことの確かめ（手順）**: `anchor-diff-175-177.py`（folio を呼ばない）が ① は 1 行の中の末文のほかが同じ、② は前の 10 行が同じ、を assert で確かめて通った（anchor-diff.log）。
6. **変えないもの。** 門の式・印のほかの欄・面の名札・`folio graph --print` と `check_index` の判定・`node-digest.py` と `node-digest-anchor.txt`・人の書く正本・本流の印（次の --stamp まで rest と nodes を持つ＝読み手は無い）・門の印の fixture。

### (c) 歯（f177_・base で 0 件）

1. **f177_no_region_claims_the_stamp_node_table（`tests/place_name.rs`・binary）。** folio2 自身の置き場・骨格の命令が書いた置き場・tsuzuri の名で `schema --write` した置き場の 9 本の生成区間に、手書きの字「天井の印はこの要約値の表」「残差」が無い（便 175 の道具 f175_no_false_claims に字の一覧を渡す）。**base は 3 つとも索引の欄の決まりで落ちる（RED）。**
2. **f177_the_stamp_has_no_node_table（`tests/stamp.rs`・binary）。** `--stamp` の書いた印の最上位の欄が 8 つでこの順、要約値を落とした印が凍結 anchor stamp-expected.yaml（10 行 560 byte）と byte で同じ。**base は rest・nodes が在って落ちる（RED）。**
3. RED の本文は `red177.log`（歯だけ = r177-teeth.patch を 501c39a の写しに当てた・2 本とも落ちる）。

### (d) 採らなかった形

1. **便 175 と 1 便にまとめる。** 差分が上限を超える（(a) の前の出所）。
2. **rest と nodes を残し、注だけ「印は書くが読み手は無い」に直す。** 読み手 0 の欄が印に残り、注の字も凍結 anchor も同じだけ動く。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **外す歯 3 本**（印の節点の表を縛る）: `tests/stamp.rs` の f99_the_stamp_carries_the_node_table・f99_the_stamp_rest_is_recomputable・f99_the_stamp_node_rows_are_the_index（道具 node_rows も）。**字を合わせる**: without_digests（rest の行を読み飛ばさない）・凍結 anchor の定数（(b) の 5）。
2. **便を当てた写し。** nextest **998 / 998**（999 − 3 + 2）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file（要約 6a4c51d7c0fb66c7 = base と同じ）。
3. **突然変異（mut.log）。**

| 変異 | 落ちる f177_ |
| --- | --- |
| M4 印が rest の行を書く | stamp_has_no_node_table |
| M5 索引の注に印の残差の 1 文を残す | no_region_claims_the_stamp_node_table |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 縮む（`-`）は src 3 本・歯の file 1 本（stamp）・生成区間 1 本・anchor 2 本。増えるのは歯の file 2 本（place_name・schema_docs）。tests/graph.rs と tests/gate.rs は verify の --test の scope で中身は変わらない。差分 26,714 byte（9 file・+54 −168・`git diff 501c39a ef26066`）。
2. **余地（CapHeadroom）。** python と awk が一致（lines.log）。

| file | base | base の余地 | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/graph.rs` | 836 | 664 | 805 | 695 |
| `crates/folio/src/main.rs` | 682 | 818 | 682 | 818 |
| `crates/folio/src/stamp.rs` | 511 | 989 | 504 | 996 |

3. **size は S。** src は外すだけ（−38 正規化行）。余地の最小は base の graph.rs の 664（≥ 100）。
4. **verify 8 行 = done の 8 の塊。** 1 `--test stamp f177_`（(c) の 2）・2 `--test place_name f177_`（(c) の 1）・3 `--test stamp`（16 本）・4 `--test place_name`（8 本）・5 `--test schema_docs`（28 本・graph.yaml の生成区間と凍結 anchor の byte 一致）・6 `--test graph`（16 本・`graph --print` と節点の要約値と残差の anchor は変わらない）・7 `--test gate`（15 本・門の答えは変わらない）・8 clippy 0 警告。便の後の写しで 8 行とも rc 0（verify177.log）。

### (g) 門・受付・並行の便・外の置き場

1. **門。** write-set の設計文書の正本は graph.yaml の 1 本（生成区間だけ）。efe3e7b の binary で 0（通す・gate.log）。受付の本流で撃ち直す。
2. **受付。** 便 175 の着地の後（(a) の base は便 175 の見本の後）。受付の本流が便 175 の見本の中身と違えば (i) の 3。
3. **外の置き場（tsuzuri）。** 本便の binary で `folio schema --dir design-intent --write` → commit で、索引の欄の決まりの生成区間から印の残差の字が消える（tz2 の写しで便 175 と続けて実測・tsuzuri.log）。

### (h) 数え直す手順（行 D-13）

記録と script は便 175 と同じ（`d175-draft.md`・`d175-scripts`）。1 apply-177.sh（apply-src-177.py → apply-tests-177.py → schema --write）を 501c39a の写しに当てると ef26066 の中身。1b anchor-diff-175-177.py。2 run-175.sh。3 red-175-177.sh。4 mut-175-177.py（M4・M5）。5 lines-175.sh。6 verify-175-177.sh <写し> 177。7 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-177.md#fx`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** FR20 の規範文の印の中身の列（周の id・観点の 3 値・止める の場所・正本と面と束の要約値・起動の記録・読んだ文書の id）は rest と nodes を名指さない＝字の直しは要らない。FR20 の注の来歴の字（第 1.28 版で足した・後続の便で消す）は一括の候補。
2. **言えないこと。** 節点の要約値を印に持たせた判断（ADR-13 決定 (8)）の本文は凍結の来歴（ADR-30 決定 (1)）で直さない。
3. **撤退条件。** (1) (e) の 1 のほかに既存の歯が落ちたら止めて席へ返す。(2) 着地の後の本流で `folio schema --dir design-intent --check` が一致しないか、凍結 anchor 2 本のほかの anchor が動いたら止める。(3) 受付の本流に便 175 が着地していなければ止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜5・歯（f177_ の 2 本と (e) の 1）。
- 入れない: 周の引き金（便 175）・node-digest.py と残差の anchor・人の書く正本・本流の印・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| stamp | 印 | stamp.rs の rest・nodes と graph.rs の stamp_table |
| note | 注 | graph.rs の digest_note と check_index の字 |
| teeth | 歯 | f177_ の 2 本と凍結 anchor 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 前提の便 = 便 175（行 fv）。外部 crate・新しい dir・host の命令は無い。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の素の床が合格・次の --stamp で印が 8 欄になる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fx"
title = "印の節点の表（rest・nodes）と graph.rs の stamp_table を外し、索引の欄の決まりの注 digest_note から印の 1 文を落とす（判断の記録 ADR-30 決定 (6)・便 175 の後半）: stamp.rs の印は rest と nodes を書かず最上位の欄は round・at・verdict・sources・faces・viewpoints・refutes・reads の 8 つ、graph.rs は stamp_table とそれだけが使う use を外し、check_index の注と違反の字から天井の印を外し、main.rs の注を合わせる。design-intent/graph.yaml の生成区間を folio schema --write の出力にし、凍結 anchor 2 本（graph-region.txt・stamp-expected.yaml）を前の anchor を直した写しで測り直す。folio graph --print と check_index と node-digest の anchor は変えない。実装の見本は origin の枝 impl/d177 の commit ef26066（base 501c39a = 便 175 の見本）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。受付は便 175 の着地の後。base = 便 175 の見本の後の 501c39a"
req = ["FR19", "FR20"]
section = "1"
write-set = ["-crates/folio/src/graph.rs", "-crates/folio/src/main.rs", "-crates/folio/src/stamp.rs", "crates/folio/tests/place_name.rs", "crates/folio/tests/schema_docs.rs", "-crates/folio/tests/stamp.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/gate.rs", "-design-intent/graph.yaml", "-tests/fixtures/ceiling/findings/stamp-expected.yaml", "-tests/fixtures/schema/graph-region.txt"]
verify = ["cargo nextest run -p folio --test stamp f177_", "cargo nextest run -p folio --test place_name f177_", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --test place_name", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test gate", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/stamp.rs の f177_ の 1 本（印の最上位の欄が 8 つでこの順・要約値を落とした印が凍結 anchor stamp-expected.yaml と byte で同じ）が緑、tests/place_name.rs の f177_ の 1 本（folio2・骨格・tsuzuri の名の置き場の 9 本の生成区間に 天井の印はこの要約値の表・残差 の字が無い）が緑、tests/stamp.rs・place_name.rs・schema_docs.rs（graph.yaml の生成区間と凍結 anchor の byte 一致）・graph.rs（graph --print と節点の要約値と残差の anchor は変わらない）・gate.rs（門の答えは変わらない）の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
