# 設計: 便 138 — 要件書の面の頭と入口の棚のカードに、効いている版（effective_version）を出す（版の欄が先に進んでいれば 起草・承認待ち の札）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は要件書の面の頭の 3 か所（鮮度の札・表紙の状態・承認欄のリード）と入口の棚の要件書のカードの 1 か所の字を、正本の欄 effective_version（効いている版）に合わせて直すだけで、FR4 の規範文も面の部品・色・並びも変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する＝正本に在る欄 effective_version を面が黙って読み落とさない）/ P-4.2（判定できないものは まだ分からない として表に出す＝状態が発効なのに効いている版の欄が無い正本は、面の頭に「効く版はまだ分からない」と出す）/ P-5.1（名札は型付きの定数で持つ）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-6.2（生成物を手で直さない＝面の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 天井の 34 周目（2026-09-25・一括 20 の枝の上）の読みやすさの所見 **F-2**（重さ 直す・場所 要件書の meta.status_note）。一括 20 の仕分け（`docs/design/batch20-triage.md`）の便の候補 **B-6**。台帳の控え **f2-648.210**。同じ形は 29 周目の読みやすさ F-4（一括 17 の仕分けは「承認で消える・便を起こす利益が無い」として便にしなかった）と、一括 15 の起草中（`docs/design/srs-vB.md` のまだ分からない点 1）にも出ており、版を上げる枝の上で周を回すたびに立つ。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ek` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 11 本（書き換える 7 本 + 本文が変わらない verify の scope 4 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-25・独立の検証 d138-verify.md〔条件付き支持・blocking 0・数値は全部一致〕の文面 3 点と任意の提案 3 点）: (b) の 1 の表の外の Err が入口の面では新しい振る舞いであることの字・(b) の 1 に `face_srs.rs` に足してよい行数（base から 41 行まで）の 1 文・done の末尾と (i) の 3 の撤退条件 (2) の比べる相手を着地の直前の main に。任意の提案から、(b) の 2 に status_note が無いときのリードの字、(b) の 3 に draft のカードの字、(i) の 2 に札とカードの版の並びが逆なことの予告、§5 に外部の利用者が発効になったときの Unknown の控えを、どれも字だけで足した。歯・verify・write-set・size・数値は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 11 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提の着地は無い。**base = main f82dba1。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: base の時点で、便 135（行 eh）・136（行 ei）・137（行 ej）は main に着地済みで、契約の枝のうち main に着地していない便は無い（起草役の実測）。一括 20 の便の候補 B-5（台帳 f2-648.208・直す先は `crates/folio/src/floor_note.rs` の見込み）は本便の write-set と重ならない。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。

## 1. 設計

### (a) いま起きていること（実測・base main f82dba1・数は参考値）

1. **正本の欄。** 要件書の正本 `design-intent/srs.yaml` の欄 meta は、版の欄 version・状態 status・効いている版 effective_version を持つ。版を上げる一括の枝では、席が version を次の版へ上げ、effective_version は承認まで前の版のまま置き、承認の後に上げる（各一括の仕分けの「承認の後に席が書く欄」の 1 行目・例 一括 20 の仕分けの effective_version v1.41 → v1.42）。status は枝の上でも effective のまま（前の版が効いているため）。base の本流では version = effective_version = v1.42。
2. **面は effective_version を脚でしか読まない。** 要件書の面の生成器 `crates/folio/src/face_srs.rs` は、effective_version を機械のための面（脚の dl）に字で出すだけで（関数 foot）、頭の 3 か所は version と status だけから組む。
   - 鮮度の札（部品 freshness-stamp・関数 head が共有の口 Frame::head に渡す版と名札）: 版 = version、名札 = status の表引き（effective なら 発効・拘束力あり）。
   - 表紙の状態（関数 cover の cover-status）: status が effective なら「発効・拘束力あり（承認 <最後の承認の行の日付>）」。
   - 承認欄のリード（関数 approval）: 「<status の名札> — <status_note の最初の句点までの要旨>」。
