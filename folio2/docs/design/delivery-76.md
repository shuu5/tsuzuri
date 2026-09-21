# 設計: 便 76 — 入口の棚の閉じた id の集合 4 つを実装の定数から入口の正本の生成区間へ導出し、命令 folio schema の 5 本目の対象に足す（ADR-11 決定 (3)(ウ)(オ)・(4)④・FR19 / FR5 / FR4）

- 要件: FR19（決まりの部分を床の定数から導出する = 対象に入口の正本を足す）/ AC17 / FR5（床）/ FR4（面は正本から生成する）
- 条: P-5.1・P-5.6（実装の型付きの定数を規則の正本とするあいだ、写しを設計文書の置き場へ決定的に導出する）/ P-6.2・P-6.3・P-6.4 / P-2.3（元の値は 1 か所）/ P-10.1（凍結 anchor は生成側から独立）/ P-14.2 と同じ型の検査 / N-3.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)(ウ)(オ) と (4)④。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)④。値は 1 つも変えない（置き場を足し、写しを導出するだけ）。
- 材料: 下調べ docs/design/adr-11-survey.md §1-c（A12〜A15）と §5 の入口の節。既に着地した同型の便 = 便 47・48（天井の正本・ADR-11 決定 (4)①）と便 51・53（規則の表・決定 (4)②）。
- 先行（本便の受付の前に要る）: 席の PR で要件書 第 1.22 版（FR19 の規範文と注・受入基準 AC17 の固定の材料に入口の正本を足す・持ち主の承認）。FR19 の注が「対象の file は増えるたびにこの要件の文に足す」と命じているためで、この 1 本だけは便では運べない。門との関係は §1 (g)。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 by が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + を付けた 1 本）。
- 門: 本便は design-intent/index.yaml を書き換えるので、天井の門（folio ceiling --gate）は第 2 の枝に入る（§1 (g)）。

## 1. 目的と中身

入口の正本 design-intent/index.yaml の棚は、行の id の集合を実装の表が固定し、行の中身（種別・説明・まだ無い理由・札の字）は人が書く形である。実測（main 26fe980）= crates/folio/src/face.rs の 4 つの表 SHELF_DOCS（692 行・4 行）・SHELF_RELATIONS（732 行・4 行）・ANNEXES（740 行・2 行）・SHELF_LEGEND（743 行・4 行）で、crates/folio/src/face_index.rs の関数 exact（409 行）が 文書（512 行）・付録（542 行）・関係（577 行）の 3 つを過不足なしと重複禁止で数え、凡例だけ 771 行の片方向の lookup で数えている。下調べ §1-c の A12〜A15 がこの 4 本で、どれも「中身を人が書く行の id の集合を実装が固定する形」＝ ADR-11 決定 (3)(ウ) の対象である。入口の正本の側には、この 4 つの id の集合を人が読める形で持つ欄が 1 つも無い（P-5.6 の写しが無い）。

本便は決定 (3)(イ)(ウ) のとおり、実装の型付きの定数を正本のままに、その写しを入口の正本の中の生成区間へ決定的に導出する。新しい命令は足さない（命令の閉じた一覧 11 本は crates/folio/tests/check.rs の 678 行の歯 p1_commands_closed_list が凍結している）。既存の命令 folio schema の 5 本目の対象に足す。

線引き（下調べ §5 の入口の節の指摘への答え）: 生成区間へ出すのは **id の列だけ**である。表 SHELF_DOCS は id のほかに 置き場の class・面の file 名・原語の札・区切りの字を持ち、SHELF_LEGEND は sw の class を持つが、これらは見た目の値なので生成区間へ出さない（出すと入口の正本に css の class 名が載り P-2.3 と折り合わない）。ANNEXES の（憲法の章の番号・数の単位）も面の側の持ち物なので出さない。行の並びの見た目も表が決めるままにする。

範囲の判断 2 つ: (1) 入口の正本の最上位の節の閉じた一覧（crates/folio/src/entrance.rs の 18 行の定数 INDEX_TOP_LEVEL）も同じ生成区間に載せる。生成区間を足す以上この定数に 1 語足す必要があり、天井の正本・規則の表の生成区間も最上位の節の一覧を先頭に持つ形で揃っているためである。ADR-11 決定 (4)⑤ は要件書・語彙・入口・相談窓口の同じ一覧を別便に置くが、入口の分だけ本便が先に片付く（残りの 3 file は (4)⑤ のまま）。(2) 読む順番（lanes）は対象外とする。実測 = 面の生成器（face_index.rs の 1109 行）も床（entrance.rs の 100 行）も行の id の閉じた一覧を持たず、行は自由な id で足せる＝そもそも二重の記述が無い。

