# 設計: 便 129 — 周の引き金の憲法の写しの範囲を条文から改訂の範囲（schema 節・前文・条文）へ広げ、索引の欄の決まりの注 2 つと天井の正本の引き金の注の字を判断の記録 ADR-20 と ADR-14 に揃える（ADR-20 決定 (2)(5)⑤ の便・一括 16 の C-1）

- 要件: FR20（天井の門・要件書 第 1.39 版。注は ADR-20 の発効で「引き金の規範の欄のうち受入基準の確かめる要件・要件の確かめ方の中の受入基準・規則の表の行の条は、索引では辺の欄だが引き金に入り、憲法の前文と schema 節も引き金の一覧に入る」を持つ）。契約表の行の req は FR20 の 1 つ（main に在る id）。FR19（床の定数から欄の決まりの file の生成区間へ写す）も本便が 2 file の生成区間を書き換える要件だが、規範文は変えないので req に入れない（便 126 と同じ扱い）。
- 条: P-5.1・P-5.6（規則・型の一覧は実装の型付きの定数に置き、写しを設計文書の置き場へ決定的に導出する＝引き金の一覧の正本は天井の床の定数・写しは天井の正本の生成区間）/ P-15.2（止める仕掛けの判定の式は同じ関数で確かめる＝印と門が同じ関数で引き金の要約値を測る・本便はその関数の憲法の範囲だけを変える）/ P-6.2（生成物を手で直さない＝生成区間は `folio schema --write` で書き直す）/ P-10.1・P-10.2（独立した凍結 anchor・生成物どうしの突き合わせを唯一の合格判定にしない）/ P-4.2（判定できないものは「まだ分からない」として表に出す）。
- 出所: 判断の記録 **ADR-20**（案 d・発効 2026-09-24 21:33 JST・持ち主の承認・対話面 R-8・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST・main 249a7a2〔PR #308〕）の決定 (2)「引き金の一覧の型付きの定数（crates/folio/src/ceiling.rs）の憲法の写しの範囲を、条文だけから改訂の範囲（schema・precedence・articles）へ広げ、引き金の要約値の関数（gate.rs の trigger_digest）が憲法をその範囲で写す（凍結 anchor の写しの関数を同じ範囲で通す）。天井の正本の生成区間の引き金の一覧の写しも同じ便で 1 行変わる」・決定 (5) ⑤「便 1 本: 決定 (2) の定数と関数、索引の欄の決まりの注（graph.yaml の edge_fields_note）と天井の正本の引き金の注（ceiling.yaml の trigger_note）の生成区間の字、天井の正本の写しと凍結の写しと歯を直す」と、承認要求 `docs/design/adr-20.md` の §3 の #9・§4 の 5（生成区間の字の文案）・§5（便の見積・M・約 29 file）。あわせて、一括 16 の便候補 **C-1**（天井の 28 周目 整合 F-6・索引の欄の決まりの `edge_types_note`・`docs/design/batch16-triage.md` §便候補（C）の中身・台帳 f2-648.171 に束ねた項）を本便に移した（同じ `crates/folio/src/graph.rs` と同じ生成区間と凍結の写しを書き換えるため・§0 の 分割）。規則の表の開発規律行 D-11 は、この行と契約表の行の title が ADR-20 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eb` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 32 本。**新しい file は無い**（`+` は当たらない）。縮む file も消す file も無く（`-` は当たらない）、新しい dir も作らない。歯の file `crates/folio/tests/ceiling.rs` は本文を変えないが、verify が `--test ceiling` で名指すので write-set に入れる。
- 門: 本便は設計文書の正本の生成区間（design-intent の下の ceiling.yaml と graph.yaml）を書き換えるので、天井の門（規則の表の開発規律行 D-12）の対象である。起草役が write-set 32 本を本体の作業ツリー（main 249a7a2 の binary）で `folio ceiling --gate` に渡すと **2（まだ分からない・断りの字 = 印が古い（引き金の要約値の欄が無い））**。本流の印は 28 周目のもので引き金の要約値の欄を持たない。**持ち主の裁定（2026-09-24 21:33 JST・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST・ADR-20 の承認要求の問 2）で、本便は門を経ず器へ出す（この便と便 130 の 2 本に限る）。** §1 (g)。
- 前の便: 前提の便は無い。**base = main 249a7a2（ADR-20 の発効と一括 18・要件書 第 1.39 版・PR #308）。この契約の数はすべて base 249a7a2 の実測（参考値）である**（規則の表の行 D-13）。起草は枝 docs/batch18 の 1498aee の上で始め、取り込みの後に main 249a7a2 へ載せ替えて全部を測り直した（1498aee と 249a7a2 の間で `crates/` と `tests/` は 1 byte も変わらない）。
- 分割: **1 便で運ぶ。** ADR-20 決定 (5) ⑤ は定数と関数と 2 つの注を 1 便と決めた。一括 16 の C-1（`edge_types_note`）は台帳 f2-648.171 の便（便 130）の候補だったが、`crates/folio/src/graph.rs`・design-intent/graph.yaml の生成区間・凍結の写し `tests/fixtures/schema/graph-region.txt`・`crates/folio/tests/schema_docs.rs` の区間の byte 数と要約値の定数を本便と同じく書き換えるので、本便に移した。これで便 130 の write-set と本便の write-set は 1 本も重ならない（§1 (h) の 3）。
- 改訂 b（2026-09-24・独立の検証 d129-verify.md〔条件付き支持・blocking 2・文面 7〕と席の裁定）: done から区間の行数と byte 数を外し §1 を指す形にした（blocking 2・行 D-13）。(c) の 4 の floor_strs の置き場を `crates/folio/src/adr.rs` に直した（文面 2）。(d) の 2 に、単体の歯が包含でなく等しさを数える理由（ADR-20 案 d は範囲を閉じた一覧で固定する・席の裁定）を足した（文面 1）。(h) の 5 の script の置き場を名指した（文面 5）。受付の順を §0 と (g) に 1 句。数・write-set（32 本）・歯（4 本）・verify（8 行）・size は変えていない（便 130 の数の直しは本便に波及しない・模擬を撃ち直した）。
- 並行の便との重なり: 便 130（行 `ec`・台帳 f2-648.171・判断の記録と設計ノートの欄の決まりの生成区間の注）とは write-set が 1 本も重ならない。起草役は両便を main 249a7a2 に同時に当てた写しで workspace の nextest が全部緑（参考値 861 本）・床 4 本 rc 0 を実測した。write-set の意味では同時に走らせてよいが、**席は便 129 → 便 130 の順で逐次に受け付ける**（`crates/folio/tests/sheet.rs` の一時 dir は process id を持たず、同じ host で 2 本の共通の検証を同時に撃つと偶発の Failed が出うる・独立の検証役の実測・契約の中身は変えない）。