3. **入口の棚のカード。** 入口の面の要件書のカードの更新の行は、`crates/folio/src/face_index_read.rs` の関数 srs_card が「<generated>・<version>」と出す（関数 updated・憲法のカードと共有）。発効かどうかの字は出さないが、版は version だけを出す。依頼が名指した `crates/folio/src/face_index.rs` の頭（関数 head）・棚の figcaption・脚は、入口の正本 index.yaml 自身の version と status（名札 発効）を出す所で、要件書の版は出さない。
4. **枝の上の実測（一括 20 の枝の commit 89be3a3・version v1.42・effective_version v1.41・status effective）。** base の binary で組んだ面の字:

| 所 | base の字（枝の上） | 読める意味 |
| --- | --- | --- |
| 鮮度の札 | 生成 2026-09-12 · v1.42（発効・拘束力あり） | 起草中の v1.42 が発効と読める |
| 表紙の状態 | 発効・拘束力あり（承認 2026-09-25） | 日付は v1.41 の承認の行だが、版を名指さない |
| 承認欄のリード | 発効・拘束力あり — いま発効しているのは 第 1.41 版（2026-09-25）です。 | v1.41 が発効と読める（頭と逆） |
| 入口の棚のカード | 更新 2026-09-12・v1.42 | 承認待ちの札が無い |
| 題・表紙の版・図の caption・脚 | 要件書（v1.42）・v1.42 / 2026-09-12 ほか | 面の中身の版（正しい） |

   status_note の 2 文目「第 1.42 版（一括 20）は起草・承認待ちです。」は、要旨（最初の句点まで）の外なので、見出し 版ごとの来歴 の折りたたみの中に入る（所見 F-2 の証拠の字）。
5. **憲法の面と入口の面の頭も同じ形。** 憲法の面（`crates/folio/src/face_constitution.rs`）の鮮度の札は「v1.4（発効・拘束力あり）」、表紙は「発効・拘束力あり（承認 2026-09-12）」で、どちらも version と status だけから組む。入口の面の鮮度の札は「v0.5（発効）」。ただし憲法の正本 constitution.yaml と入口の正本 index.yaml の欄 meta には、効いている版を持つ欄が無い（base の `grep` で effective_version を持つ正本は要件書だけ・床の fixture を除く）。
6. **base の歯（参考値）。** workspace の nextest 889 / 889・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file。`git grep -n 'f138_' -- crates` は 0 件。歯の file の本数は face_srs 16・face_index 20・face_constitution 17・face_srs_body 18・face_srs_figure 10・badge 16・site 13。

### (b) 直す先 — 版の立場を 1 つの口で決め、要件書の面の頭 3 か所と入口のカード 1 か所で使う

1. **口（`crates/folio/src/face_labels.rs` に足す・`face.rs` が丸ごと再輸出する）。**
   - 名札の定数 2 つ: PENDING = 起草・承認待ち／UNKNOWN_EFFECTIVE = 効く版はまだ分からない。
   - 版の立場の型 Standing（4 値）と関数 standing（欄 meta を受ける）: status を文書の状態の表（既存の DOC_STATUS）で引き、表の外は Err（導出できない）。要件書の面は今も同じ理由で導出しない。入口の面は base では要件書の status も effective_version も読まず、表の外の status（例 retired）や一覧の effective_version でも終了コード 0 で書くが（検証役の実測）、本便の後は同じ理由で終了コード 2 になる。`folio build` は今も要件書の面で同じ理由で止まるので、組み立ての結果は変わらない。status が draft なら Draft（effective_version は読まない）。effective なら、effective_version が無ければ Unknown、version と字が同じなら Effective、違えば Pending（効いている版の escape した字を持つ）。新しいか古いかは比べない（版の字の順序の規則を面に持たない）。effective_version が文字列でない（一覧など）なら Err。
   - 置き場を `face_srs.rs` にしない理由: `face_srs.rs` は歯 f100（`crates/folio/tests/face_srs.rs` の f100_face_srs_is_split_and_under_the_cap）が幅 120 正規化で 1,100 行の上限を持ち、base で 1,059 行（余地 41）である。起草役が口を `face_srs.rs` に置いた模擬は 1,112 行で f100 が落ちた。入口の面も同じ口を読むので、面に共有の名札の置き場（`face_labels.rs`）に置く。**`face_srs.rs` に足してよいのは base から 41 行まで（歯 f100 の上限 1,100 − base 1,059）である。** 起草役と検証役の別々の実装の増分は +16 と +14 だった。札の字の組み立てが長くなるなら `face_labels.rs` に関数を置き、`face_srs.rs` は 3 か所で呼ぶだけにする。
