# 設計: 便 200 — 極性一覧の口 folio check --polarity と、編集時の止めの本数の下限の数え（判断の記録 ADR-33 の便 3）

- 要件: FR29（止める仕掛けの一覧〔極性一覧〕を出し、その場で止める仕掛けの本数を下限と比べる・要件書 第 1.55 版・発効済み）と受入基準 AC32。規範文・確かめ方・受入基準は変えない。
- 条: P-18.3（極性一覧を出す）・P-18.4（編集時の止めが無い構成を検査が落とす）・P-4.1 と P-4.2（読めない下限の行は まだ分からない・数えなかったことを表に出す）・P-5.1（下限は規則の表の行の値）・P-15.2（書く前の口と素の床は同じ床の関数で下限を数える）・P-10.1（凍結 anchor は手で直した写し）。
- 出所: 判断の記録 ADR-33（accepted・持ち主の承認 2026-09-28 22:08 JST・本流 08a64bc で発効）の決定 (5)(7)。決定 (7) の「便 200（行 gu）が極性一覧の口と下限の数え・読む口の名の欄（key）の値域・床の定数の仕掛けの一覧の欄を運ぶ（in_loop は空のまま・行 R-13 に key が無いので下限は数えずに 1 行出す）」。
- 承認: 欄 key の閉じた一覧に in-loop-min を足すのは ADR-33 決定 (5) の字（「その欄の閉じた一覧に同じ値を足す」）で、規則の表の行は足しも変えもしない＝行 D-17 の 4 つに当たらない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gu` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 10 本（src 4〔新 1〕・歯の file 4〔新 1・本文を変える 3〕・凍結 anchor 1・設計文書の生成区間 1）。縮む file・消す file・新しい dir は無い。
- 門: **0（通す）**。本流 5b8aa9a の組み立ての binary に write-set を渡した `folio ceiling --gate --dir design-intent --write-set …` の答え（`folio ceiling: 通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・…）`・起草の記録の `gate-19x.log`）。
- 前提: **base = 本流 5b8aa9a**（便 198 の着地 ca8640a・行 R-20・便 196 の着地の後）。この契約の数はすべて 5b8aa9a の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d200`（commit **a250799**・本流 49c4779 の上の 11e8587 に本流 5b8aa9a〔便 196 の着地〕を取り込んだ merge 237cc58〔衝突なし〕の上に、検証役の N-19x-2・N-19x-4 の直しを積んだ commit）。`git diff 5b8aa9a a250799` が便の全体の差分（10 file・+548 −16・47,267 byte）。**作業者は write-set の file をこの枝の中身にしてよい**（`git checkout impl/d200 -- <write-set の file>`）。
- 並行の便との重なり: §1 (g) の表。便 196（`crates/folio/src/check.rs`・`crates/folio/tests/schema_docs.rs` が重なる）は本流 5b8aa9a に着地済みで、本契約の数は 196 の後の実測（見本へ取り込んだ merge は衝突なし）。本便の write-set の file を書き換える便がほかに先に着地したら、本便は余地と歯の数を数え直してから受け付ける。

## 1. 設計

### (a) いま起きていること（base 5b8aa9a の実測・参考値）

1. **極性一覧を出す口が無い。** base の binary の `folio check --polarity` は clap が断る（`error: unexpected argument '--polarity' found`・終了 2）。条 P-18 の機構は live が M1 で、素の床は P-18 を機構がまだ無い条の 1 行に出す（`# 機構がまだ無い条（…）: P-11（M1）・P-13（M1）・P-15（M1）・P-17（M1）・P-18（M1）・A-3（M1）`）。
2. **下限の行を読む口が無い。** 規則の表の行 R-13（条 P-18・what = 編集の時点で止める guard の本数の下限・値 1 本以上・種別 deny・段 post）は欄 key を持たず、欄 key の閉じた一覧（`crates/folio/src/rules.rs` の定数 KEYS）は note-chapters と plan-note の 2 つ。規則の表の生成区間（`design-intent/rules.yaml` の schema 節の enums.key と key_note）も 2 つ。
3. **材料は在る。** 憲法の各条の機構（種別・段・極性）、規則の表の閾値の行（段・条）、床の定数の仕掛けの一覧（`crates/folio/src/floor_note.rs` の床の木の guards・in_loop は空・post は yaml-form・derived-diff-zero・own-id-space・prose-gate・prose-mentions の 5 つ）。
4. **本便の後に出る一覧の数（見本の binary・参考値）。** folio2 = 仕掛け 51（憲法の条 24・規則の表の閾値の行 22・床の定数 5・in-loop 0・post 51）。床の土台 = 46（条 24・行 17・床の定数 5・in-loop 0）。tsuzuri の写し f71082f = 64（in-loop 7・post 57）。folio2 と土台と tsuzuri はどれも欄 key が in-loop-min の行を持たないので、下限は数えない。
5. **base の歯（参考値）。** workspace の nextest は §1 (e) の表・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 39 file。`git grep -n f200_ -- crates` は 0 件・行 id `gu` は 0 件。

