# 設計: 便 195 — 参照 id の空間と行 R-17 の読みの一覧を索引の欄の決まりの生成区間へ写し、同じ一覧を 1 枚に寄せる（台帳 f2-648.76 の前半・f2-648.184・M）

- 要件: FR19（欄の決まりの生成区間は床の定数から決定的に導出する）・FR5（構造の床）。規範文・確かめ方・受入基準は変えない。
- 条: P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ導出する）・P-6.3（同じ値を 2 か所に持たない）・P-5.1。
- 出所: 規則の表の行 D-11。判断の記録 ADR-11 決定 (4)⑧（参照 id の形と節の一覧は写しを生成区間へ出し、実装の中の 2 枚以上を 1 枚に寄せる・台帳 **f2-648.76**）と、天井の 31 周目 実態 F-3（行 R-17 の注が名指す写しの後続・台帳 **f2-648.184**）。
- 承認: 判定の値も式も変えず、既に効いている定数の写しを足すだけ。行 D-17 の「実装の定数を変えて生成区間の規則を変える」には当たらないと読み、席の裁定で受け付ける（読みが割れたら持ち主の承認の 1 回に載せる・字は変わらない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾。審査の材料は行 `gp` が指す §1 だけ。write-set 12 本（src 6・歯の file 4〔本文不変 1〕・設計文書 1・fixture 1）・新しい file も dir も無い。
- 門: **0（通す）**。本流 44c14ac の組み立てに write-set 12 本を渡した `folio ceiling --gate --dir design-intent --write-set …` の答え（印の周 51 に、書き換える file を場所とする支持された 止める は無い）。
- 前の便: **base = 本流 44c14ac**（便 185・192〜194 の着地の後）。数は base の写しの実測（参考値・行 D-13）。組み直す手順は控え `~/.local/share/folio2/handoff-2026-09-28/copies-scripts/d195/`（run・red・mut・cap・anchor・tz の script）。
- 見本: origin の枝 `impl/d195`（**26e2956**・本流 44c14ac を merge した後に検証役の N1 の歯 2 本〔(c) の 6・7〕を積んだ commit）。`git diff 44c14ac 26e2956` が便の全体の差分。作業者は write-set の file をこの commit の中身にしてよい。

## 1. 設計

### (a) いま起きていること（参考値・base 44c14ac）

1. **写しが無い。** 索引の欄の決まりの正本 `design-intent/graph.yaml` の生成区間は、節点の種類・辺の型・辺の欄だけを持つ。参照 id の解決（行 R-4）・散文の門（行 R-16）・散文の言及の歯（行 R-17）が判定に使う次の閉じた一覧は、実装の定数にしか無い。
   - 参照 id の空間: `refs.rs` の SRS_ID_SECTIONS（要件書の id を持つ 7 節）・RULE_SECTIONS（規則の表の 2 節）・RELATION_NAMESPACES（憲法の条の relations の 4 名前空間）・SRS_ID_PREFIXES（要件書の id の頭 5 つ）と、`prose.rs` の ARTICLE（P- / A- / N-）・RULE（R- / D-）。
   - 行 R-17 の読みの 5 つの閉じた一覧: `mentions.rs` の TARGETS（対象の file 9）・TYPED（型付きの欄 21）・PROVENANCE（来歴の欄 6）・TOP_SKIPPED（読まない最上位 2）・EXCLUDED（数えない言及の語形 10）と、要件書の節ごとの行の種類 SRS_SECTIONS、受け皿の表（関数 receives の match の式・表になっていない）。
   - 行 R-17 の注は「写しを設計文書の置き場へ導出することは後続である（P-5.6）」と書くが、その後続がまだ無い。
2. **同じ一覧が 2 枚以上。** RULE_SECTIONS は refs・link・note・prose・mentions・graph の 6 file に 6 枚、SRS_ID_SECTIONS は refs・link・note に 3 枚、prose.rs の REQUIREMENT は refs.rs の SRS_ID_PREFIXES と同じ 5 語（順だけ違う・照合は頭どうしが前方一致しないので順は答えに効かない）。
3. **base の歯。** workspace の nextest 1070 / 1070・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`f195_` と行 id `gp` は 0 件。

### (b) 直す先

