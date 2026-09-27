# 設計: 便 179 — 設計ノートの面の章の上限を、置き場の規則の表の欄 key が note-chapters の閾値の行から読み、面の生成器と床が同じ関数で数える

- 要件: FR9（設計ノートを 1 つの型で生成する・面の生成器が上限を超える正本を生成しない）・FR5（床の 3 値＝上限の超過は違反、上限が読めなければ まだ分からない）・FR19（規則の表の生成区間へ床の定数から欄の決まりを写す＝行の欄 key と閉じた一覧）。どれも本流の要件書（第 1.51 版）に在る id で、字は変えない。
- 条: P-4.1 / P-4.2（上限の行が無い・2 本・値の形が違う を合格にも既定の値にもしない＝まだ分からない）・P-5.1 と行 D-11（検査の結果を分ける数の閾値は実装の定数に置かず規則の表の行に置く）・P-15.2 の精神（面の生成器と床が同じ関数で数える）・P-10.1 / P-10.2（凍結 anchor は生成側に依らず組み直す）・N-3.1（欄 key は閉じた一覧で、置き場の側から読み方を広げられない）。
- 出所: 持ち主の裁定 2026-09-28 02:34 JST（対話面 R-8・逐語「推奨で」＝案 A: 章の上限を置き場ごとの規則の表の値にし、folio2 は 12 のまま・tsuzuri は自分で値を決め、床も同じ値と式で数える）。判断の記録 ADR-11 決定 (3)(エ)・(4)⑦（実装にだけ在る閾値を規則の表の行へ・実装はその行から読む）・台帳 f2-648.75 / f2-648.250。tsuzuri の写し（HEAD b02c2ae）で本流の組み立ての `folio build` が設計ノート surface-base の 24 章で止まり 1 file も書けない（席の実測 2026-09-28 02:2x・起草役が写しで撃ち直した＝(a) の 3）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fz` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 23 本（src 4・歯の file 7・設計文書の正本 1・fixture と凍結 anchor 10・床の case の file 1）。縮む file は `crates/folio/src/face_note.rs` の 1 本（頭に `-`）。新しい file・消す file・新しい dir は無い。
- 門: 対象。write-set に設計文書の正本 `design-intent/rules.yaml` が在る。base（下の前提）の写しで、本流の組み立て（866bb78 の binary）に write-set 23 本を渡した `folio ceiling --gate --dir design-intent --write-set …` は **0（通す・印の周 2026-09-27-round51〔判定 合格〕に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）**。
- 前提: **base = 枝 docs/cap の設計の変更（commit 3bb4482 = 本流 866bb78 に、規則の表の行 R-19〔欄 key 無し〕・憲法 P-2 の relations.rules の R-19・歯 f91 の期待を足したもの、と commit 155e4d4 = 行 R-19 の注の 1 句の直し）が本流に着地した後の main。この契約の数はすべて 155e4d4 の写しの実測（参考値）である**（規則の表の行 D-13・3bb4482 で測った数と同じで、違いは行 R-19 の注の 1 句と面 constitution.html のその字だけ）。受付の時点の main が 3bb4482 の中身と違えば、その main で数え直す。行 R-19 の ruling の欄は持ち主の承認の後に席が設計の変更の側で書き、本便は ruling の欄を変えない。
- 実装の見本: origin の枝 `impl/d179`（commit 21b44b3 = 見本 db8c609〔親 3bb4482〕に枝 docs/cap の 155e4d4 を merge した dee6512 に、改訂 a の歯の直し〔(c) の 3 の見本の図を 2 枚にする・tests/note.rs だけ〕を積んだもの）が本便の後の中身で、`git diff 155e4d4 21b44b3` が便の全体の差分（file の集合は `git diff 3bb4482 db8c609` と同じ 23 file）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 21b44b3 -- <write-set の file>`）。ただし `design-intent/rules.yaml` は、受付の時点の main の行 R-19 の ruling の欄が見本と違えば見本の file を写さず、(b) の 5 の 3 か所だけを当てる（生成区間は `folio schema --dir design-intent --write` で書く）。write-set の外は変えない。
- 並行の便との重なり: base の時点で、`crates/folio/src/{rules,check,note,face_note}.rs`・`design-intent/rules.yaml`・write-set の fixture を書き換える未着地の便の契約は無い（便 178 は着地済み＝本流 866bb78）。受付の時点で precheck が重なりを見る。