## 1. 設計

### (a) いま起きていること（実測・base main 249a7a2）

1. **引き金の要約値は憲法を条文だけで写す。** `crates/folio/src/gate.rs` の trigger_digest は、憲法の正本を型付きで読み、凍結 anchor の写しの関数（`crates/folio/src/anchor.rs` の project・crate の中だけで見える）に範囲として articles の 1 語だけを渡す。project は範囲の各節のうち articles 以外を丸ごと写し、articles は条の 5 欄（id・title・tier・binds・statements）と規範文の 4 欄（id・text・pattern・strength）だけを写す。だから前文（precedence）と schema 節の字は引き金に入らない。
2. **天井の床の定数と生成区間も条文だけを持つ。** `crates/folio/src/ceiling.rs` の床の木 FLOOR の trigger.constitution は articles と statements の 2 行で（葉は同じ file の条の欄と規範文の欄の定数・判断の記録の欄の決まりの床の定数の配列そのもの）、写しの範囲の行は無い。注 trigger_note は「constitution は凍結 anchor の写しと同じ条の欄と規範文の欄」と書く。
3. **前文と schema 節の変更は門を通る（起草役の実測）。** 凍結の束の fixture（`tests/fixtures/ceiling/bundle/source/`）を写し、4 観点合格で今の正本に合う印（sources と trigger と節点の表）を置いてから、前文の平易文を 1 字変えた写しと、schema 節の値域 tier の並びを変えた写しで `folio ceiling --gate --write-set design-intent/constitution.yaml` を撃つと、base の binary はどちらも **0（通す・印の後に引き金の外の変更が在る）** を返す。ADR-20 の承認要求 §5 の実測（前文・schema 節の変更は引き金を動かさない）と同じである。
4. **改訂の範囲は 3 節。** 実の憲法の schema.amendment_scope は [schema, precedence, articles]。判断の記録の欄の決まりの床の定数（`crates/folio/src/floor_adr.rs`）の anchor.scope_minimum も同じ 3 語で、凍結 anchor の写しは置き場の amendment_scope で project を通す。
5. **索引の欄の決まりの 2 つの注が古い。** `crates/folio/src/graph.rs` の床の木の edge_fields_note の 2 文目は「節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は周の引き金にならない（判断の記録 ADR-13 決定 (8)）。」で、ADR-20 決定 (1) の後の読み（受入基準の verifies・要件の verify.ac・規則の表の行の article は規範の欄として引き金に入る）と食い違う（天井の 30 周目 整合 F-1・31 周目 整合 F-1 が拾った注）。同じ file の edge_types_note の末尾「観点が読む文書の欄（reads）と入口の棚の関係は、文書を節点にする便で足す」は、判断の記録 ADR-14 決定 (1)（文書そのものは節点にしない・節どうしのつながりは天井の正本の読む欄の表と入口の棚から引く）と食い違う（28 周目 整合 F-6・一括 16 の C-1）。
6. **生成区間と凍結の写し（参考値・base 249a7a2）。**