2. **要件書の面（`face_srs.rs`）の 3 か所。** 字は版の立場ごとに次のとおり。Effective と Draft は base と 1 byte も変えない。

| 所 | Effective・Draft | Pending（version v1.42・効いている版 v1.41） | Unknown（version v0.3・効いている版の欄なし） |
| --- | --- | --- | --- |
| 鮮度の札 | base のまま（v1.42（発効・拘束力あり）） | v1.41（発効・拘束力あり・v1.42 は起草・承認待ち） | v0.3（効く版はまだ分からない） |
| 表紙の状態 | base のまま | v1.41 が発効・拘束力あり（承認 2026-09-25）・v1.42 は起草・承認待ち | 発効・拘束力あり（承認 <日付>）・効く版はまだ分からない |
| 承認欄のリード | base のまま | 発効・拘束力あり（v1.42 は起草・承認待ち） — <要旨> | 発効・拘束力あり（効く版はまだ分からない） — <要旨> |

   status_note が無い正本の承認欄のリードは、今と同じく名札だけで、Pending と Unknown では名札に札が付いた字だけになる（例 発効・拘束力あり（v1.42 は起草・承認待ち））。鮮度の札は共有の口 Frame::head に渡す版と名札の 2 つの引数だけを変える（`face.rs` は変えない）。表紙の状態の承認の日付は今の式（最後の承認の行の日付）のまま。枝の上では承認の行はまだ前の版の分しか無いので、効いている版の承認の日付になる。
3. **入口の棚のカード（`face_index_read.rs` の srs_card）。** 更新の行の後ろに、Pending なら「（起草・承認待ち・発効は <効いている版>）」、Unknown なら「（効く版はまだ分からない）」を足す。Effective と Draft は base のまま（draft の写し・version v0.4・効いている版 v0.3 でも 更新 <generated>・v0.4 で札なし・口 standing が Draft を返すため）。枝の上の実測では「更新 2026-09-12・v1.42（起草・承認待ち・発効は v1.41）」。憲法のカードは変えない（(a) の 5）。
4. **変えないもの。** 題（要件書（<version>））・表紙の版の札・図の caption・脚の平文と機械のための面（version と effective_version の字）・部品と class・章と節の並び・承認欄の行・来歴の折りたたみ・共有の口 Frame・憲法の面・入口の面の頭と棚の figcaption と脚・判断の記録と設計ノートの面・床・設計文書。題と表紙の版の札と脚は面の中身がどの版の正本から組まれたかを示す字で、枝の上では起草中の版の中身が載っているので version のままが正しい。
5. **契約で決めること（起草役の判断）。**
   - **効いている版の欄が無い正本（P-4.2）: 面は書き、頭に「効く版はまだ分からない」と出す。** 面を導出しない（終了コード 2）形は採らない。効いている版の欄は床が要る欄と定めておらず、版の立場は面の中身の 1 点にすぎないので、1 点のために面の全部と `folio build` を止めると持ち主が面を読めなくなる。黙って version を発効と出す今の形（P-4.2 に反する）も採らない。状態 draft の正本（`folio init` の雛形・外部の利用者 tsuzuri の要件書は draft で effective_version を持たない）は Draft で、base と字が変わらない。
   - **面の fixture の要件書 `tests/fixtures/face/srs.yaml` に effective_version v0.3 を 1 行足す。** 写しは version v0.3・status effective で effective_version を持たないので、足さないと凍結の面が Unknown の字になり、実の要件書の形（効いている版を持つ）と離れる。足すと凍結の面 `tests/fixtures/face/expected-srs.html` は脚の機械のための面の 1 行（effective_version v0.3 の dt と dd が加わる）だけが変わる。Unknown の字は歯が写しから欄を消して確かめる。
   - **憲法の面と入口の面の頭は範囲に入れない（断る）。** 2 つの正本は効いている版の欄を持たず（(a) の 5）、直すには設計文書の正本の欄を足す一括（門の対象）が先に要る。本便の範囲を広げると size が変わり、設計文書の承認を前置にする。台帳の控えに同じ形の残りとして書く（§5）。
   - **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便の数え）: 当たらない。** 撤退条件 ② が数えるのは見た目だけを直す便で、本便は部品・色・余白・並び・名札の class を 1 つも変えない。直すのは、正本に在る欄（effective_version）を面の頭が読まずにどの版が効いているかを誤って出していた中身の正しさ（P-6.1・P-4.2）である。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137 の契約（撤退条件 ② に当たらないとした読み）と同じである。要件書の面の見た目の直しの数え（上限 2 に到達済み）には足さない。持ち主の walk（面の承認）は章と部品を変えないので要しないと起草役は読む（求めるかは席が決める）。