## 1. 設計

### (a) いま起きていること（base 155e4d4 の実測・参考値）

1. **章の上限は実装にだけ在る。** `crates/folio/src/face_note.rs` の定数 `MAX_CHAPTERS`（値 12・章の帯の表 `BANDS` の長さ）を関数 derive が節の数（図が在れば + 1）と比べ、超えれば `<file>: 章が N 本ある＝章が多すぎる（上限 12）` の Err で面を生成しない。配信先の組み立て `folio build` は全部か無しかなので 1 file も書かない。床（`crates/folio/src/note.rs` の関数 check_note）は章を数えない＝検査の時点では分からない（台帳 f2-648.250）。規則の表の行 R-19（設計の変更で足した・値 12 章 以下）はまだ誰も読まない。
2. **規則の表の行の id は置き場ごとに意味が違う。** folio2 は R-1〜R-17・R-19・R-21・R-22、tsuzuri は R-1〜R-12・R-16〜R-25（tsuzuri の R-19 は文の予算・R-23 は面の受入）。外の置き場で床が id で名指してよいのは行 R-8 と R-16 だけ（ADR-16 決定 (2)(オ)・`crates/folio/src/floor.rs` の定数 `RESERVED`）。
3. **tsuzuri の写し（t3v の tz3 を起草役の scratch へ clone・本物では撃たない）。** 本流の組み立て（866bb78）で床は合格 0/0 だが、`folio build` は `まだ分からない: design-note/surface-base.yaml: 章が 24 本ある＝章が多すぎる（上限 12）`（surface-board は 34 章）。天井の周の束も面の配信先を材料にするので組めない。
4. **base の歯（参考値）。** workspace の nextest 1003 / 1003・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file。binary の単体の歯 152 本・`tests/face_note.rs` 31 本・`tests/graph.rs` 16 本。`git grep -n f179_ -- crates` は 0 件・行 id `fz` は 0 件。

### (b) 直す先 — 欄 key で行を引き、面の生成器と床が同じ関数で数える