### (b) 直す先

1. **`crates/folio/src/polarity.rs`（新）。**
   - 型 Guard（新しい名・仕掛けの名・段・極性・出所）と、1 行の字 `<名> · <段> · <極性> · <出所>`。
   - 関数 list（新しい名・引数は憲法と規則の表の木）: 極性一覧を、(i) 憲法の条のうち機構の種別が reject か build-check のもの（名 = 条 id・段と極性は機構の欄・出所 `憲法の条の機構`）、(ii) 規則の表の閾値の行（名 = 行 id・段は行の欄・極性は行の article の条の機構の極性・出所 `規則の表の閾値の行`）、(iii) 床の定数の仕掛けの一覧（床の木の guards の in_loop と post の名・段は in-loop か post・極性は fail-closed・出所 `床の定数の仕掛けの一覧`）の順に、それぞれ書かれた順で組む。欄が無ければ字 `無い`。
   - 関数 check_floor（新しい名）: 欄 key が in-loop-min の閾値の行が 1 本在れば、一覧の段が in-loop の本数がその値を割るとき違反 `[P-18] 極性一覧の編集時（in-loop）の仕掛けが <n> 本で、行 <id> の下限 <m> 本以上を割る（P-18.4）` を 1 つ積む。行が無ければ数えず、呼び手に知らせを出させる（真を返す）。2 本以上か値の形の誤りは まだ分からない `rules.yaml: <理由>`。
   - 関数 render（新しい名）: `--polarity` の出力の行（一覧の行と集計の 1 行 `folio check --polarity: 仕掛け <全部>（in-loop <n>・post <p>）・<下限の字>`・下限の字は `下限の行が無い＝数えない` か `下限 <m> 本以上（行 <id>）に足りる` か `… に足りない`）。憲法と規則の表は `check.rs` の関数 load_pair で素の床と同じ読み口で読み、読めないか下限の行が読めなければ Err（まだ分からない の字の列・検証役の N-19x-4）。
   - 定数 OFF（新しい名）: 数えなかった知らせ `# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外・条 P-18.4）`。