### (c) 歯（関数名 f138_・base で 0 件）

1. **f138_standing_reads_status_version_and_effective_version（単体の歯・`crates/folio/src/face_labels.rs` に足す tests の区間）。** 2 つの名札の字が (b) の 1 の字と等しいこと（凍結の針・歯の中の手で写した字）、版の立場の 4 通り（同じ字 = Effective・違う字 = Pending で効いている版を持つ・欄なし = Unknown・draft は effective_version が違っても Draft）、効いている版の字を escape すること（山括弧）、表の外の状態 retired の Err の字、effective_version が一覧なら Err を確かめる。**base では型と定数が無く、組み立てが通らない＝RED。**
2. **f138_pending_version_names_the_effective_and_the_draft（`crates/folio/tests/face_srs.rs`・binary 経由）。** 面の fixture の写しの version を v0.4 にした面（効いている版 v0.3）で、終了コード 0・鮮度の札・表紙の状態・承認欄のリードの字が (b) の 2 の Pending の列の形でちょうど 1 つずつ在り、題は要件書（v0.4）・機械のための面は version v0.4 と effective_version v0.3・起草中の v0.4 を発効と出す字と版を名指さない表紙の発効の字が無い。**base では頭が v0.4 を発効と出す＝RED。**
3. **f138_equal_versions_keep_the_effective_labels（同）。** 変異なしの写し（version = effective_version = v0.3）の面で、鮮度の札が v0.3（発効・拘束力あり）でちょうど 1 つ・2 つの名札の字がどこにも無い・機械のための面に effective_version v0.3・面全体が凍結の面と一致。守りの歯（base の生成器でも緑・直しが Effective の字を崩したら落ちる）。
4. **f138_missing_effective_version_is_unknown（同）。** 写しから effective_version の行を消した面で、終了コード 0・3 か所が (b) の 2 の Unknown の列の字でちょうど 1 つずつ・機械のための面に effective_version が無い・鮮度の札が発効・拘束力ありと出さない。**base では v0.3 を発効と出す＝RED。**
5. **f138_draft_does_not_read_effective_version（同）。** 写しを status draft・version v0.4（効いている版 v0.3 は残す）にした面で、鮮度の札が v0.4（未承認・拘束力なし）・表紙が未承認のため拘束力なし → 持ち主の承認で発効・2 つの名札の字が無い。守りの歯（base でも緑・直しが draft で効いている版を読んだら落ちる）。
6. **f138_real_srs_head_follows_the_meta（同・実の正本）。** 実の要件書を yaml-rust2 で直に読み、version と effective_version が同じなら鮮度の札が <version>（発効・拘束力あり）、違えば Pending の形、欄が無ければ Unknown の形でちょうど 1 つ在り、起草・承認待ちの字の有無が 2 つの欄の差と一致すること。守りの歯（base の本流は同じ字で緑・枝の上で撃てば Pending の形を確かめる）。
7. **f138_srs_card_marks_a_pending_version（`crates/folio/tests/face_index.rs`・binary 経由）。** 入口の写しの要件書の version を v0.4 にした入口の面で、カードの更新の行が 更新 2026-09-01・v0.4（起草・承認待ち・発効は v0.3）でちょうど 1 つ。**base では札が無い＝RED。**
8. **f138_srs_card_marks_an_unknown_effective_version（同）。** 写しから effective_version の行を消した入口の面で、更新の行が 更新 2026-09-01・v0.3（効く版はまだ分からない）でちょうど 1 つ・変異なしの写しでは 更新 2026-09-01・v0.3 のまま（札なし）でちょうど 1 つ。**base では札が無い＝RED。**