(a) 表の鍵の列を組み立て時に取り出す。crates/folio/src/entrance.rs（床の木と同じ file・face.rs の 4 つの表は既に pub const なので face.rs は 1 行も触らない＝受付の余地が 93 行で size S の見積に足りないため〔改訂 b〕）に、鍵の列だけを取り出す小さな const fn と、それを当てた 4 本の定数 SHELF_DOC_IDS・SHELF_RELATION_IDS・ANNEX_IDS・SHELF_LEGEND_IDS を置く（表は face:: の path で参照する）。 4 つの表の型（実測・main eac56b7・face.rs）: SHELF_DOCS は 692 行 `pub const SHELF_DOCS: &[(&str, Shelf)] = &[`・SHELF_RELATIONS は 732 行 `pub const SHELF_RELATIONS: &[(&str, &str)] = &[`・ANNEXES は 740 行 `pub const ANNEXES: &[(&str, (u8, &str))]`（値は vocabulary と rules の 2 行）・SHELF_LEGEND は 743 行 `pub const SHELF_LEGEND: &[(&str, &str)] = &[`。4 つとも要素は先頭が &str の組（tuple）の slice（配列でない）で、2 つ目の型は Shelf（680 行 `pub struct Shelf {`）・&str・(u8, &str)・&str と異なる。鍵の列を取り出す口は 2 つ目の型を読まない。形は下のコード例の 1 つだけ（const fn ids・型引数 T は Copy・長さの定数引数 N・lifetime は 'static を明示・const の while で先頭の &str だけを写す・N は各表の len() を渡す）で 4 表に当てる。2 つ目の型 4 つ（Shelf は 680 行で Copy を導出済み・&str・(u8, &str)・&str）はどれも Copy なので縛りを満たす。宣言的マクロは使わない（face.rs を触らないため・代わりの形は無い）。置き場の起点: write-set の path はすべて repo の根から書く。tests/fixtures/schema/ は根の直下の dir で、既存の凍結 anchor 4 本（adr-region.txt・ceiling-region.txt・note-region.txt・rules-region.txt）と同じ置き場＝crates/folio/... と同じ起点である。形は次のとおりで、長さは表の長さから取る（手書きの 2 枚目の一覧は置かない・P-6.4）。

```rust
/// 表の鍵の列（id だけ）を組み立て時に取り出す。長さは表の長さと同じでなければ組み立てが通らない。
const fn ids<T: Copy, const N: usize>(table: &'static [(&'static str, T)]) -> [&'static str; N] {
    assert!(table.len() == N);
    let mut out = [table[0].0; N];
    let mut i = 1;
    while i < N {
        out[i] = table[i].0;
        i += 1;
    }
    out
}

/// 棚の文書の閉じた id の列（並びは表の順）。
pub const SHELF_DOC_IDS: [&str; SHELF_DOCS.len()] = ids(SHELF_DOCS);
```

pin した版（rust-toolchain.toml の 1.98.1・edition 2024）でこの形が組み立たないときは、形を変えずに質問で席へ返す（宣言的マクロや手書きの 2 枚目の一覧に替えない・P-6.4）。

(b) 床の木と凍結 anchor。crates/folio/src/entrance.rs（生の行 262・正規化 239・余地 1261）に床の木 FLOOR を置く（pub(crate) の定数・型は crates/folio/src/schema.rs の Floor・天井の正本の 86 行と規則の表の 134 行と同じ形）。葉は (a) の 4 本の定数と定数 INDEX_TOP_LEVEL を指す（同じ一覧を 2 回書かない）。INDEX_TOP_LEVEL は 5 語から 6 語になり、末尾に生成区間の節の名が入る。欄の順と字面は、次の凍結 anchor（起草役が §1 のこの逐語で独立に組んだ byte・作業者は tests/fixtures/schema/index-region.txt をこの逐語から作る＝命令の出力から作らない）のとおりとする。9 行・860 byte・sha256 = 01604d6003dde61fe970af89194215b5e0517d32610fd9cc0cc4554801f7f9ee。