2. **`crates/folio/src/rules.rs`。** 定数 IN_LOOP_MIN（新しい名・字 in-loop-min）を足し、定数 KEYS を 3 項（note-chapters・plan-note・in-loop-min）にする。関数 in_loop_min（新しい名）: 欄 key が in-loop-min の行を読み、無ければ None、1 本なら行 id と値 `<正の整数> 本以上` の数（頭の 0 と空と数字以外は形の誤り）、2 本以上か形の誤りは Err（`欄 key が in-loop-min の閾値の行が <k> 本ある`・`行 <id> の value「<値>」が「<正の整数> 本以上」の形でない`）。床の木の注 key_note の末尾に 1 文（`in-loop-min（値 = 極性一覧の段が in-loop の仕掛けの本数の下限「<正の整数> 本以上」・判断の記録 ADR-33 決定 (5)）は、行が無ければ下限を数えず床の標準エラーに 1 行出す。2 本以上在るか値の形が違えば、下限の判定が まだ分からない`）。単体の歯の KEYS の期待を 3 項にする。同じ key の行が 2 本在る違反（便 179）は変えない。
3. **`crates/folio/src/check.rs`。** 関数 check_dir の中で、規則の表の床の直後に polarity.rs の関数 check_floor を撃ち、数えなかったかを Materials の欄 in_loop_min_off（新しい名）に置く。書く前の口（便 198）は同じ check_dir を撃つので、下限の違反は口の止める行にもなる（条 P-15.2）。関数 load_all の頭の置き場そのものの断り（symlink・dir でない）を関数 place_ok（新しい名・字は今と同じ）に分け、憲法と規則の表だけを床の関数 load で読む関数 load_pair（新しい名・置き場と 2 つの file の symlink・不在・file でない・parse の断りが素の床と同じ字）を足す（`--polarity` が読む・検証役の N-19x-4）。
4. **`crates/folio/src/main.rs`。** `folio check` に旗 `--polarity` を足す（`--emit-amends`・凍結の 4 つの旗・`--emit-rulings`・`--proposed` と同時に撃てない）。旗があれば polarity.rs の関数 render の行を標準出力へ出して終了 0。Err なら標準出力に `# まだ分からない: <理由>` と `folio check --polarity: まだ分からない（一覧を組めない）` の 2 行で終了 2。旗の無い素の床は、Materials の欄 in_loop_min_off が真なら定数 OFF を標準エラーへ 1 行出す（行 R-17 の知らせの次・機構がまだ無い条の行の前）。
5. **`design-intent/rules.yaml`（生成区間）。** 見本の binary の `folio schema --dir design-intent --write` で書き直す（enums.key に in-loop-min・key_note の末尾の 1 文の 2 行・生成区間 3512 → 3842 byte）。区間の外は 1 byte も変えない。
6. **`tests/fixtures/schema/rules-region.txt`（凍結 anchor）。** 前の anchor の 2 行に同じ字を手で足した写し（生成器の出力を写さない・条 P-10.1）。32 行・3842 byte・sha256 059cb57f647227f46ed725db075441b194184caee4bd730e64ec97178b895864（sha256sum で測った）。
7. **変えないもの（ADR-33 決定 (7)・起草の記録の B4）。** 床の定数の仕掛けの一覧の in_loop に名を置くこと（空のまま）・行 R-13 に欄 key を付けること・条 P-15 と P-18 の機構の live と注・設計ノートの欄の決まりの guards の生成区間・憲法と要件書と判断の記録の字・規則の表の行・`folio build` の出力。この 3 つは、器の行が着地し、器の極性一覧に口を撃つ行（design-check）が在ることを席が確かめた後の便が運ぶ。

### (c) 歯（f200_・base で 0 件・7 本）

binary 経由の歯は `crates/folio/tests/polarity.rs`（新）。どれも床の土台（`tests/fixtures/floor_base/design-intent/`）を一時 dir に design-intent/ として写し、器の導出 file を contracts/ に写し、規則の表だけを歯の側の手書きの字で直してから git の 1 commit にして撃つ。期待の本数と字は歯の側の手書き（土台の正本を手で数えた値・folio の code から組まない）。