1. **1 枚に寄せる。** `refs.rs` の 4 本を `pub(crate)` にし、`link.rs`・`note.rs`・`prose.rs`・`mentions.rs`・`graph.rs` は自前の RULE_SECTIONS / SRS_ID_SECTIONS を消して refs の定数を引く。`prose.rs` の REQUIREMENT は消して refs::SRS_ID_PREFIXES を引き、ARTICLE・RULE は `pub(crate)` にする。層は graph が 3、ほかが 2＝下向きの辺だけ（層の歯 `tests/modules.rs` の上向き 0 は変わらない）。
2. **受け皿の表を型付きに。** `mentions.rs` の `receives` の match を、種類の名（条・規範文・規則行・判断の記録・要件・制約・受入基準・目的・登場人物・出力）で鍵を引く定数 RECEIVES に置き換え、`Kind` に名を返す口を足す。答えは 10 × 10 の全部の組で base と同じ（要件 = 要件書の requirements と nonfunctional の行）。TARGETS ほか 5 本は `pub(crate)`。
3. **写す。** `graph.rs` の FLOOR の末尾（digest_note の後）に 4 欄を足す。値は上の定数を引く（同じ一覧を 2 回書かない）。
   - `ids`: rule_sections・srs_sections・relation_namespaces・prefixes（article・rule・srs）。
   - `ids_note`: 参照 id の空間の意味（行 R-4 の解決の母集団・散文の門・行 R-17 が同じ定数を引く・article の頭は枝番を取れる）と、正本は実装の定数でこの節はその写しであること。
   - `mentions`: targets・typed・provenance・top_skipped・excluded・srs_kinds（節 → 種類の名）・receives（出所の種類 → 受けられる指す先の種類・空の一覧も書く）。
   - `mentions_note`: 行 R-17 の機械の読みの 5 つの閉じた一覧であること（散文の欄は typed・provenance・top_skipped の補集合）と、正本は `mentions.rs` の定数でこの節はその写しであること。
   - 注は、外の置き場（名が folio2-constitution でない）で番号の括弧が落ちても文が通る形にする（`floor.rs` の導出の規則 7）。
4. **設計文書と fixture。** `design-intent/graph.yaml` の生成区間は便の binary の `folio schema --dir design-intent --write` で書き直す（区間の末尾に 62 行・区間の外は 1 byte も変えない）。凍結 anchor `tests/fixtures/schema/graph-region.txt` は導出を使わない独立の script（控えの anchor-195.py）で組み、`tests/schema_docs.rs` の F95_GRAPH_* の行数・byte 数・要約値を写す。
5. **変えないもの。** 判定（違反・まだ分からない）と違反の字・節点の種類と辺の型・id の走査の式（`refs.rs` の走査の中の字・`floor.rs` の ids_in・`vocab.rs` の ID_PREFIXES）・規則の表の行と要件書の字・folio2 の床 4 本の答え・`folio build` の出力。

### (c) 歯（f195_・base で 0 件）

