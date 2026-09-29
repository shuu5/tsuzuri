# 設計: 便 183 — 計画の設計ノート: 名札の行 plan-note・節の型 3 つ・行の索引の導出と床（判断の記録 ADR-31 の便 B の前半）

- 要件: FR27（計画の名札の行を持つ置き場で、計画の設計ノートの行の索引を契約表から導き、計画だけの行と判断を床で数える・要件書 第 1.53 版）・FR26（決定の欄の裁定 id の形・判断の表の行を決定の欄に足す）・FR11（導出物の命令 folio derive）・FR19（床の定数を生成区間へ写す）。どれも本流の要件書に在る id で、字は変えない。受入は AC29。
- 条: P-6.3 / P-6.4（行の索引は契約表からの導出だけ・手の 2 面を作らない）・P-15.2（書く命令と床が同じ関数で比べる）・P-2.4（節の型の閉じた一覧に 3 つ・裁定は設計ノートの欄の決まりの meta.type_enum_ruling〔本流に承認済みの字〕）・P-4.2（判定できない名札の行と計画のノートは まだ分からない）・N-3.1（名札の行は掛ける範囲で、外しても置き場の決まりが残る）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 判断の記録 ADR-31（持ち主の承認 2026-09-28 10:29 JST・対話面 R-8・逐語「全部承認する」・台帳 f2-648.267）の決定 (2)(ア)〜(エ) と (5)（便 B の差分が審査の上限を超えるなら型・床・書く命令の便と面の便に割る）。面の描き方（決定 (7)・FR9 の範囲）は便 184（行 ge・docs/design/delivery-184.md）に割った。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gd` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 41 本（src 8〔新 1〕・歯の file 8〔新 1・本文不変 1〕・設計文書の正本 3・fixture と凍結 anchor 22）。縮む file・消す file・新しい dir は無い。
- 門: 対象。write-set に設計文書の正本（`design-intent/adr/schema.yaml`・`design-intent/design-note/schema.yaml`・`design-intent/rules.yaml` の生成区間）が在る。本流 c2e59c3 の組み立てに write-set 41 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 本流 c2e59c3**（便 182 の着地の後）。この契約の数はすべて c2e59c3 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d183`（commit **366f96f**・改訂 a〔検証役の B1 の直しと歯 3 通り〕を 1f8ea15 に積んだもの・1f8ea15 は見本 457d7be〔親は便 182 の見本 faffb99〕に本流 c2e59c3 を取り込んだ merge・本流と faffb99 の差は docs の 2 file だけ）。`git diff c2e59c3 366f96f` が便の全体の差分（40 file・+1000 −62・112,703 byte）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 366f96f -- <write-set の file>`）。ただし 3 本の生成区間は `folio schema --dir design-intent --write` の出力（見本と byte で同じ）。write-set の外は変えない。
- 並行の便との重なり: 便 184（行 ge）は本便の後（face_note.rs と tests/face_note.rs だけ・本便の write-set と重ならない）。ほかに本便の write-set を書き換える未着地の便の契約は base の時点で無い。

## 1. 設計

### (a) いま起きていること（base c2e59c3 の実測・参考値）

1. **計画の行と判断を読む床が無い。** 規則の表の欄 key の閉じた一覧は `[note-chapters]` だけ（`rules.rs` の KEYS）。設計ノートの節の型の閉じた一覧は 6 つ（`floor_note.rs` の TYPE_ENUM）で、計画の行（契約の行の一覧・まだ契約の無い行・判断）を型で持てず、外の利用者の計画のノート（tsuzuri の surface-plan・今の HEAD c43cae0）は 7 節とも散文である（4 節の行 68 本のうち 15 本は既に波のノートの契約表に在る＝字の拾いの参考値）。
2. **決定の欄の一覧に判断の表が無い。** `ruling.rs` の FIELDS は 11 項（便 182 の注「判断の表の行は、その節の型が入る便が一覧に足す」）。
3. **folio derive は契約表の導出物だけを書く**（`derive.rs`・規則の表を読まない）。
4. **base の歯（参考値）。** workspace の nextest 1027 / 1027・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 36 file。`git grep -n f183_ -- crates` は 0 件・行 id `gd` は 0 件。

### (b) 直す先

1. **`crates/folio/src/rules.rs`。** 定数 PLAN_NOTE（字は `floor_note.rs` の PLAN_KEY `plan-note` を引く＝層 1 から層 2 を名指さない）を KEYS に足す（2 項）。関数 plan_note（欄 key が plan-note の閾値の行の value・行が無ければ `Ok(None)`＝掛けない・2 本以上か値が文書 id の形〔`shelf::is_doc_id`〕でなければ Err）と、keyed の中の行の拾いを keyed_rows に分ける。床の木の注 key_note を key ごとに書き分ける（note-chapters は まだ分からない・plan-note は掛けない〔置き場の決まりは掛かる〕・2 本か形違いは まだ分からない）。既存の歯 f179_ の KEYS の期待を 2 項に。単体の歯 1 本（(c) の 7）。
2. **`crates/folio/src/floor_note.rs`。** TYPE_ENUM に row-index・row-plan・decision-table（9 つ）。行の欄 INDEX_ROW（必須 id・doc）・PLAN_ROW（必須 id・what／任意 depends・ruling・note・size・files）・PLAN_LISTS（depends・files は字の一覧）・DECISION_ROW（必須 id・text・ruling）・生成区間の印 ROWS_BEGIN（`# folio:rows:begin — 生成区間・手で直さない・正本は置き場の契約表（folio derive --write が書く）`）と ROWS_END（`# folio:rows:end`）・PLAN_KEY。FLOOR の type_note と by_type の 3 項（row-index は place・region〔begin・end〕・row_note／row-plan は lists・place・row_note／decision-table は row_note）。
3. **`crates/folio/src/plan.rs`（新）。** index_rows（読めた設計ノートを名の順に・契約表の節の順・行の順に `(行 id, 文書 id)`・id の形でなければ Err）・region（印の行は頭の空白を除いた全部が字面のとおり・それぞれちょうど 1 本で begin が先）・render（begin の頭の空白で `- {id: <id>, doc: <文書 id>}`）・**drift（床と folio derive --check が共有する 1 つの関数: 行の索引の節がちょうど 1 つ・印が 1 対・印の間が導出と byte で同じ・節の行が導出と同じ〔印が節の rows の外なら割れる〕）**・rewrite（folio derive --write）・check_plan（名札の行の読みが割れれば まだ分からない 1 つで止める → 置き場の決まり〔row-index と row-plan の節は名札の行が名指すノートにだけ・名札の行が無くても掛かる〕→ 名札の行が無ければ終わり → 計画のノートが無い・状態が effective でない は まだ分からない → drift → 計画だけの行の id の一意・索引に無い・depends が索引か計画だけの行に在る）・load_rules（derive が使う・規則の表が無ければ None）。単体の歯 1 本（(c) の 8）。層は 2（検査する・`tests/modules.rs` の LAYERS に 1 行）。
4. **`crates/folio/src/note.rs`。** check_table_rows の行の欄を 3 型に広げ、plan_shapes（計画だけの行の depends と files は字の一覧・ほかは字・判断の表の ruling は決定の欄の床に任せて重ねない）。row-index と row-plan の節は duplicate_ids を掛けない（索引は文書をまたいで id が重なりうる・計画だけの行の一意は計画の床が数える）。check_note の終わりで plan::check_plan を呼び、設計ノートの置き場が無い・設計ノートが 0 本の 2 つの早い return の前でも呼ぶ（名札の行が在れば「計画のノートが無い」を まだ分からない に出す＝folio derive --check の 2 と揃える・P-4.1・改訂 a＝検証役の B1）。
5. **`crates/folio/src/derive.rs`。** derive_all が読めた設計ノートも返し、plan_index（規則の表が無い・名札の行が無い は今のまま・読めない名札の行・計画のノートが無い・行が id の形でない は まだ分からない 2）。--write は導出物を確かめた後に、書き直した字が drift で導出と合うときだけ計画のノートを書く（全部か無しか・器の欄の決まりの file が無い置き場は導出物の段で 2 のまま書かない）。--check は drift の理由を `folio derive: DRIFT: design-note/<名>（<理由>）` の 1 行にして差分に数える。要約の行の数に計画のノートを 1 つ足す。
6. **`crates/folio/src/ruling.rs` と `floor_adr.rs`。** 決定の欄に TABLE（`design-note/*.yaml sections[decision-table].rows[].ruling`）を足して FIELDS を 12 項に・歩き手 sites が各設計ノートの判断の表の行を `§<n> の行 <id> の ruling` で拾う（名札の行に依らない）。floor_adr の注 ruling_fields_note の終わりの 1 文を直す。便 182 の単体の歯の数 11 → 12。
7. **`crates/folio/src/main.rs`。** `mod plan;` の 1 行。
8. **生成区間と fixture と凍結 anchor。** 3 本の正本の生成区間を `folio schema --dir design-intent --write` で（adr/schema.yaml +2 −1・design-note/schema.yaml +29 −2・rules.yaml +2 −2）。fixture の写し: 判断の記録の欄の決まり 17 本の ruling_fields に 1 行・床の土台の設計ノートの欄の決まりの type_enum と by_type の 3 項（fixtures-183.py）。凍結 anchor: 生成区間 3 本（adr-region 151 行・28,247 byte／note-region 164 行・19,619 byte／rules-region 32 行・3,334 byte＝前の anchor の字の置き換えと挿入で組んだ・anchors-183.py・導出の命令は使わない）・土台の索引の残差 node-digest-anchor.txt（独立の script・節点の表と出力全体は不変・anchors-18x.py の段 183）。
9. **既存の歯の字の期待（中身は変えない）。** `tests/schema.rs` の生成区間の定数 6 つ・`tests/schema_docs.rs` の rules の定数 2 つ・`tests/graph.rs` の要約値の anchor の sha256・`tests/note.rs` の欄の決まりの写しの変異の当て先（type_enum が塊の形になった）・`tests/ruling.rs` の f182_ の手書きの一覧（12 行目）・`tests/modules.rs` の LAYERS（plan を層 2）。
10. **変えないもの。** 面の描き方（便 184）・書き出し（便 C）・契約表の導出物の形・憲法と要件書と判断の記録の字・folio2 自身の床の結果と `folio build` の出力（folio2 は名札の行を持たない＝36 file が base と byte で同じ・sum ac76ec3ce9403fbc）。

