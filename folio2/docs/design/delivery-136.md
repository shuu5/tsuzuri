# 設計: 便 136 — 索引の正本の行を行の逐語で切れない置き場を床で落とす（folio check と build の床）・folio init の雛形と floor_cases の書き手を裸の形に

- 要件: FR5（構造の床の 3 値・実行できなかった検査を合格と表示しない）/ FR14（機械が読む id の索引を正本から毎回組み直す）/ FR22（folio init の骨格）。本便は 3 つの規範文を変えない。FR14 の索引（`folio graph --print`）と天井の印（FR20 の `--stamp` と `--gate`）が組めない置き場を、床（FR5）が合格と言っていた穴を塞ぎ、FR22 の骨格が自分でその穴を作っていた所を直す。契約表の行の req は FR5・FR14・FR22（main に在る id）。
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1（検査・生成が実行できなかった結果を異常なしとして扱わない）/ P-15.2 の向き（止める判定の式を事後の検査が同じ関数で確かめる＝床は索引と印と同じ切り分けの関数を呼ぶ）/ P-6.4 の向き（裸の形の定義を 2 か所に手で書かない）/ P-10.1（凍結 fixture の floor_cases の case は 1 字も変えない）/ N-3.1（例外の口を足さない）。
- 出所: 台帳 f2-648.212（利用者 tsuzuri の設計席の折り返し 2026-09-25 11:4x JST と席の再現 12:2x JST）。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ei` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 9 本（書き換える 8 本と、verify の `--test init` の scope なので入れる本文不変の 1 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 9 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提の着地は無い。**base = main 06cd100。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。改訂 b（独立の検証の後）の時点の main 66fd1e1（便 137 の着地）でも write-set の 9 本は base と同じで、便の差分をそこへ当てた写しは workspace の nextest 883 / 883・check 合格・`folio build` の 30 file が main と byte で一致した（起草役の実測）。
- 並行の便との重なり: 便 137（行 `ej`）は main 66fd1e1 に着地済みで、重なりは消えた。便 135（行 `eh`・枝 docs/d135）が `crates/folio/tests/site.rs` を write-set に持つ（本文は変えず verify の `--test site` のために入れている）。本便はこの file の末尾に歯を 1 本足す。本文の衝突は無い見込みだが、受付は逐次にする（共通の検証の一時 dir の衝突も避ける）。

## 1. 設計

### (a) いま起きていること（実測・base 06cd100）

1. **読み手が 2 つある。** 索引（`crates/folio/src/graph.rs`）は正本 4 種（憲法・規則の表・要件書・判断の記録）を 2 通りに読む。1 つ目（関数 build）は YAML として読み、引用符を外して id を取る（`graph --digest` と `folio hello` の数はこちらだけで組む）。2 つ目（型 Scan）は file を行の字のまま切り分け、節点ごとの要約値を作る。2 つ目が節点の頭と認めるのは、節の見出しの key が裸（字・数字・下線・横棒だけ）で、行頭が `- id: ` か `- {id: ` の行だけである（判断の記録の file は行頭が `id: ` の行）。`folio graph --print` と天井の印（`ceiling --stamp` と `--gate` が同じ関数 stamp_table で組む）は、2 つで組んだ節点の集合が食い違えば「まだ分からない」で止まる。
2. **床はこの食い違いを数えない。** `folio check`（`crates/folio/src/check.rs` の check_dir）は正本を YAML として読むだけで、graph.rs の関数を 1 つも呼ばない。なので引用符つきの形でも床は合格する。起草役が実の design-intent の写しの 1 行（か節の見出しの key）だけを変えた 13 通りに base の binary を撃った結果（参考値）は次のとおり。13 通りとも check は 0（合格・違反 0）で、graph --print と ceiling --gate は 2（まだ分からない）、graph --digest は 0 だった。

| 変えた所 | 形 | graph --print の断り（要点） |
| --- | --- | --- |
| 規則の表の行 R-1 | key と値とも二重引用符・値だけ二重引用符・key だけ二重引用符・値だけ単引用符 | 索引だけ: R-1（行だけ: 空か引用符つきの R-1） |
| 規則の表の節の見出し thresholds | key を二重引用符 | 索引だけ: R-1〜R-17 の 17 個 |
| 憲法の条 P-1 | 値を二重引用符・key を二重引用符 | 索引だけ: P-1（key のときは P-1.1 と P-1.2 も） |
| 憲法の規範文 P-1.1 | 値を二重引用符 | 索引だけ: P-1.1 |
| 要件書の GOAL1 | 値を二重引用符 | 索引だけ: GOAL1 |
| 判断の記録 ADR-1 | 値を二重引用符・key を二重引用符 | 索引だけ: ADR-1 か、id の行が無い |
| 引用符でない形 2 つ | 規則の表の行で id が 2 番目の欄・条の頭の行の末尾に注釈 | 索引だけ: R-1 か P-1 |

3. **穴を作っていたのは folio init の雛形。** `folio init`（`crates/folio/src/init.rs` の rules_rows）は規則の表の行 R-8 と R-16 を `yaml::write`（全部の字を二重引用符で囲む書き手・`crates/folio/src/yaml.rs`）で書く。なので init の直後の置き場の rules.yaml は節の見出し thresholds と discipline の key も、行の key と値も全部二重引用符つきで、base の binary では check は 2（git の commit の後で違反 0・まだ分からない 2 = 凍結の基準がまだ無い・commit の前は HEAD が無い 1 件が足されて 3）なのに graph --print は 2（索引だけ: R-16・R-8）だった。利用者 tsuzuri の rules.yaml は頭の注が folio init の雛形の字で（利用者の repo の 5091abb で骨格から起こした）、台帳の出所の読み（席が python の yaml で吐いた）とは違い、雛形そのものが出所だった。利用者の版 aa3964a の前の版の写しでは base の check が合格で graph --print がまだ分からない、aa3964a（裸に書き直した版）と HEAD では両方とも通る（節点 161・辺 254）。憲法の雛形のうち節点を持つ条の節と、要件書・判断の記録の雛形は裸の形である（憲法の schema の節は write で書くので引用符つきだが、節点の節でない）。ほかの init の出力は索引の正本でない。
4. **floor_cases の書き手も同じ形。** 凍結 fixture `tests/floor_cases.yaml`（146 case）を回す歯 `crates/folio/tests/floor_cases.rs` は、変異を当てた正本を dump（`yaml::write`）で書き直す。退役した Python の runner（`yaml.safe_dump`）は裸で書ける字を裸で書いていたので、全部を引用符で囲むのは Rust の写しの側の違いである。床に索引の検査を足すと、この書き手で憲法か判断の記録を書き直す 38 case が種類 索引の節点 の違反で落ちる（起草役の実測・108 / 146 case が期待どおり）。
5. **repo の中の YAML（find の全数）。** `*.yaml` は 281 本。そのうち索引の正本（constitution.yaml・rules.yaml・srs.yaml・adr の ADR-n.yaml）は 125 本で、節点の頭の行の id か節の見出しの key が引用符つきの file は 0 本である（ADR-11・12・17・23 の 4 本は amends の行の中に引用符つきの key を持つが、節点の頭の行でも節の見出しでもないので索引は組める）。同じ限定で数えると、引用符つきの形を持つ 11 本は全部が索引の正本でない（凍結 anchor 7 本・支度表 2 本・相談窓口の期待の fixture 2 本）。constitution.yaml を持つ置き場の dir は 25 個で、check が 0 か 1 の 16 個は全部 graph --print が組める。check が 2（まだ分からない）の 9 個のうち 5 個（inject の fixture・srs.yaml が無い）は graph --print も組めない。
6. **床の違反の種類に閉じた一覧は無い。** 依頼は床の種類の閉じた一覧に足すことを想定していたが、base の違反の種類は各呼び出しの字（例: 未知の節・重複キー・欄の非空・schema・anchor・index）で、型付きの一覧の定数は無い（起草役の grep の実測）。本便は新しい種類の字を graph.rs の型付きの定数 1 つに置く（(b) の 1）。字は 索引の節点 で、入口の正本（index.yaml）の違反の種類 index とは別である。
7. **base の歯（参考値）。** workspace の nextest 868 / 868・clippy 0 警告・床 4 本 rc 0・`folio build` 30 file。`git grep -n 'f136_' -- crates` は 0 件。行 id `ei` は未使用。

### (b) 直す先

1. **床の口を 1 つ足す（graph.rs）。** 関数 check_index（引数は置き場の dir と床の所見）を足す。中身は `graph --print` と stamp_table と同じ 2 つの関数（build と Scan の file）で両方の節点の集合を組み、次のとおり数える。種類の字は新しい定数 INDEX_KIND（値は 索引の節点・入口の種類 index と読み違えない字）に置く。
   1. build が組めないとき: 床の所見に違反も まだ分からない も 1 つも無ければ「索引を組めない」を まだ分からない に 1 件足す。どちらかが既に在れば足さない。読めない正本は床の側が先に数えている（例えば判断の記録の file が YAML として読めないか表でないと、床は種類 adr の違反で数える）ので、同じ原因を 2 度数えず、不合格を まだ分からない に変えない。どちらでも合格にはならない（P-4.1）。起草の最初の版の条件（まだ分からない が 0 件なら足す）では、判断の記録の file が読めない写しの終了コードが base の 1 から 2 に変わっていた（独立の検証の実測）。改訂 b の条件では 1 のまま（起草役の実測・読めない形と表でない形の 2 通り）。
   2. Scan が file を切れないとき（判断の記録の id の行が無い・節点が 2 度ある・読めない）: その file に違反 1 件（字は「<file>: 行の逐語で切れない（<断り>・id の key か値が引用符つきか裸の形でない）」）で、その file の節点は次の 3 で数えない。
   3. 切れた file の節点のうち、索引（YAML）だけに在る節点 1 つにつき違反 1 件。字は「<file>: 索引の節点 <id> の行を行の逐語で切れない（id か節の見出しの key が引用符つきか裸の形でない＝folio graph --print と天井の印が組めない）」。
   4. 行の逐語だけに在る節点は、行の末尾の注釈（空白と `#` 以降）と前後の空白と引用符を外した字（関数 bare_id）が 3 の節点に在れば数えない（同じ行を 2 度数えない）。無ければ違反 1 件（字は「行の逐語の節点 <id> が索引の節点に無い（id が引用符つきか裸の形でない）」）。同じ行を 2 度数えないと言えるのは、この外し方で 3 の id に戻る形（引用符と末尾の注釈）に限る。
2. **呼ぶ所は床の入口 2 つ（main.rs と site.rs）。** check_dir を呼ぶ所は base でこの 2 つだけである（grep の実測・受付の後も同じ grep で確かめられる）。 `folio check`（`crates/folio/src/main.rs`）は check_dir の直後・凍結の後始末（freeze::after）より前で呼ぶ。`folio build --write` の床（`crates/folio/src/site.rs` の write_after_floor）も check_dir の直後で呼ぶ（check が落とす置き場から面を書かない）。check_dir（層 2 = 検査する）の中からは呼ばない。graph は層 3（導出する）で、層が上がる辺を足すと実装の区切りの歯（`crates/folio/tests/modules.rs` の p106_edges_point_down・判断の記録 ADR-15）が落ちる（起草役の実測）。入口 2 つは層 5 なので下向きの辺である。
3. **裸の書き手を足す（yaml.rs）。** 公開の口 write_plain を足す。形は write と同じ（表は 1 欄 1 行・入れ子は 2 空白・一覧の各要素は字下げした `- `）で、文字列（key を含む）は、裸で書いても読み戻せる字だけを裸で書き、ほかは二重引用符で囲む。裸で書ける字の判定 plain_ok は保守的で、迷う字は引用符に倒す: 空でない・先頭が指示子（`-?:#&*!|>%@` と逆引用符）でも `.` でも空白でもない・末尾が空白でも `:` でもない・`: ` と ` #` を含まない・制御文字と引用符と逆斜線と流れの括弧と読点（半角）を含まない・既存の plain の型の解き方（resolve_plain）で同じ字の文字列に解ける（数・真偽・null・日付に読まれない）。write は 1 字も変えない（凍結 anchor の file・支度表・相談窓口の期待の fixture の byte を守る）。
4. **folio init の雛形（init.rs）。** rules_rows の書き手を write から write_plain に替える（1 行）。中身の木は変わらない。起草役の実測では、新しい init の rules.yaml を PyYAML で読んだ木は base の init の rules.yaml の木と等しく、`graph --print` は節点 5 で組める。
5. **floor_cases の書き手（tests/floor_cases.rs）。** dump の書き手を write から write_plain に替える（1 行・注 1 行）。凍結 fixture `tests/floor_cases.yaml` は 1 字も変えない。
6. **変えないもの。** Scan・build・field_key・head_id・drop_pairs・節点の要約値の式（凍結 anchor の独立の実装 `tests/fixtures/schema/node-digest.py` と同じ式）・stamp_table・check_dir・write・設計文書・fixture。folio2 の置き場の床の結果と `folio build` の出力（30 file）は base と 1 byte も変わらない（起草役の実測）。

### (c) 歯（関数名 f136_・base で 0 件）

1. **単体の歯 2 本（verify の `--bin folio f136_` が拾う）。**
   1. **f136_plain_writer_writes_bare_only_what_reads_back（`crates/folio/src/yaml.rs` の既存の tests の区間）。** plain_ok が裸で書ける字 9 個（R-1・P-1.1・ADR-1・thresholds・日本語の字・句点・閉じ括弧・byte・注の字）を認め、引用符が要る字 23 個（空・yes・null・波線・12・01・1.5・日付・`: ` や ` #` を含む字・先頭が `#` `-` `.` 空白・末尾が空白や `:`・`%`・半角の読点・流れの括弧・引用符・逆斜線・改行）を退けること。表 1 つを write_plain で書いた字が期待の字（key と id は裸・読点を含む値は引用符・null は裸）と byte で等しく、parse_typed で読み戻すと同じ木で、write は今までどおり key も値も引用符で書くこと。**base では write_plain と plain_ok が無く組み立てが落ちる＝RED。** この区間は `crates/folio/tests/` の 3 本（anchor・floor_cases・init）が yaml.rs を path で取り込むので、その 3 つの歯の binary でも同じ歯が走る。
   2. **f136_unbuildable_index_is_unknown_only_when_the_floor_is_silent（`crates/folio/src/graph.rs` の末尾に tests の区間を新しく置く・改訂 b）。** 正本の無い一時 dir（索引を組めない）に check_index を 3 通りの所見で撃つ。空の所見には まだ分からない がちょうど 1 件（字「索引を組めない: 」で始まる）足され違反は足されない。違反を 1 件持つ所見には まだ分からない が足されず判定は不合格のまま。まだ分からない を 1 件持つ所見には足されず 1 件のまま。**base では check_index が無く組み立てが落ちる＝RED。起草の最初の版の条件（違反を見ない）でも 2 つ目の確かめで落ちる。**
2. **`crates/folio/tests/check.rs` の 3 本（binary 経由・実の design-intent の写し・既存の Work）。**
   1. f136_quoted_rule_row_fails_the_floor_before_graph: 規則の表の行 R-1 の key と id を二重引用符つきにした写しで、check が終了コード 1・違反ちょうど 1 件で、違反が種類 索引の節点・file rules.yaml・字「 R-1 」と「引用符つき」を持つ。同じ写しで `graph --print` が終了コード 2 で、断りに「索引だけ: R-1」を持つ（check が先に落ちる）。
   2. f136_quoted_forms_in_each_source_fail_the_floor: 写しの 1 か所ずつ 7 通り（条 P-1 の値を二重引用符・規範文 P-1.1 の値を単引用符・要件書 GOAL1 の key を二重引用符・判断の記録 ADR-1 の値を二重引用符・同じ key を二重引用符・条 P-1 の頭の行の末尾に注釈〔引用符でない形〕・規則の表の節の見出し thresholds の key を二重引用符）で、check が終了コード 1、違反が全部 種類 索引の節点・その file・字「引用符つき」を持つ。前の 6 通りは違反ちょうど 1 件で、節点の id（判断の記録の key のときは字「id の行が無い」）を名指す。節の見出しの変異は件数を pin しない（行の数に依る）。
   3. f136_bare_copy_passes: 変えない写しは check が終了コード 0 で、種類 索引の節点の違反が無い（対照・base でも緑）。
   **base では床が索引を数えないので 1 と 2 は落ちる＝RED。**
3. **f136_quoted_rule_row_stops_the_build_floor（`crates/folio/tests/site.rs`・binary 経由・既存の real_copy）。** 1 と同じ変異を commit した写しで `folio build --write` が終了コード 1・出力に「床 = 不合格（違反 1・」を持ち、配信先の dir を作らない。**base では床が合格して面を書き 0 で終わる＝RED。**
4. **既存の歯が見張る所。** init の雛形を write に戻すと、既存の `crates/folio/tests/init.rs` の f125_init_writes_the_skeleton_and_the_floor_passes（init の後の床）が落ちる。floor_cases の書き手を write に戻すと、既存の floor_cases の歯（146 case）が落ちる（(e) の 2 の表）。

### (d) 採らなかった形

1. **行の字のまま切る側（節の見出しの key と節点の頭の行を拾う関数）を引用符つきの形に対応させる。** 節点の頭と認める行の形が広がり、節点の要約値の式が動く。式の凍結 anchor（folio に依らない Python の実装 `tests/fixtures/schema/node-digest.py`）と、索引の欄の決まりの生成区間の式の注（graph.yaml の digest_note）を同時に直すことになり、床で落とすより大きく、天井の印の値も動きうる。利用者も採らないでよいと言っている（台帳 f2-648.212）。
2. **床に引用符つきの形の一覧（字の型の一覧）を手で書いて数える。** 裸の形の定義が graph.rs の切り分けと床の 2 か所に分かれ、片方だけ直ると再び食い違う（P-6.4 の向き）。引用符でない形（id が 2 番目の欄・頭の行の末尾の注釈・(a) の 2 の表の最後の行）も索引を組めなくするが、一覧はそれを拾わない。本便は切り分けの関数そのものを呼ぶので、索引と印が組めない形は種類を問わず床が落とす。
3. **write そのものを裸の書き手に替える。** write の出力を byte で持つ凍結 anchor の file（`design-intent/anchors/` と fixture の土台の計 7 本）と支度表と相談窓口の期待の fixture（`tests/fixtures/intake/` の 2 本）が変わる。裸の書き手は別の口にして、索引の正本を書く 2 か所（init の規則の表と floor_cases）だけが使う。

### (e) 既存の歯のうち落ちるもの・突然変異・検算

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分（改訂 b）を base に当てた写しで、workspace の nextest 877 / 877（base 868 + 本便の歯 9 = yaml.rs の単体の歯が folio の binary と取り込む 3 本の計 4・graph.rs の単体の歯 1・check の 3・site の 1）・clippy 0 警告・床 4 本 rc 0・`folio build` 30 file（base と全 file が byte で一致）・`graph --print` は base と同じ要約（節点 224・辺 1116）。floor_cases は 146 / 146。置き場の dir 25 個の check の終了コードと要約の行は base と全部同じで、種類 索引の節点の違反は 0 件。判断の記録の file が YAML として読めない写しと表でない写しは、base と同じく終了コード 1（違反 1・まだ分からない 0）。(a) の 2 の 13 通りは全部 check が 1 で、引用符つきの 1 行か末尾の注釈 1 つの変異は違反ちょうど 1 件（節の見出しは 17 件・条の key は入れ子の規範文を含めて 3 件）。利用者 tsuzuri の写しは、aa3964a の前の版が不合格（違反 17・全部が種類 索引の節点）になり、aa3964a と HEAD は合格のまま。rustfmt の差の数は write-set の file で base と同じ（本便で増えない）。
2. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 か所ずつ変える）。**

| 変異 | 落ちる歯 |
| --- | --- |
| main.rs の check_index の呼び出しを消す | check の f136_ 2 本（site の f136_ は緑） |
| site.rs の呼び出しを消す | site の f136_（check の f136_ は緑） |
| init の書き手を write に戻す | init の f125 の床の歯 1 本・mechanism_live の init を使う歯 1 本 |
| floor_cases の書き手を write に戻す | floor_cases の歯（108 / 146 case が期待どおり） |
| plain_ok から型の解き方の条件を外す | 単体の歯 f136_ |
| 切れなかった file の節点も数える | check の f136_quoted_forms の歯（判断の記録の key の変異が 2 件になる） |
| 引用符を外した字の突き合わせを外す | check の f136_quoted_forms の歯（値だけ引用符の変異が 2 件になる） |
| 突き合わせで末尾の注釈を外さない | check の f136_quoted_forms の歯（末尾の注釈の変異が 2 件になる） |
| 索引を組めないときの条件を起草の最初の版（まだ分からない が 0 件なら足す）に戻す | graph.rs の単体の歯 f136_（違反を持つ所見に まだ分からない が足される） |
| 索引を組めないときに まだ分からない を足す行を消す | graph.rs の単体の歯 f136_（空の所見に何も足されない） |

3. **裸の書き手の検算（2 実装）。** repo の全 `*.yaml` 281 本のうち parse_typed で読める 247 本を write_plain で書き直し、(1) parse_typed で読み戻した木が全部同じ、(2) 元の file と書き直した file を PyYAML（`yaml.safe_load`）で読んだ木が 246 本で同じで、違う 1 本（`tests/fixtures/check/dup-key/rules.yaml`）は元の file の重複キー（読み手ごとに後勝ちか先勝ちかが違う）による（起草役の実測・script は (h)）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 9 本とも印なし。書き換える 8 本 = `crates/folio/src/graph.rs`・`crates/folio/src/main.rs`・`crates/folio/src/site.rs`・`crates/folio/src/yaml.rs`・`crates/folio/src/init.rs`・`crates/folio/tests/check.rs`・`crates/folio/tests/site.rs`・`crates/folio/tests/floor_cases.rs`。本文を変えない 1 本 = `crates/folio/tests/init.rs`（verify の `--test init` で名指すので入れる）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 5 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/graph.rs` | 739 | 761 | 843（+104） |
| `crates/folio/src/main.rs` | 662 | 838 | 664（+2） |
| `crates/folio/src/site.rs` | 261 | 1239 | 263（+2） |
| `crates/folio/src/yaml.rs` | 975 | 525 | 1088（+113） |
| `crates/folio/src/init.rs` | 772 | 728 | 773（+1） |

   src の増分の合計は +222 で S の見積 100 を超えるので、**size は M**（5 本とも余地が M の見積 300 を超える・最小は yaml.rs の 525）。歯の file は src の外なので余地を測らない（参考値 check.rs 1383 → 1505・site.rs 727 → 757・floor_cases.rs 760 → 761。check.rs は幅 120 で 1500 を超えるが、器の余地の門は `crates/<c>/src/` だけを測る）。
3. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f136_` = (c) の 1（単体の歯 2 本）。
   2. `cargo nextest run -p folio --test check f136_` = (c) の 2（3 本）。
   3. `cargo nextest run -p folio --test site f136_` = (c) の 3。
   4. `cargo nextest run -p folio --test floor_cases` = 凍結 fixture の 146 case（裸の書き手で全部が期待どおり）。
   5. `cargo nextest run -p folio --test init` = init の骨格の歯の全部（床の歯 f125 を含む・参考値 18 本）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
4. **verify の歯の file と write-set。** `--test` で名指す歯の file は check・site・floor_cases・init の 4 本で、全部 write-set に在る。`--bin folio` の絞り込みの語 f136_ を関数名に持つ src は `crates/folio/src/yaml.rs` と `crates/folio/src/graph.rs` の 2 本で、どちらも write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（冒頭の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。便 137 は main 66fd1e1 に着地済み・便 135（契約 main 91ddb48・run 走行中）とは `crates/folio/tests/site.rs` が重なるので、便 135 の着地の後に受け付ける（逐次）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d136-draft.md`、模擬の差分と script は同じ dir の d136-scripts（repo には入れない）。

1. 再現: repro-136.sh（binary・repo の写しの根・作業 dir）で (a) の 2 の 13 通りと対照を撃つ。placedirs-136.sh で置き場の dir 25 個、tsuzuri-136.sh で利用者の 3 つの版を、base と本便の binary で撃ち比べる。
2. 模擬: 差分 c136.patch を base に当て、run-136.sh（workspace の nextest・clippy・床 4 本・graph の要約・`folio build --write` の file 数・verify の 5 行・rustfmt の差の数）と mut-136.sh（(e) の 2 の突然変異 10 通り）。unknown-branch-136.sh で判断の記録の file が読めない写しの終了コードを、onmain-136.sh で受付の時点の main（改訂 b では 66fd1e1）に当てた写しの build の出力と nextest を撃つ。
3. RED: 歯の 4 か所だけの差分 r136-teeth.patch を base に当てると、単体の歯 2 本は組み立てで落ち、check の f136_ は 2 本が落ち（対照の 1 本は緑）、site の f136_ は落ちる。
4. 裸の書き手の検算: zz_plain_corpus.rs（本便を当てた写しの tests へ一時に置く・契約に入れない）で全 YAML を書き直し、pyyaml-compare.py で PyYAML の木を突き合わせる。
5. 余地: lines-136.py と lines-136.awk（同じ式の 2 実装）を repo の根で write-set の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** byte で切る側の変更（(d) の 1）。write と、write を使うほかの書き手（凍結 anchor・支度表・id の一覧の凍結）。憲法の schema の節の雛形（init が write で書くが、節の見出し schema は索引の節でない）。利用者の置き場の書き直し（tsuzuri は aa3964a で済み）。設計文書の字（要件書・判断の記録）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 床が落とすのは、索引の 2 つの読み手が食い違う形である。読み手が 2 つとも同じ誤りをする形（例えば YAML としても行としても別の行を節点と読む形）は数えない。裸で書ける字の判定は保守的で、裸で書けるのに引用符で書く字が残りうる（読み戻しは同じ）。PyYAML との一致は repo の中の 246 本で確かめたもので、全部の字について証したものではない。
3. **撤退条件。** (1) 本便の後に、置き換えも足しもしない既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に folio2 自身の床の結果か `folio build` の出力か `graph --print` の要約が、受付の時点の main（本便を当てる前）の出力と比べて 1 byte でも変わったら、止めて席へ返す。(3) floor_cases の 146 case のうち 1 case でも期待どおりでなくなったら、凍結 fixture を直さずに止めて席へ返す。(4) 受付の時点の main で write-set の file が base から書き換わっていたら（便 135 の着地を含む）、余地と歯の効きを測り直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/graph.rs` の床の口 check_index と種類の定数 INDEX_KIND と、行の逐語の id を突き合わせる字に直す関数 bare_id と、単体の歯 1 本（新しい tests の区間）。`crates/folio/src/main.rs` と `crates/folio/src/site.rs` の呼び出し各 1 か所。`crates/folio/src/yaml.rs` の裸の書き手 write_plain と判定 plain_ok と単体の歯 1 本。`crates/folio/src/init.rs` の規則の表の行の書き手（1 行）。`crates/folio/tests/floor_cases.rs` の dump の書き手（1 行）。`crates/folio/tests/check.rs` の歯 3 本。`crates/folio/tests/site.rs` の歯 1 本。
- 入れない: 切り分けの関数と要約値の式・stamp_table・check_dir・write・凍結 fixture と凍結 anchor・設計文書・利用者の repo・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| floor | 索引の床の口 | `crates/folio/src/graph.rs` の check_index（build と Scan の食い違いを種類 索引の節点の違反に数える） |
| entry | 床の入口 2 つ | `crates/folio/src/main.rs`（folio check）と `crates/folio/src/site.rs`（folio build --write の床）が check_dir の直後に呼ぶ |
| writer | 裸の書き手 | `crates/folio/src/yaml.rs` の write_plain と plain_ok（裸で読み戻せる字だけ裸） |
| init | 骨格の規則の表 | `crates/folio/src/init.rs` の rules_rows が裸の書き手で書く |
| cases | floor_cases の書き手 | `crates/folio/tests/floor_cases.rs` の dump が裸の書き手で書く |
| teeth | 歯 | yaml.rs と graph.rs の単体の歯・check.rs の 3 本・site.rs の 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無い。
- 本便の着地の後に席が見ること: 台帳 f2-648.212 を閉じる。利用者 tsuzuri の設計席へ、雛形が出所だったこと（(a) の 3）と、本便の後は引用符つきの形を床が落とすことを返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ei"
title = "床の穴（台帳 f2-648.212）: 索引の正本（憲法・規則の表・要件書・判断の記録）の節点の行の id か節の見出しの key が引用符つき（ほか行の逐語で切れない形）だと folio check は合格なのに graph --print と天井の印が組めない。graph.rs に床の口 check_index（graph --print と stamp_table と同じ build と Scan で節点の集合の食い違いを数え、種類 索引の節点 の違反にする・組めなければ、床がほかに何も数えていないときだけ まだ分からない）を足し、folio check（main.rs）と folio build --write の床（site.rs）が check_dir の直後に呼ぶ（層 2 の check_dir からは呼ばない）。出所の folio init の雛形の規則の表と floor_cases の歯の書き直しは、yaml.rs に足す裸の書き手 write_plain（裸で読み戻せる字だけ裸・write は不変）で書く。切り分けの関数と要約値の式・凍結 fixture・設計文書は変えない"
req = ["FR5", "FR14", "FR22"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/src/main.rs", "crates/folio/src/site.rs", "crates/folio/src/yaml.rs", "crates/folio/src/init.rs", "crates/folio/tests/check.rs", "crates/folio/tests/site.rs", "crates/folio/tests/floor_cases.rs", "crates/folio/tests/init.rs"]
verify = ["cargo nextest run -p folio --bin folio f136_", "cargo nextest run -p folio --test check f136_", "cargo nextest run -p folio --test site f136_", "cargo nextest run -p folio --test floor_cases", "cargo nextest run -p folio --test init", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "単体の歯 f136_ の 2 本（yaml.rs = 裸の書き手が裸で読み戻せる字だけを裸で書き、読み直すと同じ木・write は全部を引用符のまま / graph.rs = 索引を組めないとき床がほかに何も数えていなければ まだ分からない を 1 件足し、違反か まだ分からない が在れば足さない）が緑、check の f136_ の 3 本（規則の表の行・条・規範文・要件書・判断の記録・頭の行の末尾の注釈・節の見出しを引用符つきか裸でない形にした写しで folio check が種類 索引の節点 の違反で終了コード 1・同じ写しで graph --print が まだ分からない・変えない写しは合格）が緑、site の f136_（同じ変異で folio build --write の床が不合格で何も書かない）が緑、floor_cases の 146 case が全部期待どおり、init の歯の全部（init の後の床の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致を返す"
<!-- contracts:end -->
