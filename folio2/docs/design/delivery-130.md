# 設計: 便 130 — 判断の記録の欄の決まりの生成区間の注（enums_note・anchor_note の 1 項・limits_note の 3 項）と設計ノートの欄の決まりの注 index_note の古い字を直す（台帳 f2-648.171・天井の 30 周目 実態 F-4 / F-5・31 周目 整合 F-5・一括 16 の C-2 / C-3）

- 要件: FR19（床の定数から欄の決まりの file の生成区間へ写す・要件書 第 1.39 版）。本便は 2 file の生成区間の注の字だけを変え、規範文は変えない。契約表の行の req は FR19 の 1 つ（main に在る id・注の字を直した先例の便 123 と同じ）。
- 条: P-6.2（生成物を手で直さない＝生成区間は `folio schema --write` で書き直す）/ P-5.6（実装の型付きの定数の写しを設計文書の置き場へ決定的に導出する）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する＝機構の状態は憲法の条の機構の欄が持ち、注には写さない）/ P-4.2（判定できないものは「まだ分からない」として表に出す＝撤退条件の種類が組み立てた列に無ければまだ分からない）/ P-10.1（独立した凍結 anchor）。
- 出所: 台帳 **f2-648.171**（便 ③ の後続・memo・2026-09-24）の description と notes。
  - description: 判断の記録の欄の決まりの生成区間の注 enums_note（`crates/folio/src/floor_adr.rs` の床の定数）は「retreat_kind が床の定数と違えば落ちる」と書くが、便 122 で比較を部分集合に揃えたので古い。
  - notes（一括 16 の発効 2026-09-24 12:06 JST）: 一括 16 の C の 4 件（C-1〜C-3・天井の 28 周目 整合 F-6 / F-8・実態 F-3 / F-4）をこの項に束ねる。後の字の案は `docs/design/batch16-triage.md` §便候補（C）の中身。
  - notes（2026-09-24 16:15 JST）: 天井の 31 周目 整合 F-5（判断の記録の欄の決まりの limits_note の 3 か所）を足す（一括 18 の仕分け `docs/design/batch18-triage.md` §便候補（C）の中身・30 周目 実態 F-4・F-5 と同じ項）。32 周目の前に着地させる。
  - 所見の逐語は持ち主の home の下の `.local/share/folio2/ceiling/` の 2026-09-24-round30 の実態の findings.yaml（F-4・F-5・どちらも重さ 直す・場所 判断の記録の欄の決まりの limits_note）と 2026-09-24-round31 の整合の findings.yaml（F-5・直す・同じ場所）。根拠の字は (a) の 2。
  - 一括 16 の C-1（索引の欄の決まりの edge_types_note・`crates/folio/src/graph.rs`）は、同じ file と生成区間と凍結の写しを書き換える便 129（行 `eb`）へ移した。本便が運ぶのは C-2（整合 F-8 / 実態 F-4）と C-3（実態 F-3）と、enums_note と 30・31 周目の 3 件である。
  - 規則の表の開発規律行 D-11 は、この行と契約表の行の title が台帳の項と所見を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ec` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 7 本。**新しい file は無い**（`+` は当たらない）。縮む file も消す file も無く（`-` は当たらない・§1 (f) の 1）、新しい dir も作らない。
- 門: 本便は設計文書の正本の生成区間（design-intent の下の adr/schema.yaml と design-note/schema.yaml）を書き換えるので、天井の門（規則の表の開発規律行 D-12）の対象である。起草役が write-set 7 本を本体の作業ツリー（main 249a7a2 の binary）で `folio ceiling --gate` に渡すと **2（まだ分からない・断りの字 = 印が古い（引き金の要約値の欄が無い））**。**持ち主の裁定（2026-09-24 21:33 JST・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST・判断の記録 ADR-20 の承認要求の問 2）で、本便は門を経ず器へ出す（この便と便 129 の 2 本に限る）。** §1 (g)。
- 前の便: 前提の便は無い（便 ③ = 便 122 は着地済み）。**base = main 249a7a2（ADR-20 の発効と一括 18・要件書 第 1.39 版・PR #308）。この契約の数はすべて base 249a7a2 の実測（参考値）である**（規則の表の行 D-13）。
- 改訂 b（2026-09-24・独立の検証 d129-verify.md〔条件付き支持・blocking 2・文面 7〕と席の裁定）: anchor_note の新しい字の指し先を「limits_note の末項」（末項は 16 項目めの列の根の項で誤り）から「limits_note の条の廃止（status）の項」へ（blocking 1）。limits_note の 4 項目めの括弧に、fixture の不在を落とす側（歯 crates/folio/tests/floor_cases.rs）を名指した（文面 3）。判断の記録の欄の決まりの生成区間の byte 数と要約値を 2 実装で取り直した（(b) の 7・参考値）。done から区間の行数と byte 数を外し §1 を指す形にした（blocking 2・行 D-13）。歯 f130_ の新しい字に anchor_note の字を足した（5 つ → 6 つ）。受付の順を §0 と (g) に 1 句。write-set（7 本）・歯の本数（2）・verify（3 行）・size は変えていない。
- 並行の便との重なり: 便 129（行 `eb`・ADR-20 の生成区間の字の便）とは write-set が 1 本も重ならない。起草役は両便を main 249a7a2 に同時に当てた写しで workspace の nextest が全部緑（参考値 861 本）・床 4 本 rc 0 を実測した。write-set の意味では同時に走らせてよいが、**席は便 129 → 便 130 の順で逐次に受け付ける**（`crates/folio/tests/sheet.rs` の一時 dir は process id を持たず、同じ host で 2 本の共通の検証を同時に撃つと偶発の Failed が出うる・独立の検証役の実測・契約の中身は変えない）。

## 1. 設計

### (a) いま起きていること（実測・base main 249a7a2）

1. **生成区間の注の字の正本は床の定数。** 判断の記録の欄の決まり（design-intent/adr/schema.yaml）の生成区間は `crates/folio/src/floor_adr.rs` の床の木 FLOOR から、設計ノートの欄の決まり（design-intent/design-note/schema.yaml）の生成区間は `crates/folio/src/floor_note.rs` の床の木 FLOOR から、`folio schema --write` が導出する。末尾が _note の欄は床が一切読まない説明の注で（limits_note の 10 項目め）、直すには床の定数の字を直して導出し直す。
2. **古い字は 6 か所（逐語）。**

| # | 欄（file） | 今の字（逐語の抜き書き） | 今の実態 |
| --- | --- | --- | --- |
| 1 | enums_note（floor_adr.rs） | retreat_kind は憲法 schema.enums.retreat_kind と同じ（食い違えば落ちる）。 | 便 122 の後、`crates/folio/src/link.rs` の retreat_kind は、置き場の憲法の値域の retreat_kind が判断の記録の床の定数（組み立て時に憲法から導出した列・便 49）の部分集合かを数え、列に無い値が在れば「まだ分からない」1 件で違反は出さない（順と重複は問わない・FR25） |
| 2 | anchor_note の 10 項目め（floor_adr.rs） | （番号は消さない・P-7。条の廃止（status）の機構は day-1 の憲法 schema に無い＝M0 で決める）。 | M0 は 2026-09-18 に着地したが決まっていない。判断の記録 ADR-2 の帰結は「M0 では決めておらず…台帳に起票した（f2-648.100）」と書く（30 周目 実態 F-4 の note が名指す字） |
| 3 | limits_note の 4 項目め（floor_adr.rs） | P-10.1（live は M0）は未発効で、day-1 の代替として tests/floor_cases.yaml を置くが、床の入力ではない（fixture の不在は検出されない）＝M0 で床の入力に取り込む。 | 憲法の条 P-10 の機構の欄は live now。floor_cases.yaml を読むのは歯 `crates/folio/tests/floor_cases.rs` だけで、床の実装は読まない（判断の記録 ADR-2 の注・30 周目 実態 F-4・31 周目 整合 F-5） |
| 4 | limits_note の 6 項目め（floor_adr.rs） | …の機構は憲法のとおり便 0 の検査（CI）で置く＝day-1 の床では効いていない。…行の本文の書き換えは止めない（便 0 の検査の領分）。 | 憲法の条 P-17 の機構の欄は live M1 で、注は「この差分を数える口は未実装である（便 0 は 2026-09-17 に着地したが、その検査は置かれなかった）」と書く（30 周目 実態 F-5） |
| 5 | limits_note の 12 項目め（floor_adr.rs） | 条の廃止（status）の機構は day-1 の憲法 schema に無い（article の欄に status が無く、足しても写しの外）。…形は M0 で決める。 | M0 で決めていない（台帳 f2-648.100・ADR-2 の帰結）。便 128 の後、条に status を足すと床が未知の欄の違反を出す（起草役の実測: 実の design-intent の写しの条 P-10 に status: retired を足すと `[未知の欄] constitution.yaml: 条 P-10 の未知の欄「status」`・不合格 違反 1） |
| 6 | index_note（floor_note.rs） | 節点と辺の欄も、節点の種類と辺の型の閉じた一覧も、索引の欄の決まり design-intent/graph.yaml が正本として持つ | 正本は実装の型付きの定数（`crates/folio/src/graph.rs` の NODE_KINDS・EDGE_TYPES・EDGE_FIELDS ほかの床の木）で、graph.yaml はその写しの生成区間を持つ（28 周目 実態 F-3・一括 16 の C-3） |

3. **生成区間と凍結の写し（参考値・base 249a7a2）。**

| 写し | 行 | byte | sha256 |
| --- | ---: | ---: | --- |
| 判断の記録の欄の決まりの生成区間（`tests/fixtures/schema/adr-region.txt` と byte 一致） | 138 | 25,326 | 7a500d7a811bd698dac4f8feb10756785e449e973f3e9f434e737767f16f2ee1 |
| 設計ノートの欄の決まりの生成区間（`tests/fixtures/schema/note-region.txt` と byte 一致） | 137 | 16,721 | d10d261b77f94b3410fdda4f4e3672f94ba11647d1e14ff222685b4e895ed83b |

   - `crates/folio/tests/schema.rs` の定数（行数・byte 数・要約値の 2 組）がこの値を持つ。src の単体の歯 2 本（`crates/folio/src/adr.rs` と `crates/folio/src/note.rs` の、床の木の導出と凍結 anchor の byte 一致）も同じ anchor を読む。
   - **写しを全数で列挙した（床の定数の写し）。** `find tests -path '*adr/schema.yaml'` は 17 本、`find tests -path '*design-note/schema.yaml'` は 1 本。生成区間の印を持つのは凍結の土台（`tests/fixtures/floor_base/design-intent/`）の 2 本だけで、どちらも凍結の古い字のまま（判断の記録の側は enums_note と limits_note の古い字を持ち、設計ノートの側は index_note を持たない）。ほかの 16 本は生成区間を持たない。土台の 2 本を突き合わせる歯は無く（天井の正本の写しの歯 f102 のような写しの全数の歯は判断の記録と設計ノートの欄の決まりには無い）、本便も変えない。土台は節点の要約値の凍結 anchor（`tests/fixtures/schema/node-digest-anchor.txt`）の残差の母集団に入るので、変えると便 129 と write-set が重なる。
4. **base の歯と床（参考値）。** workspace の nextest 855 / 855。床 4 本（`folio check --dir design-intent` 合格 違反 0・まだ分からない 0／`folio inject --check` 68 行 / 7,912 byte 一致／`folio schema --dir design-intent --check` 9 file 一致／`folio derive --out ../contracts --check` 一致 1）はどれも rc 0。`git grep -n 'f130_' -- crates` は 0 件。

### (b) 直す先 — 6 か所の字（逐語）

字の正本は床の定数の字で、`folio schema --write` が生成区間へ導出する。どの字も YAML の引用符を要る字（コロンと空白の並び・空白と井桁の並び・行頭の記号）を持たないので、導出は base と同じ形（limits_note と anchor_note は字下げの一覧・enums_note と index_note は素の字）になる。

1. enums_note: 「retreat_kind は憲法 schema.enums.retreat_kind と同じ（食い違えば落ちる）。」を「retreat_kind は組み立て時に憲法 schema.enums.retreat_kind から導出した列（便 49）。置き場の憲法の retreat_kind にこの列に無い値が在れば「まだ分からない」（部分集合の比較・順と重複は問わない・FR25・便 122）。」へ。後ろの surface の文は変えない。
2. anchor_note の 10 項目め: 括弧「（番号は消さない・P-7。条の廃止（status）の機構は day-1 の憲法 schema に無い＝M0 で決める）。」を「（番号は消さない・P-7。条の廃止の機構は無い＝limits_note の条の廃止（status）の項）。」へ（項の番号で指すと並びが変わると古くなるので、項の名で指す）。項の残りは変えない。
3. limits_note の 4 項目め: 3 文目以降「P-10.1（live は M0）は未発効で、day-1 の代替として tests/floor_cases.yaml を置くが、床の入力ではない（fixture の不在は検出されない）＝M0 で床の入力に取り込む。」を「P-10.1 の機構の状態は憲法の条 P-10 の機構の欄が持つ（ここには写さない）。tests/floor_cases.yaml は歯の入力で、床の入力ではない（fixture の不在は床では検出されず、歯 crates/folio/tests/floor_cases.rs が落とす・判断の記録 ADR-2 の注）。」へ（一括 16 の C-2 ① の案に、不在を落とす側を足した・改訂 b。歯は fixture が読めなければ panic で落ちる）。
4. limits_note の 6 項目め: 「の機構は憲法のとおり便 0 の検査（CI）で置く＝day-1 の床では効いていない。」を「の機構と今の状態は憲法の条 P-17 の機構の欄が持つ（ここには写さない）。」へ、末尾の括弧「（便 0 の検査の領分）」を「（P-17 の機構の領分）」へ。C-2 ② の案（「便 0 は着地したがその検査は置かれず、床では効いていない」）は機構の今の状態を注に写すので、状態が変わるとまた古くなる。3 と同じく状態は憲法の機構の欄に任せる形にした（P-6.3）。
5. limits_note の 12 項目め: 「条の廃止（status）の機構は day-1 の憲法 schema に無い（article の欄に status が無く、足しても写しの外）。条は消せず、廃止も表せない＝P-7.2「廃止は状態で」を条に適用する形は M0 で決める。」を「条の廃止（status）の機構は憲法 schema に無い（条の欄に status が無く、足すと床が未知の欄で落とす・便 128）。条は消せず、廃止も表せない＝P-7.2「廃止は状態で」を条に適用する形は M0 では決めておらず、台帳 f2-648.100 に起票した（判断の記録 ADR-2 の帰結）。」へ（C-2 ③ の案に、便 128 の後の実測を足した）。
6. index_note: 「節点と辺の欄も、節点の種類と辺の型の閉じた一覧も、索引の欄の決まり design-intent/graph.yaml が正本として持つ＝この節はその置き場を指すだけで写しを持たない（P-6.3）。」を「節点と辺の欄と、節点の種類と辺の型の閉じた一覧の正本は実装の型付きの定数（crates/folio/src/graph.rs）で、その写しは索引の欄の決まり design-intent/graph.yaml の生成区間に在る＝この節はその置き場を指すだけで写しを持たない（P-6.3）。」へ（一括 16 の C-3 の案どおり）。
7. **数（参考値・base 249a7a2 に本便の字を当てた実測・起草役の独立の 2 実装が一致・(h) の 1）。** 行数は変わらない（注の項の数も変わらない）。改訂 b の前の字（指し先が末項・4 項目めの括弧が base の案どおり）では 25,619 byte、blocking 1 の直しだけを当てると 25,643 byte・19eb9bac…（独立の検証役の参考値と一致）で、4 項目めの括弧の直しで 50 byte 増えて下の値になる。

| 写し | 行 | byte | sha256 |
| --- | ---: | ---: | --- |
| 判断の記録の欄の決まりの生成区間 | 138 | 25,693 | 8aca8f764952001b1e2eca1b7f69c1b61e6edb09a7389a466df0120b9789ddc5 |
| 設計ノートの欄の決まりの生成区間 | 137 | 16,806 | 21dc7f7cb7304f2da4890cb501aab731697d6b2a620899c23c5610791f994a20 |

8. **版は上げない。** 2 file の meta（版・承認欄）は変えない（生成区間の注だけを直した先例の便 123 に倣う）。
9. **folio2 自身の結果。** 床 4 本は rc 0 のまま（`folio schema --check` は 2 file の byte 数が上の値になって 9 file 一致）。`folio build --dir design-intent --out <置き場> --write` の出力は base と 1 byte も変わらない（27 file・起草役の実測）。_note の欄は床が読まないので、`folio check` の結果も変わらない。天井の周の引き金の一覧は判断の記録の欄の決まりと設計ノートの欄の決まりを数えないので、引き金の要約値も動かない。

### (c) 歯（新しく 2 本）

関数名は f130_ で始める（verify の絞り込みの語・base で `git grep -n 'f130_' -- crates` は 0 件）。2 本とも `crates/folio/tests/schema.rs` に置き、同じ file の生成区間を切り出す補助（begin の行の次から end の行の手前まで）で実の正本の区間を読む。

1. **f130_the_adr_region_notes_carry_no_stale_milestone_text。** 実の design-intent/adr/schema.yaml の生成区間に、古い字 7 つ（「と同じ（食い違えば落ちる）」・「P-10.1（live は M0）は未発効」・「＝M0 で床の入力に取り込む」・「便 0 の検査（CI）で置く」・「（便 0 の検査の領分）」・「＝M0 で決める」・「形は M0 で決める。」）が 1 つも無く、(b) の 1〜5 の新しい字 6 つ（部分集合の比較の文・P-10 の機構の欄に任せる文・floor_cases.yaml は歯の入力で不在は歯が落とす文・anchor_note の条の廃止の項を名で指す括弧・P-17 の機構の欄に任せる文・f2-648.100 に起票した文）が全部在る。**base では古い字が在り落ちる＝RED**（起草役の実測）。
2. **f130_the_note_region_index_note_names_the_graph_constants。** 実の design-intent/design-note/schema.yaml の生成区間に、(b) の 6 の新しい字（閉じた一覧の正本は実装の型付きの定数〔crates/folio/src/graph.rs〕で、その写しは索引の欄の決まり design-intent/graph.yaml の生成区間に在る）が在り、古い字「design-intent/graph.yaml が正本として持つ」が無い。**base では新しい字が無く落ちる＝RED**（起草役の実測）。

### (d) 採らなかった形

1. **一括 16 の C-2 ② の案の字（便 0 は着地したがその検査は置かれず、床では効いていない）。** (b) の 4。
2. **凍結の土台の 2 本の生成区間も直す。** 土台は凍結の古い字のまま置く（便 122 の独立の検証の実測・直す先でない）。直すと節点の要約値の凍結 anchor の残差が動き、便 129 と write-set が重なって同時に走らせられない。
3. **anchor_note の 10 項目めを残す。** 30 周目 実態 F-4 の note が「同じ欄の条の廃止の機構は…M0 で決める」と名指す字は、limits_note の 12 項目めと anchor_note の 10 項目めの両方に在る。片方だけを直すと、次の周が残りを同じ所見で拾う。
4. **limits_note の 9 項目めの「（day-1 では想定しない）」も直す。** 写しの取り方の移行を初版で想定しなかったという当時の記録で、所見は無く、今の実態とも食い違わない（§1 (i) の 2）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか

1. **凍結 anchor 2 本と定数。** `tests/fixtures/schema/adr-region.txt` と `tests/fixtures/schema/note-region.txt` を (b) の 1〜6 の字に置き換え（字面の置き換え・区間の外の字は無い）、`crates/folio/tests/schema.rs` の判断の記録の側の byte 数と要約値の定数 2 つを (b) の 7 の値（参考値 25,693・8aca8f76…）に、設計ノートの側の 2 つを (b) の 7 の値（参考値 16,806・21dc7f7c…）に（行数 138・137 は不変）。定数の注と頭の注に便 130 の 1 行を足す。
2. **直さないと落ちる歯（起草役の実測）。** src 2 本と design-intent の 2 file だけを直した写しで、workspace の nextest は 11 本が落ちる。
   - `crates/folio/tests/schema.rs` の 9 本: 判断の記録の側の実の正本の要約値・1 byte のずれ・書き直しの冪等・区間の外を変えない・凍結 anchor と orchestrator 席の歯と、設計ノートの側の実の正本の要約値・1 byte のずれ・書き直しの冪等・凍結 anchor と導出の命令の名の歯。
   - src の単体の歯 2 本（adr.rs と note.rs の、床の木の導出と凍結 anchor の byte 一致）。
   上の 1 を当てると 11 本とも緑に戻り、workspace の nextest は 857 / 857（base 855 + 歯 2）・clippy 0 警告。
3. **ほかの凍結 anchor は動かない。** 天井の正本・索引の欄の決まりほかの生成区間の写し 7 本と、節点の要約値の凍結 anchor と、凍結の土台は 1 byte も動かない。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file も縮む file も無い（7 本とも印なし）。7 本は src 2 本（`crates/folio/src/floor_adr.rs`・`crates/folio/src/floor_note.rs`）、歯の file 1 本（`crates/folio/tests/schema.rs`）、設計文書の生成区間 2 本（design-intent/adr/schema.yaml・design-intent/design-note/schema.yaml）、凍結 anchor 2 本。`wc -l` の行数はどの file も変わらず、幅 120 の正規化の行数は floor_adr.rs が 1 行増える（509 → 510・改訂 b の字）。
2. **余地（CapHeadroom）。** 各行を ceil(字数 / 120) で数えて足す（空行は 1）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/floor_adr.rs` | 509 | 991 | 510 |
| `crates/folio/src/floor_note.rs` | 489 | 1,011 | 489 |

   2 本とも余地は S の見積 100 を超える。