### (c) 歯（f183_・base で 0 件）

`crates/folio/tests/plan.rs`（新）は土台の写しを一時 dir の `design-intent/` に、器の導出 file を `contracts/` に作り、歯の中の最小の手書き（計画のノート plan.yaml・規則の表の名札の行 R-27〔article P-2・欄 key plan-note・値 plan〕・条 P-2 の関係の行に R-27）を足す。もう 1 本の設計ノートは土台の見本 example.yaml（契約表の行 a）。版管理は作らない（違反の行と まだ分からない の行の増減を数える）。

1. **f183_a_new_contract_row_is_one_violation_until_derive_write**（AC29 の 1）: 合格の写しで example.yaml に契約表の行 a2 を足すと違反ちょうど 1 行 `[note] design-note/plan.yaml: 行の索引の生成区間が契約表からの導出と違う（folio derive --write で書き直す）`・derive --check は 1。器の導出 file を消すと derive --write は 2 で計画のノートを変えない。戻して --write の後に `- {id: a, doc: example}` と `- {id: a2, doc: example}` が印の間に入り、床の違反 0・--check 0。
2. **f183_a_planned_row_in_the_index_a_dangling_depends_or_a_twin_is_one_violation**（AC29 の 2・3）: 計画だけの行の id を索引の a にする・depends に zz・id を重ねる、でそれぞれ違反ちょうど 1 行（字は歯の手書き）。
3. **f183_no_plan_note_a_draft_two_rows_or_a_bad_value_is_unknown**（AC29 の 4 ほか）: 計画のノートを消す・状態 draft・名札の行 2 本・値 `Plan`・設計ノートを全部消す（欄の決まりだけ）・設計ノートの置き場ごと消す（改訂 a の 2 通り）で、まだ分からない がちょうど 1 つ増え（字は歯の手書き）種別 note の違反は 0（2 本は欄 key の床が種別 schema の違反 1 を別に出す）。
4. **f183_the_index_outside_the_plan_note_is_a_violation_with_or_without_the_row**（AC29 の 5）: 名札の行を消し索引の節を残した写しで違反ちょうど 1 行（`…§2: 節の型 row-index は計画の名札の行…にだけ置ける（名札の行が無い）`）・計画だけの行の節も残せば 2・名札の行が在っても example.yaml に索引の節を置けば 1（`（名指すのは plan）`）。
5. **f183_size_files_shapes_and_the_decision_ruling_are_read**: size を一覧・files を字にするとそれぞれ違反 1・size の値 XL は違反 0（値は照らさない）・判断の表の ruling から台帳の id を消すと種別 裁定 id の違反 1（便 A の関数）・名指されない example.yaml の判断の表も数える。
6. **f183_the_index_follows_file_names_then_table_order**: a-wave.yaml（契約表の行 z・y）と、file 名の順と文書 id の順が割れる組 ex-b.yaml（行 x2）と ex.yaml（行 x1）を足して --write すると、索引は z・y（a-wave）・x2（ex-b）・x1（ex）・a（example）の順（file 名の順・表の中の順）で、2 度目は 書いた 0 file・字は変わらない。
6b. **f183_markers_outside_the_section_fail_the_floor_and_derive_check_alike**（改訂 a）: 印を散文の節の body の中に置き（区間の字は導出と合う）行の索引の節の rows を空にした写しで、床は違反ちょうど 1 行（`…行の索引の節の行が生成区間の導出と違う（印が節の rows の外に在る・folio derive --write で書き直す）`）、folio derive --check は 1 と同じ理由の DRIFT の行（床と --check が同じ関数で比べる）。
7. **単体（`rules.rs` の tests）f183_plan_note_reads_the_keyed_row_and_none_means_off**: 値 surface-plan は Some・行が無い（note-chapters だけ・開発規律の行に在る）は None・値の形違い 6 通りと 2 本は Err。
8. **単体（`plan.rs` の tests）f183_the_region_is_written_and_read_by_one_function**: rewrite の字は drift で None・2 度目は同じ字・印が 1 対でない・節が 2 つ・区間の字の違い・印が散文の body の中（節の外）はそれぞれ理由を返す。
9. **RED の実測。** base c2e59c3 に歯の file（tests/ の 7 本）だけを当てると f183_ の binary の歯が全部落ち（名札の行の欄 key が閉じた一覧に無い・節の型が一覧に無い）、workspace で 25 本が落ちる（改訂 a の前の歯の file で数えた・f183_ 6・字の期待を直した既存 19〔modules 2 を含む〕・起草の記録の red-183*.log）。base の `--bin folio f183_` は 0 本（rc 4）。改訂 a の歯は、見本から B1 の直しを外すと歯 3 が落ち、検証役の変異 V4（索引を文書 id の順）で歯 6 が、V5（derive --check が区間の字だけで比べる）で歯 6b が落ちる（mut-reva.log・失敗の本文つき）。