| 写し | 行 | byte | sha256 |
| --- | ---: | ---: | --- |
| 天井の正本の生成区間（`tests/fixtures/schema/ceiling-region.txt` と byte 一致） | 45 | 4,537 | 6d095a75be4c5113b653bfcc938f4ef8fdaddb87f322796a6a66117579d7484e |
| 索引の欄の決まりの生成区間（`tests/fixtures/schema/graph-region.txt` と byte 一致） | 35 | 3,601 | da15ba4a87eb45f772c05717751b6e310e9bb41f2e3df94590587646ca65ca04 |
| 節点の要約値の凍結 anchor `tests/fixtures/schema/node-digest-anchor.txt` | 192 | 3,007 | 7874472e73f1a6f76ab922c0369b548f64b22bdd42e825453a6d0eb60c272a51 |

   - 天井の正本の生成区間の写しは 21 か所ある。design-intent/ceiling.yaml と、`tests/fixtures/` の下の ceiling.yaml 20 本（write-set の tests/fixtures の下の 20 本・区間は 20 本とも凍結 anchor と byte 一致）。組み直す手順は `find tests -name ceiling.yaml | sort`（20 行）。`crates/folio/tests/ceiling.rs` の歯 f102_every_ceiling_copy_carries_the_frozen_region_and_the_same_documents が 21 か所を全部歩いて byte 一致を数える。凍結の土台 `tests/fixtures/floor_base/design-intent/ceiling.yaml` もその 1 本である。
   - 索引の欄の決まりの生成区間の写しは design-intent/graph.yaml の 1 か所だけ（`find tests -name graph.yaml` は 0 行）。
   - 節点の要約値の凍結 anchor は、独立の script `tests/fixtures/schema/node-digest.py` が凍結の土台に当てた出力で、末尾の 2 行が残差（節点にも辺の欄にも属さない byte）の要約値と byte の内訳である。残差の母集団は天井の正本の観点の reads が指す文書の file で、土台の ceiling.yaml を含む（残差の byte 142,879・合計 391,051）。`crates/folio/tests/graph.rs` の定数 F99_ANCHOR_SHA256 がこの file の sha256 を持つ。
7. **門の歯の独立の実装は条文だけを写す。** `crates/folio/tests/gate.rs` の関数 trigger_hex は、凍結 anchor ceiling-region.txt の trigger の欄を yaml-rust2 で読み、憲法は articles と statements の欄だけで写した木を正規化の json にして sha256 を測る（folio の code を呼ばない・便 126）。印を今の正本に合わせる歯（put_stamp の fresh）はこの値を印に書く。
8. **base の歯と床（参考値）。** workspace の nextest 855 / 855。床 4 本（`folio check --dir design-intent` 合格 違反 0・まだ分からない 0／`folio inject --check` 68 行 / 7,912 byte 一致／`folio schema --dir design-intent --check` 9 file 一致／`folio derive --out ../contracts --check` 一致 1）はどれも rc 0。`git grep -n 'f129_' -- crates` は 0 件。

### (b) 直す先 — 写しの範囲の定数と 3 つの注

1. **写しの範囲の型付きの定数。** `crates/folio/src/ceiling.rs` に、憲法の写しの範囲の定数（crate の中だけで見える・字の配列 schema・precedence・articles の 3 つ・この順）を足す。置き場は同じ file の憲法の条の欄と規範文の欄の定数の直前。床の木 FLOOR の trigger.constitution の頭に行 scope を足し、葉をこの定数に向ける（同じ一覧を 2 回書かない）。trigger.constitution の行の順は scope・articles・statements になる。
2. **引き金の要約値の関数。** `crates/folio/src/gate.rs` の trigger_digest は、憲法を (1) の定数の 3 語で project に渡す（凍結 anchor の写しの関数を同じ範囲で通す・ADR-20 決定 (2)）。project は範囲の順に schema と precedence を丸ごと、articles を条の 5 欄と規範文の 4 欄で写す（project の本文は変えない）。印（`crates/folio/src/stamp.rs`）は同じ関数を呼ぶので、印と門は同じ範囲で測る（P-15.2・stamp.rs は変えない）。天井の床の定数の注の字（下の 4）以外に、判断の記録・要件書・規則の表・天井の正本の写しと、門の判定の式と印の欄は変えない（ADR-20 決定 (3)）。
3. **置き場の amendment_scope は読まない。** 範囲は組み立てた型付きの定数で閉じ、置き場の憲法の schema.amendment_scope を実行時に読まない（ADR-18 決定 (1)「一覧の正本は実装の型付きの定数」・置き場ごとに引き金を広げる口を持たない）。定数が判断の記録の床の定数 anchor.scope_minimum と同じ 3 語であることは src の歯 1 本が数える（(c) の 4）。
4. **生成区間の字（逐語・`folio schema --write` の導出と、凍結 anchor を直す独立の置換の script の両方がこの字になる）。**
   - 天井の正本の生成区間: 行 `    constitution:` の直後に 1 行 `      scope: [schema, precedence, articles]` が入る。trigger_note の「constitution は凍結 anchor の写しと同じ条の欄と規範文の欄。」を「constitution は凍結 anchor の写しと同じ範囲（schema 節と前文は丸ごと・条は条の欄と規範文の欄）。」へ、末尾の「何にするかの裁定の正本は判断の記録 ADR-18 決定 (1)」を「何にするかの裁定の正本は判断の記録 ADR-18 決定 (1)（ADR-20 が改訂）」へ（ADR-20 の承認要求 §4 の 5 の文案どおり）。区間の外は 1 byte も変わらない。
   - 索引の欄の決まりの生成区間の edge_fields_note: 2 文目「節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は周の引き金にならない（判断の記録 ADR-13 決定 (8)）。」を「節点の要約値はこの欄を落とした本文だけを数えるので、この欄に id を足すだけの変更は節点の要約値を動かさない。周の引き金になるかは天井の正本の引き金の一覧が決め、この欄のうち受入基準の verifies・要件の verify.ac・規則の表の行の article は規範の欄として引き金に入る（判断の記録 ADR-18 決定 (1)・ADR-20）。ほかの辺の欄に id を足すだけの変更は周の引き金にならない（ADR-13 決定 (8)）。」へ（同 §4 の 5 の文案どおり）。残りの文は変えない。
   - 同じ区間の edge_types_note: 末尾「観点が読む文書の欄（reads）と入口の棚の関係は、文書を節点にする便で足す」を「観点が読む文書と入口の棚の関係は辺にしない＝文書そのものを節点にしないので（判断の記録 ADR-14 決定 (1)）、その関係は天井の正本の読む欄の表と入口の棚から引く」へ（一括 16 の C-1 の案どおり）。
   - 3 つの注の新しい字はどれも YAML の引用符を要る字（コロンと空白の並び・空白と井桁の並び・行頭の記号）を持たないので、導出は素の字で書く（base の注と同じ形）。