3. **size は S。** src の増分は 1 file あたり 0〜数行（字の置き換えだけ）。
4. **verify は 3 行**で、done の 3 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test schema f130_` = (c) の歯 2 本。
   2. `cargo nextest run -p folio --test schema` = 生成区間の歯の全部（参考値 20 本・(e) の 2 の 9 本を含む）。
   3. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は schema の 1 本で、write-set に在る。絞り込みの語 f130_ を関数名に持つ file はその 1 本だけである。src の単体の歯 2 本（(e) の 2）は共通の検証（`.vessel.toml` の common-verify の workspace の nextest）が走らせる。絞り込みで名指すと、同じ語を名に持つ天井の正本の単体の歯（`crates/folio/src/ceiling.rs`・便 129 の write-set）に広がるので名指さない。

### (g) 門と受付（規則の表の開発規律行 D-12）

1. **門の実測。** 本便は design-intent の下の 2 file の生成区間を書き換えるので門の対象で、本体の作業ツリー（main 249a7a2 の binary）で write-set 7 本を渡すと 2（まだ分からない・印が古い（引き金の要約値の欄が無い））である。
2. **持ち主の裁定。** 行 D-12 は、門が まだ分からない のときは器へ出さず、所見の解消か持ち主の裁定を先に取ると決める。持ち主は 2026-09-24 21:33 JST（対話面 R-8・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST）に、判断の記録 ADR-20 の承認要求の問 2「発効の後の生成区間の字の便 2 本（ADR-20 の §3 の #9 と台帳 f2-648.171）を、門が まだ分からない のまま器へ出す」を推奨どおり承認した。**本便は門を経ず器へ出す（この便と便 129 に限る）。**
3. **門を経ないことの代わり。** 器へ出す前に、席が床 4 本と workspace の nextest を撃つ。本便の字は、2 本の着地の後に本流で回す天井の 32 周目が初めて読む。
4. **受付の順。** 席は便 129 → 本便の順で逐次に受け付ける（§0 の 並行の便との重なり）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d129-draft.md`（便 129 と同じ記録）、script と模擬の差分は同じ dir の d129-scripts（repo には入れない）。