1. **f200_polarity_lists_each_guard_and_a_summary_on_the_floor_base。** `--polarity` は終了 0・標準エラーは空・標準出力は 47 行（条 24・行 17・床の定数 5・集計 1）で、末尾がちょうど `folio check --polarity: 仕掛け 46（in-loop 0・post 46）・下限の行が無い＝数えない`。出所の字で数えると 24・17・5。1 行目は `P-1 · post · fail-closed · 憲法の条の機構`、25 行目は `R-1 · post · fail-closed · 規則の表の閾値の行`、42 行目は `yaml-form · post · fail-closed · 床の定数の仕掛けの一覧`。機構の種別が human-review の条（P-16・A-1・A-4）の行は無い。
2. **f200_floor_fails_when_in_loop_guards_are_below_the_bound（AC32 の 2 つ目）。** 行 R-13 に欄 key in-loop-min を付け値を 99 本以上にした写しで、素の床は終了 1・違反の行がちょうど `[P-18] 極性一覧の編集時（in-loop）の仕掛けが 0 本で、行 R-13 の下限 99 本以上を割る（P-18.4）` の 1 行・知らせの行は無い。`--polarity` の集計は `… ・下限 99 本以上（行 R-13）に足りない`。続けて、値を 1 本以上にし行 R-7 の段を in-loop にした写しでは、素の床は終了 0・違反 0、一覧に `R-7 · in-loop · fail-closed · 規則の表の閾値の行` が在り、集計は `folio check --polarity: 仕掛け 46（in-loop 1・post 45）・下限 1 本以上（行 R-13）に足りる`。
3. **f200_two_bound_rows_are_unknown（AC32 の 3 つ目）。** 行 R-13 と行 R-7 の 2 本に欄 key in-loop-min を付けた写しで、`--polarity` は終了 2・標準出力はちょうど `# まだ分からない: 欄 key が in-loop-min の閾値の行が 2 本ある` と `folio check --polarity: まだ分からない（一覧を組めない）` の 2 行。素の床は終了 2（まだ分からない）で、標準エラーの まだ分からない の行がちょうど `# まだ分からない: rules.yaml: 欄 key が in-loop-min の閾値の行が 2 本ある` の 1 行、違反の行はちょうど便 179 の `[schema] rules.yaml: 行 R-13 の key「in-loop-min」を持つ閾値の行が 2 本以上ある` の 1 行、知らせの行は無い。
4. **f200_a_bound_value_out_of_form_is_unknown。** 行 R-13 に欄 key を付け、値を 0 本以上・3 本・いくつか 本以上 の 3 通りにした写しで、素の床はどれも終了 2・違反 0・まだ分からない の行がちょうど `# まだ分からない: rules.yaml: 行 R-13 の value「<値>」が「<正の整数> 本以上」の形でない` の 1 行・知らせの行は無い。`--polarity` はどれも終了 2。
5. **f200_no_bound_row_prints_one_line_and_does_not_count（AC32 の 4 つ目）。** 土台そのままの写しで、素の床は終了 0・違反 0・まだ分からない 0・標準エラーに知らせ（定数 OFF の字）がちょうど 1 行。同じ置き場で、行 R-13 に欄 key と値 99 本以上を付けた規則の表を `--proposed rules.yaml` に渡すと終了 1 で、標準出力に `行 R-13 の下限 99 本以上を割る` を含む止める行が在り、口の標準エラーに知らせの行は無い（口も同じ床の関数で数える）。
6. **f200_row_polarity_follows_its_article（検証役の N-19x-2）。** 憲法の条 P-11（行 R-7 の条）の機構の極性を fail-open にした写しで、`--polarity` は終了 0・一覧に `P-11 · post · fail-open · 憲法の条の機構` と `R-7 · post · fail-open · 規則の表の閾値の行` が在り、fail-open の行はちょうど 2 行・集計は土台と同じ（行の極性は行の条の機構の極性で、一律の fail-closed ではない）。
7. **f200_polarity_refuses_a_symlinked_source_like_the_floor（検証役の N-19x-4）。** 規則の表を置き場の外の file への symlink にした写しで、`--polarity` は終了 2・標準出力はちょうど `# まだ分からない: rules.yaml: symlink は認めない` と `folio check --polarity: まだ分からない（一覧を組めない）` の 2 行。素の床も終了 2 で、まだ分からない の行がちょうど同じ字の 1 行（同じ読み口）。
8. **既存の歯の期待の直し。** `crates/folio/tests/mechanism_live.rs`: 土台と骨格は下限の行を持たないので、知らせの行が行 R-17 の知らせの次に出る。手書きの定数 IN_LOOP_OFF（新しい名・定数 OFF と同じ字）を足し、f131_floor_base_lists_the_articles_before_the_summary の 2 か所（素の床と `--emit-amends`）・f131_no_line_when_no_article_waits_for_its_mechanism・f156_a_place_without_r17_says_the_mentions_are_not_counted（末尾の 2 行が行 R-17 の知らせと IN_LOOP_OFF）の標準エラーの期待に 1 行足す。`crates/folio/tests/schema_docs.rs`: 定数 RULES_REGION_BYTES を 3842・RULES_REGION_SHA256 を上の値にする（行数 32 は同じ・注 1 行）。`crates/folio/tests/modules.rs`: 層の割り当ての表に区切り polarity を層 2 として 1 行（読む相手は verdict〔0〕・constitution_enums と floor と floor_note と yaml〔1〕・rules〔2〕、名指す側は check〔2〕と main〔5〕）。
9. **RED の実測。** 歯の file だけ（`r200-teeth.patch`＝`tests/polarity.rs`・`tests/mechanism_live.rs`・`tests/modules.rs`・`tests/schema_docs.rs`・`tests/fixtures/schema/rules-region.txt` の差分）を base に当てると、f200_ の 7 本とも落ちる（clap が旗を断る・欄 key の値域の外は便 179 の違反・知らせの行が無い）・`tests/mechanism_live.rs` の 3 本が落ちる（知らせの行が無い）・`tests/schema_docs.rs` の f85_rules_region_matches_the_new_anchor が落ちる（生成区間 3512 byte）・`tests/modules.rs` の 2 本（p106_layers_cover_every_module・p106_edges_point_down）が落ちる（表に区切り polarity が在るのに src に file が無い）。log は起草の記録の `red-200d.log`。

### (d) 採らなかった形