5. **数（参考値・起草役の独立の 2 実装が一致・(h) の 1）。**

| 写し | 行 | byte | sha256 |
| --- | ---: | ---: | --- |
| 天井の正本の生成区間 | 46 | 4,655 | 8eacece06c52ecda9f12919241b83d420f454141c6c439f39cfb8c520f802bf0 |
| 索引の欄の決まりの生成区間 | 35 | 4,068 | c318f899640d7d603d38ba26c4bc293112e2147276f465ad7e28b1ad73b93a6c |
| 節点の要約値の凍結 anchor（残差の 2 行だけが動く） | 192 | 3,007 | c76bd9acddcfccb3c0caaa7d396ca4cbfe474937891ae3cd7b8a3e9e85016946 |

   - 節点の要約値の凍結 anchor の新しい末尾 2 行は `# 残差 sha256 c2c6fe61c3218d82e48aeefbddc11ce54bcbbbad502e3446f657a4c298a6a2ec` と `# 節点 189・本文の byte 243360・辺の欄の byte 4812・残差の byte 142997・合計 391169`。節点の行 189 本は 1 行も変わらない（土台の天井の正本は節点を持たない・残差だけが 118 byte 増える＝天井の正本の生成区間の増分 4,655 − 4,537 と同じ）。
6. **版は上げない。** 天井の正本と索引の欄の決まりの meta（版・承認欄）は変えない。生成区間だけを書き換えた先例（便 126 が天井の正本の生成区間に 18 行を足した・便 123 が索引の欄の決まりの注を直した）に倣う。ADR-20 の承認要求 §7 の「版を上げるかは契約で決める」への答えである。
7. **folio2 自身の結果。** 床 4 本は rc 0 のまま（`folio schema --check` は 2 file の byte 数が上の値になって 9 file 一致）。`folio build --dir design-intent --out <置き場> --write` の出力は base と 1 byte も変わらない（27 file・起草役の実測）。引き金の要約値は本便の着地で動く（前文と schema 節が入る＝ADR-20 の帰結の 2 つ目・発効の後に本流で周が 1 本要る）。門は本便の着地の後も、本流の印が 28 周目のもので trigger の欄を持たないので 2（まだ分からない）のままである。

### (c) 歯（新しく 4 本）

関数名は f129_ で始める（verify の絞り込みの語・base で `git grep -n 'f129_' -- crates` は 0 件）。

1. **f129_the_gate_is_stale_on_a_precedence_or_schema_edit（`crates/folio/tests/gate.rs`）。** 土台は同じ file の Repo（凍結の束の fixture の写し）と印を置く補助（put_stamp_with_nodes・fresh の印に rest の仮の値と節点の表を足す）。
   - 写し 1 つ目は何も変えずに、write-set design-intent/constitution.yaml で門を撃つ → 終了コード 0・標準出力に 通す と 正本の要約値が同じ。
   - 写し 2 つ目は前文の平易文の行 `  plain: 迷ったら、揃っている方を選びます。` を `  plain: 迷ったら、揃っている方を選ぶ。` に、写し 3 つ目は schema 節の行 `    tier: [always, ask-first, never]` を `    tier: [always, never, ask-first]` にする。どちらも終了コード 2・標準出力に 印が古い と 引き金の要約値が違う。
   - 写し 4 つ目は条 P-1 の関係の欄の `articles: [A-1], sections: [§6]}` を `articles: [A-1, N-1], sections: [§6]}` にする（規範の欄に入らない辺の欄）→ 終了コード 0・標準出力に 通す と 引き金の外の変更。
   - **base で RED。** base の binary は写し 2・3 を 0（通す）で返す（(a) の 3）。さらに印を置く補助の独立の実装（(e) の 1）が写しの範囲を読むので、base の凍結 anchor（scope の行が無い）では補助が落ち、新しい凍結 anchor と base の binary の組では写し 1 つ目が 2（引き金の要約値が違う）で落ちる（起草役の実測）。
2. **f129_the_ceiling_region_lists_the_constitution_scope（`crates/folio/tests/schema_docs.rs`）。** 実の design-intent/ceiling.yaml の生成区間に、字 `    constitution:` の行・`      scope: [schema, precedence, articles]` の行・`      articles: ` で始まる行がこの順で続く。実の憲法の schema.amendment_scope（yaml-rust2 で読む）が [schema, precedence, articles] と一致する。区間に (b) の 4 の trigger_note の新しい 2 つの字が在り、古い字「同じ条の欄と規範文の欄。」が無い。**base では scope の行が無く落ちる＝RED。**
3. **f129_the_graph_region_notes_follow_adr20_and_adr14（`crates/folio/tests/schema_docs.rs`）。** 実の design-intent/graph.yaml の生成区間に、(b) の 4 の edge_fields_note の新しい 3 つの文（節点の要約値を動かさない・3 つの欄は規範の欄として引き金に入る・ほかの辺の欄は引き金にならない）と edge_types_note の新しい字（文書そのものを節点にしないので〔判断の記録 ADR-14 決定 (1)〕）が在り、古い字「周の引き金にならない（判断の記録 ADR-13 決定 (8)）」と「文書を節点にする便で足す」が無い。**base では新しい字が無く落ちる＝RED。**
4. **f129_trigger_constitution_scope_is_the_anchor_scope_minimum（`crates/folio/src/ceiling.rs` の tests）。** (b) の 1 の定数が判断の記録の床の定数 anchor.scope_minimum（`crates/folio/src/floor_adr.rs` の床の木・`crates/folio/src/adr.rs` の floor_strs で読む）と長さ・字・並びまで一致し、床の木 FLOOR の trigger.constitution の行の名の列が scope・articles・statements である。**base では定数が無く組み立たず落ちる＝RED。**