fixture は新しい file を足さない。面の fixture の要件書に effective_version の 1 行を足し（(b) の 5）、変異は歯の中で写しを書き換える（歯の file の既存の口 fixture_srs と index_fixture_copy・edit と同じ形）。

### (d) 採らなかった形

1. **effective_version が無い正本は面を導出しない（終了コード 2）。** (b) の 5 のとおり、版の立場の 1 点のために面の全部と組み立てを止めることになる。まだ分からないの字を頭に出せば P-4.2 を満たす。
2. **効いている版を承認欄の最後の承認の行の version から読む。** 承認の行と effective_version の 2 つの面が同じ事実を持つことになり（P-6.3）、欄を読まずに別の欄から推すのは P-6.4 の 2 面一致の形に近づく。承認の行の version は任意の欄で、第 1.9 版の作成の行のように持たない行もある。
3. **版の字を数として比べ、新しいときだけ Pending にする。** v1.9 と v1.10 のような字の順序の規則を面に持つことになる。枝の上の形は効いている版が前の版で字が違うことだけなので、字が違えば Pending で足りる。効いている版のほうが新しい字の食い違いは面では判定しない（(i) の 2）。
4. **status_note の要旨の切り方を変え、2 文目の起草・承認待ちもリードに出す。** 所見が挙げた 2 つ目の案。要旨の切り方は便 81 の決め（最初の句点まで）で、正本の文の並びに依る。欄から決める札のほうが、status_note の書き方に依らずに揃う。
5. **憲法の面と入口の面の頭まで直す。** (b) の 5 のとおり、正本に効いている版の欄が無い。
6. **題（title）も効いている版にする。** 枝の上の面の中身は起草中の版なので、題が前の版を名乗ると中身と食い違う。題は中身の版、頭の札は効いている版と役を分ける。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **落ちる既存の歯は 8 本で、どれも本便の fixture の 1 行と凍結の面の 1 行で直る（起草役の実測）。** 実装だけを base に当てた写し（歯の file と fixture と凍結の面は base のまま）で、workspace の nextest は 889 本のうち 8 本が落ちる。面の fixture の要件書が effective_version を持たないので、写しの面が Unknown の字になり、凍結の面と byte が違うためである。

| 歯 | 直し方 |
| --- | --- |
| face_srs_body の face_srs_write_matches_the_frozen_fixture | fixture の 1 行と凍結の面の 1 行（本文は変えない） |
| face_srs_figure の face_srs_figure_chapter_is_the_only_difference_from_the_figureless_face | 同上 |
| face_srs の f118_handwritten_scope_m3_adds_one_escaped_block・f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required | 同上 |
| face_index の face_index_write_matches_the_frozen_fixture・face_index_write_with_a_sheet_matches_the_frozen_fixture | fixture の 1 行（カードが Effective に戻る・入口の凍結の面は変えない） |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures | fixture の 1 行と凍結の面の 1 行 |
| site の site_write_matches_the_frozen_fixture | 同上 |

   実装に fixture の 1 行と凍結の面の 1 行を足した写し（歯を除く）で 889 / 889。本便の差分の全部を base に当てた写しで、workspace の nextest は 897 / 897（base 889 + 単体 1 + face_srs 5 + face_index 2）・clippy 0 警告・床 4 本 rc 0・face_srs 21 / 21・face_index 22 / 22・face_srs_body 18 / 18・face_srs_figure 10 / 10・badge 16 / 16・site 13 / 13。歯 f100 の `face_srs.rs` の数は 1,075 行（上限 1,100）。