### (d) 採らなかった形

1. **便 184 と 1 便で運ぶ。** 本便だけで 112,703 byte（改訂 a）、面（便 184 の 9,429 byte）を足すと 122,132 byte で審査の上限 120,000 を超える（ADR-31 決定 (5)）。型と床と書く命令が先で、面は後（本便の後から便 184 までの間は、3 つの型の節を持つ設計ノートの面は まだ分からない のまま＝folio2 自身は持たない）。
2. **生成区間の印を持たず、行の索引の節の rows を字の走査で探して書き直す。** 流れの形と塊の形の両方を字で解くことになり、印の型（`folio schema`・`folio inject` と同じ）から外れる。
3. **比べ方を節の行の読みだけにする（区間の byte を比べない）。** 手で直した字が見えない（P-6.2）。印が節の外に在る写しを割るため、読みの比べも残す（両方）。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **既存の歯。** 本便の差分（改訂 a）を base に当てた写しで、workspace の nextest **1036 / 1036**（1027 + f183_ の 9 本）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 36 file（base と byte で同じ）。字の期待を直した既存の歯は (b) の 9 のとおりで、直さないと 19 本が落ちる。
2. **突然変異（見本の src だけを 1 通りずつ変え f183_ と f184_ を撃つ・mut-183.log と改訂 a の mut-reva.log）。** 起草役の 14 通りと改訂 a の 3 通り（R0・V4・V5）は全部落ちる。検証役の変異のうち V4b（1 本のノートの中の契約表の節を逆順）・V6（状態 example を発効扱い）・V7（行の索引に id の重なりの床）・V9（印の順を見ない）・V10（行の欄に表を許す）は歯で縛らない（見本の振る舞いは正しく、ADR の字の決まりでもない・起草の記録）。本便の分は次の 14 通り。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 生成区間の字を比べない | 1 |
| M2 名札の行が無いとき置き場の決まりを掛けない | 4 |
| M3 計画だけの行と索引の二重を見ない | 2 |
| M4 宙に浮いた依存を見ない | 2 |
| M5 下書きの計画のノートを数える | 3 |
| M6 名札の行 2 本で先の 1 本を取る | 3 |
| M7 索引を id の字の順にする | 6 |
| M8 --write が計画のノートを書かない | 1・6 |
| M9 size に字の一覧を許す | 1〜6 |
| M10 判断の表の行を決定の欄に数えない | 5 |
| M11 --check が索引を比べない | 1 |
| M14 印の頭の空白を落として書く | 1〜6 のうち 5 本 |
| R0 設計ノートが 0 本・置き場が無いとき計画の床を呼ばない | 3 |
| V4 索引を文書 id の順にする | 6 |
| V5 derive --check が区間の字だけで比べる | 6b |