1. **`--polarity` が下限を割ると終了 1 にする。** 判定は素の床が持つ（違反の行・同じ関数）。口が 2 つ目の判定を持つと床と口の 2 面が同じ判定を持つ（条 P-6.3）。`--polarity` は一覧と集計の字だけを返し、読めなければ まだ分からない（2）。
2. **下限の行が読めなくても一覧は出す。** 集計の字が数えていない下限を名乗れず、読めない値を黙って落とす（条 P-4.1）。一覧を組めないとして断る。
3. **知らせを標準出力に出す。** 素の床の標準出力は違反の行と要約の 1 行で、行 R-17 の知らせ（便 156）も標準エラー。同じ扱いに揃える。
4. **`--polarity` が正本を自分で読む（`fs::read_to_string`）。** 素の床が断る symlink の正本を一覧の口だけが読み、床と口で読める置き場が食い違う（条 P-15.2・P-4.1）。起草の 11e8587 はそうなっていて、検証役の N-19x-4 で床の関数 load に寄せた（歯 (c) 7・変異 M21）。
5. **in_loop に口の族の名（yaml-form・prose-gate）を置く・行 R-13 に欄 key を付ける。** ADR-33 決定 (7) と起草の記録の B4 のとおり、器の行の着地の前に置くと、編集の時点で止める仕掛けが 1 本も動かないのに条 P-18.4 の床が合格になる（条 P-4.1・P-15.1・P-3.3）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 本便の差分を base に当てた写し（見本 a250799）の数は次の表（参考値・起草の記録の `verify-200d-summary.txt`・`CARGO_BUILD_JOBS=4`・`--test-threads 2`）。字の期待を直した既存の歯は (c) の 8 の 5 本と単体の 1 本（rules.rs の f179_key_is_a_closed_list_and_one_threshold_row_per_key）。

| 数え | base 5b8aa9a | 見本 a250799 |
| --- | ---: | ---: |
| workspace の nextest（全部合格） | 1131 | 1138（+7 の歯） |
| clippy の警告 | 0 | 0 |
| 床 4 本（check・inject --check・schema --check・derive --check） | 4 本とも rc 0 | 4 本とも rc 0 |
| `folio build --write` の file 数 | 39 | 39（base と全 file が byte で同じ） |

2. **突然変異（見本の写しの src だけを 1 通りずつ変え、`--test polarity` を撃つ・`mut-200.py`・歯の番号は (c)）。**

| 変異 | 落ちる歯 |
| --- | --- |
| M1 条の機構の種別の絞りから build-check を外す | 1・2・6 |
| M2 条の機構の種別の絞りから reject を外す | 1・2・6 |
| M3 条の機構の種別で絞らない（human-review も出す） | 1・2・6 |
| M4 規則の表の閾値の行を一覧に入れない | 1・2・6 |
| M5 床の定数の仕掛けの一覧を入れない | 1・2・6 |
| M6 行の極性を条の機構でなく行の欄から読む | 1・2・6 |
| M7 床の定数の仕掛けの極性を fail-open にする | 1・6 |
| M8 in-loop の本数に post を数える | 1・2・6 |
| M9 下限の比べを「以下」にする（等しくても落とす） | 2 |
| M10 下限の行が無いとき知らせない | 5 |
| M11 下限の行の読めなさを まだ分からない にしない | 3・4 |
| M12 下限の行が 2 本でも 1 本目を読む | 3 |
| M13 下限の値の 0 始まりを許す | 4 |
| M14 下限の値の「以上」を読まない | 2・4・5 |
| M15 欄 key の閉じた一覧に in-loop-min を置かない | 2・3・4 |
| M16 素の床が下限を数えない | 2・3・4・5 |
| M17 素の床が知らせの行を出さない | 5 |
| M18 `--polarity` が読めない正本で 0 を返す | 3・4・7 |
| M19 集計の足りる / 足りないを逆にする | 2 |
| M20 行の極性を fail-closed に固定する（検証役の Z10） | 6 |
| M21 `--polarity` が正本を床の読み口を通さずに読む（検証役の N-19x-4） | 7 |

   21 通りとも落ちる（生き残り 0・`mut-200d.log`・直列・a250799）。M20 は検証役の変異 Z10 で、11e8587 の歯では生き残った（土台の条はどれも fail-closed）＝歯 (c) 6 を足した。