```
schema:
  top_level: [meta, audience, shelf, lanes, intake, schema]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす）。meta・audience・shelf・lanes・intake は人が書き、schema は生成区間
  shelf:
    documents: [constitution, srs, design-note, adr]
    annexes: [vocabulary, rules]
    relations: [binds, before-build, inside, amends]
    legend: [readable, absent, binds, inside]
  shelf_note: 棚の閉じた id の集合（documents = 文書・annexes = 付録・relations = 文書どうしの関係・legend = 凡例）。棚の行はこの id に 1 つずつ要る。行の中身は人が書き、置き場の class や面の file 名のような見た目の値は面の生成器が持つ（P-2.3）。何にするかの裁定の正本は判断の記録 ADR-5 決定 (2) と入口の正本の承認欄
```

末尾が `_note` の 2 欄は人が読む説明の注で、導出（schema.rs の 269 行の関数 derive）が書き、突き合わせ（schema.rs の 81 行の関数 floor_diff）は読まない。注の 2 文めが裁定の正本を名指すのは ADR-11 決定 (3)(オ) の形である（天井の正本の生成区間と同じ書き方）。anchor を書き換えて実装に合わせてはいけない（変えるなら席へ問う）。

(c) 命令の対象。crates/folio/src/schema.rs（生の行 467・正規化 438・余地 1062）の 31〜36 行の一覧 TARGETS に 1 行足す = 入口の正本 index.yaml と entrance.rs の FLOOR。順は 5 本目（判断の記録 → 設計ノート → 天井の正本 → 規則の表 → 入口の正本）。ほかの関数（region_of・run_one・run）は変えない。印は 1 file に 1 対で位置を問わない形なので、生成区間が file の末尾に在る本件もそのまま扱える。

(d) 床。entrance.rs の 21 行の関数 check_entrance は、22 行の未知の節の判定に定数 INDEX_TOP_LEVEL を渡しているので、(b) で 1 語増えることにより生成区間の節が未知の節にならなくなる。これだけが床の変更で、床は生成区間の中身を床の木と突き合わせない（写しの fixture 20 本を直さずに済ませるため。実の file の一致は folio schema --check と歯が数える）。実測 = 入口の正本の写しは tests/fixtures の下に 20 本（anchors/index.yaml の 3 本は憲法の凍結 anchor の索引で別 file）。この線引きは規則の表の便 53 と同じで、床に足すのは後続の便とする。生成区間の字は語彙の検査にも通さない（body へ積まないまま・天井の正本と同じ持ち方）。

(e) 入口の正本。design-intent/index.yaml の末尾（相談窓口の節の最後の行の後）に、空行 1 本・人が書く注釈 1 行・印 2 本と (b) の生成区間を置く。人が書く注釈の逐語は次のとおり（印の行は crates/folio/src/schema.rs の 22〜23 行の定数の字面をそのまま使う）。

```
# 決まりの部分（生成区間）。最上位の節の閉じた一覧と、棚の閉じた id の集合 4 つ（文書・付録・関係・凡例）。棚の行はこの id に 1 つずつ要る（行の中身は人が書く）。何にするかの裁定の正本は設計文書（判断の記録 ADR-5 決定 (2)・この file の承認欄）で、機械が執行する規則の正本は実装の定数（判断の記録 ADR-11 決定 (3)(ウ)・(4)④）。直すときは実装の側を便で直す。
```

生成区間の中身は手で書かず、(a)〜(c) を入れた後に folio schema --write を実の置き場へ当てて書かせる（P-6.2）。書いた結果が (b) の anchor と 1 byte でも違えば、直すのは実装の側である。人が書く節（meta・audience・shelf・lanes・intake）と承認欄は 1 字も触らない。面の出力は 1 byte も変わらない（面の生成器は最上位の未知の節を読まない）ので、持ち主の walk の承認は取り直さない。

(f) 凡例を過不足なしに。face_index.rs（生の行 1363・正規化 1291・余地 209）の 767 行の関数 shelf の中の凡例の繰り返しを、ほかの 3 つと同じ関数 exact に替える（771 行の片方向の lookup をやめる）。ADR-11 決定 (3)(ウ) の「規則の全 id に行が 1 つずつ在る」の実体の検査に 4 本を揃えるためで、今は入口の正本から凡例の行を 1 つ消しても面が出てしまう。実測の影響 = 実の正本も凍結の見本 tests/fixtures/face/index.yaml も凡例 4 行を表と同じ並びで持つので、面の出力は 1 byte も変わらず、凍結の写し tests/fixtures/face/expected-index.html は再生成しない。表の外の id を書いた場合の既存の歯（tests/face_index.rs の 1005 行）は、文言が変わっても終了コード 2 と「まだ分からない」を見るだけなので期待は不変。