1. **`crates/folio/src/rules.rs`（規則の表の床の定数）。** 閾値の行の任意の欄に `key` を足し（任意の欄の一覧の末尾・欄名の定数 ROW_KEY）、欄 key の閉じた一覧 KEYS（値 note-chapters の 1 つ・定数 NOTE_CHAPTERS）を床の木の enums の子 key と説明の注 key_note として生成区間へ写す（FR19）。関数 3 つを足す: keyed（置き場の規則の表の thresholds のうち欄 key が指定の値の行がちょうど 1 本ならそれを返し、0 本は `欄 key が … の閾値の行が無い`・2 本以上は `… が N 本ある` の Err）・chapter_cap（keyed で note-chapters の行を引き、value が「<正の整数> 章 以下」の字＝半角の数字だけで頭が 0 でなく、後ろが半角空白・章・半角空白・以下 のときだけその数を返し、ほかは `行 <id> の value「…」が「<正の整数> 章 以下」の形でない` の Err）・key_violations（欄 key の値が閉じた一覧に無い行と、同じ値を持つ 2 本目以降の閾値の行の字を返す）。行の id は見ない＝置き場ごとの番号に依らない。開発規律の行に欄 key を書くのは今の未知の欄の検査が落とす（任意の欄の一覧を閾値の行にだけ足すため）。
2. **`crates/folio/src/check.rs` の関数 check_rules。** 行の検査の後に key_violations の字を種別 schema の違反として出す（3 行）。
3. **`crates/folio/src/note.rs`（床）。** 関数 chapters（節の数に、図が 1 枚でも在れば 1 を足す＝承認欄は数えない）と関数 over_cap（章の数が上限を超えるときだけ `<file>: 章が N 本ある＝章が多すぎる（上限 M）` を返す）を足す。check_note は散文の門の行を読む所の隣で chapter_cap を 1 回読み、読めなければ `rules.yaml: 設計ノートの章の上限が読めない: <理由>` の まだ分からない を 1 つ出す（設計ノートが 1 本も無い置き場では読まない＝今の早い戻りのまま）。読めれば設計ノートごとに sections と figures の要素の数で chapters と over_cap を呼び、超えれば種別 note の違反にする。
4. **`crates/folio/src/face_note.rs`（面の生成器）。** 定数 `MAX_CHAPTERS` を消し、関数 derive は置き場の rules.yaml を読んで chapter_cap（読めなければ `rules.yaml: 設計ノートの章の上限が読めない: <理由>` の Err）・chapters・over_cap を床と同じ関数で呼ぶ。章の帯の表 `BANDS` は前半の 6 つだけにし（後半 6 つは前半と同じ字）、章の数だけ先頭から順に回した列を Frame に渡す（Frame の欄 bands は 'static なので、1 面に 1 本だけ leak する＝同じ関数の source と同じ扱い）。12 章までの面は base と byte で同じ（(e) の 1）。
5. **`design-intent/rules.yaml`。** (i) 生成区間を `folio schema --dir design-intent --write` で書き直す（threshold_row.optional の末尾に key・enums に key の 1 行・enums_note の後に key_note の 1 行）。(ii) 行 R-19 の 2 か所だけを直す: population の欄の前に `key: note-chapters, ` を足す／注の末尾の 1 文「便 179 の着地までは、面の生成器が実装の定数（同じ値 12）で数え、床は章を数えず、この行はまだ欄 key を持たない。」を消す。ほかの欄（値・種別・段・ruling・注のほかの字）は変えない。この 2 か所は持ち主が承認した行の最終の字（設計の変更の承認の要求に最終の字として示した）で、承認を超えない。
6. **fixture と凍結 anchor（字の期待だけ・どれも手で数えた値）。** (i) 面の凍結 fixture `tests/fixtures/face/rules.yaml` に閾値の行 R-2（欄 key note-chapters・値 12 章 以下）を足し、その写しを材料にする凍結の面 3 本（`expected.html` の数値の表の 1 行と件数の字 2 か所・`expected-index.html` と `expected-index-sheet.html` の付録の行数の字 1 か所）を組み直す。(ii) 床の土台 `tests/fixtures/floor_base/design-intent/rules.yaml` に同じ欄 key の行 R-19 を足し、同じ置き場の憲法 P-2 の relations.rules に R-19 を足す（行 R-4 の双方向）。その索引の凍結 anchor 3 本（`graph-anchor.txt`・`graph-digest-anchor.txt`・`node-digest-anchor.txt`）は節点 1（R-19）と辺 2（P-2 → R-19 の relations.rules・R-19 → P-2 の article）が増えた字にする。node-digest-anchor は独立の script `tests/fixtures/schema/node-digest.py` の出力、graph-anchor は base の出力（旧 anchor の要約値と一致を確かめたもの）に手で書いた 3 行を id の byte 順の位置へ差し込んだ字の要約値で、folio の出力と byte で一致した。(iii) 規則の表の生成区間の凍結 anchor `tests/fixtures/schema/rules-region.txt` に同じ 3 か所を手で足す（実装の導出と byte で一致）。(iv) 床の case の file `tests/floor_cases.yaml` の case mentions-rule-row-counts-the-frozen-base の置き場の添字を thresholds の 16 から 17 へ（土台の末尾に行 R-19 が入ったため・case の中身は変えない）。
7. **既存の歯の字の期待（中身は変えない）。** `tests/graph.rs`（土台の索引の数 189 → 190 節点・576 → 578 辺・出力の行と byte・anchor の byte と行と要約値・辺の表の要約値の頭 8 字）・`tests/hello.rs`（土台の数の 1 行）・`tests/schema_docs.rs`（規則の表の生成区間の行数・byte・要約値の定数 3 つ）・`tests/unknown_fields.rs`（生成区間の threshold_row.optional の行の字）。
8. **変えないもの。** 散文の門（行 R-16 を id で引く・prose.rs）・外の置き場で床が id で名指す行の一覧 RESERVED・導出物の命令 folio derive（上限を掛けない＝ADR-16 決定 (5)）・骨格の命令 folio init（外の利用者は tsuzuri だけ・持ち主の裁定 2026-09-27 22:38）・要件書・憲法・判断の記録・設計ノートの字・面の見た目（12 章までの設計ノートの面は byte で同じ）。