### (d) 採らなかった形

1. **置き場の憲法の schema.amendment_scope を実行時に読んで範囲にする。** ADR-18 決定 (1) は一覧の正本を実装の型付きの定数に置くと決め、ADR-20 決定 (2) も定数の範囲を広げると書く。置き場の字で範囲を広げる・狭める口は、引き金を置き場ごとに黙らせる口になる（N-3.1 の向き）。
2. **anchor.scope_minimum の配列を floor_adr.rs で名前つきの定数に上げ、ceiling.rs がそれを指す（便 126 が条の欄と規範文の欄で採った形）。** floor_adr.rs を書き換えると、同じ file の注を直す便 130 と write-set が重なり、2 本を同時に走らせられない。同じ 3 語であることは歯 4 が数えるので、手書きの写しの食い違いは落ちる。歯 4 は包含（引き金の範囲が改訂の範囲の下限を覆う）でなく等しさを数える。ADR-20 案 d は引き金の範囲を改訂の範囲の 3 節の閉じた一覧で固定するので、下限が増えれば引き金の範囲も同じ便で増やす（黙って下限だけが広がる形を歯が落とす）（P-6.4 の「2 面を人が書き一致を検査で強制する」形に近いが、範囲の下限と引き金の範囲は別の役の一覧で、同じ字を強制する根拠は ADR-20 決定 (2) の「改訂の範囲」の 1 語である）。後の便で一本化するなら floor_adr.rs を触る便に載せる（§5）。
3. **注だけを直し、定数は変えない（ADR-20 の案 a）。** ADR-20 は案 d を採った（承認要求 §2 の問 1）。
4. **天井の正本と索引の欄の決まりの版を上げる。** (b) の 6。
5. **edge_types_note を便 130 に残し、便 130 を本便の着地の後に回す。** 2 本とも門を経ずに出す裁定を持つが、順に走らせると実時間が伸びる。移すと本便の write-set は 1 本も増えない（graph.rs・graph.yaml・graph-region.txt・schema_docs.rs は本便が既に書き換える）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか

1. **門の歯の独立の実装を範囲に合わせる（`crates/folio/tests/gate.rs`）。** 関数 trigger_hex は、凍結 anchor ceiling-region.txt の trigger.constitution の scope の一覧を読み、範囲の各節のうち articles 以外を憲法の写しから丸ごと（yaml-rust2 の値をそのまま正規化の json へ）、articles を今の条の欄と規範文の欄で写した表を作る（範囲に articles が無ければ articles の鍵を置かない）。folio の code は今のまま呼ばない。直さないと、印を今の正本に合わせる歯 6 本（4 観点合格で通す歯・不合格の観点で止める歯・未知の観点でまだ分からないにする歯・便 126 の節点の表の歯 3 本〔引き金の外の変更で通す・平易文の直しで節点 1 個を数える・節点の表が無いと通さない〕）が、印の trigger と今の引き金の要約値の食い違いで 2 に落ちる（独立の検証役の模擬 849 / 855 と起草役の模擬で同じ 6 本）。便 126 の歯 f126_the_gate_is_stale_on_each_normative_edit ほかの期待（終了コードと字）は 1 つも変えない。頭の注に便 129 の 1 行を足す。
2. **天井の正本の写し 20 本。** `tests/fixtures/` の下の ceiling.yaml 20 本の生成区間を、新しい凍結 anchor と byte 一致に書き換える（区間の外は 1 byte も変えない）。直さないと f102 の歯と、写しを床に通す歯（判断の記録・凍結 anchor・床・参照・語彙・link・凍結の命令・id の一覧の歯と床の凍結の場合）が生成区間の食い違いで落ちる。起草役の模擬では、src 3 本と design-intent の 2 file だけを直した写しで 43 本が落ち、写し 20 本と (e) の 3 の定数を直すと 7 本（門の 6 本と節点の要約値の 1 本）に減った。
3. **区間の byte 数と要約値の定数（`crates/folio/tests/schema_docs.rs`）。** 天井の正本の生成区間の行数・byte 数・要約値の定数 3 つを (b) の 5 の値（参考値 46・4,655・8eacece0…）に、索引の欄の決まりの生成区間の byte 数と要約値の定数 2 つを (b) の 5 の値（参考値 4,068・c318f899…）に（行数は不変）。頭の注に便 129 の 1 行を足す。直さないと、天井の正本の側の歯 3 本（実の正本の要約値・1 byte のずれ・書き直しの冪等）と索引の側の歯 3 本（区間と anchor・9 本目の file・ずれ）が落ちる。
4. **凍結 anchor 3 本。** `tests/fixtures/schema/ceiling-region.txt`・`tests/fixtures/schema/graph-region.txt` を (b) の 4 の字に、`tests/fixtures/schema/node-digest-anchor.txt` を (b) の 5 の末尾 2 行に。`crates/folio/tests/graph.rs` の定数 F99_ANCHOR_SHA256 を (b) の 5 の値（参考値 c76bd9ac…）にし、その定数の注に便 129 の 1 行を足す（行数 192・byte 3,007 の歯の字は不変）。src の単体の歯 ceiling_floor_derives_the_frozen_anchor_byte_for_byte（`crates/folio/src/ceiling.rs`・床の木の導出と凍結 anchor の byte 一致）は anchor を直せば緑に戻る。
5. **ほかの歯は落ちない（起草役の実測）。** 上の 1〜4 を当てた写しで workspace の nextest が 859 / 859（base 855 + 歯 4）・clippy 0 警告。便 126 の印の歯（`crates/folio/tests/stamp.rs`）は trigger の形（sha256 と 64 字）だけを見るので落ちない。印の凍結 fixture（`tests/fixtures/ceiling/findings/` の stamp-pass・stamp-fail・stamp-unknown）の trigger は 64 字の 0 の仮の値で、歯が今の値に置き換えるので書き換えない。
6. **凍結の土台の節点の表は動かない。** 節点の要約値の凍結 anchor で動くのは残差の 2 行だけで、節点 189 本の行と辺の表・索引の出力（`tests/fixtures/schema/graph-anchor.txt`）は 1 byte も動かない。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file も縮む file も無い（32 本とも印なし）。32 本の内訳は、src 3 本（`crates/folio/src/ceiling.rs`・`crates/folio/src/gate.rs`・`crates/folio/src/graph.rs`）、歯の file 4 本（`crates/folio/tests/gate.rs`・`crates/folio/tests/schema_docs.rs`・`crates/folio/tests/graph.rs`・`crates/folio/tests/ceiling.rs`〔本文不変・verify の scope〕）、設計文書の生成区間 2 本（design-intent/ceiling.yaml・design-intent/graph.yaml）、凍結 anchor 3 本、天井の正本の写し 20 本。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの 3 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、全部一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/ceiling.rs` | 717 | 783 | 741（+24・定数 1 行・床の木 1 行・注の字・歯 4） |
| `crates/folio/src/gate.rs` | 451 | 1,049 | 453（+2） |
| `crates/folio/src/graph.rs` | 738 | 762 | 739（+1・注の字が 1 行長くなる） |

   3 本とも余地は M の見積 300 を超える。
3. **size は M。** src の増分は 1 file あたり 100 行を超えない見積だが、write-set が 32 本と大きく、ADR-20 の承認要求 §5 と帰結が M と見積もった（先例の便 126 も M・37 本）。
4. **verify は 8 行**で、done の 8 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test gate f129_` = (c) の歯 1。
   2. `cargo nextest run -p folio --test schema_docs f129_` = (c) の歯 2・3。
   3. `cargo nextest run -p folio --bin folio f129_` = (c) の歯 4（src の単体の歯）。
   4. `cargo nextest run -p folio --test gate` = 門の歯の全部（参考値 14 本・(e) の 1 の 6 本を含む）。
   5. `cargo nextest run -p folio --test schema_docs` = 区間の byte 数と要約値の歯の全部（参考値 29 本・(e) の 3 を含む）。
   6. `cargo nextest run -p folio --test graph f99_the_independent_script` = 節点の要約値の凍結 anchor の歯 1 本。
   7. `cargo nextest run -p folio --test ceiling f102_every_ceiling_copy` = 天井の正本の写し 21 か所の byte 一致の歯 1 本。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は gate・schema_docs・graph・ceiling の 4 本で、どれも write-set に在る（ceiling は本文不変）。`--bin folio f129_` の絞り込みの語 f129_ を関数名に持つ src の file は `crates/folio/src/ceiling.rs` の 1 本だけで、write-set に在る。graph と ceiling の絞り込みは関数名の頭の語を長く取り、src の別の file（`crates/folio/src/bundle.rs` の f102_ の単体の歯）に広がらない形にした。

### (g) 門と受付（規則の表の開発規律行 D-12）