1. 生成区間の字と数の独立の 2 実装: b_regions.py（folio を呼ばない・base の凍結 anchor 2 本に b_texts.py の置換の表〔(b) の 1〜6 の前の字と後の字〕を当てて行・byte・sha256 を出す）と、同じ表を床の定数の字に当てた写しの `folio schema --dir design-intent --write` の導出。2 つの区間が byte 一致（参考値 25,693・16,806）。
2. 模擬の差分 b-measured.patch を main 249a7a2 に当てて、workspace の nextest（857 / 857）・clippy・床 4 本・`folio build --write` の出力の差 0。便 129 の差分と両方を当てても 861 / 861。
3. 門: 本体の作業ツリーで `folio ceiling --gate --dir design-intent --write-set <write-set の 7 本>`。
4. 余地: `python3 d119-draft-lines.py crates/folio/src/floor_adr.rs crates/folio/src/floor_note.rs`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 憲法・規則の表・要件書・判断の記録（ADR-2 の注と帰結を含む）・語彙の字。2 file の meta と生成区間の外の字。_note でない欄（床の定数の値）。凍結の土台の 2 本。索引の欄の決まりの edge_types_note（便 129）。台帳への記帳（台帳 f2-648.171 と f2-648.100 は席）。外部 crate。
2. **言えないこと。**
   - limits_note の 9 項目めの「（day-1 では想定しない）」と、ほかの _note の欄の字の全部が今の実態と合うかは、本便では数えていない。本便が見たのは所見の 5 件と台帳の項が名指す 6 か所だけである。
   - (b) の 3 の新しい字は「P-10.1 の機構の状態は憲法の条 P-10 の機構の欄が持つ」と書き、fixture の不在を落とすのは歯だと名指す。その機構の欄の注（凍結 fixture を検査の入力に持ち、fixture 不在はまだ分からないに落とす・live now）と、床が tests/floor_cases.yaml を読まず歯が panic で落とす実態が合うかは、本便の外（憲法の側の字）で、次の周が拾いうる。