3. **外の置き場（tsuzuri の写し・今の HEAD 1eb50a7〔b23084f の後〕・参考値）。** 本便と便 184 の binary では、tsuzuri の欄の決まりの写し（adr・design-note）が古いので違反 8 → `folio schema --dir design-intent --write`（3 本の生成区間）の後に合格 0/0。移行の手順は起草の記録の tz-run.sh と tz-migrate.py（tsuzuri の今の HEAD で撃ち直す形・patch は surface-plan が変わると当たらない）: 規則の表に名札の行 R-27（欄 key plan-note・値 surface-plan・種別 build-check・段 post・refs ADR-14）と条 P-2 の関係の行・surface-plan の 3 節の持ち主の裁定 **30 行**を判断の表（§10）へ・4 節の行のうち契約表に無い **62 行**を計画だけの行（§9）へ・契約表に既に在る **6 行**（e-hold・g-keyed・g-strip・g-reads・g-policy-all・i-3）は外す・依存の字のうち id でないか括弧つきの 25 行は note へ・索引の節（§8・印だけ）を足す → `folio derive --write`（索引 **126 行**）→ 床 合格 0/0・derive --check 一致 15・`folio build --write` 37 file（surface-plan の面に 3 章）。**本便だけの binary では build が まだ分からない（節の型 row-index を面が知らない）＝tsuzuri の移行は便 184 の着地の後**（ADR-31 決定 (5)）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/src/plan.rs` と `+crates/folio/tests/plan.rs`。`crates/folio/tests/derive.rs` は本文を変えないが verify の `--test derive` の scope なので入れる。差分 112,703 byte（`git diff c2e59c3 366f96f | wc -c`・40 file）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 8 本（python と awk の 2 実装で一致・cap-183.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/note.rs` | 971 | 529 | 1010（+39） | 490 |
| `crates/folio/src/main.rs` | 687 | 813 | 688（+1） | 812 |
| `crates/folio/src/floor_note.rs` | 511 | 989 | 576（+65） | 924 |
| `crates/folio/src/floor_adr.rs` | 511 | 989 | 511（0） | 989 |
| `crates/folio/src/rules.rs` | 428 | 1072 | 473（+45） | 1027 |
| `crates/folio/src/derive.rs` | 358 | 1142 | 426（+68） | 1074 |
| `crates/folio/src/ruling.rs` | 357 | 1143 | 370（+13） | 1130 |
| `crates/folio/src/plan.rs`（新） | — | 1500 | 274 | 1226 |

