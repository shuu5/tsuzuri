# 設計: 便 147 — 入口の棚のカードの「更新 <日付>」を、その型の文書の面の鮮度の札の日付の最大に揃える

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は入口の面の棚の 4 枚のカード（憲法・要件書・設計ノート・判断の記録）の 2 行目の 更新 <日付> を、それぞれの文書の面が鮮度の札に出す日付から 1 つの口で組むように直すだけで、FR4 の規範文も面の部品・class・色・並びも様式の定義も変えない。判断の記録と設計ノートの面の生成器（FR16・FR9）は日付の口を共有の口へ移すだけで、出力は 1 byte も変わらない（(e) の 4）。契約表の行の req は FR4 の 1 つ（main に在る id）。
- 条: P-6.3（同じ内容を 2 つの面が持つときは一方を正本とする＝入口のカードの日付は、その文書の面の鮮度の札と同じ口から出す）/ P-6.1（人が読むページは正本から逐語で生成する）/ P-2.1（生成器は 1 つ＝更新 の定めは共有の口 1 か所に置く）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-10.1（期待の字は歯の側の手書きで持つ）。
- 出所: 便 146（台帳 f2-648.219・行 es）の契約 §1 (i) の 1 と、起草役の問い 7（席の裁定 = 控え）。台帳の控え **f2-648.220**。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `et` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 10 本（書き換える 6 本 + 本文が変わらない verify の scope 4 本）。新しい file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 10 本を base の binary（写しの組み立て）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便を当てた写しの binary でも同じ。
- 前の便: 前提の着地は無い。**base = main 13af548（便 146 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草役は `git archive 13af548` の写しを組み立てて測った（本流の作業ツリーの binary は使っていない）。
- 並行の便との重なり: base の時点で、契約の枝のうち main に着地していない便は無い（便 146 は着地済み・起草役の実測）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main 13af548・数は参考値）

1. **4 枚のカードの 更新 の日付の出どころが型ごとに違い、定めが無い。**

| カード | 字を組む所 | base の日付の出どころ | 同じ文書の面の鮮度の札 |
| --- | --- | --- | --- |
| 憲法 | `crates/folio/src/face_index_read.rs` の constitution_card | 今の版の承認の日付（draft は生成日・便 144） | 同じ（便 145） |
| 要件書 | face_index_read.rs の srs_card | 承認欄の最後の 承認 の行（無ければ生成日・便 144・145） | 同じ |
| 設計ノート | `crates/folio/src/face_index.rs` の note_rows | 各設計ノートの meta.generated の最大 | 承認欄の日付（draft と見本は読まない・便 146）・無ければ生成日 |
| 判断の記録 | face_index.rs の adr_rows | 各記録の欄 date（起草の日）の最大 | 承認欄の日付（提案中は読まない・便 146）・無ければ記録の欄 date |

   憲法と要件書のカードは面の鮮度の札と同じ日付を出す。判断の記録のカードは面と違う欄（date）を読み、設計ノートのカードも発効した設計ノートでは面と違う欄（generated）を読む。
2. **folio2 自身の面の字（`folio build --dir design-intent --out <置き場> --write` の出力 30 file・1,946,214 byte）。** 入口の面の 4 枚のカードの 2 行目は次のとおり。

| カード | base の字（index.html の行） | 同じ文書の面 |
| --- | --- | --- |
| 憲法 | 更新 2026-09-25・v1.4（38 行目） | 鮮度の札 承認 2026-09-25 |
| 要件書 | 更新 2026-09-26・v1.44（45 行目） | 鮮度の札 承認 2026-09-26 |
| 設計ノート | 更新 2026-09-19（52 行目） | 2 本とも見本で、鮮度の札は 生成 2026-09-17 と 生成 2026-09-19 |
| 判断の記録 | **更新 2026-09-24**（61 行目） | 最も新しい承認は ADR-22 と ADR-23 の承認欄 2026-09-25。ADR-23 の面の鮮度の札は 承認 2026-09-25 |

   判断の記録は 23 本とも発効で承認欄を持つ（`grep -L approval design-intent/adr/ADR-*.yaml` は空）。記録の欄 date の最大は 2026-09-24、承認欄の日付の最大は 2026-09-25 で、同じ記録について入口のカードと面の日付が違う（台帳 f2-648.220 の観測）。