### (c) 歯（f179_・base で 0 件）

1. **f179_face_reads_the_cap_from_the_keyed_row_whatever_its_id（`crates/folio/tests/face_note.rs`・binary 経由）。** 面の fixture の写しの full.yaml（節 6・図 1）に節 7〜13 を足した 14 章の設計ノートは、行 R-2 の値 12 章 以下では 2 で `design-note/full.yaml: 章が 14 本ある＝章が多すぎる（上限 12）` を出して面を書かず、同じ行の id を R-26・値を 14 章 以下にすると 0 で面を書く（行の id に依らない）。その面の章 1・6・7・12・13・14 の帯の class は band-1・band-6・band-1・band-6・band-1・band-2（7 章目から繰り返す）で、crumb の分母は 15（帯の章 14 + 承認欄）・章 15 は無い。**base は値 14 でも上限 12 で止まる＝RED。**
2. **f179_face_is_unknown_when_the_keyed_row_is_missing_doubled_or_malformed（同）。** 今の fixture（7 章）のまま行だけを崩す 4 通り＝欄 key を消す（`note-chapters の閾値の行が無い`）・同じ欄 key の行を 2 本（`2 本ある`）・値 12章以下（`行 R-2 の value「12章以下」が「<正の整数> 章 以下」の形でない`）・値 0 章 以下（`の形でない`）。どれも 2・まだ分からない・`設計ノートの章の上限が読めない`・面を書かない（既定の値に倒れない）。**base は 0 で面を書く＝RED。**
3. **f179_floor_counts_chapters_with_the_same_cap_and_words_as_the_face（`crates/folio/tests/note.rs`）。** 本流の置き場の写しの見本 example.yaml（節 6・図 1）に節 7〜12 と 2 枚目の図 fig-2 を足した 13 章（図の章は枚数に依らず 1 章）で、床は不合格 1・違反 1 件（種別 note・`design-note/example.yaml: 章が 13 本ある＝章が多すぎる（上限 12）`）。同じ写しで `folio face --face note --id example` は 2 で同じ字を出し、面を書かない。行の値を 13 章 以下にすると床は合格。図を 2 枚にするので、面か床の片方でも図を枚数で数えれば（章 14）字が割れて落ちる（(e) の 2 の V4・V6）。**base は床が合格（章を数えない）＝RED。**
4. **f179_floor_is_unknown_without_the_keyed_row（同）。** 写しの規則の表から欄 key を消すと、床は まだ分からない 2・違反 0・`設計ノートの章の上限が読めない`。**base は合格＝RED。**
5. **f179_rules_key_outside_the_closed_list_or_doubled_is_a_schema_violation（`crates/folio/tests/check.rs`）。** 本流の写しで行 R-19 の欄 key を note-chapter にすると `[schema] rules.yaml: 行 R-19 の key「note-chapter」が閉じた一覧` の違反・行 R-2 にも欄 key note-chapters を足すと `rules.yaml: 行 R-19 の key「note-chapters」を持つ閾値の行が 2 本以上ある`・開発規律の行 D-1 に欄 key を書くと行 D-1 の未知の欄。**base は欄 key を未知の欄として落とす（閉じた一覧の字が無い）＝RED。**
6. **単体の歯 3 本（`crates/folio/src/rules.rs` と `crates/folio/src/note.rs` の既存の tests の区間）。** f179_chapter_cap_reads_the_keyed_row_and_only_the_positive_integer_form（id R-19 と R-26 で 12 と 40・崩れた値 9 通り〔12章以下・12 章・12 章 以上・0・頭の 0・全角の数字・負・数の無い字・usize を超える数〕は行の id と「の形でない」の Err・欄 key の無い行と開発規律の行だけでは「行が無い」・2 本は「2 本ある」）・f179_key_is_a_closed_list_and_one_threshold_row_per_key（閉じた一覧の外と一覧の形の値は 1 件ずつ・2 本目は字が 1 つ・1 本だけなら空・KEYS は note-chapters の 1 つ）・f179_chapters_add_one_figure_chapter_and_over_cap_names_the_counts（chapters の 4 組と over_cap の 3 組）。**base は関数が無く組み立てが落ちる（E0425）＝RED。**
7. **RED の実測。** 歯だけの差分（binary の 5 本 + 面の fixture の行 + 行 R-19 の欄 key）を base に当てると 5 本とも落ちる（本文は起草の記録の red-bin.log・理由は上の各項のとおり＝面は 2 でなく 0、床は合格、欄 key は未知の欄）。単体の歯 3 本だけを当てると組み立てが落ちる（E0425 chapter_cap・chapters・over_cap・key_violations・KEYS）。
8. **「当たらないもの」の歯。** 12 章までの面が変わらないことは既存の凍結 fixture の歯（`face_note_write_matches_the_frozen_fixture`・full.yaml 7 章）が byte で見る（verify の 5）。

