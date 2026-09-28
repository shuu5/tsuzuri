# 設計: 便 182 — 判断の記録の欄の決まりの生成区間へ、裁定 id の文法と決定の欄の閉じた一覧を写す（判断の記録 ADR-31 の便 A の後半）

- 要件: FR26（決定の欄の裁定 id の形・要件書 第 1.53 版）と FR19（文書の決まりの部分を、機械の中の決まりから各 file の生成区間へ写す）。どちらも本流の要件書に在る id で、字は変えない。FR26 の注「切り出しの文法は…判断の記録の欄の決まりの ruling_pattern が写す」を実装に写す。
- 条: P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを人が読める型付きデータとして設計文書の置き場へ決定的に導出する）・P-6.2 / P-6.3（生成区間は手で直さず、導出の命令が書く）・N-3.1（欄の決まりは床の定数の写しで、置き場の側から動かせない）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 判断の記録 ADR-31（持ち主の承認 2026-09-28 10:29 JST・対話面 R-8・逐語「全部承認する」・PR #371・本流 78a793f・台帳 f2-648.267）の決定 (1)（決定の欄の閉じた一覧は実装の定数で、欄の決まりの生成区間に写す）と (3)（欄の決まりの ruling_pattern とその注は文法の写しに直す・生成区間）。便 A を 2 便に割った後半で、前半は便 181（行 gb・docs/design/delivery-181.md）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gc` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 26 本（src 3・歯の file 3・設計文書の正本 1・fixture と凍結 anchor 19）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象。write-set に設計文書の正本 `design-intent/adr/schema.yaml` が在る。本流の組み立て（78a793f の binary）に write-set 26 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 便 181（行 gb）が本流 78a793f の上に着地した後の main**（見本 a2f8606 と同じ中身）。この契約の数はすべて a2f8606 の写しの実測（参考値・規則の表の行 D-13）。便 181 が見本と違う中身で着地したら、その main で数え直す。
- 実装の見本: origin の枝 `impl/d182`（commit **394510d**・親 a2f8606）が本便の後の中身で、`git diff a2f8606 394510d` が便の全体の差分（26 file・+461 −33・51,065 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 394510d -- <write-set の file>`）。ただし `design-intent/adr/schema.yaml` は `folio schema --dir design-intent --write` の出力（見本の中身と byte で同じ）。write-set の外は変えない。
- 並行の便との重なり: 便 181 と write-set が重なる（`crates/folio/src/adr.rs`・`crates/folio/src/ruling.rs`・`crates/folio/tests/ruling.rs`・`crates/folio/tests/graph.rs`・`tests/fixtures/schema/node-digest-anchor.txt`）＝本便は便 181 の着地の後に受け付ける。ほかに本便の write-set を書き換える未着地の便の契約は base の時点で無い。

## 1. 設計

### (a) いま起きていること（base a2f8606 の実測・参考値）

1. **実装の定数と欄の決まりの写しが食い違う。** 便 181 の後、床は `crates/folio/src/ruling.rs` の関数 rulings（語頭の台帳の id・question / notes-time / bead）で決定の欄を数え、決定の欄の名 11・骨格の欄 SKELETON・数えない役 SKIP_ROLES・形の名 Form::NAMES を実装の定数に持つ。判断の記録の欄の決まり（`design-intent/adr/schema.yaml` の生成区間・`crates/folio/src/floor_adr.rs` の床の木 FLOOR から `folio schema --write` が導く）は、ruling_pattern が前の字 `[a-z]\d-[0-9a-z]+(\.\d+)?`（語頭を問わない・`.数字` は 1 段）と注「裁定 id は台帳の id（f2-648.2 / s2-07l.149 の形）を 1 つ以上含む…」のままで、決定の欄の一覧・骨格の欄・数えない役・形の種類の写しを持たない（条 P-5.6 の写しの不足・便 181 の (d) の 2）。
2. **写しは置き場ごとに在る。** 生成区間の写しは folio2 の正本 1・床の土台 1・古い形の fixture 16 組・生成区間の凍結 anchor `tests/fixtures/schema/adr-region.txt` の 19 か所で、床は各置き場の写し（注の `_note` の欄を除く）が FLOOR と一致することを見る（`adr/schema.yaml schema.<欄> が床の定数と違う` の違反・種別 adr）。tsuzuri の写し（d927a57）も同じ生成区間を持つ。
3. **base の歯（参考値）。** workspace の nextest 1024 / 1024・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 36 file。`git grep -n f182_ -- crates` は 0 件・行 id `gc` は 0 件。

### (b) 直す先 — 実装の定数を床の木に載せ、生成区間へ導く

1. **`crates/folio/src/ruling.rs`。** 文法の字面の定数 PATTERN（`(?<![0-9A-Za-z_.-])[a-z]\d-[0-9a-z]+(\.\d+)*(:\d{8}T\d{4}Z-\d+| notes( \d{4}-\d{2}-\d{2}( \d{2}:\d[0-9x])?| \d{2}:\d[0-9x])( JST)?)?`・人が読む字面で、床は字の走査で判定する）と、決定の欄の閉じた一覧 FIELDS（11 項・ADR-31 決定 (1) の順: `constitution.yaml meta.approval.ruling`・`constitution.yaml articles[].amended_by[].ruling`・`rules.yaml thresholds[].ruling`・`rules.yaml discipline[].ruling`・`adr/ADR-*.yaml approval.ruling`・`design-note/*.yaml meta.approval[].ruling`・`srs.yaml`・`index.yaml`・`ceiling.yaml`・`intake.yaml`・`graph.yaml` の `meta.approval[].stamp`）を足す。頭の注を写しの在り処に直す。単体の歯 1 本（(c) の 3）。
2. **`crates/folio/src/floor_adr.rs` の FLOOR。** 前の定数 RULING_PATTERN を消し、欄 ruling_pattern の値を ruling::PATTERN に、注 ruling_pattern_note を文法の説明（ADR-31 決定 (3) の字・例 `f2-648 notes 2026-09-28 07:18 JST`・`t3-hub.56:20260927T2259Z-1`・`s2-07l.149`・欄の中の裁定 id は全部切り出す・実在と形の種類は床で決めない）に直す。その後に欄 ruling_forms（Form::NAMES）・ruling_fields（FIELDS）・ruling_skeleton（SKELETON）・ruling_skip_roles（SKIP_ROLES）と注 ruling_fields_note（床の数え方・骨格の欄だけ まだ分からない・作成とレビューの行は数えない・全ての置き場に掛け選ぶ行を持たない・一覧の外〔supersedes_v1・支度表の承認欄・憲法の rationale・凍結 anchor の承認一覧〕・判断の表の行は便 B が足す）を足す。字は見本 394510d のとおり。頭の注に 1 項。
3. **`crates/folio/src/adr.rs`。** 単体の歯の注の欄の数 24 → 25（注 ruling_fields_note が 1 本増えた）だけ。
4. **`design-intent/adr/schema.yaml`。** 生成区間を `folio schema --dir design-intent --write` で書き直す（ruling_pattern とその注の 2 行が変わり、19 行が増える＝+21 −2）。生成区間の外は変えない。
5. **fixture と凍結 anchor（字の期待だけ）。** 床の土台と古い形の fixture 16 組の `adr/schema.yaml`（17 本）の ruling_pattern を新しい字面（単引用符の YAML の字）にし、直後に 4 欄を足す（注は足さない＝床は注を比べない・fixtures-181.py の段 182）。生成区間の凍結 anchor `tests/fixtures/schema/adr-region.txt` は前の anchor の字面の置き換えと挿入で組み直した（region-181.py・導出の命令は使わない）。土台の索引の要約値の anchor `node-digest-anchor.txt` は残差の 2 行だけが動く（独立の script の出力・anchors-18x.py の段 182・節点の行と索引の出力 graph-anchor.txt は不変）。
6. **既存の歯の字の期待（中身は変えない）。** `tests/schema.rs` の生成区間の凍結 anchor の行数・byte・sha256 の定数 3 つ（131 行・25,271 byte → 150 行・28,083 byte）・`tests/graph.rs` の要約値の anchor の sha256。
7. **変えないもの。** 床の判定（便 181 の check_rulings と歩き手と文法の関数）・決定の欄の数と形の種類の判定・書き出し（便 C）・ほかの欄の決まり（設計ノート・規則の表・憲法ほか）の生成区間・憲法と要件書と判断の記録の字・folio2 自身の床の結果と `folio build` の出力（36 file・便 181 の後と byte で同じ）。

### (c) 歯（f182_・base で 0 件）

1. **f182_the_region_holds_the_grammar_and_the_lists（`crates/folio/tests/ruling.rs`）。** folio2 の正本 `design-intent/adr/schema.yaml` が `  ruling_pattern: <文法の字面>` の行を、床の土台の写しが単引用符の同じ行を持ち、どちらも手書きの 4 欄の字（ruling_forms の 1 行・ruling_fields の 11 行・ruling_skeleton の 3 行・ruling_skip_roles の 1 行）を持つ（期待の字は歯の側の手書き＝凍結 anchor）。**base は前の字面で 4 欄が無い＝RED。**
2. **f182_a_copy_that_drops_a_list_drifts（同）。** 床の土台の写しの欄の決まりから ruling_skip_roles の行を消すと、違反はちょうど 1 行 `[adr] adr/schema.yaml schema.ruling_skip_roles（欠落） が床の定数と違う（…）`、文法の字面を前の字に戻すと `schema.ruling_pattern が床の定数と違う` の 1 行。**base は消す行が無い＝RED。**
3. **単体の歯（`crates/folio/src/ruling.rs` の tests の区間）。** f182_the_fields_are_closed_and_hold_the_skeleton（FIELDS は 11 項で重ならず、骨格の欄 3 と 5 正本の stamp の欄 5 を含む）。**base は FIELDS が無く組み立てが落ちる（`--bin folio f182_` は歯が無い）＝RED。**
4. **RED の実測。** 歯の file だけ（見本の tests/ruling.rs）を base に当てると f182_ の 2 本とも落ちる（起草の記録の red-182.log）。base の `--bin folio f182_` は 0 本（nextest の rc 4）。

### (d) 採らなかった形

1. **便 181 と 1 便で運ぶ。** 差分が審査の上限 120,000 byte を超える（便 181 の (d) の 1）。
2. **一覧を注の字にだけ書く（型の無い写し）。** fixture の写し 17 本は動かずに済むが、条 P-5.6 は型付きデータの写しを求め、行 D-11 は閉じた一覧を P-5.6 の範囲に置く。
3. **憲法の条 P-12 の機構の注に欄の名 ruling_fields を書く。** 便 181 は注に「ADR-31 決定 (1) の閉じた一覧」と書いた（正本は判断の記録・写しの在り処が動いても注は変わらない）。本便は憲法を書き換えない。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写し（見本 394510d）で、workspace の nextest **1027 / 1027**（1024 + f182_ の 3 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0・schema --check 一致）・`folio build --write` 36 file（便 181 の後と全 file が byte で同じ）。字の期待を直した既存の歯は (b) の 6 のとおりで、直さないと 7 本が落ちる（schema 6・graph 1・old-teeth-182.log）。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f182_ と f181_ を撃つ）。** 5 通りとも落ちる（mut-182.log）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 決定の欄の一覧の 1 つを重ねる | f182_ の単体・drifts |
| M2 生成区間から ruling_skip_roles を落とす | drifts |
| M3 文法の字面を前の字にする | drifts |
| M4 形の種類の順を替える | f181_ の単体 2 本・drifts |
| M5 骨格の欄から開発規律を落とす | f181_ の単体 1 本と歯 4・drifts |

3. **外の置き場（tsuzuri の写し d927a57）。** 本便の binary では、tsuzuri の欄の決まりの写しが古いので違反 5（ruling_pattern・ruling_forms・ruling_fields・ruling_skeleton・ruling_skip_roles が床の定数と違う）。写しで `folio schema --dir design-intent --write` を撃つと `design-intent/adr/schema.yaml` の生成区間だけが +21 −2 動き、床は合格 0/0（起草の記録）。**tsuzuri は本便の binary を取り込むときに `folio schema --write` を 1 回撃つ**（生成区間の定数が変わる便のいつもの手順）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** どれも印なし（新しい file・縮む file は無い）。差分 51,065 byte（`git diff a2f8606 394510d | wc -c`・26 file・+461 −33）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 3 本（python と awk の 2 実装で一致・cap-18x.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/ruling.rs` | 337 | 1163 | 357（+20） | 1143 |
| `crates/folio/src/floor_adr.rs` | 493 | 1007 | 511（+18） | 989 |
| `crates/folio/src/adr.rs` | 972 | 528 | 972（0） | 528 |

3. **size は S。** src の増分は +38 で S の見積 100 の内、余地の最小 528 は S の 100 を超える。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。便の後の写しで 6 行とも rc 0。
   1. `cargo nextest run -p folio --test ruling f182_` = (c) の 1・2（2 本）。
   2. `cargo nextest run -p folio --bin folio f182_` = (c) の 3（1 本）。
   3. `cargo nextest run -p folio --bin folio adr::tests` = 判断の記録の床の定数の単体の歯（注の欄の数 25 を含む）。
   4. `cargo nextest run -p folio --test schema` = 生成区間の凍結 anchor と書き戻し。
   5. `cargo nextest run -p folio --test graph` = 土台の索引の凍結 anchor。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（ruling・schema・graph）は全部 write-set に在る。`--bin folio` の歯の在り処（ruling.rs・adr.rs）も write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象・0（通す）。
2. **受付。** 便 181 の着地の後に受け付ける。受付の先撃ち（precheck）は、便 181 の見本 a2f8606 の上に本契約を置いた写しで契約に起因する断り 0（起草の記録の precheck-182-on-181.log）。本流 78a793f の上の枝 docs/d181m で撃つと、便 181 の新しい file 2 本（ruling.rs の src と歯）が base に無いので write-set-item-unresolved が 2 つ出る（便 181 の着地で消える）。着地の後、席は tsuzuri へ「本便の binary を取り込むときは `folio schema --write` を 1 回」を返す（(e) の 3）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/d181-draft.md`（便 181 と共通）、script と log は同じ dir の d181-scripts。

1. 模擬: 見本 394510d（`git diff a2f8606 394510d`）。fixture は fixtures-181.py（段 182）、生成区間の anchor は region-181.py、索引の anchor は anchors-18x.py（段 182）で組み直せる。run-181.sh。
2. RED と既存の歯: red-18x.sh・old-teeth-182.sh。
3. 突然変異: mut-18x.py（段 182・5 通り）。
4. 外の置き場: 見本の binary で tsuzuri の写しに `folio check` と `folio schema --write`（起草の記録の手順）。
5. 余地: cap-18x.sh。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 床の判定（便 181）・書き出し（便 C）・判断の表の行を一覧に足すこと（便 B）・ほかの欄の決まりの生成区間・憲法と要件書と判断の記録の字・tsuzuri の写しの書き直し（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) ruling_pattern は人が読む字面（後読みの否定 `(?<!…)` を使う正規表現の方言）で、床はこの字面を解釈せず、同じ文法を字の走査で判定する（字面と走査の一致は単体の歯と、起草の記録の文法の 2 実装の突き合わせで見た）。(2) fixture の写しの注の欄（ruling_pattern_note ほか）は床が比べないので、土台の写しの注は前の字のまま。
3. **撤退条件。** (1) 本便が要件書 FR26 / FR19 の字か憲法の条文を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `floor_adr.rs` の FLOOR・`ruling.rs` の定数が base（a2f8606）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) の 6 に挙げた外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `ruling.rs` の定数 PATTERN と FIELDS と単体の歯 1 本・`floor_adr.rs` の FLOOR の ruling_pattern とその注・4 欄と注 ruling_fields_note・RULING_PATTERN を消す・`adr.rs` の注の欄の数・`design-intent/adr/schema.yaml` の生成区間・fixture の写し 17 本・生成区間の凍結 anchor・索引の残差の anchor・既存の歯の字の期待 2 本・歯の file の f182_ の 2 本。
- 入れない: 床の判定・書き出し・判断の表・ほかの生成区間・憲法と要件書と判断の記録の字・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| consts | 定数 | `ruling.rs` の PATTERN（文法の字面）と FIELDS（決定の欄の閉じた一覧） |
| floor | 床の木 | `floor_adr.rs` の FLOOR の ruling_pattern・ruling_forms・ruling_fields・ruling_skeleton・ruling_skip_roles と注 2 本 |
| region | 生成区間 | `design-intent/adr/schema.yaml`（`folio schema --write`）と fixture の写し 17 本と凍結 anchor |
| teeth | 歯 | f182_ の 3 本（binary 2・単体 1）と既存の歯の字の期待 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 181（行 gb）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ `folio schema --write` の 1 回を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gc"
title = "判断の記録の欄の決まりの生成区間へ、裁定 id の文法と決定の欄の閉じた一覧を写す（判断の記録 ADR-31 の便 A の後半・決定 (1)(3)・要件書 第 1.53 版の FR26 と FR19・便 181 の後）: crates/folio/src/ruling.rs に文法の字面の定数 PATTERN と決定の欄の閉じた一覧 FIELDS（11 項）を足し、crates/folio/src/floor_adr.rs の床の木 FLOOR の ruling_pattern を PATTERN に・注 ruling_pattern_note を文法の説明に直して前の定数 RULING_PATTERN を消し、その後に ruling_forms（形の名 3）・ruling_fields（FIELDS）・ruling_skeleton（骨格の欄 3）・ruling_skip_roles（作成・レビュー）と注 ruling_fields_note を足す。design-intent/adr/schema.yaml の生成区間は folio schema --dir design-intent --write で書き直し、床の土台と古い形の fixture 16 組の欄の決まりの写し 17 本に同じ字面と 4 欄を足し、生成区間の凍結 anchor と土台の索引の残差の anchor と既存の歯の字の期待（tests/schema.rs の生成区間の行数と byte と sha256・tests/graph.rs の要約値の anchor・adr.rs の注の欄の数 25）を直す。床の判定は変えない。歯は f182_ の 3 本（crates/folio/tests/ruling.rs の 2 本と ruling.rs の単体の 1 本）。実装の見本は origin の枝 impl/d182 の commit 394510d（親 a2f8606）で、作業者は write-set の file をその中身にしてよく（adr/schema.yaml は folio schema --write の出力と同じ）、write-set の外は変えない。base = 便 181（行 gb）が本流 78a793f の上に着地した後の main"
req = ["FR26", "FR19"]
section = "1"
write-set = ["crates/folio/src/ruling.rs", "crates/folio/src/floor_adr.rs", "crates/folio/src/adr.rs", "crates/folio/tests/ruling.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/graph.rs", "design-intent/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt"]
verify = ["cargo nextest run -p folio --test ruling f182_", "cargo nextest run -p folio --bin folio f182_", "cargo nextest run -p folio --bin folio adr::tests", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test graph", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/ruling.rs の f182_ の 2 本（folio2 の正本と床の土台の写しの判断の記録の欄の決まりが文法の字面の ruling_pattern の行と手書きの 4 欄〔ruling_forms・ruling_fields 11 行・ruling_skeleton 3 行・ruling_skip_roles〕を持ち、土台の写しから ruling_skip_roles を消すか字面を前の字に戻すと種別 adr の床の定数と違う違反がちょうど 1 行）が緑、binary の単体の f182_ の 1 本（FIELDS は 11 項で重ならず骨格の欄と 5 正本の stamp の欄を含む）が緑、binary の単体の adr::tests の全部（注の欄の数 25 を含む）が緑、tests/schema.rs の歯の全部（判断の記録の欄の決まりの生成区間の凍結 anchor 150 行・28,083 byte を含む）が緑、tests/graph.rs の歯の全部（土台の索引の凍結 anchor を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