2. **動く凍結 anchor は 1 本で、1 か所だけ。** `tests/fixtures/face/expected-srs.html` の脚の機械のための面の 1 行に、effective_version v0.3 の dt と dd が status の後に加わる（起草役は手で書き換え、生成器の出力と byte で一致した）。fixture と凍結の面の 2 行を base の生成器に当てても緑（base の脚は effective_version を在れば出すため）。ほかの面の凍結 anchor（入口 2 本・憲法・判断の記録 2 本・設計ノート）・床の凍結の土台・天井の束の凍結 anchor は 1 byte も変えない。
3. **RED（起草役の実測）。** 歯と fixture と凍結の面だけを base に当てた写しで、binary の f138_ の 7 本のうち 4 本（(c) の 2・4・7・8）が落ち、3 本（(c) の 3・5・6 の守りの歯）は緑。単体の歯は組み立てが通らない。
4. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変える・f138_ の 8 本を撃つ）。** 7 通りとも 1 本以上が落ちる。

| 変異 | 落ちる f138_ |
| --- | --- |
| M1 鮮度の札を version と status の名札に戻す | (c) の 2 |
| M2 表紙の状態で版を名指さない | (c) の 2 |
| M3 承認欄のリードに札を添えない | (c) の 2 |
| M4 入口のカードに札を添えない | (c) の 7 |
| M5 draft でも effective_version を読む | (c) の 1・5 |
| M6 欄が無いとき Effective と読む | (c) の 1・4・8 |
| M7 効いている版を escape しない | (c) の 1 |

5. **folio2 自身の面の変化（起草役の実測）。** base の本流（version = effective_version = v1.42）では、`folio build --dir design-intent --out <置き場> --write` の出力は 30 file のまま、base と 1 byte も変わらない（`diff -r` で差 0）。一括 20 の枝の commit 89be3a3 の設計文書で組むと 30 file のうち 2 file だけが変わる＝要件書の面の 3 行（鮮度の札・表紙の状態・承認欄のリード）と入口の面の 1 行（カードの更新の行）で、字は (b) の 2 と 3 の Pending の列のとおり。天井の束の面の要約値は、次に版を上げる枝の上の周でだけ変わるが、面は周の引き金ではない（引き金は設計文書の正本の規範の欄）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 11 本とも印なし（書き換える 7 本 = `crates/folio/src/face_labels.rs`・`crates/folio/src/face_srs.rs`・`crates/folio/src/face_index_read.rs`・`crates/folio/tests/face_srs.rs`・`crates/folio/tests/face_index.rs`・`tests/fixtures/face/srs.yaml`・`tests/fixtures/face/expected-srs.html`／本文不変の 4 本 = `crates/folio/tests/face_srs_body.rs`・`crates/folio/tests/face_srs_figure.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 3 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。空行を 0 と数える式では face_srs.rs は 989 行と出るが、器と歯 f100 の式（空行は 1）に揃える。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_srs.rs` | 1,059 | 441 | 1,075（+16） | 425 |
| `crates/folio/src/face_index_read.rs` | 622 | 878 | 629（+7） | 871 |
| `crates/folio/src/face_labels.rs` | 255 | 1,245 | 318（+63） | 1,182 |

   余地はどれも S の見積 100 を超える。ただし `face_srs.rs` は歯 f100 の上限 1,100 が先に効き、便の後に残るのは 25 行である（(b) の 1）。歯の file と fixture は src の外なので余地を測らない（参考値 face_srs 730 → 827・face_index 810 → 843・srs.yaml 124 → 125・expected-srs.html 442 → 442）。