3. **正本の欄の形。** 判断の記録の承認欄は 1 つの表 approval（date を持つ・発効した判断に必須）。設計ノートの承認欄も 1 つの表 approval（date を持つ・欄の決まり `design-intent/design-note/schema.yaml` の approval_required_when = 発効）で、今の folio2 の 2 本は見本（status example）なので承認欄を持たない。判断の記録と設計ノートの面の（名・日付）は、それぞれ `crates/folio/src/face_adr.rs` と `crates/folio/src/face_note.rs` の中の非公開の関数 dated（便 146）が、`crates/folio/src/face_labels.rs` の named と approval_date を読まない状態の一覧（提案中・draft と見本）の字と一緒に呼んでいる。入口の読み手（face_index_read.rs の records と notes）はこの関数を呼ばず、Record は欄 date・Note は欄 generated だけを持つ。
4. **fixture。** 面の fixture の判断の記録 ADR-2（`tests/fixtures/face/adr/ADR-2.yaml`）は提案中・date 2026-09-06・承認欄なし。設計ノート full（`tests/fixtures/face/design-note/full.yaml`）は draft・generated 2026-09-18・承認欄なし。どちらも面の日付は base と本便の後で同じなので、凍結 anchor（`tests/fixtures/face/expected-index.html` ほか）のカードの字は変わらない。
5. **base の歯（参考値）。** workspace の nextest 933 / 933・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file（1,946,214 byte）。`git grep -n 'f147_' -- crates` は 0 件。既存の歯で判断の記録のカードの 更新 の日付を固定しているのは `crates/folio/tests/face_index.rs` の f139_adr_card_has_no_hit_area_and_opens_the_newest（写しの 2 本とも提案中で 更新 2026-09-06）だけで、実の正本の 更新 2026-09-24 を固定する歯は無い（`crates/folio/tests/face_index_shelf.rs` にも無い）。

### (b) 直す先 — 更新 の定めを 1 か所に書き、4 枚のカードが同じ口を通る