1. **f195_the_graph_region_copies_the_id_space_and_the_mention_lists**（`tests/graph.rs`）。実の graph.yaml の生成区間の末尾 4 欄の順と、ids・mentions の各欄の値が歯の中に手で書いた字と同じ（鍵の過不足も落とす）・2 つの注が「この節はその写しである」を持つ。**base では区間に無い＝RED。**
2. **f195_the_id_space_lists_are_declared_only_in_refs**（`tests/modules.rs`）。src の `#[cfg(test)]` より前で RULE_SECTIONS・SRS_ID_SECTIONS・RELATION_NAMESPACES・SRS_ID_PREFIXES を宣言するのは `refs.rs` だけで、`prose.rs` は要件書の id の頭を自分で持たない。**base では 6 枚・3 枚＝RED。**
3. **graph::tests::f195_abroad_region_keeps_the_lists_and_the_unmarked_notes**（単体・`graph.rs`）。外の置き場の名で導出しても型付きの欄の行は folio2 と同じ字で、2 つの注は番号の括弧だけが落ちた字になる。**base では欄が無い＝RED。**
4. **mentions::tests::f195_receives_answers_every_pair_as_before**（単体・`mentions.rs`）。受け皿の 10 × 10 の答え（受ける組 40）を歯の中の行列で固定する。**答えを保つ歯＝base でも緑が正しい**（変異 M2〜M6 を落とす）。
5. **mentions::tests::f195_receives_table_is_keyed_by_the_kind_names**（単体・`mentions.rs`）。RECEIVES の鍵が 10 の種類の名を宣言の順に持ち、値がどれも種類の名である。見本の新しい定数を引くので base には当てない。
6. **graph::tests::f195_the_real_region_equals_the_constants**（単体・`graph.rs`）。実の graph.yaml の生成区間の ids と mentions の 11 の一覧が、正本の定数そのものと字も順も同じ（床の木が定数を引かずに字を手で持つと、定数を変えた途端に落ちる）。見本の定数を引くので base には当てない。
7. **f195_the_merged_files_spell_no_id_space_list**（`tests/modules.rs`）。1 枚に寄せた 6 file（refs・link・note・prose・mentions・graph）のうち、`#[cfg(test)]` より前で参照 id の空間の一覧を字で持つ（空白を除いて数える・式の中の 2 枚目も数える）のは正本の file の 1 回だけ。**base では link・note ほかが持つ＝RED。** ほかの file に残る 2 枚目（(i) の 2）は数えない。
- 既存の歯の直し: `tests/schema_docs.rs` の F95_GRAPH_*（graph.yaml の区間の行数・byte 数・要約値）を便の後の値に。歯だけを base に当てると、落ちるのは 1〜3・7 と F95 の 3 本だけ（1〜5 と F95 は 215 本中 6・7 は `tests/modules.rs` で 2 と並んで落ちる）。

### (d) 採らなかった形

1. **規則の表 rules.yaml の生成区間に R-17 の一覧を置く。** その区間は規則の表の行の欄の決まりで、R-17 の読みは id を持つ行と型付きの欄（索引の節点と辺）の話である。索引の欄の決まりは写しの fixture を持たず、区間の歯は anchor 1 本で済む。
2. **一覧を行 R-17 の value に移し、file を正本にする（行 R-16 の形）。** 規則の表の行の変更＝行 D-17 の持ち主の承認が要り、台帳 .184 の「写しを生成区間へ」とも向きが逆。
3. **新しい生成区間の file を足す。** FR19 の規範文の対象の一覧を足す要件書の版上げが要る。
4. **id の走査の式も 1 つにする。** `vocab.rs`・`floor.rs`・`refs.rs` の走査は文法が少しずつ違い、寄せると答えが変わりうる＝別の便で設計する。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は無い**（F95 の 3 本は (c) のとおり同じ便で値を直す）。便の全部を当てた写しで workspace の nextest 1077 本（+7 = f195_・80aec6c の全体の run が 1075 / 1075 で、積んだ歯 2 本は verify の行で緑）・clippy 0 警告・床 4 本 rc 0・`folio build` 37 file は base と byte で同じ。
2. **突然変異 14 通り・生き残り 0**（見本の src だけを 1 通りずつ変え、verify の歯の束を撃った・base 44c14ac の見本 80aec6c で撃ち直し・落ちる歯の顔ぶれは前の base と同じ）。検証役の変異のうち V3（床の木が TARGETS を字で持ち、定数の順を変える）は (c) の 6 が、V4（link.rs が規則の表の 2 節を式の中に字で持つ）は (c) の 7 が落とす（26e2956 で実測）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 RULE_SECTIONS の順を入れ替え / M9 写しの srs_kinds の節を取り違え / M10 写しの prefixes.srs を RULE に / M14 写しの鍵 receives の綴り違い | F95 |
| M2 受け皿の 要件 から 目的 を落とす | 4・F95 |
| M3 表の値を読まず鍵が在れば全部受ける / M4 出所と指す先を取り違え | 4・R-17 の既存の歯 f93_ |
| M5 Kind::name の 要件 を 非機能要件 に / M6 鍵 受入基準 の綴り違い | 4・5・F95 |
| M7 prose.rs が要件 id の頭を自分で持つ / M8 link.rs が規則の表の節を自分で持つ | 2 |
| M11 写しから excluded を落とす / M12 注の番号を括弧の外の文に書く | 3・F95 |
| M13 TYPED の verify を綴り違える | f93_・F95 |

### (f) 大きさ・余地・verify と done