1. **門の実測。** 本便は design-intent の下の 2 file の生成区間を書き換えるので門の対象で、本体の作業ツリー（main 249a7a2 の binary）で write-set 32 本を渡すと 2（まだ分からない・印が古い（引き金の要約値の欄が無い））である。
2. **持ち主の裁定。** 行 D-12 は、門が まだ分からない のときは器へ出さず、所見の解消か持ち主の裁定を先に取ると決める。持ち主は 2026-09-24 21:33 JST（対話面 R-8・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST）に、ADR-20 の承認要求の問 2「発効の後の生成区間の字の便 2 本（ADR-20 の §3 の #9 と台帳 f2-648.171）を、門が まだ分からない のまま器へ出す」を推奨どおり承認した。**本便は門を経ず器へ出す（この便と便 130 に限る）。** この裁定は ADR-19 決定 (4) で置き換わった 2026-09-22 の裁定の前例を使うものではない。
3. **門を経ないことの代わり。** 器へ出す前に、席が床 4 本と workspace の nextest を撃つ。本便の中身（引き金の定数と 3 つの注）は、2 本の着地の後に本流で回す天井の 32 周目が初めて読む（ADR-20 の承認要求 §5 の順序 5）。
4. **受付の順。** 席は本便 → 便 130 の順で逐次に受け付ける（§0 の 並行の便との重なり）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d129-draft.md`、script と模擬の差分は同じ dir の d129-scripts（repo には入れない）。

1. 生成区間の字と数の独立の 2 実装: a_regions.py（folio を呼ばない・base の凍結 anchor 2 本に (b) の 4 の字の置換を当てて行・byte・sha256 を出す）と、(b) の 1・2・4 を当てた写しの `folio schema --dir design-intent --write` の導出。2 つの区間が byte 一致（天井 4,655・索引 4,068）。
2. 残差の独立の 2 実装: `python3 tests/fixtures/schema/node-digest.py tests/fixtures/floor_base/design-intent`（写しの 20 本を直した後）と、a_rest_check.py（base の残差の byte 列の中の天井の正本の生成区間を新しい anchor の字に置き換えて sha256 を測る）。どちらも c2c6fe61… と 142,997 byte。
3. 模擬の差分 a-measured.patch を main 249a7a2 に当てて、workspace の nextest（859 / 859）・clippy・床 4 本・`folio build --write` の出力の差 0。便 130 の差分 b-measured.patch と両方を当てても 861 / 861 で、2 つの差分が触る file は重ならない。
4. 門: 本体の作業ツリーで `folio ceiling --gate --dir design-intent --write-set <write-set の 32 本>`。
5. 余地: 持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d119-draft-lines.py` を repo の根で `python3 <その path> crates/folio/src/ceiling.rs crates/folio/src/gate.rs crates/folio/src/graph.rs`（便 119 の script・同じ式）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 憲法・規則の表・要件書・判断の記録・語彙の字（ADR-20 の §3 の #1〜#8・#12 は main 249a7a2 に在る）。天井の正本と索引の欄の決まりの meta と生成区間の外の字。判断の記録と要件書と規則の表と天井の正本の引き金の定数（3 つの辺の欄を持つ定数を含む・ADR-20 決定 (3)）。門の判定の式・印の欄・面の生成器。`crates/folio/src/anchor.rs`・`crates/folio/src/stamp.rs`・`crates/folio/src/floor_adr.rs`。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 次の周が、ADR-20 の改訂の欄と注と本便の字だけで ADR-18 決定 (1)(2) の食い違いを解けたと読むかは、確率的な審査なので言えない（ADR-20 の承認要求 §7）。憲法の改訂の範囲の外の節（north_star・amendment・sources・rules_pointer・glossary_pointer）は本便の後も引き金に入らない（ADR-20 の承認要求 §7 の最後から 2 つ目）。
3. **撤退条件。** (1) 本便の後に (e) の 1〜4 の外の既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に folio2 自身の床の結果（合否・違反と「まだ分からない」の件数）か `folio build` の出力が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で、`crates/folio/src/gate.rs` の trigger_digest の憲法の段・`crates/folio/src/ceiling.rs` の trigger の床の木・`crates/folio/src/graph.rs` の 2 つの注・天井の正本と索引の欄の決まりの生成区間・凍結 anchor 3 本のどれかが base と違っていたら、base を取り直して (a) の 6 と (b) の 5 を測り直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/ceiling.rs` の写しの範囲の定数・床の木の trigger.constitution の行 scope・trigger_note の字・単体の歯 1 本。`crates/folio/src/gate.rs` の trigger_digest の憲法の段（範囲を定数で渡す）。`crates/folio/src/graph.rs` の edge_fields_note と edge_types_note の字。design-intent/ceiling.yaml と design-intent/graph.yaml の生成区間（`folio schema --write`）。凍結 anchor 3 本。天井の正本の写し 20 本の生成区間。`crates/folio/tests/gate.rs` の独立の実装と歯 1 本。`crates/folio/tests/schema_docs.rs` の定数 5 つと歯 2 本。`crates/folio/tests/graph.rs` の定数 1 つ。
- 入れない: 設計文書の生成区間の外の字・meta・版。`crates/folio/src/anchor.rs` の project の本文。`crates/folio/src/stamp.rs`。`crates/folio/src/floor_adr.rs` と判断の記録と設計ノートの欄の決まりの生成区間（便 130）。印の凍結 fixture。`crates/folio/tests/ceiling.rs` の本文。新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| scope | 憲法の写しの範囲の定数 | `crates/folio/src/ceiling.rs`（schema・precedence・articles・床の木の trigger.constitution の頭の行） |
| digest | 引き金の要約値の関数の憲法の段 | `crates/folio/src/gate.rs` の trigger_digest（範囲を定数で project に渡す） |
| notes | 3 つの注の字 | ceiling.rs の trigger_note・graph.rs の edge_fields_note と edge_types_note |
| regions | 生成区間と凍結の写し | design-intent の 2 file・凍結 anchor 3 本・天井の正本の写し 20 本 |
| indep | 門の歯の独立の実装 | `crates/folio/tests/gate.rs` の trigger_hex（範囲を凍結 anchor から読む） |
| teeth | 歯 | f129_ の 4 本（gate 1・schema_docs 2・ceiling.rs の単体 1） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無い。base は main 249a7a2（ADR-20 の発効の後）。
- 並行の便: 便 130（行 `ec`）と write-set が重ならない（§0）。起草の時点で、ほかに write-set が重なる未着地の契約は契約表に無い。
- 本便の着地の後に席が見ること: 台帳の本便の項を閉じる。便 130 の着地と合わせて、本流で天井の 32 周目を回す（ADR-20 の承認要求 §5 の順序 5）。ADR-20 の撤退条件を数える期間（便の着地の時刻から 14 日）の起点を台帳に書く。
- 後の便への申し送り:
  - **数えの道具を組み直す。** ADR-20 の撤退条件 (1) の道具（持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/adr20-scripts/` の intervals.py）と、便 126・127 の起草役の引き金の独立の script は、憲法を条文だけで写す。本便の着地の後は範囲に schema と precedence を足して組み直す（ADR-20 の撤退条件の字「撃つのは便の着地の後に組み直した道具」）。
  - **範囲の一本化。** (d) の 2 の形（anchor.scope_minimum を floor_adr.rs の名前つきの定数に上げ、ceiling.rs の定数がそれを指す）は、floor_adr.rs を触る後の便で載せられる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eb"