1. **定め（`crates/folio/src/face_labels.rs`・face.rs が再輸出する）。** 関数 shelf_updated を足し、その doc comment に定めを書く: **更新 = その型の文書の面が鮮度の札に出す日付（効いた日）のうち最も新しいもの**。型ごとに、憲法 = 今の版の承認の日付（draft は生成日・便 144）／要件書 = 承認欄の最後の 承認 の行（無ければ生成日・便 145）／判断の記録 = 各記録の adr_dated の最大／設計ノート = 各設計ノートの note_dated の最大。関数は日付の列を受けて最大を返す（1 つも無ければ空の字）。4 枚のカードがこの口を通る（憲法と要件書は 1 つの日付を渡す）。
2. **型ごとの面の日付の口（同じ file）。** 読まない状態の一覧を型付きの定数 ADR_UNREAD（提案中）と NOTE_UNREAD（draft と見本）にし、face_adr.rs と face_note.rs の非公開の dated を公開の関数 adr_dated と note_dated として face_labels.rs へ移す（字と振る舞いは便 146 のまま＝named と approval_date を呼ぶ）。face_adr.rs と face_note.rs は自前の dated を外し、共有の口を呼ぶ。これで面の鮮度の札・足の行（判断の記録と設計ノート）と入口のカードが、同じ関数と同じ一覧を読む（P-6.3・便 146 の契約 (b) の 3・4 と同じ意味）。
3. **入口の読み手（`crates/folio/src/face_index_read.rs`）。** Record の欄 date を dated（adr_dated の日付）に、Note の欄 generated を dated（note_dated の日付）に替える。欄 date と欄 generated は承認欄が在っても今までどおり必須で読む（断りの字は base と同じ）。憲法と要件書のカードの 更新 の日付も shelf_updated を通す（出力の字は変わらない）。
4. **入口の面（`crates/folio/src/face_index.rs`）。** adr_rows と note_rows の 更新 を shelf_updated（各記録・各設計ノートの dated）から取る。設計ノートの「開く →」と当たり判定は、その日付を持つ設計ノートへ（同じ日付が 2 本以上なら id の順で後の 1 本・base と同じ決まり）。判断の記録の「開く →」は最も新しい番号の記録のまま（便 139）。
5. **folio2 自身の面の字（本便の後）。** 入口の面の判断の記録のカードが 更新 2026-09-24 → **更新 2026-09-25** になる（1 行・1 字）。憲法・要件書・設計ノートのカードの字は変わらない。ほかの 29 file は byte で一致する（(e) の 4）。
6. **設計ノートのカードについて（依頼との差・起草役の判断）。** 依頼は設計ノートのカードを「生成日の最大（承認欄を持たないので今のまま）」とした。欄の決まりは発効した設計ノートに承認欄を求め、便 146 の後の設計ノートの面は発効なら 承認 <承認欄の日付> を鮮度の札に出す。カードだけ生成日のままにすると、発効した設計ノートが来たときに判断の記録と同じ食い違いが起きる。本便は設計ノートのカードも note_dated の最大にする。今の folio2 の 2 本は見本で、面の fixture の full は draft なので、**出力の字は folio2 自身の面でも凍結 anchor でも 1 byte も変わらない**（変わるのは発効して承認欄を持つ設計ノートが来たときだけ）。席が依頼の字どおりを採るなら、(e) の 3 の歯 f147_note_card_updated_is_the_latest_note_face_date と NOTE_UNREAD の note 側の読みを外し、note_rows を base のまま残す（問い 1）。
7. **変えないもの。** 部品・class・様式の定義・HTML のタグの並び。span.up に title 属性などの説明の字は足さない（(d) の 2）。憲法と要件書と設計ノートのカードの出力の字。判断の記録のカードの 1 行目・リンク・「開く →」の先・番号と見出しの一覧。判断の記録と設計ノートの面の出力。床。設計文書（index.yaml の凡例と部品目録を含む）。凍結 anchor（`tests/fixtures/face/` の expected 7 本・`design-intent/anchors/`・天井の束）。
8. **ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は、入口のカードが面と違う欄を読んで同じ記録について違う日付を出していた中身の正しさの直し（P-6.3）で、部品 shelf-card の class・構造・タグの並び・色・余白を 1 つも変えない。変わるのは 1 行の日付の 1 字で、タグの並びは base の同じ行と一致し、`folio parts --check` も合格する（(e) の 4）。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137〜139・141・144〜146 の契約と同じ読みである。部品ごとに数える読みで、もし中身の正しさの直しも数えるなら、shelf-card の字を変えた便は少なくとも便 138（要件書のカードに効いている版の札）・便 139（判断の記録のカードの番号と見出しの一覧・当たり判定の撤去）・便 144（憲法と要件書のカードの日付）で、本便は少なくとも 4 本目になり上限を超える（便 146 が変えたのは棚の札でカードではない・起草役が `git show` で 3 便の差分を確かめた）。よって 当たらない の根拠は、本便が見た目でなく中身の正しさの直しであるという前段の読みだけである。席はこの読みを採る（便 146 の契約 (i) と同じ判定・持ち主への次の報告で明記し、持ち主はこの読みを覆せる）。

### (c) 歯（関数名 f147_・base で 0 件）

