# 設計: 便 120 — 契約表の欄の決まり（器の導出 file）の要否の値域に conditional を足し、folio2 の写しを器の今の出力に揃える（FR10・FR11・便 119 の §1 (i) の 5）

- 要件: FR10（契約表の欄は器の導出 file から読む・第 1.33 版）/ FR11（導出物を生成し、差分 0 を数える・第 1.33 版）。本便は FR10 の読み手が受ける要否の値を 1 つ広げ、その読み手を共有する FR11 の命令（folio derive）が器の今の形の導出 file で動くようにする。規範文はどちらも変えない。FR19（床の定数から生成区間への写し）は入れない＝本便の定数は生成区間に写しを持たない（§1 (a) の 3）。
- 条: P-4.1（実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは「まだ分からない」として表に出す＝値域の外の要否は今までどおり「まだ分からない」）/ N-3.1（規則の例外機構を足さない＝値域を開き放しにせず 1 語だけ足す）/ P-6.2・P-6.3（folio2 の写しは器の命令の出力をそのまま置き、手で直さない）/ P-10.1・P-10.2（手書きの凍結の対で確かめ、生成物どうしの突き合わせを唯一の合格判定にしない）/ P-5.6 と規則の表の開発規律行 D-11（値域の定数の値を変える便は根拠の判断の記録か裁定 id を名指す）
- 出所: 台帳 f2-648.166 の notes 2026-09-24 07:35 JST（scribe2 orchestrator の回答・逐語の要旨）= 器の contracts/schema.toml の verify と done は要否 conditional を持つ。揃え方は folio2 が値域に conditional を足す（判断の記録 ADR-16 決定 (2) の範囲）。意味は器の現物（器の repo の crates/scribe2/src/pipe/table/parse.rs の conditional_missing）どおり＝約束の行（of がその行の id を名指す後続の行）を 1 つ以上持つ親の行だけ verify と done を省け、それ以外の行では required と同じ字面で断る。folio の床は値域に在ることだけを検査すれば足り、判定は器の受付が持つ。器の schema.toml の 2 行は変えない。あわせて、着地済みの便 119 の契約（`docs/design/delivery-119.md`・行 `dr`）の §1 (i) の 5 が「この読み手を器の今の形に合わせる便が先に要る。folio2 の repo の写しの更新も同じ便で扱う」と書く。根拠の判断の記録は ADR-3 決定 (2)（欄の集合と値域は器の導出 file から読む）と ADR-16 決定 (2)（行 D-11 の名指し）で、上の器の回答は出所であって裁定 id ではない。設計ノートの欄の決まりの生成区間の external_schema_note は「欄の追加・値域の変更は器の版上げで足り、folio2 の判断の記録は要らない」と書くが、本便が要ること自体がその注と実装（閉じた値域 EXTERNAL_NEED で断る）の食い違いの実例である（§1 (h) の 3・根拠には引かない）。ADR-16 決定 (7) の便の列の外の追加（便 118 と同じ扱い）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ds` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 6 本。**新しい file は 2 本（凍結の対）で、先頭に `+` を付けて宣言する**。どちらも既に在る dir `tests/fixtures/design-note/` に置き、**新しい dir は作らない**。**縮む file も消す file も無い**（`contracts/schema.toml` は行が増える）。`~` は使わない。
- 門: write-set の 6 本はどれも設計文書の置き場（`design-intent/`）の外で、起草役の実測（2026-09-24・41ed484 に便 119 の実測 patch と本便の patch を当てた木の binary・`folio ceiling --gate --write-set` に 6 本）は **0（通す・設計文書の正本を書き換えない便）**。規則の表の開発規律行 D-12 の対象ではない。
- 前の便: **便 119（行 `dr`・契約は main 4bba5fd・便は main c33ee44 で 2026-09-24 09:14 JST に着地）の着地が前提（満たされた）**。本便は便 119 が足す `crates/folio/tests/derive.rs` と `crates/folio/src/derive.rs` の上に歯を足し、便 119 と同じ file（`crates/folio/src/floor_note.rs`）を書き換える。**base = c33ee44**。この契約の数は、main 41ed484 に便 119 の実測 patch（`~/.local/share/folio2/handoff-2026-09-24/d119-measured.patch`・sha256 の先頭 f3fb3d015e24）を当てた木（以下 base′）の起草役の実測で、参考値である（規則の表の行 D-13）。検証役は 73a6bf7 に便 119 と本便の patch を当てた木でも同じ値（792 → 796・clippy 0・床 合格・derive --check 0）を得た（`~/.local/share/folio2/handoff-2026-09-24/d120-verify.md`）。
- 並行の便との重なり: 便 119 とは write-set が 2 本重なる（`crates/folio/src/floor_note.rs`・`crates/folio/tests/derive.rs`）ので、**受付は便 119 の Landed の後に限る**（器の受付は重なる write-set を断る）。一括 15（main 73a6bf7 で着地）は設計文書の正本と `docs/design/` だけを書き換え、本便の 6 本とは 1 本も重ならない（FR10 と FR11 の規範文は一括 15 で変わっていない・変わったのは FR11 の注）。便 119 の §5 の申し送り（`contracts/schema.toml` の欄の順を変える便は `contracts/example.toml` を write-set に入れる）は本便には当たらない（§1 (b) の ③）。申し送りの括弧は本便（§1 (i) の 5 の FR10 の読み手の便）を名指すが、それは欄の順が変わる見込みで書かれたもので、実測では既存 16 行の順は変わらず導出物は byte 一致・`--check` は 0 である。

## 1. 設計

### (a) いま起きていること（実測・base′）

1. **器の今の導出 file は要否 conditional を持つ。** 器の命令 `scribe2 contracts schema` の出力（器の repo の main の `contracts/schema.toml` と同じ byte・2026-09-24 の実測で器の main d7f388b と a277a29 の間でこの file は動いていない）は、欄 18 行（参考値・`grep -c '^\[\[field\]\]' contracts/schema.toml` で数え直す）のうち verify と done の要否を conditional と書き、その後に約束の行の欄の表（見出し promise-field の行・鍵は promise-name・promise-need・promise-shape）を持つ。器の側の歯（器の repo の crates/scribe2/src/pipe/table.rs）は conditional の欄が verify と done の 2 つであることを固定している。
2. **folio の読み手はこの値を知らない。** 床と導出器が共有する読み手（`crates/folio/src/note.rs` の load_external）は、器の導出 file の各行の要否を `crates/folio/src/floor_note.rs` の定数 EXTERNAL_NEED（required と optional の 2 語）で受け、無い値なら「まだ分からない」を返す。起草役が base′ の写しに要否 conditional の最小の導出 file を置いて撃つと、`folio check` も `folio derive --write` も終了コード 2 で、断りの字は「器の導出 file が読めない: 欄「verify」の need「conditional」を知らない」だった。利用者の repo（器の repo）はこの形の導出 file を持つので、ADR-16 の M3 の判定点 ①（利用者の repo で床が合格）と ②（導出物の差分 0）はこの読み手のままでは測れない。
3. **定数 EXTERNAL_NEED は生成区間に写しを持たない。** 設計ノートの欄の決まりの床の木（同じ file の FLOOR）の external_schema の節は、置き場・先頭の字・行の見出し・行の欄の名（EXTERNAL_PATH・EXTERNAL_HEAD・EXTERNAL_ROWS_KEY・EXTERNAL_ROW_FIELDS）を写し、値域は写さず value_domains の葉に from-file の字を置く。したがって本便で定数を変えても `design-intent/design-note/schema.yaml` の生成区間・凍結 anchor `tests/fixtures/schema/note-region.txt`・床の凍結の土台 `tests/fixtures/floor_base/` の欄の決まり・節点の要約値の anchor は 1 byte も動かない（起草役の実測で `folio schema --check` は 9 file とも一致）。名が似た定数 NEED_ENUM（生成区間の fields-table の need_enum）は folio2 自身の欄の表の値域で、器の値域の写しではない（同じ生成区間の row_note が明記）。本便はそれに触らない。組み直す手順:

   ```
   git grep -n 'EXTERNAL_NEED\|need_enum\|value_domains' -- crates/folio/src design-intent tests/fixtures
   ```

4. **読み手の要否の使い方は 2 か所で、どちらも required の字だけを見る。** 床（`crates/folio/src/note.rs` の契約表の行の検査）は要否が required の欄だけを非空に数え、導出器（`crates/folio/src/derive.rs` の行の導出）は要否が required の欄が無いか空なら断り、ほかの欄は値が無ければ鍵ごと省く。どちらも required でない値を optional と同じに扱うので、**値域に conditional を足すだけで、床は conditional の欄の有無を数えず、導出器は行の値をそのまま写す（在れば写し、無ければ鍵を省く）**。これは器との合意（床は値域だけ・導出器は required だが親の行は省略可と読む）と一致し、**導出器の本文は 1 字も触らない**。
5. **folio2 の写しは器の今の出力より古い。** `contracts/schema.toml`（便 23 で置いた器の導出 file の写し）は欄 16 行（参考値・同じ数え方）で、verify と done を required と書き、targets と growth の 2 行と約束の行の欄の表を持たない。行の順は器の今の出力の先頭 16 行と同じで、器の出力は末尾に 2 行と表を足した形である。
6. base′ の workspace の nextest は全部緑（参考値 792 本・`cargo nextest run --workspace --no-tests=fail --no-fail-fast`）。

### (b) 直す先 — 値域に 1 語を足し、写しを器の出力に替え、凍結の対と歯を足す

**① 定数（`crates/folio/src/floor_note.rs`・base′ の 90〜91 行目・参考値）。** この節は二重引用符を置けないので、Rust の字の literal の二重引用符を「」で書いた。実物は base の字の literal と同じ二重引用符である。前:

```
/// 導出 file の need / shape の値域（読めない値は「まだ分からない」）。
pub(crate) const EXTERNAL_NEED: &[&str] = &[「required」, 「optional」];
```

後（値域の末尾に conditional の 1 語・注に 1 文）:

```
/// 導出 file の need / shape の値域（読めない値は「まだ分からない」）。conditional は器の条件付きの欄（約束の行を持つ親の行だけ
/// 省ける・省いてよいかの判定は器の受付が持つ）で、床は値域に在ることだけを見て欄の有無を数えない（便 120・判断の記録 ADR-16 決定 (2)）。
pub(crate) const EXTERNAL_NEED: &[&str] = &[「required」, 「optional」, 「conditional」];
```

EXTERNAL_SHAPE・FLOOR・`crates/folio/src/note.rs`・`crates/folio/src/derive.rs`・面の生成器は 1 字も触らない。

**② folio2 の写し（`contracts/schema.toml`）。** 器の命令の出力をそのまま置く。人は手で書かない（P-6.2）。

```
scribe2 contracts schema > contracts/schema.toml
```

実装役の環境で器の命令が撃てないときは、器の repo の main の `contracts/schema.toml` を byte のまま写す（2026-09-24 の実測で両者は同じ byte）。差分は verify と done の要否の 2 行が conditional に替わり、末尾に targets と growth の 2 行と約束の行の欄の表が足される形で、既存の 16 行の順と名と形は変わらない。

**③ 写しを替えても導出物は動かない。** folio2 自身の導出物 `contracts/example.toml`（便 119 が版管理に置く）は、実の見本 `design-intent/design-note/example.yaml` の契約表の行が targets も growth も持たず、verify と done を全部の行に持つので、写しを替えた後も 1 byte も変わらない（起草役の実測: 替えた後の置き場で `folio derive --dir design-intent --out ../contracts --check` が 0 で「一致（1 file）」・器の今の出力を置いた写しの置き場で `folio derive --write` した example.toml が版管理の example.toml と byte 一致）。**便 119 の §5 の申し送りは欄の順を変える便に掛かり、本便は既存の欄の順を変えないので、`contracts/example.toml` を write-set に入れない。** 便 119 の歯のうち、実の置き場から導出して版管理の導出物と突き合わせる歯がこの主張を確かめる（§1 (e) の 2）。

**④ 凍結の対（手書き・最小・生成器から独立・P-10.1）。** 既に在る dir `tests/fixtures/design-note/` に 2 本を新しく置く。

1. `tests/fixtures/design-note/need-conditional-schema.toml` = 器の導出 file の最小の写し。先頭の注 4 行・schema = 1・欄 7 行（id・title・req・section が required・verify が conditional・size が required・done が conditional）。**verify と done の 3 行組は器の今の出力の字を 1 字も変えずに写す**（名・要否・形の 3 行）。ほかの 5 行は契約表の行に要る required の欄だけで、器の出力の同じ名の行と同じ字。約束の行の欄の表は持たない。
2. `tests/fixtures/design-note/need-conditional.yaml` = 契約表を持つ最小の設計ノート（文書 id は need-conditional・status example・profile design-note）。散文の節 2 つ（n 1 と n 2・body は 1 行ずつ）と契約表の節 1 つ（n 3）で、契約表は 2 行。**行 a は verify と done を持ち（section 1）、行 b は verify も done も持たない（section 2）**。req は 2 行とも FR10、size は 2 行とも S。先頭の注 3 行に、対の相手と、床が値域だけを見て行 b を数えないことと、省いてよいかは器の受付が判定することを書く。

### (c) 採らなかった形

1. **床で conditional を required と同じに数える（folio の契約表は約束の行を持てないので、どの行も省けないと読む）。** 器の判定（約束の行を持つ親の行か）を folio が別の式で持つことになり、判定の置き場が 2 つになる（器との合意は「判定は器の受付が持つ」・ADR-3 決定 (3) の意味の検査の持ち主も器）。folio の契約表が後で約束の行を持つようになったとき、床が正しい行を落とす。採らない。§1 (d) の歯 1 がこの形を落とす（変異 M1）。
2. **要否の値域の検査をやめ、器の導出 file の値を何でも受ける（value_domains が from-file の字どおり）。** 知らない値の意味（非空に数えるか）を読み手が黙って optional と同じに決めることになる（P-4.2 の逆・FR10 の規範文の「読めないときは『まだ分からない』」の向きの逆）。採らない。§1 (d) の歯 2 がこの形を落とす（変異 M3・M4）。
3. **値域の写しを設計ノートの欄の決まりの生成区間へ出す（FLOOR の external_schema の reader_expects に要否の値域の葉を足す）。** 生成区間・凍結 anchor・床の凍結の土台・節点の要約値の anchor の 4 か所が動き、`design-intent/` を書き換えるので天井の門（D-12）の対象になる。便 119 が同じ 4 か所を書き換えたばかりで、生成区間の注（value_domains は from-file）との向きを先に決める要がある。本便では採らず、§1 (h) の 3 に「まだ分からない」として残す。
4. **folio2 の写しを替えない（凍結の対だけで確かめる）。** 便 119 の契約の §1 (i) の 5 が写しの更新を同じ便に割り当てている。写しを替えると、folio2 自身の床と導出物の歯（実の置き場を撃つ既存の歯すべて）が器の今の形の上で走り、定数から conditional を外す変異で 100 本を超える歯が落ちる（§1 (e) の 3）＝値域の 1 語が凍結の対の外でも固定される。採らない。
5. **面の生成器（`crates/folio/src/face_note.rs`）で verify と done の無い行も描けるようにする。** 面の範囲（FR4 と部品目録）へ広がり、folio の契約表が約束の行を持てない今は、その行は器の受付で断られる（§1 (h) の 2）。加えて、M3 の判定点 ①（床）と ②（導出物の差分 0）はどちらも面を撃たないので、面の側を直さなくても判定点は測れる。本便では採らない。

### (d) 歯（新しく 4 本・置き場は `crates/folio/tests/note.rs` に 3 本と `crates/folio/tests/derive.rs` に 1 本）

関数名は f120_ で始める（verify の絞り込みの語）。**どれも実の repo の数を pin しない**＝欄の数・行の数・値域の長さを見ず、性質だけを見る（歯 4 が見る「書いた 1 file」は空の置き場に置いた対の file の数で、実の repo の数ではない）。凍結の対は §1 (b) の ④ の 2 本で、ほかに新しい fixture の file も dir も作らない。`crates/folio/tests/note.rs` の 3 本は、同じ file の既存の写し（実の design-intent の全部と repo の器の導出 file を一時 dir に置き git init 済み）を使い、凍結の対を使う 2 本は写しから実の見本 example.yaml を外して（契約表を持つ設計ノートを対の 1 本にするため）対を置く小さな helper を 1 つ足す。

1. **要否 conditional の導出 file で、conditional の欄を持つ行も省いた行も床を通る**（note.rs）。対を置いて `folio check` → 終了コード 0・違反 0・合格の 1 行（違反 0・まだ分からない 0）。前提として、置いた導出 file が conditional の字を持つことと、対の設計ノートの行 b が verify も done も持たないことを見る。**base′ では「まだ分からない」の 2 で落ちる＝RED**。
2. **値域の外の要否は、conditional を知った後も「まだ分からない」のまま**（note.rs）。対の導出 file の verify の行の要否を、3 つの値（sometimes・大文字で始まる Conditional・required? のように末尾に字を足した値）へ 1 つずつ替えて `folio check` → 終了コード 2・違反 0・標準エラーにその値を名指す「まだ分からない」の行（need「…」を知らない）。base′ でも緑の歯で（base′ も最初に読む verify の行の値を名指して断る）、値域が 1 語だけ広がったこと（開き放しや大小文字の同一視を採っていないこと）を確かめる。
3. **実の器の導出 file は器の今の形を持ち、実の置き場で床が通る**（note.rs）。repo の `contracts/schema.toml` が要否 conditional の字と約束の行の欄の表の見出しの字を持つことを見て、実の design-intent の写しに `folio check` → 合格（違反 0・まだ分からない 0）。約束の行の欄の表が契約表の欄に登録されず床の結果を変えないことも、この合格が確かめる（§1 (h) の 4）。**base′ では写しが conditional を持たないので落ちる＝RED**。
4. **導出物は conditional の欄を行の値のまま写す**（derive.rs）。便 119 の同じ file の一時 dir に対を置いて `folio derive --write` → 終了コード 0・書いた 1 file。導出物を行の見出しで割り、行 a の塊が verify と done と goal の鍵を持ち、行 b の塊が verify と done の鍵を持たず goal を持つこと、導出物のどこにも conditional の字が無い（要否の値を導出物へ写さない）ことを見て、続けて `folio derive --check` → 0（一致）。**base′ では読み手が断って 2 で落ちる＝RED**。

歯の file の頭の注に、便 120 の歯と凍結の対の名を 1〜3 行で足す（note.rs は凍結 fixture の一覧の行の次・derive.rs は歯の一覧の 7 番目）。

歯の効き（起草役が変異を当てて測った・base′ に本便の patch を当てた木）:

| 当てた形 | 歯 1 | 歯 2 | 歯 3 | 歯 4 |
| --- | --- | --- | --- | --- |
| base′（定数も写しも base′ のまま・歯と対だけを当てる） | 落ちる | 通る | 落ちる | 落ちる |
| 本便の形 | 通る | 通る | 通る | 通る |
| M1 床が conditional の欄を required と同じに非空に数える | 落ちる | 通る | 通る | 通る |
| M2 導出器が conditional の欄の欠けを required と同じに断る | 通る | 通る | 通る | 落ちる |
| M3 読み手が要否の値域を検べない | 通る | 落ちる | 通る | 通る |
| M4 読み手が要否を小文字に畳んでから値域を引く | 通る | 落ちる | 通る | 通る |
| M5 定数に conditional を足さない（写しは替える） | 落ちる | 通る | 落ちる | 落ちる |
| M6 導出器が conditional の欄の行に要否の字を書き足す | 通る | 通る | 通る | 落ちる |

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くもの・門

1. **落ちる既存の歯は 0 本。** 起草役の実測（base′）で、定数だけを当てた木も、定数と写しを当てた木も、workspace の nextest は全部緑（参考値 792 本）。歯の本文は 1 本も直さない。
2. **写しを替えた後も緑のままの歯が、写しの主張を確かめる。** 便 119 の derive.rs の実の置き場の歯（実の design-intent から導出し、版管理の `contracts/` の導出物と一致して --check が 0）と、`crates/folio/tests/note.rs` の実の写しを撃つ歯（未知の欄が 0・実の見本が合格）と、`crates/folio/tests/site.rs` と `crates/folio/tests/badge.rs` の配信の組み立ての歯が、器の今の出力を置いた写しの上で緑である（§1 (b) の ③ の主張の歯）。
3. **写しだけを替えて定数を替えない木では、既存の歯の大半が落ちる**（参考値 170 本・床が「まだ分からない」を返すため）。定数と写しは同じ便で運ぶ（§1 (h) の 5 の撤退条件）。
4. **凍結 anchor が動くもの・動かないもの。** 動く凍結 anchor は無い。生成区間 9 file・`tests/fixtures/schema/` の anchor・`tests/fixtures/floor_base/`・面の凍結 fixture・要件書 AC13 の凍結 fixture `tests/fixtures/design-note/schema-plus-one/`（器の導出 file の古い形 + 欄 1 つ・欄の追従を見る材料）は 1 byte も変えない。新しく足すのは §1 (b) の ④ の対 2 本だけ。組み直す手順:

   ```
   cargo nextest run --workspace --no-tests=fail --no-fail-fast
   cargo run -q -p folio -- schema --check
   cargo run -q -p folio -- derive --dir design-intent --out ../contracts --check
   ```

5. **門（規則の表の開発規律行 D-12）。** write-set は設計文書の置き場の外だけで、起草役の実測は **0（通す）**（§0 の 門）。

### (f) 大きさ（器の行数の式・幅 120 で正規化）

1. **write-set の印。** 新しい file 2 本（凍結の対）に `+`。ほかの 4 本は base に在る file の書き換えで印なし（`contracts/schema.toml` は行が増える・`-` は当たらない）。`~`（着地で消える file）は使わない。
2. **余地（CapHeadroom）。** 測るのは write-set のうち `crates/folio/src/` の下の .rs、つまり `crates/folio/src/floor_note.rs` の 1 本だけである。

| file | base′ の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | size S の見積 | 本便の後（参考値） |
| --- | --- | --- | --- | --- |
| `crates/folio/src/floor_note.rs` | 487 | 1013 | 100 | 488（注が 1 行増える） |

   測り方: 各行の字数を 120 で割って切り上げ（空行は 1）、全行で足す（`wc -l` ではない）。参考に、歯の 2 本は note.rs が 645 から 709、derive.rs が 431 から 486、写しは 82 から 139（src の外なので余地を測らない）。
3. **size は S。** 変える src は 1 本で定数 1 語と注 1 文。

### (g) verify と done の対応

verify は 3 行で、done の 3 つの塊と 1 対 1 に揃える。

1. `cargo nextest run -p folio --test note --test derive f120_` = (d) の新しい歯 4 本。
2. `cargo nextest run -p folio --test note --test derive` = 書き換える歯の file 2 本の歯の全部（便 119 の derive.rs の実の置き場の歯と、note.rs の実の写しを撃つ歯を含む）。
3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

verify が `--test` で名指す歯の file 2 本はどちらも write-set に在る。絞り込みの語 f120_ は 2 本の歯の file の中だけで効き、src の単体の歯に同じ語で始まる関数は無い（base′ で `git grep -n 'fn f120_'` が 0 件）。共通の検証（`.vessel.toml` の common-verify）が workspace の nextest の全部と clippy を撃つ。

### (h) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 床の判定の式（要否が required の欄だけを非空に数える）・導出器の本文・面の生成器・生成区間とその注（value_domains・external_schema_note）・EXTERNAL_SHAPE・要件書・判断の記録・憲法・規則の表・語彙・台帳への記帳・器の側の file。
2. **言えないこと — 約束の行を省いた行は器の受付で断られる。** folio の契約表（設計ノートの YAML）は約束の行を持てないので、verify か done を省いた行は、床も導出器も通るが、器の受付（conditional_missing）が必須の key の欠けとして断る。検出は器の受付の 1 か所で、黙らない（器との合意どおり・ADR-3 決定 (3)）。folio の歯はこの断りを確かめない（器の読み手は folio2 の歯の外）。
3. **まだ分からない — 値域の定数の写し（P-5.6 と行 D-11）。** 行 D-11 は「まだ分からない」を出す判定に使う値域を P-5.6 の範囲に入れるので、EXTERNAL_NEED（と EXTERNAL_SHAPE）は写しを要する定数に読める。一方、生成区間の external_schema は値域を from-file と書き、判断の記録 ADR-11 の下調べ（`docs/design/adr-11-survey.md` の 2-b）はこの 2 つを写し無しと数えている。本便は値を 1 語変えるだけで向きを決めない（D-11 の名指しは §0 の出所）。写しを生成区間へ出すか、生成区間の注を実装に合わせるかは、席が決める（§1 (c) の 3）。
4. **まだ分からない — 約束の行の欄の表の読み方。** folio の読み手は行の見出し field の行だけを数え、見出し promise-field の行の鍵（promise-name 等）は直前の field の行（growth）に付いたまま読まれて使われない。床の結果は変わらない（§1 (d) の歯 3）が、約束の行の欄は folio に登録されない。folio の契約表が約束の行を持つかは M3 の後の判断で、本便では決めない。
5. **まだ分からない — 面の生成器は verify と done の無い行を描かない。** 起草役の実測で、対の設計ノート（行 b が verify も done も持たない）に `folio face --face note --write` を撃つと終了コード 2（まだ分からない・断りの字は 欄 done が無い）で、行 b を外すと書けた。床が通す行を面が断るので、そのような行を持つ設計ノートは配信の組み立てで「まだ分からない」になる（黙らない）。上の 2 のとおり、その行は器の受付でも断られる。
6. **言えないこと — 写しが器の出力と同じ byte であること。** 歯 3 は写しが器の今の形の目印（conditional と約束の行の欄の表）を持つことだけを見て、器の出力との byte 一致は見ない（folio2 の歯は器の命令を撃たない）。起草役の変異で、写しの verify の行だけを required に戻しても歯 3 は通った（done の行の conditional が残るため）。一致は §1 (b) の ② の手順と取り込みの審査で見る。
7. **撤退条件。** (1) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(2) 写しを器の出力に替えた後に `folio derive --dir design-intent --out ../contracts --check` が 0 でなければ（便 119 の着地の後の main で契約表を持つ設計ノートが増えた・または器の出力が既存の欄の順を変えた）、止めて席へ返す（`contracts/example.toml` を write-set に入れる便に直す）。(3) 受付の時点の器の出力が本便の実測（verify と done が conditional・既存 16 行の順が同じ）と違ったら、数を測り直してから運ぶ。(4) 定数と写しのどちらか一方だけでは運ばない（§1 (e) の 3）。(5) 受付の時点の main で `crates/folio/src/floor_note.rs` の EXTERNAL_NEED の周りが便 119 の実測 patch と違う形になっていたら、逐語を測り直してから運ぶ。

### (i) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草役の実測 patch は `~/.local/share/folio2/handoff-2026-09-24/d120-measured.patch`（main 41ed484 に便 119 の実測 patch を当てた木に `git apply --check` が通る・repo には入れない）。起草の記録は同じ dir の d120-draft.md。

1. base′ の写し: `git worktree add --detach <写し> 41ed484` の後に便 119 の実測 patch を当てる（便 119 の着地の後は、その sha の写しに替える）。
2. RED: 本便の patch のうち `crates/folio/tests/` と `tests/fixtures/` の差分だけを当てて `cargo nextest run -p folio --test note --test derive f120_ --no-fail-fast` → 歯 1・3・4 が落ち、歯 2 は通る。
3. 落ちる既存の歯: patch のうち `crates/folio/src/` と `contracts/` の差分だけを当てて workspace の nextest → 全部緑。`contracts/` だけを当てると大半が落ちる（§1 (e) の 3）。
4. 全部: patch の全部を当てて `cargo nextest run --workspace --no-tests=fail --no-fail-fast`（全部緑）・`cargo clippy --workspace --all-targets -- -D warnings`（0 警告）・`cargo run -q -p folio -- check`（合格・違反 0・まだ分からない 0）・`cargo run -q -p folio -- schema --check`（9 file とも一致）・`cargo run -q -p folio -- derive --dir design-intent --out ../contracts --check`（0）。
5. 写し: `scribe2 contracts schema | cmp - contracts/schema.toml`（差 0）。
6. 変異: (d) の表の M1〜M6 を 1 つずつ当てて `cargo nextest run -p folio --test note --test derive f120_ --no-fail-fast`。
7. 門: `cargo run -q -p folio -- ceiling --gate --write-set <write-set の 6 本（接頭辞を剥がす）>`。
8. 余地: (f) の 2 の式で `crates/folio/src/floor_note.rs` を数える。

## 2. 範囲

- 入れる: `crates/folio/src/floor_note.rs` の定数 EXTERNAL_NEED の末尾に conditional の 1 語と注の 1 文。`contracts/schema.toml` を器の命令 `scribe2 contracts schema` の出力に置き換える。新しい凍結の対 2 本（`tests/fixtures/design-note/` の need-conditional.yaml と need-conditional-schema.toml）。`crates/folio/tests/note.rs` の f120_ の歯 3 本と対を置く helper 1 つと頭の注の 2 行。`crates/folio/tests/derive.rs` の f120_ の歯 1 本と頭の注の 3 行。
- 入れない: 床の判定の式と断りの字面・導出器の本文・面の生成器・FLOOR と生成区間 9 file・凍結 anchor（生成区間の anchor・床の凍結の土台・節点の要約値・面の fixture・AC13 の fixture）・`contracts/example.toml`・要件書・判断の記録・憲法・規則の表・語彙・新しい命令と旗・新しい dir・外部 crate・台帳への記帳・器の repo。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| domain | 要否の値域 | `crates/folio/src/floor_note.rs` の EXTERNAL_NEED（末尾に conditional） |
| copy | folio2 の写し | `contracts/schema.toml`（器の命令の出力・手で直さない） |
| pair | 凍結の対 | `tests/fixtures/design-note/` の need-conditional.yaml と need-conditional-schema.toml（手書き・最小） |
| teeth | 歯 | `crates/folio/tests/note.rs` の f120_ の 3 本と `crates/folio/tests/derive.rs` の f120_ の 1 本 |

## 4. 検査（歯）

§1 (d) と (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は作らない。host に要る命令は無い（写しの置き換えに器の命令を使うが、撃てなければ器の repo の file を写す・§1 (b) の ②）。
- 前提の着地: 便 119（行 `dr`）。本便の歯の 1 本は便 119 が足す歯の file に在り、便 119 と同じ src の file を書き換える。受付は便 119 の Landed の後で、base を着地の sha に取り直す改訂を先に当てる。
- 並行の便: 一括 15（main 73a6bf7 で着地）とは重ならない。
- 本便の着地で、利用者の repo（器の repo）の導出 file のままで床（M3 の判定点 ①）と導出物（判定点 ②）を測る前提のうち、要否の値域の分が満ちる。器の導出 file の置き場を版管理の根で解くこと（ADR-16 決定 (7) の ④）は別の便。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ds"
title = "器（scribe2）の導出 file contracts/schema.toml の今の形（verify と done の要否が conditional）を folio の床と導出器が読めるようにする。床と導出器が共有する読み手の要否の値域の定数（crates/folio/src/floor_note.rs の EXTERNAL_NEED）の末尾に conditional の 1 語を足し、注に 1 文（器の条件付きの欄・省いてよいかの判定は器の受付・床は値域だけを見る・便 120・判断の記録 ADR-16 決定 (2)）を足す。根拠は判断の記録 ADR-3 決定 (2) と ADR-16 決定 (2)（規則の表の行 D-11 の名指しはこの 2 つで満たす）、出所は器との合意（台帳 f2-648.166 notes 2026-09-24 07:35 JST・器の orchestrator 席の回答であり持ち主の裁定ではない）。床の判定の式（required の欄だけを非空に数える）と導出器の本文（required でない欄は値が無ければ鍵を省く）は 1 字も触らず、conditional の欄は床が有無を数えず導出器が行の値をそのまま写す。あわせて便 119 の契約の §1 (i) の 5 のとおり folio2 の写し contracts/schema.toml を器の命令 scribe2 contracts schema の出力に置き換える（verify と done の要否・末尾の targets と growth・約束の行の欄の表。既存の欄の順は変わらず contracts/example.toml は動かない）。手書きの最小の凍結の対 2 本（tests/fixtures/design-note/ の need-conditional.yaml = 契約表 2 行のうち行 a だけが verify と done を持つ設計ノート・need-conditional-schema.toml = 器の出力の verify と done の 3 行組を字のまま写した最小の導出 file）を既に在る dir に置き、歯を crates/folio/tests/note.rs に 3 本（対で床が合格／値域の外の要否は 3 つの値とも まだ分からない のまま／実の写しが conditional と約束の行の欄の表を持ち実の置き場で床が合格）と crates/folio/tests/derive.rs に 1 本（対から導出した行 a は verify と done を持ち行 b は持たず要否の字を写さず --check が 0）足す。どれも数を pin しない。生成区間とその注・凍結 anchor・面の生成器・要件書・判断の記録・規則の表は 1 字も触らず、file も消さない。受付は便 119 の着地の後"
req = ["FR10", "FR11"]
section = "1"
write-set = ["crates/folio/src/floor_note.rs", "contracts/schema.toml", "+tests/fixtures/design-note/need-conditional.yaml", "+tests/fixtures/design-note/need-conditional-schema.toml", "crates/folio/tests/note.rs", "crates/folio/tests/derive.rs"]
verify = ["cargo nextest run -p folio --test note --test derive f120_", "cargo nextest run -p folio --test note --test derive", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f120_ の歯 4 本（凍結の対を置いた写しに folio check が合格〔違反 0・まだ分からない 0〕を返し、対の導出 file が conditional を持ち行 b が verify も done も持たない／対の導出 file の verify の要否を sometimes・Conditional・required? に 1 つずつ替えると終了コード 2 でその値を名指す まだ分からない が出る／repo の contracts/schema.toml が conditional と約束の行の欄の表の見出しを持ち実の置き場の写しに folio check が合格／対から folio derive --write した導出物の行 a が verify と done と goal を持ち行 b が verify と done を持たず goal を持ち、導出物に conditional の字が無く、続く --check が 0）が緑、crates/folio/tests/note.rs と crates/folio/tests/derive.rs の既存の歯が全部緑（実の置き場から導出して版管理の contracts/ の導出物と一致し --check が 0 の歯と、実の写しに未知の欄が無く合格する歯を含む）、clippy が 0 警告で、workspace の nextest が全部緑で CI が通る"
<!-- contracts:end -->