### (d) 採らなかった形

1. **行を id で引く（例 folio2 の R-19 を名指す）。** tsuzuri の R-19 は別の行（文の予算）で、同じ id を外の置き場で名指すには、床が id で名指す閉じた一覧（ADR-16 決定 (2)(オ)・`RESERVED`）を広げる新しい判断の記録が要り、しかも利用者の番号空間に folio2 の番号を予約することになる。欄 key なら一覧は今の R-8 と R-16 のまま。
2. **行が無い置き場では道具の既定 12 を使う。** 行 D-11（検査の結果を分ける数の閾値を実装の定数に置かない）と、ADR-16 決定 (6) の「欄が無ければ既定に倒れる形を置かない」の向きに反する。行が無いか崩れていれば まだ分からない にし、置き場の持ち主に値を決めさせる。
3. **行が無い置き場では上限を掛けない。** fixture の手直しは要らなくなるが、行を消す（または綴りを誤る）だけで上限の検査が黙って外れ、ADR-16 決定 (4) の物差し（検査される側の data を書き換えて落ちるはずの検査を通せるか）に掛かる。行 R-16 の散文の門（行が無ければ まだ分からない）と同じ型にする。
4. **章の上限を密度 profile のデータか設計ノートの欄の決まりに置く。** 部品目録と密度 profile は道具の側の 1 つの閉じた一覧で置き場ごとに持てず（ADR-16 決定 (2)(エ)）、欄の決まりの人の書く部分には裁定 id と時刻・値を緩める前の検証（P-17.1・P-17.4）が掛からない。持ち主が選んだのは規則の表の値。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **既存の歯（起草役の実測・改訂 a の見本 21b44b3 で撃ち直した）。** 本便の差分を base に当てた写しで、workspace の nextest **1011 / 1011**（1003 + f179_ の 8 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file。base の設計の変更だけの写しの組み立てと比べ、変わるのは `constitution.html` の行 R-19 の注の 1 文だけ（設計ノートの面 7 本を含むほかの 34 file は byte で同じ）。字の期待を直した既存の歯は (b) の 6・7 のとおりで、歯の中身（何を数えるか）は変えていない。直す前は nextest で落ちた（面の fixture と土台に欄 key の行が無い 71 本・生成区間の数と字 5 本・土台の索引の数 9 本）。
2. **突然変異（見本の写しの実装だけを 1 通りずつ変え、f179_ の 8 本を撃つ）。** 起草役の 10 通り（M1〜M10）と検証役の 2 通り（V4・V6）の 12 通りとも f179_ のどれかが落ちる（改訂 a の見本 21b44b3 で撃ち直した reva-mut.log・各変異の本文は reva-mut/mut-<名>.out）。V4・V6 は改訂 a の前の見本 dee6512 では f179_ も workspace の全部も落とさなかった（検証役の記録）。

| 変異 | 落ちる f179_ の本数 |
| --- | --- |
| M1 行を読まず 12 に固定 | 5 |
| M2 欄 key でなく id R-19 で引く | 4 |
| M3 図の章を数えない | 3 |
| M4 上限と同じ数も超えたと読む | 3 |
| M5 床が章を数えない | 1 |
| M6 同じ欄 key の 2 本目を字にしない | 2 |
| M7 行が無ければ 12 に倒れる | 3 |
| M8 0 と頭の 0 を許す | 2 |
| M9 面が上限を数えない | 2 |
| M10 7 章目から帯を繰り返さない | 1 |
| V4 床が図を枚数で数える（共有の関数 chapters を呼ばない） | 1 |
| V6 面が図を枚数で数える（共有の関数 chapters を呼ばない） | 1 |