1. **f147_type_dates_and_shelf_updated（単体の歯・`crates/folio/src/face_labels.rs` の既存の tests の区間 face_labels_tests）。** adr_dated が発効と廃止で（承認・承認欄の日付）、提案中は承認欄が在っても（生成・記録の日付）、承認欄の無い発効は（生成・記録の日付）を返すこと。note_dated が発効で（承認・承認欄の日付）、draft と見本は承認欄が在っても（生成・生成日）、承認欄の無い発効は（生成・生成日）。ADR_UNREAD と NOTE_UNREAD の字（凍結の針）。shelf_updated が最大を返し、1 つだけならそれ、空の列で空の字。**base では口が無く組み立てが通らない＝RED。**
2. **f147_adr_card_updated_is_the_latest_record_face_date（`crates/folio/tests/face_index.rs`・binary 経由・面の fixture の写し）。** 写しの ADR-2 と、ADR-2 を写した ADR-10 を歯の中で書き換える 4 通り: ADR-10 が発効（記録の日付 09-07・承認 09-10）で 更新 2026-09-10／ADR-10 が提案中で承認欄あり（読まない）で 更新 2026-09-07／ADR-10 が廃止で承認 09-08・ADR-2 が承認欄の無い発効で記録の日付 09-09 で 更新 2026-09-09／両方に承認欄（09-11 と 09-10）で 更新 2026-09-11。どれも 更新 <日付> がちょうど 1 つで、カードの span.up の総数も 1 つ。「開く →」は最も新しい番号の ADR-10 のまま。**base では 1 通り目で 更新 2026-09-07（記録の日付の最大）を出す＝RED。**
3. **f147_real_adr_card_updated_is_the_latest_record_face_date（同・実の正本）。** 歯の側の手書きの読み（yaml-rust2 で各記録を直に読み、提案中か承認欄が表でなければ欄 date・そうでなければ承認欄の date を escape して最大を取る）の日付で、判断の記録のカードに 更新 <日付> がちょうど 1 つ・更新 の行がカードに 1 つ。**base では 更新 2026-09-24（手書きの読みは 2026-09-25）＝RED。**
4. **f147_note_card_updated_is_the_latest_note_face_date（同・面の fixture の写し）。** 写しの full を写した 2 本目 second（draft・生成 09-20）を足し、full を 4 通りに書き換える: draft のまま承認欄 09-21 を足した写し（読まない＝更新 2026-09-20・開く → note-second.html）／発効にして承認欄 09-21 を足した写し（更新 2026-09-21・開く → note-full.html）／発効で承認欄の無い写し（生成日＝更新 2026-09-20・開く → note-second.html）／発効にして承認欄 09-20 を足した写し（full の承認と second の生成が同じ 2026-09-20＝id の順で後の note-second.html を開く）。どれも 更新 <日付> がちょうど 1 つで、カードの span.up の総数も 1 つ。**base では発効の写しが 更新 2026-09-20 と note-second.html を出す＝RED。**

fixture は新しい file を足さない。変異は歯の中で一時 dir の写しを書き換える（既存の口 index_fixture_copy・index_from・adr_article・note_card と同じ形・helper は up_span と index_with_edited_records の 2 つを足す）。正本の読みは歯の側の手書き（生成側の口を呼ばない・P-10.1）。

**既存の歯の期待の字の直し（全数）。** 0 本。凍結 anchor の手直しも 0 本（(e) の 1）。

### (d) 採らなかった形