3. **外の置き場（tsuzuri の写し f71082f・参考値・`tz-19x.log`）。** 見本の binary の素の床は base の binary と同じ判定（便 196 の着地の後の base では、tsuzuri の設計ノートの欄の決まりの写しが古いので違反 1〔known_values の欠落〕で終了 1）で、標準エラーに知らせの 1 行が増える（tsuzuri の規則の表は欄 key が in-loop-min の行を持たない）。`--polarity` は仕掛け 64（in-loop 7 = 憲法の条 4・規則の表の行 3）。見本の binary の `folio schema --write` は規則の表の生成区間の enums.key と key_note を書き直す（ほかの生成区間の差は便 190・195・196・197 の着地の分で、base の binary でも tsuzuri の `schema --check` はすでに 1）。その後の素の床は終了 0（知らせの 1 行は残る）・`schema --check` は 0。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file は `+crates/folio/src/polarity.rs` と `+crates/folio/tests/polarity.rs`。差分 47,267 byte（`git diff 5b8aa9a a250799 | wc -c`・10 file・+548 −16）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 4 本（python と awk の 2 実装で一致・`cap-19x.sh`・`cap-200d.log`）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/check.rs` | 1144 | 356 | 1171（+27） | 329 |
| `crates/folio/src/main.rs` | 758 | 742 | 778（+20） | 722 |
| `crates/folio/src/rules.rs` | 475 | 1025 | 498（+23） | 1002 |
| `crates/folio/src/polarity.rs` | 0（新） | 1500 | 120（+120） | 1380 |

3. **size は M。** src の増分は +190 で S の見積 100 を超え、M の見積 300 の内。余地の最小（check.rs の base 356・本便の後 329）は M の 300 を超える。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。見本の写しで 6 行とも rc 0。
   1. `cargo nextest run -p folio --test polarity` = (c) の 1〜7（7 本）。
   2. `cargo nextest run -p folio --test mechanism_live` = 知らせの行の順（(c) の 8・6 本）。
   3. `cargo nextest run -p folio --test schema_docs` = 規則の表の生成区間の凍結 anchor の行数・byte 数・要約値と歯の file の上限（28 本）。
   4. `cargo nextest run -p folio --test modules` = 層の割り当ての表と層が上がる辺（便 106 の歯・4 本）。
   5. `cargo nextest run -p folio --bin folio rules::tests` = 規則の表の床の単体の歯（KEYS の 3 項と生成区間の凍結 anchor との byte 一致を含む・6 本）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（polarity・mechanism_live・schema_docs・modules）は全部 write-set に在る。`--bin folio` の歯の在り処（rules.rs）も write-set に在る。素の `folio check` の歯（`--test check`）は common-verify の workspace の nextest が撃つ。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり 0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/guard2 の本契約で 契約に起因する断り 0（起草の記録の `precheck-200.log`）。ADR-33 と要件書 第 1.55 版は発効済み（req の FR29 は本流に在る）。
3. **重なり（2026-09-29 02:4x と 03:0x の実測・origin の各レーンの見本の枝と本便の見本を git merge-tree で重ねた）。**

| レーン（便・見本） | 重なる file | 扱い |
| --- | --- | --- |
| 写しの負債（196・本流 5b8aa9a） | `crates/folio/src/check.rs`・`crates/folio/tests/schema_docs.rs` | 着地済み。見本へ取り込んだ merge で自動で合わさった（衝突なし）。check.rs は base 1144 → 本便の後 1171（余地 329）。schema_docs.rs は本便の後 1177 行（器の式で 1178・歯の file の上限 1200 の内・歯 f89_schema_teeth_are_split_and_under_the_cap）。本便の注 1 行は定数 RULES_REGION_* の上に置き、196 と同じ頭の注の列の末尾には足さない（頭の注の列の末尾に足した初めの形は 196 と衝突した）。生成区間は 196 の要件書の ids_anchor（srs.yaml 1771 byte）と設計ノートの欄の決まり（20312 byte）と並び、見本の binary の `folio schema --check` は 9 本とも一致。数は 196 の後の本流の上で数え直した（この節の表と (e)(f)）。 |
| 便 199（impl/d199 85b6d84） | 無し | 無し（衝突なし・どちらが先でも数は変わらない） |
| 便 201（impl/d201 8dbdd4d） | `crates/folio/src/main.rs` | 自動で重なる（衝突なし） |

4. **着地の後（席へ）。** (1) tsuzuri へ: 本便の binary では素の床の標準エラーに知らせの 1 行が増える（判定と終了は同じ）。`folio schema --write` で規則の表の生成区間（enums.key と key_note）が変わる（ほかの着地の分と 1 回にまとめてよい）。設計ノートの欄の決まりの guards の生成区間は本便では変わらない（in_loop は空のまま・ADR-33 の帰結の〔guards〕は in_loop に名を置く後の便で起きる）。下限は、tsuzuri が下限の行に欄 key を付けるまで数えない。(2) 器の席へ: 口は `--polarity` の一覧で器の行（design-check · in-loop）を数えない（器の極性一覧は器が持つ）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/guard-draft.md`、script と log は同じ dir の `guard-scripts/`。