title = "判断の記録 ADR-20（案 d・発効 2026-09-24 21:33 JST・裁定 id = 台帳 f2-648 notes 2026-09-24 21:33 JST）の決定 (2) と (5) ⑤ の便と一括 16 の C-1（天井の 28 周目 整合 F-6）: 周の引き金の一覧の憲法の写しの範囲を、条文だけから条 A-2 の改訂の範囲（schema・precedence・articles）へ広げる。crates/folio/src/ceiling.rs に範囲の型付きの定数を足して床の木の trigger.constitution の頭の行 scope に写し、crates/folio/src/gate.rs の trigger_digest が憲法をその範囲で凍結 anchor の写しの関数に通す（印と門は同じ関数・前文と schema 節は丸ごと）。天井の正本の trigger_note と、索引の欄の決まりの edge_fields_note（3 つの辺の欄は規範の欄として引き金に入る・ADR-20）と edge_types_note（文書そのものは節点にしない・ADR-14 決定 (1)）の字を直し、folio schema --write で 2 file の生成区間を書き直す。凍結 anchor 3 本・天井の正本の写し 20 本・区間の byte 数と要約値の定数・門の歯の独立の実装を合わせる。版は上げない。門は まだ分からない だが持ち主の裁定で門を経ず出す（この便と便 130 に限る）。歯は f129_ の 4 本"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/ceiling.rs", "crates/folio/src/gate.rs", "crates/folio/src/graph.rs", "crates/folio/tests/gate.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/ceiling.rs", "design-intent/ceiling.yaml", "design-intent/graph.yaml", "tests/fixtures/schema/ceiling-region.txt", "tests/fixtures/schema/graph-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/adr/effective-no-approval/ceiling.yaml", "tests/fixtures/adr/schema-drift/ceiling.yaml", "tests/fixtures/adr/two-adopted/ceiling.yaml", "tests/fixtures/anchor/no-anchor/ceiling.yaml", "tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "tests/fixtures/ceiling/bundle/source/ceiling.yaml", "tests/fixtures/check/dup-key/ceiling.yaml", "tests/fixtures/check/empty-field/ceiling.yaml", "tests/fixtures/check/missing-file/ceiling.yaml", "tests/fixtures/check/unknown-section/ceiling.yaml", "tests/fixtures/face/ceiling.yaml", "tests/fixtures/floor_base/design-intent/ceiling.yaml", "tests/fixtures/link/adr-id-missing/ceiling.yaml", "tests/fixtures/link/amended-by-orphan/ceiling.yaml", "tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "tests/fixtures/refs/bad-counts/ceiling.yaml", "tests/fixtures/refs/dangling-id/ceiling.yaml", "tests/fixtures/refs/orphan-rule/ceiling.yaml", "tests/fixtures/vocab/exemptions/ceiling.yaml", "tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio --test gate f129_", "cargo nextest run -p folio --test schema_docs f129_", "cargo nextest run -p folio --bin folio f129_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test graph f99_the_independent_script", "cargo nextest run -p folio --test ceiling f102_every_ceiling_copy", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "gate の f129_ の歯 1 本（凍結の束の写しに今の正本に合う 4 観点合格の印を置き、何も変えなければ門が 0 で通す・前文の平易文の 1 字と schema 節の値域の並びの変更はそれぞれ 2 で印が古い・引き金の要約値が違う・条の関係の欄に id を足すだけなら 0 で引き金の外の変更）が緑、schema_docs の f129_ の歯 2 本（天井の正本の生成区間の trigger.constitution に scope の行が articles の前に在り実の憲法の改訂の範囲と同じ 3 語で trigger_note が新しい字・索引の欄の決まりの生成区間の edge_fields_note と edge_types_note が新しい字で古い字が無い）が緑、ceiling.rs の単体の f129_ の歯 1 本（範囲の定数が判断の記録の床の定数 anchor.scope_minimum と一致し trigger.constitution の行が scope・articles・statements）が緑、gate の歯の全部（範囲を凍結 anchor から読む独立の実装で印を合わせる 6 本を含む）が緑、schema_docs の歯の全部（天井の正本と索引の欄の決まりの生成区間の行数・byte 数・要約値の定数〔参考値は §1 (b) の 5〕）が緑、節点の要約値の凍結 anchor の歯（残差の 2 行と要約値）が緑、天井の正本の写し 21 か所の byte 一致の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --check が 9 file 一致を返す"
<!-- contracts:end -->