1. **write-set 12 本**（印なし＝書き換えるだけ）: src 6・歯の file 4（`tests/check.rs` は verify の scope で本文不変）・`design-intent/graph.yaml`・`tests/fixtures/schema/graph-region.txt`。
2. **余地（参考値）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: |
| graph.rs | 1036 | 1166 | 334 |
| link.rs | 661 | 647 | 853 |
| mentions.rs | 348 | 449 | 1051 |
| note.rs | 1016 | 1002 | 498 |
| prose.rs | 327 | 327 | 1173 |
| refs.rs | 431 | 434 | 1066 |

3. **size は M**（最小の余地は graph.rs の 334 ≥ 300）。
4. **verify は 6 行**で、done の塊と 1 対 1: `--test graph f195_`・`--test schema_docs f95_ f89_`（写しの anchor と歯の file の上限）・`--test modules`（層と 1 枚の歯 2 本）・`--bin folio f195_`（単体の歯 4 本・外の置き場の導出と定数との一致を含む）・`--test check f93_`（行 R-17 の既存の歯）・clippy。外の置き場の区間の字は単体の歯 3 が見る（`tests/place_name.rs` は小さな直しの便 194 が書くので verify に入れない）。base では f195_ の行が 0 件で終了コード 4、ほかは緑。

### (g) 受付・並行の便

- **重なり**（本流 44c14ac の上の見本どうしの `git merge-tree` の実測・2026-09-28 20:0x・読むだけ）: 便 187（床の穴・先に着地・impl/d187 430c9c7）と便 190（検査の信頼・持ち主の承認待ち・83f049d）とは write-set が重ならない（字の衝突も無い）。便 198（編集時の止め・後・15b235a）とは `tests/modules.rs` を両方が書くが、別の所で自動で合わさる。便 192〜194 は base に入った: 192 と両方が書いた `note.rs` は merge で衝突なし。194 は `floor.rs` の外の置き場の字の落とし方（括弧を落とした跡の空白・指す語で始まる文）を変えたが、(c) の 3 の期待の字は変わらない（2 つの注の番号の括弧は和字どうしか文の末尾に接し、丸ごと落ちる文も無い＝base 44c14ac の上で単体の歯と 194 の歯 f194_ が緑）。
- **同じレーン**: 便 196・197 とは `tests/schema_docs.rs` の頭の注に便ごとの行を隣り合わせに足す所だけが当たる（後に着地する側が両方の行を残す）。3 本を本流 44c14ac の上に 195 → 196 → 197 の順で重ねた写し（控え copies-scripts/stack44/・commit を作らない git apply -3）では、字の衝突はその頭の注の 2 塊だけ（両方の行を残して解いた）で、9 本の生成区間は重ねたまま schema --check が一致し（--write は何も書き直さない）、workspace の nextest 1083 / 1083（base 1070 + 5 + 3 + 5）・clippy 0 警告・床 4 本 rc 0・`folio build` は本流と byte で同じ・3 本の verify 17 行が全部 rc 0、`tests/schema_docs.rs` は器の式で 1177 行（上限 1200）・`tests/schema.rs` は 700 行（上限 700）。着地の順は 187 → 190 → 195 → 196 → 197 を既定とし、190 は持ち主の承認待ちなので 195〜197 が先に着地してもよい（187 → 190 → 195 → 196 → 197 と 187 → 195 → 196 → 197 → 190 の 2 つの順で 44c14ac の上に重ねた写しは、最後の木が byte で同じで、workspace の nextest 1107 / 1107・clippy 0 警告・床 4 本 rc 0・歯 f89 が緑）。
- **受付の時点の main が 44c14ac と違えば**、graph.rs の余地・graph.yaml の区間・graph-region.txt・F95_GRAPH_* を数え直してから運ぶ（手順は控え）。

### (h) 今の置き場の床が変わらないこと

1. **folio2 自身。** 床 4 本の答えは同じ（schema --check の graph.yaml の byte 数の 1 行だけが違う）。`folio build` の出力は byte で同じ。
2. **tsuzuri**（写しへ cp・本物では何も撃たない・写し 59af37e・本流 44c14ac の binary と比べた）。本流の binary では素の写しで check と derive が合格（schema --check は写しが本流より古い分の adr/schema.yaml と design-note/schema.yaml で落ちる＝本便の外）。便の binary の `folio schema --write` が本流の binary の書き直しのほかに書き直すのは graph.yaml だけ（区間の末尾に 62 行・消えた行 0）。その後の check・schema --check・derive は 0 で、check と derive の出力は本流の binary の素の写しの答えと byte で同じ。注は番号の括弧だけが落ちる（folio2 の番号の跡 0）。