(g) 先行と門。先行 = 席の PR で要件書 第 1.22 版（FR19 の規範文の対象に入口の正本を足し、注の「対象の file は増えるたびに」に従い、受入基準 AC17 の固定の材料にも足す・持ち主の承認）。その PR は要件書を書き換えるので天井の印（design-intent/preview/ceiling-stamp.yaml）の正本の要約値が合わなくなる＝門は「印が古い」で まだ分からない を返す。したがって本便の受付は、その PR の後の周で folio ceiling --stamp を撃ち直して門が 通す を返す状態になってから行う。受付のとき本便の write-set には design-intent/index.yaml が在るので、門は第 2 の枝（印が 4 観点とも合格 ∧ 正本の要約値が今と同じなら 通す）で判定される。着地の後は入口の正本が変わるため印は再び古くなる＝次の周で --stamp を撃ち直すまで、設計文書の正本を書き換える次の便は まだ分からない で受からない（実装だけの便は第 1 の枝で 通す）。この付け直しは周ごとの決まった手順（天井の 15 周目と同じ）で、本便は印を書かない。

(h) 歯（関数名は f76_ で始める・`grep -rn 'fn f76_' crates/folio/tests` は今 0 本・src の側も同じ語で始める）。

1. f76_index_floor_derives_the_frozen_anchor_byte_for_byte（crates/folio/src/entrance.rs の単体の歯）: schema.rs の derive に FLOOR を渡した結果が凍結 anchor tests/fixtures/schema/index-region.txt と byte 一致。
2. f76_index_floor_is_the_frozen_closed_id_sets（同）: FLOOR の top_level が 6 語、shelf の下の 4 つの列が (b) の字面と順のとおり（字面を歯に直に書く凍結の針）。かつ 4 つの列が entrance.rs の 4 本の定数と同一の中身であること（face.rs の表から取り出した列であることの針）。
3. f76_schema_check_matches_the_real_index_file_and_its_frozen_digest（crates/folio/tests/schema.rs）: 実の設計文書の置き場の写しに --check → 0 ∧ 標準出力が 5 行 ∧ 5 行目に index.yaml と 860 byte ∧ 生成区間を sha256 で測り直して (b) の値と同じ ∧ 行数 9 ∧ 生成区間が schema の行で始まる ∧ 印の前に相談窓口の節が在り印の後は file の終わり。既存の定数 TARGETS は 4 から 5 に上げる（ほかの歯の期待は不変）。
4. f76_schema_check_fails_on_one_byte_drift_inside_the_index_region（同）: 写しの入口の正本の生成区間の 1 byte を書き換えて --check → 1 ∧ index.yaml ∧ 導出と違う旨 ∧ 先の 4 本の file は触らない。
5. f76_schema_check_is_unknown_without_the_index_begin_marker（同）: 入口の正本の begin の印を消す → --check も --write も 2 ∧ index.yaml の印が 1 対でない旨。
6. f76_schema_write_restores_the_index_region_and_is_idempotent（同）: 歯 4 の写しに --write → 0 ∧ 入口の正本の全体が元と byte 一致（人が書く節 meta・audience・shelf・lanes・intake と承認欄と注釈も不変）∧ もう 1 度 --write で変わらない。
7. f76_entrance_accepts_the_generated_region（crates/folio/tests/entrance.rs）: 実の置き場の写しで床が合格（生成区間の節が未知の節にならない）∧ 写しから生成区間の節だけを消しても床の結果が同じ（床は突き合わせない線引き・(d)）。本便の前の床の定数に本便の入口の正本を当てると未知の節で落ちるので、この歯は本便の中で赤から緑になる。
8. f76_face_index_unknown_when_a_legend_row_is_missing（crates/folio/tests/face_index.rs）: 凍結の見本の凡例の行を 1 つ消した写しで folio face --face index → 2 ∧ まだ分からない ∧ 出力先に書かない（今の main では 0 で面が出るので赤）。
9. f76_face_index_legend_order_comes_from_the_table（同）: 凡例の 4 行の並びを入れ替えた写しから出した面が、入れ替える前の面と byte 一致（並びは表が決める）。
10. 回帰（期待不変・verify の 2 行目）: tests/schema.rs・tests/entrance.rs・tests/face_index.rs の既存の歯すべて。tests/check.rs（命令の閉じた一覧 11 本を含む）・tests/floor_cases.rs（凍結の場合 134 件）・面と束の歯は共通の検証が回す。実測 = 凍結の場合 134 件に入口の正本を書き換える場合は 1 つも無く、床の写しの入口の正本 20 本は生成区間を持たないまま通る。