1. 模擬: 見本 impl/d200 の a250799（`git diff 5b8aa9a a250799` = `c200.patch`・歯だけ = `r200-teeth.patch`）。`verify-198.sh <見本> <base> <log>`。
2. RED: `red-19x.sh <base> r200-teeth.patch <log> polarity - mechanism_live - schema_docs f85_ modules -`。突然変異: `mut-200.py`（M1〜M21・直列）。余地: `cap-19x.sh <base> <見本> polarity.rs rules.rs check.rs main.rs`。資源は `CARGO_BUILD_JOBS=4`・nextest `--test-threads 2`。
3. 外の置き場: `tz-19x.sh`（tsuzuri の `.git` だけを写して clone・remote を外した写し）。

### (i) 本便が運ばないもの・読み・席へ返すこと・撤退条件

1. **運ばないもの。** (b) の 7 のとおり。器の hook の行と器の極性一覧（器の判断の記録と便）。条 P-18 の機構の注と行 R-13 の注（どちらも「（2026-09-20 時点）」と日付の付いた事実の字）は本便では変えない。直すのは後の B4 の便で、R-13 の欄 key・in_loop の名・live と一緒に、行の変更の裁定 id と持ち主の承認（行 D-17）を取って直す（席の裁定・台帳 f2-648.275.2 の notes）。
2. **読み（起草の記録の N5 (iii)）。** 条 P-18.3 の「その一覧（極性一覧）を生成時に出す」は、`folio check --polarity` が求められたときに正本の型付きのデータ（憲法の機構・規則の表の行・床の定数）から一覧をその場で組んで出すことで満たすと読む（手で書く一覧を持たない・`folio build` の面には出さない）。ADR-33 決定 (5) はすでに発効し封が在るので、この読みは本契約に置く（席の裁定: 承認済みの決定 (5) の範囲内の読みで、新しい判断の記録は起こさない・台帳 f2-648.275.2 の notes）。
3. **後の便の縛り（起草の記録の N5 (ii)）。** 行 R-13 に欄 key in-loop-min を付ける後の便は、規則の表の行の変更なので、行に裁定 id と時刻を付け（条 P-17.1・行 D-17 の持ち主の承認）、R-13 の注の「極性一覧を出す口も本数を数える口も未実装」の字と、この行が床の判定に数えられない旨の字を同じ便で書き換える。
4. **注の字（席の裁定）。** 条 P-18 の機構の注（「極性一覧を出す口と R-13 を数える口は未実装である（2026-09-20 時点・…）」）と行 R-13 の注は日付の付いた事実の字なので、本便の着地の後も字の上では偽にならない。直すのは後の B4 の便（1 の末尾）。
5. **言えないこと。** (1) 一覧の段の札は、仕掛けが設計文書の編集を止めるかを言わない（tsuzuri の in-loop 7 は serve の接続先や席の停止の条と行で、設計文書を書く時点の止めではない）。(2) 下限の違反は条 P-18 の機構の live に依らず数える（live が M1 のまま欄 key の行を置けば数える）。folio2 は R-13 に欄 key を付けるときに live を一緒に改める（ADR-33 決定 (7)）。(3) 床の定数の仕掛けの極性は一律 fail-closed（ADR-33 決定 (5)）で、仕掛けごとの測れない周の倒し方は数えない。
6. **撤退条件。** (1) 本便が要件書 FR29 か ADR-33 の字か憲法の条文か規則の表の行を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の関数 check_dir の規則の表の床の撃ち方か素の床の標準エラーの行の順が base（5b8aa9a）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 口（`--proposed`）が下限の違反を止めた中身を、同じ中身を書いた置き場の素の床が合格にする周が見つかったら、止めて席へ返す。

## 2. 範囲