3. **外の置き場（tsuzuri の写し・起草役の scratch）。** 本便の組み立てで、(1) 生成区間を直す前も後も床は まだ分からない 1（`設計ノートの章の上限が読めない: 欄 key が note-chapters の閾値の行が無い`）・`folio build` も同じ字の まだ分からない。(2) `folio schema --write` の後に tsuzuri の番号の空きで閾値の行 R-26（欄 key note-chapters・値 40 章 以下）を足し、tsuzuri の憲法 P-2 の relations.rules に R-26 を足すと、床は合格 0/0・`schema --check` 一致・`folio build` 21 file・天井の束 4 観点が組める。(3) 値を 30 にすると床は不合格 1（`design-note/surface-board.yaml: 章が 34 本ある＝章が多すぎる（上限 30）`）で、`folio build` は床の不合格で書かない。行と値と条の選び方と承認は tsuzuri の持ち主の手番。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** `crates/folio/src/face_note.rs` だけが縮む（頭に `-`）。ほかは増えるか同じ大きさ。差分は参考値 73,935 byte（`git diff 155e4d4 21b44b3 | wc -c`・23 file・+392 −68）＝審査の上限 120,000 byte の内なので割らない（行 ga は使わない）。
2. **余地（CapHeadroom）。** 各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、4 本とも一致した（lines-179.sh）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/rules.rs` | 316 | 1184 | 428 | 1072 |
| `crates/folio/src/check.rs` | 1053 | 447 | 1056 | 444 |
| `crates/folio/src/note.rs` | 942 | 558 | 977 | 523 |
| `crates/folio/src/face_note.rs` | 1037 | 463 | 1023 | 477 |

3. **size は M。** src の増分は 4 本で合わせて参考値 +136 行（S の見積 100 を超える）。4 本とも余地が M の見積 300 を超える（最小 447）。
4. **verify は 7 行**で、done の 7 つの塊と 1 対 1 に揃える。便の後の写しで 7 行とも rc 0（verify-impl.log）。base では 1〜4 が 0 件で rc 4、5〜7 は rc 0。
   1. `cargo nextest run -p folio --test face_note f179_` = (c) の 1・2（2 本）。
   2. `cargo nextest run -p folio --test note f179_` = (c) の 3・4（2 本）。
   3. `cargo nextest run -p folio --test check f179_` = (c) の 5（1 本）。
   4. `cargo nextest run -p folio --bin folio f179_` = (c) の 6（3 本）。
   5. `cargo nextest run -p folio --test face_note` = 設計ノートの面の歯の全部（凍結 fixture の byte 一致と章の上限の旧い歯を含む・参考値 33 本）。
   6. `cargo nextest run -p folio --test graph` = 土台の索引の凍結 anchor 3 本の歯を含む索引の歯の全部（参考値 16 本）。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file（face_note・note・check・graph）は 4 本とも write-set に在る。`--bin folio` の絞り込みの語 f179_ を関数名に持つ src は `crates/folio/src/rules.rs` と `crates/folio/src/note.rs` で、どちらも write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、本契約を commit した作業ツリー（設計の変更 3bb4482・155e4d4 を含む枝 docs/cap）で撃った（起草の記録）。共通の検証は同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/cap-draft.md`、差分と script は同じ dir の cap-scripts（repo には入れない）。