3. **size は S。**
4. **verify は 10 行**で、done の 10 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f138_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_srs f138_` = (c) の 2〜6（5 本）。
   3. `cargo nextest run -p folio --test face_index f138_` = (c) の 7・8（2 本）。
   4. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（歯 f100 の上限・凍結の面を読む f118 を含む・参考値 21 本）。
   5. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（入口の凍結の面 2 本との byte 一致を含む・参考値 22 本）。
   6. `cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture` = 凍結の面との byte 一致。
   7. `cargo nextest run -p folio --test face_srs_figure` = 図の章の差が凍結の面との唯一の差であることを含む図の歯の全部（参考値 10 本）。
   8. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   9. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   10. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_srs・face_index・face_srs_body・face_srs_figure・badge・site の 6 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f138_ を関数名に持つ src は `crates/folio/src/face_labels.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d138-draft.md`、模擬の差分と script は同じ dir の d138-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-138.py（実装と fixture と凍結の面）と apply-138-teeth.py（歯）を撃つ。その差分が c138.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -r`・verify の 10 行を撃つ。
2. RED: apply-138-teeth.py と fixture と凍結の面だけを base に当てると (e) の 3 のとおり。実装だけを当てると (e) の 1 の 8 本が落ちる。
3. 突然変異: mut-138.sh（(e) の 4）。
4. 枝の上の面: 一括 20 の枝の commit 89be3a3 の design-intent と contracts を写しの外に展開し、base と模擬の binary の `folio build` の出力を比べる（(e) の 5）。
5. 余地: lines-138.py と lines-138.awk（同じ式の 2 実装）を repo の根で write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 憲法の面と入口の面の頭の同じ形（(b) の 5・正本の欄が先に要る）。入口の棚の憲法のカード。status_note の要旨の切り方（(d) の 4）。題・表紙の版の札・脚。設計文書の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 効いている版の字が本当に承認された版かは面も床も確かめない（承認欄の行との突き合わせは持たない・(d) の 2）。効いている版が version より新しい字の食い違いも Pending と出す（(d) の 3）。鮮度の札は効いている版を先に（v1.41（…・v1.42 は起草・承認待ち））、入口のカードは版の欄を先に（v1.42（起草・承認待ち・発効は v1.41））出し、並びが逆である。カードの更新の行はその file の最新の版を示す行なので版の欄を先に置いたが、次の枝の上の周で読みやすさの観点が拾う見込みがある。枝の上の次の周で、読みやすさの観点が F-2 を解けたと読むかは周の結果でしか分からない。面の見た目が持ち主に受け入れられるかは walk でしか分からない。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 8 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と file 数か byte で 1 つでも違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、要件書の正本の effective_version の置き方（version と別の欄で効いている版を持つ）か、歯 f100 の `face_srs.rs` の上限 1,100 が base と違えば、(b) と (f) を数え直してから運ぶ（欄が無くなっていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の名札 2 つ・型 Standing・関数 standing・単体の歯 1 本。`crates/folio/src/face_srs.rs` の鮮度の札・表紙の状態・承認欄のリードの 3 か所。`crates/folio/src/face_index_read.rs` の要件書のカードの更新の行。`crates/folio/tests/face_srs.rs` の歯 5 本と `crates/folio/tests/face_index.rs` の歯 2 本。面の fixture の要件書の 1 行と凍結の面の 1 行。
- 入れない: 憲法の面・入口の面の頭と棚の figcaption と脚・憲法のカード・題と表紙の版の札と脚・部品目録・名札の class・共有の口 Frame（`face.rs`）・判断の記録と設計ノートの面・床・設計文書・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| standing | 版の立場の口 | `crates/folio/src/face_labels.rs` の PENDING・UNKNOWN_EFFECTIVE・Standing・standing |
| head | 要件書の面の頭 | 鮮度の札・表紙の状態・承認欄のリード（`crates/folio/src/face_srs.rs`） |
| card | 入口の棚の要件書のカード | 更新の行の札（`crates/folio/src/face_index_read.rs`） |
| anchor | 面の fixture と凍結の面 | `tests/fixtures/face/srs.yaml` の 1 行・`tests/fixtures/face/expected-srs.html` の 1 行 |
| teeth | 歯 | 単体の f138_ 1 本・face_srs の f138_ 5 本・face_index の f138_ 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main f82dba1）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.210）を閉じる。憲法の面と入口の面の頭の同じ形（効いている版の欄が正本に無い）を台帳の控えとして残すか、次の一括で正本の欄を足すかを決める（§1 (b) の 5）。次に版を上げる枝の上の周で、読みやすさの F-2 と同じ所見が立たないかを見る。外部の利用者の要件書（今は draft で effective_version を持たない）が承認で effective になると面の頭は 効く版はまだ分からない になるが、effective_version はどの欄の決まりにも `folio init` の雛形にも載っていないので、欄の置き場（雛形か欄の決まり）を台帳の控えに残すかを決める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ek"
title = "一括 20 の B-6（天井の 34 周目 読みやすさ F-2・台帳 f2-648.210）: 要件書の面の頭が、効いている版 meta.effective_version でなく版の欄 meta.version を発効・拘束力ありと出し、版を上げる枝の上で承認欄のリードと食い違う問題を直す。版の立場の口（crates/folio/src/face_labels.rs の Standing・standing・名札 PENDING = 起草・承認待ち・UNKNOWN_EFFECTIVE = 効く版はまだ分からない）を足し、status が draft なら今のまま、effective で effective_version が version と同じなら今のまま、違えば鮮度の札を 効いている版（発効・拘束力あり・<version> は起草・承認待ち）・表紙の状態を <効いている版> が発効・拘束力あり（承認 <日付>）・<version> は起草・承認待ち・承認欄のリードの名札に（<version> は起草・承認待ち）を添え、effective_version が無ければ 3 か所に 効く版はまだ分からない を出す（面は書く・P-4.2）。入口の棚の要件書のカードの更新の行にも同じ札を添える（crates/folio/src/face_index_read.rs）。題・表紙の版の札・脚・部品・class・憲法の面と入口の面の頭は変えない。面の fixture の要件書に effective_version v0.3 の 1 行を足し、凍結の面 expected-srs.html の脚の 1 行を手で直す。歯は単体の f138_ 1 本と face_srs の f138_ 5 本と face_index の f138_ 2 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_labels.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_index_read.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_index.rs", "tests/fixtures/face/srs.yaml", "tests/fixtures/face/expected-srs.html", "crates/folio/tests/face_srs_body.rs", "crates/folio/tests/face_srs_figure.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --bin folio f138_", "cargo nextest run -p folio --test face_srs f138_", "cargo nextest run -p folio --test face_index f138_", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face_srs_body face_srs_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test face_srs_figure", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f138_（2 つの名札の字が §1 (b) の 1 のとおりで、版の立場が 同じ字 = Effective・違う字 = Pending・欄なし = Unknown・draft = Draft の 4 通りに決まり、効いている版を escape し、表の外の状態と一覧の effective_version が Err）が緑、face_srs の f138_ の 5 本（version を先に進めた写しで鮮度の札・表紙の状態・承認欄のリードが効いている版と 起草・承認待ち を名指し題と脚は version のまま・版が揃った写しは今の字で凍結の面と一致・effective_version の無い写しは 3 か所が 効く版はまだ分からない で面を書く・draft は effective_version を読まない・実の要件書の鮮度の札が meta の 2 つの欄に従う）が緑、face_index の f138_ の 2 本（入口の棚の要件書のカードの更新の行が Pending なら 起草・承認待ち・発効は <効いている版>、欄なしなら 効く版はまだ分からない、揃っていれば札なし）が緑、face_srs の歯の全部（歯 f100 の上限 1100 を含む）が緑、face_index の歯の全部（入口の凍結の面 2 本との byte 一致を含む）が緑、face_srs_body の凍結の面 expected-srs.html との byte 一致が緑、face_srs_figure の歯の全部が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数も byte も変わらない"
<!-- contracts:end -->