### (i) 連絡・運ばないもの・撤退条件

1. **利用者への連絡**（195〜197 の着地の後に 1 回・席が送る）: folio を便 197 の後の版にしたら、置き場の根で `folio schema --dir design-intent --write` を 1 回撃ち、書き直った file を全部 commit する（本便の分は graph.yaml）。撃つまでは schema --check が落ちる。
2. **運ばないもの。** 式の中に残る参照 id の空間の字の 2 枚目（`vocab.rs` の規則の表の 2 節と要件 id の頭・`adr.rs` の要件 id と条と規則行の頭・`rules.rs` の規則の表の 2 節・面の 4 file〔`face_note.rs`・`face_srs.rs`・`face_adr.rs`・`face_constitution_read.rs`〕・範囲の外の写し）・配信の接続先の式（台帳 .76 の後半・置き場が決まっていない）・行 R-17 の注の「後続である」の字（本便の後に古くなる＝席の一括）。
3. **撤退条件。** (1) F95 のほかに既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で、folio2 の床 4 本か `folio build` の出力が (h) の 1 の差のほかで着地の直前と違えば止めて席へ返す。

## 2. 範囲

- 入れる: §1 (b) の 1〜4 と (c) の歯 7 本と F95 の値の直し。
- 入れない: §1 (b) の 5・(i) の 2・命令の旗・新しい file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| U1 | 1 枚の定数 | refs.rs の 4 本と prose.rs の 2 本を正本にし、ほかの file は引く |
| U2 | 受け皿の表 | mentions.rs の RECEIVES と Kind の名 |
| U3 | 写しの欄 | graph.rs の FLOOR の ids・mentions と 2 つの注 |
| U4 | 写しの data | graph.yaml の生成区間・graph-region.txt・F95_GRAPH_* |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は便 185（base に入る）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .184 を閉じる。**.76 は閉じない**: 前半のうち式の中の 2 枚目（(i) の 2 の vocab.rs・adr.rs・rules.rs・面の 4 file）と後半（serve.rs の配信の接続先の式）が残る＝残りの在り処を .76 の notes に書いて開けたままにする。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gp"
title = "台帳 f2-648.76 の前半と f2-648.184: 参照 id の空間（要件書の id を持つ節・規則の表の節・憲法の relations の名前空間・id の頭）と行 R-17 の読みの 5 つの閉じた一覧（対象の file・型付きの欄・来歴の欄・読まない最上位・数えない語形と、節ごとの行の種類・受け皿の表）を、索引の欄の決まり graph.yaml の生成区間へ写す（行 D-11）。refs.rs の定数を 1 枚の正本にして link・note・prose・mentions・graph の写しを消し、mentions.rs の受け皿の match を種類の名の表 RECEIVES にし、graph.rs の FLOOR に ids と mentions と注 2 つを足して folio schema --write で書く。判定の答えは変えない。歯は f195_ 7 本と F95 の値の直し。base = main 44c14ac"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/src/link.rs", "crates/folio/src/mentions.rs", "crates/folio/src/note.rs", "crates/folio/src/prose.rs", "crates/folio/src/refs.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/check.rs", "design-intent/graph.yaml", "tests/fixtures/schema/graph-region.txt"]
verify = ["cargo nextest run -p folio --test graph f195_", "cargo nextest run -p folio --test schema_docs f95_ f89_", "cargo nextest run -p folio --test modules", "cargo nextest run -p folio --bin folio f195_", "cargo nextest run -p folio --test check f93_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f195_ の binary 経由の 3 本（tests/graph.rs 1・tests/modules.rs 2）と単体の 4 本が緑、tests/schema_docs.rs の f95_ と f89_ の歯（graph.yaml の生成区間の凍結 anchor・行数・byte 数・要約値と歯の file の上限）が緑、tests/modules.rs の層の歯が緑、tests/check.rs の f93_ の歯（行 R-17）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が 9 file とも一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と byte で同じ"
<!-- contracts:end -->