3. **撤退条件。** (1) 本便の後に (e) の 2 の外の既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に folio2 自身の床の結果（合否・違反と「まだ分からない」の件数）か `folio build` の出力が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で、`crates/folio/src/floor_adr.rs` の enums_note・anchor_note・limits_note か `crates/folio/src/floor_note.rs` の index_note の字、2 file の生成区間、凍結 anchor 2 本のどれかが base と違っていたら、base を取り直して (a) の 3 と (b) の 7 を測り直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/floor_adr.rs` の enums_note・anchor_note の 10 項目め・limits_note の 4・6・12 項目めの字。`crates/folio/src/floor_note.rs` の index_note の字。design-intent/adr/schema.yaml と design-intent/design-note/schema.yaml の生成区間（`folio schema --write`）。凍結 anchor 2 本。`crates/folio/tests/schema.rs` の定数 4 つと注と歯 2 本。
- 入れない: 床の定数の値（_note でない欄）。2 file の meta・版・生成区間の外の字。凍結の土台。索引の欄の決まり（便 129）。ほかの _note の欄の字。新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| adrnotes | 判断の記録の欄の決まりの注 5 か所 | `crates/folio/src/floor_adr.rs` の enums_note・anchor_note・limits_note |
| notenote | 設計ノートの欄の決まりの注 1 か所 | `crates/folio/src/floor_note.rs` の index_note |
| regions | 生成区間と凍結の写し | design-intent の 2 file・凍結 anchor 2 本・`crates/folio/tests/schema.rs` の定数 |
| teeth | 歯 | f130_ の 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無い。base は main 249a7a2。
- 並行の便: 便 129（行 `eb`）と write-set が重ならない（§0）。起草の時点で、ほかに write-set が重なる未着地の契約は契約表に無い。
- 本便の着地の後に席が見ること: 台帳 f2-648.171 を本便の行で閉じる（C-1 は便 129 が運ぶ）。便 129 の着地と合わせて本流で天井の 32 周目を回し、判断の記録の欄の決まりの limits_note（30 周目 実態 F-4・F-5・31 周目 整合 F-5）が支持されないことを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ec"
title = "台帳 f2-648.171（判断の記録の欄の決まりの注 enums_note の直し・一括 16 の C を束ねた項）と天井の 30 周目 実態 F-4・F-5 と 31 周目 整合 F-5 の 1 便: 判断の記録の欄の決まりの生成区間の注 5 か所（crates/folio/src/floor_adr.rs の enums_note を撤退条件の種類の部分集合の比較の字へ・anchor_note の条の廃止の括弧と limits_note の 12 項目めを M0 で決めず台帳 f2-648.100 に起票した字へ・limits_note の 4 項目めと 6 項目めを P-10 と P-17 の機構の状態を憲法の機構の欄に任せる字へ）と、設計ノートの欄の決まりの注 index_note（crates/folio/src/floor_note.rs・索引の一覧の正本は crates/folio/src/graph.rs の定数で graph.yaml は写しの生成区間を持つ）の字を直し、folio schema --write で 2 file の生成区間を書き直し、凍結 anchor 2 本と tests/schema.rs の byte 数と要約値の定数を合わせる。行数・床の結果・面の出力は変わらない。版は上げない。門は まだ分からない だが持ち主の裁定（2026-09-24 21:33 JST・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST）で門を経ず出す（この便と便 129 に限る）。一括 16 の C-1 は便 129 が運ぶ。歯は f130_ の 2 本"
req = ["FR19"]
section = "1"
write-set = ["crates/folio/src/floor_adr.rs", "crates/folio/src/floor_note.rs", "crates/folio/tests/schema.rs", "design-intent/adr/schema.yaml", "design-intent/design-note/schema.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/note-region.txt"]
verify = ["cargo nextest run -p folio --test schema f130_", "cargo nextest run -p folio --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の f130_ の歯 2 本（実の判断の記録の欄の決まりの生成区間に古い字 7 つが無く新しい字 6 つが在る・実の設計ノートの欄の決まりの生成区間の index_note が graph.rs の定数を正本と名指し古い字が無い）が緑、schema の歯の全部（判断の記録と設計ノートの欄の決まりの生成区間の行数・byte 数・要約値の定数〔参考値は §1 (b) の 7〕と凍結 anchor との byte 一致を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑（判断の記録と設計ノートの床の木の導出と凍結 anchor の byte 一致の単体の歯を含む）で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --check が 9 file 一致を返す"
<!-- contracts:end -->