- 入れる: `folio check --polarity`（旗・一覧の行と集計の行・読めなければ まだ分からない）・`polarity.rs`・`rules.rs` の定数 IN_LOOP_MIN と KEYS の 3 項と関数 in_loop_min と key_note の 1 文・`check.rs` の下限の数え・`main.rs` の旗と知らせの 1 行・規則の表の生成区間と凍結 anchor・歯の file の f200_ の 5 本と既存の歯の期待の直し。
- 入れない: in_loop の名・行 R-13 の欄 key・条 P-15 と P-18 の機構の live と注・設計ノートの欄の決まりの guards の生成区間・器の hook と器の極性一覧・憲法と要件書と判断の記録の字・規則の表の行・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| list | 極性一覧 | `polarity.rs` の型 Guard と関数 list（憲法の条の機構 → 規則の表の閾値の行 → 床の定数の仕掛けの一覧） |
| read | 読み口 | `check.rs` の関数 place_ok と load_pair（`--polarity` が素の床と同じ字で断る） |
| bound | 下限の数え | `rules.rs` の関数 in_loop_min・`polarity.rs` の関数 check_floor・`check.rs` の check_dir の 1 か所 |
| mouth | 一覧の口 | `main.rs` の `--polarity`・`polarity.rs` の関数 render |
| key | 欄 key の値域 | `rules.rs` の定数 KEYS と key_note・規則の表の生成区間と凍結 anchor |
| teeth | 歯 | `tests/polarity.rs` の f200_ の 7 本・既存の歯の期待の直し |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 本流 5b8aa9a（便 198 の着地 ca8640a と便 196 の着地の後）。ADR-33 と要件書 第 1.55 版は発効済み。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) の 4 を返す。§1 (i) の 2〜4 を決める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gu"
title = "極性一覧の口 folio check --polarity と編集時の止めの本数の下限の数え（判断の記録 ADR-33 の便 3・決定 (5)(7)・要件書 第 1.55 版の FR29 と AC32）: crates/folio/src/polarity.rs（新）が憲法の条の機構（種別 reject と build-check）・規則の表の閾値の行（極性は行の条の機構）・床の定数の仕掛けの一覧（floor_note の guards・極性 fail-closed）の順に 1 仕掛け 1 行の極性一覧を組み、--polarity は末尾に集計の 1 行を出す（正本は crates/folio/src/check.rs の load_pair で素の床と同じ読み口で読み、読めなければ まだ分からない 2）。crates/folio/src/rules.rs の欄 key の閉じた一覧に in-loop-min を足し（関数 in_loop_min・key_note の 1 文・規則の表の生成区間と凍結 anchor rules-region.txt）、crates/folio/src/check.rs の check_dir が欄 key が in-loop-min の行が 1 本在れば段が in-loop の本数を下限と比べて割れば違反 1、無ければ数えずに素の床の標準エラーへ 1 行、2 本以上か値の形の誤りは まだ分からない。in_loop の名・行 R-13 の欄 key・条 P-15 と P-18 の機構の live は運ばない（base = 本流 5b8aa9a・見本 impl/d200 a250799）"
req = ["FR29"]
section = "1"
write-set = ["+crates/folio/src/polarity.rs", "crates/folio/src/rules.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "+crates/folio/tests/polarity.rs", "crates/folio/tests/mechanism_live.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/modules.rs", "tests/fixtures/schema/rules-region.txt", "design-intent/rules.yaml"]
verify = ["cargo nextest run -p folio --test polarity", "cargo nextest run -p folio --test mechanism_live", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test modules", "cargo nextest run -p folio --bin folio rules::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "crates/folio/tests/polarity.rs の f200_ の 7 本（床の土台の極性一覧の行の数と集計の 1 行が手で数えた数と一致・行の極性は行の条の機構の極性・正本の symlink は --polarity も素の床と同じ字で断る・下限の行の値を割る写しは素の床が違反 1 で足りる写しは合格・下限の行が 2 本の写しは まだ分からない・値の形が違う写しは まだ分からない・下限の行が無い写しは数えなかった 1 行で書く前の口も同じ床の関数で下限を数える）が緑、crates/folio/tests/mechanism_live.rs の歯の全部（知らせの行が行 R-17 の知らせの次に出る）が緑、crates/folio/tests/schema_docs.rs の歯の全部（規則の表の生成区間の凍結 anchor の行数・byte 数・要約値と歯の file の上限）が緑、crates/folio/tests/modules.rs の歯の全部（層の割り当ての表に polarity）が緑、rules.rs の単体の歯 6 本（KEYS の 3 項と生成区間の凍結 anchor との byte 一致）が緑、clippy 0 警告"
<!-- contracts:end -->