1. **判断の記録のカードだけ承認欄の date を直に読む（adr_rows の中で approval.date か date の最大）。** 差分は小さいが、提案中を読まない一覧が face_adr.rs と入口の 2 か所に割れ、面と入口の日付が同じ口から出る保証が無くなる（P-6.3）。
2. **span.up に title 属性で定めを添える（例: 面の鮮度の札の日付の最大）。** 読み手に定めが見えるが、4 枚のカードの字がすべて変わり、入口の凍結 anchor 2 本（expected-index.html・expected-index-sheet.html）と既存の歯の期待の字が動く。定めを読み手に見せる字は入口の正本 index.yaml（凡例か説明）に置くのが筋で、それは設計文書の改訂（版の承認）になる＝問い 2 として席へ返す。
3. **更新 を「その型の文書の中で最も新しく変わった日（git の日付など）」にする。** 正本に無い値で、P-6.3 と便 146 の（名・日付）の口の外になる。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **実装だけを base に当てた写し（歯の file は base のまま）で、workspace の nextest は 933 / 933（落ちる歯は 0 本・起草役の実測）。** 凍結 anchor 7 本・天井の束の凍結 anchor・床の凍結の土台・凍結 anchor の列はどれも動かない（面の fixture の記録は提案中・設計ノートは draft で、面の日付が base と同じ）。
2. **本便の差分の全部を base に当てた写しで、** workspace の nextest は 937 / 937（base 933 + 単体 1 + face_index 3）・clippy 0 警告・床 4 本 rc 0・verify の 9 行すべて rc 0。
3. **RED（起草役の実測）。** binary の歯だけを base に当てた写し（936 本）で 3 本が落ちる＝(c) の 2・3・4。単体の歯だけを当てると組み立てが通らない（E0425 が 7 つ＝shelf_updated 3・adr_dated・ADR_UNREAD・note_dated・NOTE_UNREAD）。
4. **folio2 自身の面の変化（起草役の実測）。** base と本便の写しで `folio build --dir design-intent --out <置き場> --write` の出力は 30 file・1,946,214 byte のまま。違う file は index.html の 1 枚だけで、違う行は 61 行目の 1 行（更新 2026-09-24 → 更新 2026-09-25・タグの並びは同じ）。判断の記録 23 枚・設計ノート 2 枚・憲法・要件書・様式・script は byte で一致する。本便の写しの index.html は `folio parts --check --page index=<path>` で合格（違反 0・まだ分からない 0）。
5. **突然変異（起草役の実測・本便を当てた写しの実装だけを 1 通りずつ変え、単体の歯 f146_ と f147_・歯の file face_index・face_index_shelf・face_adr・face_note・face_labels を撃つ）。** 8 通りとも 1 本以上が落ちる。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 入口の記録の日付を欄 date に戻す | (c) の 2・3（2 本） |
| M2 ADR_UNREAD を空にする（提案中でも承認欄を読む） | (c) の 1・2 と便 146 の f146_adr_stamp_and_foot_follow_the_approval（3 本） |
| M3 NOTE_UNREAD から draft を外す | (c) の 1・4 と便 146 の f146_note_stamp_cover_and_foot_follow_the_approval（3 本） |
| M4 入口の設計ノートの日付を meta.generated に戻す | (c) の 4（1 本） |
| M5 shelf_updated を最小にする | (c) の 1・2・3・4（4 本） |
| M6 設計ノートの「開く →」を id の順の最後にする | (c) の 4（1 本） |
| M7 判断の記録のカードが最初の記録の日付だけを読む | (c) の 2・3（2 本） |
| M8 設計ノートの「開く →」を同じ日付の id の順で先の 1 本にする（rfind を find に） | (c) の 4（1 本） |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 10 本とも印なし。書き換える 6 本 = `crates/folio/src/face_labels.rs`・`crates/folio/src/face_index_read.rs`・`crates/folio/src/face_index.rs`・`crates/folio/src/face_adr.rs`・`crates/folio/src/face_note.rs`・`crates/folio/tests/face_index.rs`。本文不変の 4 本 = `crates/folio/tests/face_index_shelf.rs`・`crates/folio/tests/face_adr.rs`・`crates/folio/tests/face_note.rs`・`crates/folio/tests/face_labels.rs`（verify の scope）。face_adr.rs と face_note.rs は数行縮むが、新しい file へ移す分割ではないので `-` を付けない（便 146 の face.rs と同じ扱い）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 5 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_labels.rs` | 590 | 910 | 648（+58・うち単体の歯 約 33） | 852 |
| `crates/folio/src/face_index_read.rs` | 629 | 871 | 645（+16） | 855 |
| `crates/folio/src/face_index.rs` | 800 | 700 | 794（−6） | 706 |
| `crates/folio/src/face_adr.rs` | 920 | 580 | 915（−5） | 585 |
| `crates/folio/src/face_note.rs` | 1,019 | 481 | 1,011（−8） | 489 |

   余地はどれも S の見積 100 を超える（最小は face_note.rs の 481）。行数の上限の歯（face.rs 1,250・face_srs.rs 1,100）の対象の file は触らない。歯の file は src の外なので余地を測らない（参考値 wc -l で face_index 1,157 → 1,300）。
3. **size は S。**
4. **verify は 9 行**で、done の 9 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f147_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_index f147_` = (c) の 2・3・4（3 本）。
   3. `cargo nextest run -p folio --bin folio face_labels_tests` = face_labels.rs の単体の歯の全部（便 145・146 の named・approval_date・last_approval・dated の歯を含む・参考値 5 本）。
   4. `cargo nextest run -p folio --test face_index` = 入口の面の歯の全部（凍結 anchor 2 本・便 139 の判断の記録のカード・便 144 の憲法と要件書のカード・便 146 の入口の日付を含む・参考値 34 本）。
   5. `cargo nextest run -p folio --test face_index_shelf` = 棚のカードの歯の全部（参考値 11 本）。
   6. `cargo nextest run -p folio --test face_adr` = 判断の記録の面の歯の全部（凍結 anchor・便 146 の鮮度の札と足の行を含む・参考値 39 本）。
   7. `cargo nextest run -p folio --test face_note` = 設計ノートの面の歯の全部（同・参考値 31 本）。
   8. `cargo nextest run -p folio --test face_labels` = face.rs の行数の上限と face_labels.rs の定義の置き場（参考値 1 本）。
   9. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告（外した dated と欄 date・generated が残っていないことを含む）。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_index・face_index_shelf・face_adr・face_note・face_labels の 5 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f147_ と face_labels_tests を関数名か区間の名に持つ src は `crates/folio/src/face_labels.rs` だけで write-set に在る。base では verify の 1・2 が 0 本の実行で rc 4、3〜9 が rc 0（起草役の実測）。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d147-draft.md`、模擬の差分と script は同じ dir の d147-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-147.py（実装）・apply-147-teeth.py（歯）を撃つ。その差分が c147.patch。workspace の nextest・clippy・床 4 本（floor4.sh）・`folio build --write` の出力の file 数と base との `diff -rq`・`folio parts --check`・verify の 9 行（verify-147.sh）を撃つ。
2. RED: r147-teeth.patch（binary の歯だけ）を base に当てると (e) の 3 のとおり。実装だけを当てると落ちる歯は 0 本。
3. 突然変異: mut-147.sh（(e) の 5）。
4. 余地: lines-147.py と lines-147.awk（同じ式の 2 実装）を write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 定めを読み手に見せる字（index.yaml の凡例か説明・span.up の title 属性）。憲法・要件書・設計ノートのカードの出力の字。判断の記録の表紙の 日付 の枡と機械のための面の date。凍結 anchor。設計文書の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 更新 は面の鮮度の札の日付の最大で、最後に中身が変わった日ではない（正本に在る承認と生成の日付だけを読む）。提案中の判断の記録と承認欄の無い記録は起草の日（欄 date）を数える（便 146 の面と同じ）。承認欄は在るが日付の欄が無い記録と、発効で日付の欄の無い承認欄を持つ設計ノートは、base では入口の面が書けたが本便の後は入口の面も まだ分からない になる（その記録と設計ノートの面は便 146 の後すでに まだ分からない で、床は承認欄の date を必須にしているので、床を通った置き場では起きない）。日付は正本の字をそのまま比べ、形（年-月-日）を面で確かめない。次の周で読みやすさの観点がこの食い違いを解けたと読むかは周の結果でしか分からない。
3. **撤退条件。** (1) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と比べて index.html 以外の file で 1 byte でも違うか、index.html の違う行が判断の記録のカードの 更新 の行以外に在るか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、便 146 の named・approval_date と face_adr.rs・face_note.rs の dated・入口の Record と Note の欄の形が base と違えば、(b) と (f) を数え直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の ADR_UNREAD・NOTE_UNREAD・adr_dated・note_dated・shelf_updated と単体の歯 1 本。`crates/folio/src/face_adr.rs` と `crates/folio/src/face_note.rs` の dated を共有の口へ移すこと。`crates/folio/src/face_index_read.rs` の Record と Note の日付の欄と、憲法と要件書のカードが shelf_updated を通ること。`crates/folio/src/face_index.rs` の adr_rows と note_rows の 更新。歯の file `crates/folio/tests/face_index.rs` の f147_ の歯 3 本。
- 入れない: 部品目録・名札の class・様式の定義・床・設計文書・凍結 anchor・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| updated | 更新 の口 | `crates/folio/src/face_labels.rs` の shelf_updated（定めの doc comment） |
| dated | 型ごとの面の日付 | face_labels.rs の adr_dated・note_dated・ADR_UNREAD・NOTE_UNREAD |
| reader | 入口の読み手 | `crates/folio/src/face_index_read.rs` の records・notes・constitution_card・srs_card |
| cards | 棚のカード | `crates/folio/src/face_index.rs` の adr_rows・note_rows |
| teeth | 歯 | 単体の f147_ 1 本・binary の f147_ 3 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main 13af548）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.220）を閉じる。定めを読み手に見せる字を index.yaml に置くかを決める（§1 (d) の 2・問い）。次の周で、読みやすさの観点が入口と判断の記録の面の日付の食い違いを所見に立てないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "et"
title = "便 146 の問い 7 の控え（台帳 f2-648.220）: 入口の棚の判断の記録のカードの 更新 <日付> が各記録の欄 date（起草の日）の最大を出し、同じ記録の面の鮮度の札（承認欄の日付・便 146）と違う日付になる問題（folio2 自身の面で 更新 2026-09-24 と ADR-23 の 承認 2026-09-25）を直し、4 枚のカードの 更新 の定めを 1 か所に置く。定め = その型の文書の面が鮮度の札に出す日付（効いた日）のうち最も新しいもの。共有の口（crates/folio/src/face_labels.rs）に、定めを doc comment に持つ shelf_updated（日付の列の最大）と、読まない状態の一覧 ADR_UNREAD（提案中）・NOTE_UNREAD（draft と見本）と、判断の記録と設計ノートの面の（名・日付）の口 adr_dated・note_dated（crates/folio/src/face_adr.rs と crates/folio/src/face_note.rs の dated を字と振る舞いを変えずに移す）を足す。入口の読み手（crates/folio/src/face_index_read.rs）の Record と Note は面と同じ口の日付を持ち、入口の面（crates/folio/src/face_index.rs）の判断の記録と設計ノートのカードは shelf_updated で 更新 を出し、憲法と要件書のカードも shelf_updated を通す。憲法・要件書・設計ノートのカードと判断の記録と設計ノートの面の出力の字は変えない（folio2 自身の面で変わるのは index.html の 1 行だけ）。部品・class・タグの並び・設計文書・凍結 anchor は変えない。歯は単体の f147_ 1 本と face_index 3 本。既存の歯の期待の字の直しは 0 本。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main 13af548・受付の時点の main で数え直す"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_labels.rs", "crates/folio/src/face_index_read.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/face_index_shelf.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face_labels.rs"]
verify = ["cargo nextest run -p folio --bin folio f147_", "cargo nextest run -p folio --test face_index f147_", "cargo nextest run -p folio --bin folio face_labels_tests", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face_index_shelf", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test face_labels", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f147_（adr_dated が発効と廃止で承認欄の日付と名 承認・提案中と承認欄の無い記録で記録の日付と名 生成、note_dated が発効で承認欄の日付・draft と見本と承認欄の無い発効で生成日、ADR_UNREAD と NOTE_UNREAD の字、shelf_updated が最大と空の列で空の字）が緑、face_index の f147_ の 3 本（面の fixture の写し 4 通りで判断の記録のカードの 更新 が各記録の面の日付の最大・実の判断の記録の全本数で手書きの読みの最大・設計ノートの写し 4 通りで 更新 と開く の先が各設計ノートの面の日付の最大に従い同じ日付の 2 本は id の順で後の 1 本を開く・どの写しでもカードの 更新 の行は 1 つ）が緑、face_labels.rs の単体の歯の全部が緑、face_index の歯の全部が緑、face_index_shelf の歯の全部が緑、face_adr の歯の全部が緑、face_note の歯の全部が緑、face_labels の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで index.html 以外は byte 一致・index.html の違う行は判断の記録のカードの 更新 の行だけ"
<!-- contracts:end -->