(i) 大きさと接続。新規 file は 1 本（tests/fixtures/schema/index-region.txt・9 行）。既存 = entrance.rs（+約 73 行・余地 1261 → 約 1188・face.rs は触らない）・face_index.rs（+約 6 行・余地 209 → 約 203）・schema.rs（+約 3 行・余地 1062 → 約 1059）・tests/schema.rs（正規化 826・+約 110 行）・tests/entrance.rs（正規化 271・+約 30 行）・tests/face_index.rs（正規化 1379・+約 30 行）・design-intent/index.yaml（+13 行）。size S（src の各 file の余地はどれも 100 以上）。行数は生の行と正規化（空行を除き幅 120 で折る）の両方を書いたが、受付のときは席が測り直す。crates/folio/src/main.rs・check.rs・ceiling.rs・rules.rs・adr.rs・note.rs・build.rs・部品目録 design-intent/preview/parts.json・様式 folio.css・凍結の写しの面・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。先行の便は無い（便 74 行 bw は main 26fe980 に着地済みで、本便が触る face.rs の表の並びとは重ならない）。

## 2. 範囲

- 入れる: 表の鍵の列を取り出す口と 4 本の定数（entrance.rs）・床の木 FLOOR と最上位の節の 1 語（entrance.rs）・命令の 5 本目の対象（schema.rs）・入口の正本の生成区間（design-intent/index.yaml）・凡例を過不足なしに（face_index.rs）・凍結 anchor 1 本・歯 9 本。
- 入れない: 要件書 第 1.22 版（席の PR・持ち主の承認・先行）・床が入口の正本の生成区間を床の木と突き合わせること（写しの fixture 20 本を直さないため・便 53 と同じ線引き）・見た目の値（置き場の class・面の file 名・原語の札・区切り・章の番号と単位・sw の class）を生成区間へ出すこと・読む順番（lanes）の行（閉じた id の集合が無い）・要件書と語彙と相談窓口の最上位の節の一覧（ADR-11 決定 (4)⑤）・天井の印の付け直し（周の手順）・入口の面の見た目と正本の行の中身。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| ids | 鍵の列 | entrance.rs の const fn と 4 本の id の定数（face.rs の pub const の表から取り出す） |
| floor | 床の木 | entrance.rs の FLOOR（凍結 anchor と byte 一致・最上位の節は 6 語） |
| target | 対象 | schema.rs の TARGETS の 5 本目（index.yaml と entrance.rs の FLOOR） |
| region | 生成区間 | design-intent/index.yaml の末尾（注釈 1 行 + 印 2 本 + 9 行） |
| anchor | 凍結 | tests/fixtures/schema/index-region.txt（860 byte・sha256 01604d60…） |
| legend | 凡例 | face_index.rs の凡例を exact に（過不足なし・重複禁止） |
| teeth | 歯 | 単体 2 本・schema 4 本・entrance 1 本・face_index 2 本 |

## 4. 検査（歯）

§1 (h) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "by"
title = "入口の棚の閉じた id の集合 4 つ（文書・付録・関係・凡例）と最上位の節の一覧を、実装の表から取り出した定数を正本として入口の正本 index.yaml の生成区間へ導出し、命令 folio schema の 5 本目の対象に足す。凡例の検査を片方向から過不足なしに揃える（見た目の値は生成区間へ出さない・床は突き合わせない・ADR-11 決定 (3)(ウ)(オ) と (4)④）"
req = ["FR4", "FR5", "FR19"]
section = "1"
write-set = ["crates/folio/src/entrance.rs", "crates/folio/src/face_index.rs", "crates/folio/src/schema.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/entrance.rs", "crates/folio/tests/face_index.rs", "+tests/fixtures/schema/index-region.txt", "design-intent/index.yaml"]
verify = ["cargo nextest run -p folio f76_", "cargo nextest run -p folio --test schema --test entrance --test face_index", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f76_ の歯 9 本（床の木の導出が凍結 anchor と byte 一致・閉じた id の集合の凍結の針・実の入口の正本の一致と要約値と行数・ずれ・印・書き直し・生成区間の節が未知の節にならない・凡例の行を 1 つ消すと面が出ない・凡例の並びは表が決める）が緑、tests/schema.rs と tests/entrance.rs と tests/face_index.rs の既存の歯が全部緑（命令の標準出力の行数は 4 から 5 へ）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