1. 設計の変更: design.py（既定 = 枝 docs/cap の形・`--final` = 本便の行 R-19 の 2 か所）。
2. 模擬: c179.patch を 3bb4482 に当て、その上に r179-reva.patch（改訂 a の歯の直し）を当てる（155e4d4 の上では見本 21b44b3 の中身）。fixture と anchor は fixtures.py・anchor.py・regen_face_anchors.sh・floor_base_anchors.py・tests_adjust.py で組み直せる。verify-179.sh（verify の 7 行）。
3. RED: red-179.sh（binary の歯 5 本と単体の歯 3 本を別々に当てる）。
4. 突然変異: mut-179.py（(e) の 2 の M1〜M10・最後に戻して組み直す）と、検証役の mut-verify.py（同じ dir の d179-verify-scripts・M1〜M10 と V4・V6 を名で選べる）。撃ち直しの逐次の手順は reva-179.sh（f179_・workspace の nextest・clippy・床 4 本）。
5. 外の置き場: tsuzuri-179.sh（t3v の写しを scratch に clone し、本流と本便の binary で撃つ）。
6. 余地と差分の byte: lines-179.sh（lines.py と lines.awk）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 行 R-19 の ruling の欄（席が設計の変更の側で書く）・規則の表のほかの行・要件書と憲法と判断の記録の字・台帳 f2-648.75 の残り 2 つ（相談窓口の質問の数の上限・版管理の照合の待ち時間＝道の外）・骨格の命令 folio init の雛形（外の置き場に章の上限の行を焼かない）・tsuzuri の行（tsuzuri の手番）・外部 crate・新しい dir。
2. **言えないこと。** (1) 設計ノートが 1 本も無い置き場では上限の行を読まない（無くても床は黙る）。(2) 値の形は「<正の整数> 章 以下」の 1 つだけで、全角の数字・桁区切り・単位の前後の空白の違いは まだ分からない（広げるなら値の欄の決まりを変える便）。(3) 行を欄 key で引くので、置き場が欄 key を別の行へ付け替えると、その行の値を読む（付け替えは規則の表の行の変更で裁定 id を要する・P-17.1）。(4) 上限の大きさに道具の上限は無い（帯は 6 つを繰り返す）。
3. **撤退条件。** (1) 本便が要件書 FR5 / FR9 / FR19 の字か憲法の条文を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の `face_note.rs` の derive・`note.rs` の check_note・`rules.rs` の床の木が base（155e4d4）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に (b) の 6・7 に挙げた外の既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が `constitution.html` の行 R-19 の注の 1 文のほかで 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `rules.rs` の欄 key・閉じた一覧・関数 keyed / chapter_cap / key_violations・床の木の 2 か所・単体の歯 2 本。`check.rs` の check_rules の 3 行。`note.rs` の関数 chapters / over_cap・check_note の上限の読みと数え・単体の歯 1 本。`face_note.rs` の上限の読みと帯の繰り返し（`MAX_CHAPTERS` と帯の表の後半を消す）。`design-intent/rules.yaml` の生成区間と行 R-19 の 2 か所。歯 f179_ の binary 5 本。fixture・凍結 anchor・字の期待（(b) の 6・7）。
- 入れない: 散文の門・`RESERVED`・導出物の命令・骨格の命令・要件書・憲法・判断の記録・行 R-19 の ruling・外の置き場の行・外部 crate・新しい dir・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| key | 行の欄 key | 閾値の行を置き場ごとの id でなく閉じた一覧の値で引く（rules.rs・生成区間） |
| cap | 章の上限の読み | rules.rs の chapter_cap（欄 key note-chapters の行の「<正の整数> 章 以下」） |
| count | 章の数え | note.rs の chapters と over_cap（面の生成器と床が同じ関数を呼ぶ） |
| floor | 床 | note.rs の check_note（上限の超過は違反・読めなければ まだ分からない）・check.rs の欄 key の違反 |
| face | 面の生成器 | face_note.rs の derive（同じ関数で数え、帯を 6 つで繰り返す） |
| teeth | 歯 | f179_ の binary 5 本と単体 3 本・fixture と凍結 anchor の組み直し |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い（歯の node-digest.py は既存の python3 の独立の script）。
- 前提の着地: 枝 docs/cap の設計の変更（行 R-19・憲法 P-2 の relations・歯 f91 の期待）と、その持ち主の承認（行の追加と変更・生成区間の規則の変更＝規則の表の行 D-17）。
- 本便の着地の後に席が見ること: 台帳の本便の件と f2-648.250 を閉じ、f2-648.75 の章の上限の分を記帳する。本流の `target/debug/folio` を組み直す。tsuzuri へ (e) の 3 と行の見本（起草の記録）を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fz"
title = "設計ノートの面の章の上限を置き場の規則の表の値にする（持ち主の裁定 2026-09-28 案 A・判断の記録 ADR-11 決定 (3)(エ)(4)⑦・台帳 f2-648.75 と f2-648.250）: 規則の表の閾値の行に任意の欄 key（閉じた一覧 note-chapters）を足し、crates/folio/src/rules.rs に関数 keyed と chapter_cap（欄 key が note-chapters の閾値の行がちょうど 1 本で値が 正の整数 章 以下 の形ならその数・無い 2 本 形が違うは Err）と key_violations（閉じた一覧の外と同じ値の 2 本目）を足して check.rs の check_rules が種別 schema の違反に出す。crates/folio/src/note.rs に関数 chapters（節の数に図が在れば 1）と over_cap を足し、床の check_note は設計ノートが在る置き場で chapter_cap を読み（読めなければ まだ分からない）設計ノートごとに超過を種別 note の違反にする。面の生成器 face_note.rs は MAX_CHAPTERS を消して同じ 3 つの関数で数え（読めなければ まだ分からない で面を書かない）、章の帯の表を 6 つにして 7 章目から繰り返す（12 章までの面は byte で同じ）。design-intent/rules.yaml は folio schema --write で生成区間を書き、行 R-19 に key: note-chapters を足して注の末尾の便 179 の着地までの 1 文を消す（ruling ほかは変えない）。面の fixture と床の土台に同じ欄 key の行を足し、凍結の面 3 本・土台の索引の凍結 anchor 3 本（独立の script と手で差し込んだ行で組み直した）・生成区間の凍結 anchor・床の case の添字と歯の数の字を直す。行の id では引かない（外の置き場で名指す行 R-8 と R-16 の一覧は変えない）。歯は f179_ の 8 本（face_note 2・note 2・check 1・binary の単体 3）。実装の見本は origin の枝 impl/d179 の commit 21b44b3（見本 db8c609 に枝 docs/cap の 155e4d4 を merge し、図 2 枚の歯の直しを積んだもの）で、作業者は write-set の file をその中身にしてよく（rules.yaml は行 R-19 の ruling が違えば §1 (b) の 5 の 3 か所だけを当てる）、write-set の外は変えない。base = 枝 docs/cap の設計の変更 3bb4482 と 155e4d4 が本流に着地した後の main"
req = ["FR9", "FR5", "FR19"]
section = "1"
write-set = ["crates/folio/src/rules.rs", "crates/folio/src/check.rs", "crates/folio/src/note.rs", "-crates/folio/src/face_note.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/note.rs", "crates/folio/tests/check.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/hello.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/unknown_fields.rs", "design-intent/rules.yaml", "tests/fixtures/face/rules.yaml", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/floor_base/design-intent/rules.yaml", "tests/fixtures/floor_base/design-intent/constitution.yaml", "tests/fixtures/schema/rules-region.txt", "tests/fixtures/schema/graph-anchor.txt", "tests/fixtures/schema/graph-digest-anchor.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/floor_cases.yaml"]
verify = ["cargo nextest run -p folio --test face_note f179_", "cargo nextest run -p folio --test note f179_", "cargo nextest run -p folio --test check f179_", "cargo nextest run -p folio --bin folio f179_", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test graph", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "tests/face_note.rs の f179_ の 2 本（14 章の設計ノートが欄 key の行の値 12 では 2 で 章が 14 本ある＝章が多すぎる（上限 12） を出して面を書かず、同じ行の id を R-26・値を 14 にすると面を書いて 7 章目から帯の組を繰り返す／欄 key が無い・2 本・値が 12章以下 か 0 章 以下 の 4 通りは 2 と まだ分からない と 設計ノートの章の上限が読めない で面を書かない）が緑、tests/note.rs の f179_ の 2 本（図 2 枚の 13 章の見本で床が不合格 1・違反は種別 note の 章が 13 本ある＝章が多すぎる（上限 12）で、面の生成器が同じ字で止まり、値を 13 にすると合格／欄 key を消すと まだ分からない）が緑、tests/check.rs の f179_ の 1 本（欄 key の閉じた一覧の外・同じ値の 2 本目は種別 schema の違反・開発規律の行の欄 key は未知の欄）が緑、binary の単体の f179_ の 3 本（chapter_cap の正の整数の形と行の id に依らないこと・key_violations・chapters と over_cap）が緑、tests/face_note.rs の歯の全部（凍結 fixture の byte 一致を含む）が緑、tests/graph.rs の歯の全部（土台の索引の凍結 anchor 3 本を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