3. **size は M。** file ごとの増分の最大は plan.rs の 274（M の見積 300 の内）で、余地の最小 490（note.rs）は M の 300 を超える。
4. **verify は 11 行**で、done の 11 の塊と 1 対 1 に揃える。便の後の写しで 11 行とも rc 0。
   1. `cargo nextest run -p folio --test plan f183_` = (c) の 1〜6b（7 本）。
   2. `cargo nextest run -p folio --bin folio f183_` = (c) の 7・8（2 本）。
   3. `cargo nextest run -p folio --bin folio rules::tests` = 規則の表の床の単体の歯（KEYS 2 項を含む）。
   4. `cargo nextest run -p folio --test derive` = 導出の命令の既存の歯（名札の行の無い置き場は今のまま）。
   5. `cargo nextest run -p folio --test note` = 設計ノートの床の既存の歯（写しの変異の当て先）。
   6. `cargo nextest run -p folio --test ruling` = 決定の欄の歯（一覧 12 行）。
   7. `cargo nextest run -p folio --test schema` = 判断の記録と設計ノートの生成区間の凍結 anchor。
   8. `cargo nextest run -p folio --test schema_docs` = 規則の表の生成区間の凍結 anchor。
   9. `cargo nextest run -p folio --test graph` = 土台の索引の凍結 anchor。
   10. `cargo nextest run -p folio --test modules` = 層の表（plan は層 2）。
   11. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（plan・derive・note・ruling・schema・schema_docs・graph・modules）は全部 write-set に在る。`--bin folio` の歯の在り処（rules.rs・plan.rs）も write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は本流 c2e59c3 の上の枝 docs/d183 で契約に起因する断り 0（起草の記録の precheck-183.log）。着地の後、席は tsuzuri へ「本便と便 184 の binary を取り込むときは `folio schema --write` を 1 回・移行は便 184 の着地の後（手順は起草の記録の tz-run.sh と tz-migrate.py・今の HEAD で撃ち直す）」を返す。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/d183-draft.md`（便 184 と共通）、script と log は同じ dir の d183-scripts。

1. 模擬: 見本 366f96f（`git diff c2e59c3 366f96f`）。fixture は fixtures-183.py、生成区間の anchor は anchors-183.py、索引の anchor は anchors-18x.py（段 183）、既存の歯の定数は consts-183.py で組み直せる。床 4 本と build は floor4.sh。
2. RED: red-183.sh（改訂 a は mut-reva.py の R0）。突然変異: mut-183.py（14 通り）と mut-reva.py（V4・V5）。余地: cap-183.sh。
3. 外の置き場: prep-tz.sh（.git だけを写す）→ tz-run.sh <写し> <見本の binary>（`folio schema --write` → tz-migrate.py → `folio derive --write` → 床・derive --check・build）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 3 つの型の面の描き方（便 184）・書き出し（便 C）・現在地（器の持ち分・ADR-31 決定 (2)(オ)）・folio2 に名札の行を置くこと（持ち主の裁定＝置かない）・tsuzuri の写しの書き直し（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) 名札の行と行の索引・計画だけの行の節を一緒に消す変更は床に見えない（ADR-31 の帰結・閉じるのは条 P-17.2 の差分の検査か器）。(2) 計画だけの行の size と files の値は器の語と照らさない（形だけ）。(3) 行の索引の行 id は文書をまたいで重なりうる（床は重なりを数えない・契約 id は `<文書 id>#<行 id>`）。(4) 状態 retired と example の計画のノートも draft と同じく まだ分からない に数える（ADR-31 は 無い・下書き だけを名指す）。
3. **撤退条件。** (1) 本便が要件書 FR27 / FR26 の字か憲法の条文か判断の記録の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `note.rs`・`derive.rs`・`rules.rs`・`ruling.rs`・`floor_note.rs` が base（c2e59c3）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) の 9 に挙げた外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: (b) の 1〜9。
- 入れない: 面の描き方・書き出し・現在地・憲法と要件書と判断の記録の字・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| key | 名札の行 | `rules.rs` の PLAN_NOTE・plan_note・key_note |
| types | 節の型 | `floor_note.rs` の 3 型と行の欄と印 |
| plan | 計画の床 | `plan.rs` の index_rows・drift・rewrite・check_plan |
| derive | 書く命令 | `derive.rs` の --write と --check |
| table | 判断の表 | `ruling.rs` の TABLE（決定の欄） |
| teeth | 歯 | f183_ の 9 本と既存の歯の字の期待 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 182（行 gc・本流 c2e59c3）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。続けて便 184 を受け付ける。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gd"
title = "計画の設計ノートの名札の行 plan-note と節の型 3 つと行の索引の導出と床（判断の記録 ADR-31 の便 B の前半・決定 (2)(ア)〜(エ)・要件書 第 1.53 版の FR27・AC29）: crates/folio/src/rules.rs の欄 key の閉じた一覧に plan-note を足し、関数 plan_note（行が無ければ掛けない・2 本か値が文書 id の形でなければ まだ分からない）と key ごとの key_note を置く。crates/folio/src/floor_note.rs の節の型の閉じた一覧に row-index・row-plan・decision-table と行の欄と生成区間の印を足す。新しい crates/folio/src/plan.rs に行の索引の導出（file 名の順・表の中の順の行 id と文書 id）と、床と folio derive --check が共有する関数 drift と、置き場の決まり（行の索引と計画だけの行の節は名札の行が名指すノートにだけ・名札の行が無くても掛かる）と計画の床（計画のノートが無い・effective でない は まだ分からない・索引の節ちょうど 1 つで導出と一致・計画だけの行の id は一意で索引に無い・depends は索引か計画だけの行に在る）を置き、crates/folio/src/note.rs が 3 型の行の欄と形を数えて check_plan を呼ぶ（設計ノートが 0 本・置き場が無いときも）。crates/folio/src/derive.rs の --write は計画のノートの生成区間も書き（全部か無しか）、--check は同じ関数で比べる。crates/folio/src/ruling.rs の決定の欄に判断の表の行を足す（12 項）。3 本の生成区間は folio schema --dir design-intent --write で書き、fixture の写し 18 本と凍結 anchor 4 本と既存の歯の字の期待を直す。面の描き方は便 184。歯は f183_ の 9 本（crates/folio/tests/plan.rs の 7 本と rules.rs・plan.rs の単体の 2 本）。実装の見本は origin の枝 impl/d183 の commit 366f96f（改訂 a・親 1f8ea15 = 見本 457d7be と本流 c2e59c3 の merge）で、作業者は write-set の file をその中身にしてよく（3 本の生成区間は folio schema --write の出力と同じ）、write-set の外は変えない。base = 本流 c2e59c3"
req = ["FR27", "FR26", "FR11", "FR19"]
section = "1"
write-set = ["crates/folio/src/derive.rs", "crates/folio/src/floor_adr.rs", "crates/folio/src/floor_note.rs", "crates/folio/src/main.rs", "crates/folio/src/note.rs", "+crates/folio/src/plan.rs", "crates/folio/src/rules.rs", "crates/folio/src/ruling.rs", "crates/folio/tests/derive.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/modules.rs", "crates/folio/tests/note.rs", "+crates/folio/tests/plan.rs", "crates/folio/tests/ruling.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/schema.rs", "design-intent/adr/schema.yaml", "design-intent/design-note/schema.yaml", "design-intent/rules.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/design-note/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/schema/note-region.txt", "tests/fixtures/schema/rules-region.txt", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test plan f183_", "cargo nextest run -p folio --bin folio f183_", "cargo nextest run -p folio --bin folio rules::tests", "cargo nextest run -p folio --test derive", "cargo nextest run -p folio --test note", "cargo nextest run -p folio --test ruling", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test modules", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/plan.rs の f183_ の 7 本（契約表に行を足すと違反 1 で folio derive --write の後に合格・計画だけの行と索引の二重と宙に浮いた依存と id の重なりがそれぞれ違反 1・計画のノートが無い・下書き・名札の行 2 本・値の形違い・設計ノートが 0 本・設計ノートの置き場が無いがそれぞれ まだ分からない 1・名札の行を消して索引の節を残すと違反 1・size と files の形と判断の表の ruling の形・索引は file 名の順〔文書 id の順ではない〕と表の中の順・印が節の外の写しで床の違反 1 と folio derive --check の 1 が同じ理由）が緑、binary の単体の f183_ の 2 本（plan_note の読みと生成区間の読み書き）が緑、binary の単体の rules::tests の全部（KEYS 2 項を含む）が緑、tests/derive.rs の歯の全部が緑、tests/note.rs の歯の全部が緑、tests/ruling.rs の歯の全部（決定の欄の一覧 12 行を含む）が緑、tests/schema.rs の歯の全部（生成区間の凍結 anchor adr 151 行・28,247 byte と design-note 164 行・19,619 byte を含む）が緑、tests/schema_docs.rs の歯の全部（rules の生成区間 32 行・3,334 byte を含む）が緑、tests/graph.rs の歯の全部が緑、tests/modules.rs の歯の全部（plan は層 2）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数と中身は着地の直前の main と同じである"
<!-- contracts:end -->
