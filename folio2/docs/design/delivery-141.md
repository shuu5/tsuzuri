# 設計: 便 141 — 憲法の面の機構の欄の名札から「段」を外し、値を憲法が書く実在の予定と読める字にする（床は判定しない・憲法が書く実在の予定は M1）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す）。本便は憲法の面の機構の小窓の名札のうち、機構がまだ無い条の意味の 4 つの字（値 M0・delivery-0・M1・adr）を、型付きの定数の表の側で直すだけで、FR4 の規範文も面の部品・色・並び・class も変えない。契約表の行の req は FR4 の 1 つ（main に在る id・便 132 と同じ）。
- 条: P-6.1（人が読むページは正本から逐語で生成する＝値は正本の live の字のまま出し、読み違える字を足さない）/ P-4.2（判定できないものは表に出す＝機構がまだ無いことを名札で出す向きは便 132 のまま）/ P-5.1（名札は値域の型の上の閉じた表で持つ）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-6.2（生成物を手で直さない＝面の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ）。
- 出所: 天井の 33 周目（2026-09-25・本流 94e6d3e）の読みやすさの所見 **F-1**（重さ 直す・場所 adr の ADR-23.title・面の側）。一括 20 の仕分け（`docs/design/batch20-triage.md`）の便の候補 **B-2**。台帳の控え **f2-648.206**。正本の側（判断の記録 ADR-23 の見出しと決定の字）は一括 20 で語彙の「段」と「機構」の項の断りで閉じ、面の名札の字だけが面の生成器の便として残った（仕分けの判定 2）。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `en` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 7 本（書き換える 4 本 + 本文が変わらない verify の scope 3 本・便 132 と同じ 7 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-25・席の裁定と指示）: 便 140 と便 139 の着地を受けて枝を main e5b8930 に載せ替え、base を e5b8930 に一本化した（起草を始めた時点の main の数の列を外した）。数はすべて e5b8930 の写しで撃ち直した（nextest 903 → 906・`folio build` の出力 1,924,927 → 1,924,999 byte・binary の単体の歯 135）。write-set 7 本の行数・凍結 anchor の数と sha256・実装だけで落ちる既存の歯 5 本・RED 3 本・突然変異 6 通りの結果・門・床・部品目録の検査・verify の 8 行の結果は撃ち直しても同じだった。§0 の重なりの欄を並行の便なしの形にした。席は起草役の問い 7 点を全部推奨どおりに裁定した（新しい字は 憲法が書く実在の予定は <値>・S・ADR-5 の撤退条件 ② に当たらない・語彙の段の項の断りは本便で直さず次の一括へ〔席が台帳の控え f2-648.209 に足す〕・受付順は問題なし〔便 139 は着地済み〕・base は main e5b8930・単体の歯は `face_labels.rs` の既存の tests の区間）。walk は前置せず、着地後の面を持ち主への次の報告で見せる（席の裁定）。設計・歯・write-set・verify・done・size は変えない。
- 改訂 c（2026-09-25・独立の検証 d141-verify.md〔支持・blocking 0・数値は全部一致〕の任意の文面 4 点）: (e) の 5 と (i) の 1 に、語彙の段の項の断りの句が便の後も憲法の面と要件書の面の用語集に出て名札と食い違うので、次の天井の周の前に一括で直す（台帳の控え f2-648.209）ことを書いた。(g) の 3 に、撤退条件 ② の上限が行 R-7 の 2 回で、見た目の直しと数えれば便 84・132・141 で 3 回になる帰結を書き、そのうえで数えない根拠を置いた。(a) の 6 の全数の表に `design-intent/vocabulary.yaml` の行（直さない・次の一括）を足した。関数の上の doc 注の扱いを (a) の 6 の表と (b) の 1 で 直す に揃えた。数値・write-set・verify・done・size・歯は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 7 本を base の binary で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便の全差分を当てた写しの binary で撃っても同じ字で 0 だった。
- 前の便: 前提の着地は無い。**base = main e5b8930（便 139 と便 140 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 139（行 el・台帳 f2-648.205）と便 140（行 em・台帳 f2-648.208）はどちらも着地済み（main e5b8930＝本便の base）で、並行の便は無い。便 139 の write-set と本便の write-set は本文を変えない verify の scope の 2 本（`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`）を共に持っていたが、便 139 は 2 本の本文を変えずに着地した。**共通の検証（workspace の nextest）の本数は受付の時点の main で数え直す**（base e5b8930 で 903・本便の後 906）。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。

## 1. 設計

### (a) いま起きていること（実測・base main e5b8930・数は参考値）

1. **名札の今の字。** 憲法の面の各条の機構の小窓は、名札を「<守らせ方>・<いつから動くかの名札>（<その意味>）・<編集時か事後か>・<開くか閉じるか>」の形で出す（`crates/folio/src/face_constitution.rs` の条の行の組み立て）。いつから動くかの名札と意味は `crates/folio/src/face_labels.rs` の 2 関数 mechanism_live_label と mechanism_live_meaning が、憲法の値域 schema.enums.mechanism_live の 5 値（now・M0・delivery-0・M1・adr）の上の網羅の場合分けで持つ（便 84 と便 132）。now でない 4 値の意味の字は次のとおり。

| 値 | 今の意味の字（mechanism_live_meaning） |
| --- | --- |
| M0 | 床は判定しない・憲法の段の値は M0 |
| delivery-0 | 床は判定しない・憲法の段の値は delivery-0 |
| M1 | 床は判定しない・憲法の段の値は M1 |
| adr | 床は判定しない・憲法の段の値は adr |

2. **folio2 自身の憲法の面の今の字（`folio build --dir design-intent --out <一時 dir> --write` の constitution.html から逐語）。** 機構の小窓は 30 個で、機構がまだ無い条の小窓は 6 個（P-11・P-13・P-15・P-17・P-18・A-3・どれも値 M1）。名札の部分（機構の注の区切り — の前）は次の 2 通りである。

```
機械が拒む・機構がまだ無い（床は判定しない・憲法の段の値は M1）・事後・閉じる   （3 個）
生成時の検査・機構がまだ無い（床は判定しない・憲法の段の値は M1）・事後・閉じる （3 個）
```

   ほかの 24 個はいま動く（今の folio に在る）の名札で、本便は変えない。値 M0・delivery-0・adr は folio2 自身の憲法には無く、面の fixture の憲法（`tests/fixtures/face/constitution.yaml`）の P-1（M0）と N-1（delivery-0）が凍結 anchor `tests/fixtures/face/expected.html` の 86 行目と 121 行目に出る。
3. **何が読み違えられるか。** 「段」は憲法では条の段（絶対にやらない／確認してから／いつも守る＝語彙の項 段）の語で、面では同じ条の区画の見出しの隣に段の札が在る（例 P-11 の区画は、見出しの隣に いつも守る と この段 · いつも守る と §1 で 3 段を見る → を出し、同じ区画の機構の小窓に 憲法の段の値は M1 を出す）。名札の「憲法の段の値は M1」は、機構がいつ実在するか（語彙の項 機構 の live）の値を言うのに、条の段の値と読める（33 周目の読みやすさ F-1）。一括 20 は語彙の「段」の項に、判断の記録 ADR-23 と憲法の機構の注と面の名札の「段の値」は条の段ではないという断りを足して正本の側を閉じたが、面の名札そのものは面の生成器の字なので残った。
4. **値 M1 は予定として読める字が要る。** 判断の記録 ADR-23 決定 (1) ⑤ は、6 本の条の値 M1 を変えないと決めた（値域に今の実態＝予定の段階が決まっていないを言う値が無い・M3 に移すと別の古い約束になる・値域を足すのは条 A-2 の改訂）。M1 の段階は着地済みなので、値 M1 は古い約束のまま残る（ADR-23 の文脈 (7)）。面は値を正本の字のまま出すが（P-6.1）、それが実態でなく憲法が書いた予定であることが字で読めないと、M1 が済んだのに機構が無いことが矛盾に見える。
5. **語彙の字（base の design-intent/vocabulary.yaml）。** 項 機構 の定義は、守らせ方と「いつ実在するか（live・判断の記録と憲法の注と面の名札での別名は「段」の項の断りのとおりで、条の段とは別）」を持つ。項 段 の断りは、ADR-23 と憲法の機構の注と面の名札が言う「実在の段」「機構の段（の規則）」「段の値」を（その条の機構がいつ実在するか＝機構の項の live の値）と言い換える。規則の表 rules.yaml の注も同じ値を「いつ実在するか〔live〕」と呼ぶ。憲法の schema 節の値域の注は「その機構がいつ実在するか」、意味の表 mechanism_live_meaning は M1 を「M1 で実在する」と書く。
6. **同じ字を持つ所の全数（`grep -rn 憲法の段の値 crates tests design-intent` と `grep -rln 床は判定しない`・target と .git を除く）。**

| file | 何か | 本便 |
| --- | --- | --- |
| `crates/folio/src/face_labels.rs` | 正本の表（mechanism_live_meaning の 4 つの腕と上の doc 注） | 直す |
| `crates/folio/src/face.rs` | 単体の歯 face_labels_are_frozen_needles_for_the_string_tables の凍結の針（同じ表を逐語で持つ・4 行） | 直す |
| `crates/folio/tests/face_constitution.rs` | 歯 f132_articles_without_a_mechanism_say_so の期待の字（1 行） | 直す |
| `tests/fixtures/face/expected.html` | 憲法の面の凍結 anchor（2 か所） | 手で直す |
| `design-intent/vocabulary.yaml` | 語彙の項 段 の断り（面の名札が言う「段の値」を名指す・grep の字には掛からない・憲法の面と要件書の面の用語集に出る） | 直さない（面の名札を名指す・次の一括・台帳の控え f2-648.209） |
| `design-intent/adr/ADR-23.yaml` | 判断の記録の決定と帰結の字（面の名札の字を名指す来歴・1 か所） | 直さない（正本の来歴・門の外を保つ） |
| `docs/design/` の設計ノート（batch20-triage.md・adr-23.md・adr-23-grill.md・delivery-132.md） | 記録 | 直さない |

   床の fixture の憲法の写し（`tests/fixtures/floor_base/design-intent/`）と実の憲法の正本と凍結の版（`design-intent/anchors/constitution-v1.*.yaml`）は、意味の表 mechanism_live_meaning（M1 で実在する など）を持つが、「段の値」の字は持たない。本便は正本の意味の表を変えない。
7. **base の歯（参考値）。** workspace の nextest 903 / 903・clippy 0 警告・床 4 本（check・inject --check・schema --check・derive --check）rc 0・`folio build` の出力 30 file（1,924,927 byte）・部品目録の検査 `folio parts --check` 合格（違反 0・まだ分からない 0）。`git grep -n 'f141_' -- crates` は 0 件。歯の file の本数は face_constitution 17・face 24・badge 16・site 13・binary の単体 135。

### (b) 直す先 — 意味の表の 4 つの字と、同じ字を逐語で持つ針・期待の字・凍結 anchor

1. **意味の表（`crates/folio/src/face_labels.rs` の mechanism_live_meaning）。** now でない 4 つの腕の字を次にする。now の腕（今の folio に在る）と名札の関数 mechanism_live_label（機構がまだ無い）は変えない。値は正本の live の字のまま括弧の末尾に置く。

| 値 | 今の字 | 新しい字 |
| --- | --- | --- |
| M0 | 床は判定しない・憲法の段の値は M0 | 床は判定しない・憲法が書く実在の予定は M0 |
| delivery-0 | 床は判定しない・憲法の段の値は delivery-0 | 床は判定しない・憲法が書く実在の予定は delivery-0 |
| M1 | 床は判定しない・憲法の段の値は M1 | 床は判定しない・憲法が書く実在の予定は M1 |
| adr | 床は判定しない・憲法の段の値は adr | 床は判定しない・憲法が書く実在の予定は adr |

   面の小窓の名札は、たとえば「機械が拒む・機構がまだ無い（床は判定しない・憲法が書く実在の予定は M1）・事後・閉じる」になる。1 つの字は 4 字増え、UTF-8 で 12 byte 増える。関数の上の doc 注の「憲法の段の値」も同じ向きの字に直す（字は実装が決める・歯は見ない）。これで便の後は `grep -rn 憲法の段の値 crates/folio/src` が 0 件になり（起草役の模擬で 0 件・crates の下で残るのは歯 f141_ が古い字の無いことを確かめる 1 行だけ）、着地の後の確かめが 1 行で済む。
2. **新しい字の読み方と語彙との揃い。** 「実在の予定」は、語彙の項 機構 の「いつ実在するか（live）」と、項 段 の断りの言い換え（その条の機構がいつ実在するか＝機構の項の live の値）を名詞にした字で、規則の表の注の「いつ実在するか〔live〕」とも同じ事を指す。「憲法が書く」は、値が憲法の正本の書いた約束であって実態の測りではないことを言い、M1 が済んだ後も値が M1 のまま残る ADR-23 決定 (1) ⑤ の形を、予定として読める字にする。「段」の字は名札から消えるので、条の段と読み違える字が面の機構の小窓の名札の部分に 1 つも残らない。語彙の 2 つの項の字は本便で変えない（(i) の 1）。
3. **凍結の針（`crates/folio/src/face.rs` の単体の歯 face_labels_are_frozen_needles_for_the_string_tables）。** mechanism_live_meaning の表の 4 行を (b) の 1 の新しい字に直す（歯の中の手で写した字・行数は変わらない）。ほかの表の針は変えない。
4. **既存の歯の期待の字（`crates/folio/tests/face_constitution.rs` の f132_articles_without_a_mechanism_say_so）。** 期待の字の組み立ての 1 行を「機構がまだ無い（床は判定しない・憲法が書く実在の予定は <値>）」に直す。数え方（正本の値ごとの条の数と小窓の数が等しい・古い名札 5 つが無い）は変えない。
5. **凍結 anchor（`tests/fixtures/face/expected.html`）。** 86 行目（P-1・M0）と 121 行目（N-1・delivery-0）の小窓の名札の中の「憲法の段の値は」を「憲法が書く実在の予定は」に字面で置き換える（folio の出力から写さない）。起草役は手で直した anchor が、(b) の 1 を当てた binary の出力と **byte で一致**することを確かめた（歯 face_write_matches_the_frozen_fixture が緑）。

| もの | 行数 | byte | sha256 |
| --- | ---: | ---: | --- |
| `expected.html` の前 | 224 | 25,705 | 248b6df3ccc6201495c3a20e3ed7ddae6014a3a9911ee6ab9e842c29cbd36acf |
| 同 後 | 224 | 25,729 | c5fef562e372738c971b1da6f317a70746b6a5e721e31fb1098a4361ab19021b |

   差は 2 か所で各 12 byte の合わせて 24 byte だけで、行数は変わらない（sha256 は `sha256sum` と python の hashlib の 2 実装で一致）。ほかの凍結 anchor（入口の面 2 本・要件書の面・判断の記録の面 2 本・設計ノートの面・天井の束・床の土台）は 1 byte も変えない。
6. **変えないもの。** 名札の関数 mechanism_live_label と now の意味の字・守らせ方と編集時か事後かと開くか閉じるかの名札・小窓の部品と class と並び・機構の注・憲法の正本の意味の表 mechanism_live_meaning（M1 で実在する など）と値域と各条の値・語彙・判断の記録 ADR-23 の字・folio check の標準エラーの 1 行（便 131）・入口と要件書と判断の記録と設計ノートの面・床・設計文書。

### (c) 歯（関数名 f141_・base で 0 件）

1. **f141_live_meanings_name_the_plan_and_keep_the_value（単体の歯・`crates/folio/src/face_labels.rs` の既存の tests の区間 face_labels_tests に足す）。** 値域の 5 値のそれぞれで、now は今の folio に在る、now でない 4 値は「床は判定しない・憲法が書く実在の予定は 」（歯の中で手で写した凍結の針）に正本の値の字をそのまま続けた字に等しいこと、5 つの意味の字と名札 機構がまだ無い のどれも字「段」を含まないことを確かめる。**base では字が違い「段」を含む＝RED。**
2. **f141_each_live_value_names_the_plan_in_the_chip（`crates/folio/tests/face_constitution.rs`・binary 経由）。** 実の置き場の写しの憲法の live: M1 の先頭 3 つを M0・delivery-0・adr に替え（正本に live: M1 が 4 つ以上在ることを前提として確かめる）、4 つの値を 1 枚の面に並べる。終了コード 0 で、機構の小窓の名札の部分（注の区切りの前）に「機構がまだ無い（床は判定しない・憲法が書く実在の予定は <値>）」が値ごとに写しのその値の条の数（M0・delivery-0・adr は 1・M1 は残りの数）だけ在り、どの小窓の名札の部分も字「段」を含まないこと。**base では古い字＝RED。**
3. **f141_no_chip_head_names_the_tier（同）。** 実の憲法の面と、凍結 anchor `expected.html` の 2 つで、字「憲法の段の値」がどこにも無く、機構の小窓が 1 つ以上在り、どの小窓の名札の部分も字「段」を含まないこと。加えて凍結 anchor に M0 と delivery-0 の新しい名札がちょうど 1 つずつ在ること。**base では古い字＝RED。**

fixture は新しい file を足さない。変異は歯の中で実の置き場の写しを書き換える（歯の file の既存の口 real_copy と folio_face と hint_bodies と chip_head を使う）。名札の部分に限るのは、機構の注（実の P-18 の注の「機構の段の規則」など・正本の字）が「段」を持つためである。

### (d) 採らなかった形

1. **「段」だけを別の語に替える（例 憲法の実在の値は M1）。** 値が予定であることが字で読めず、所見の後半（M1 が済んだのに値が M1 のまま残るのを予定と読めない）が残る。
2. **語彙の字を逐語で使う（例 憲法が書く、いつ実在するかの値は M1）。** 語彙との揃いは強いが、名札の括弧の中が長くなり、小窓の 1 行が延びる。「実在の予定」は同じ事を名詞 1 つで言い、括弧の中の読点も増やさない。
3. **値を人の言葉に置き換える（例 M1 で実在する予定）。** 意味の表を憲法の正本 mechanism_live_meaning から読む形になり、面が正本の意味の字（M1 で実在する）を引く別の便になる。値の字をそのまま出す便 132 の形（P-6.1）を保つ。
4. **値 M1 を面で「予定なし」などに読み替える。** ADR-23 決定 (1) ⑤ が値を変えないと決めた判断を面が上書きすることになる。値域を足すのは条 A-2 の改訂である。
5. **語彙の「段」の項の断りの字（面の名札が言う「段の値」）も同じ便で直す。** 設計文書の正本の書き換えで天井の門の対象になり、一括（持ち主の承認）が要る。本便は面の生成器の字だけにし、語彙の字は次の一括へ回す（席が台帳の控え f2-648.209 に足す・§5）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **実装だけを当てた写しでは、既存の歯が 5 本落ちる（起草役の実測・903 本中）。** どれも本便の write-set の中の直しで緑に戻る。

| 歯（file） | 落ちる理由 | 直し方 |
| --- | --- | --- |
| bin/folio の face::face_tests::face_labels_are_frozen_needles_for_the_string_tables | 凍結の針が古い字 | (b) の 3 |
| face_constitution の f132_articles_without_a_mechanism_say_so | 期待の字が古い | (b) の 4 |
| face の face_write_matches_the_frozen_fixture | 凍結 anchor が古い字 | (b) の 5 |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures | 同上 | (b) の 5 |
| site の site_write_matches_the_frozen_fixture | 同上 | (b) の 5 |

   (b) の 3〜5 まで当てた写し（歯を除く）で 903 / 903。本便の差分の全部を base に当てた写しで、workspace の nextest は **906 / 906**（base 903 + 単体 1 + face_constitution 2）・clippy 0 警告・床 4 本 rc 0・face_constitution 19 / 19。
2. **動く凍結 anchor は 1 本で、2 か所だけ**（(b) の 5）。
3. **RED（起草役の実測）。** 歯だけを base に当てた写しで、f141_ の 3 本とも落ちる。
4. **突然変異（起草役の実測・本便を当てた写しの file を 1 通りずつ変える・src の変異は match の腕の字だけを替え、歯の中の凍結の針は替えない）。** 6 通りとも f141_ の 1 本以上が落ちる。

| 変異 | f141_ 単体 | f141_ 値ごと | f141_ 段なし | 針（face.rs） | f132 | face・badge・site |
| --- | --- | --- | --- | --- | --- | --- |
| M1 4 つの字を古い字へ戻す | 落ちる | 落ちる | 落ちる | 落ちる | 落ちる | 落ちる |
| M2 adr の 1 つだけ古い字 | 落ちる | 落ちる | 緑 | 落ちる | 緑 | 緑 |
| M3 語彙と違う字（実在の時期は） | 落ちる | 落ちる | 緑 | 落ちる | 落ちる | 落ちる |
| M4 段を別の形で戻す（実在の段は） | 落ちる | 落ちる | 落ちる | 落ちる | 落ちる | 落ちる |
| M5 adr の腕に値 M1 を焼く | 落ちる | 落ちる | 緑 | 落ちる | 緑 | 緑 |
| M6 凍結 anchor を直さない | 緑 | 緑 | 落ちる | 緑 | 緑 | 落ちる |

   M2 と M5 は folio2 自身の憲法にも面の fixture にも無い値 adr の字だけを崩す変異で、凍結 anchor を読む歯では拾えない。f141_ の単体の歯と値ごとの歯が 4 つの値を全部並べるのはこのためである。
5. **面の変化（起草役の実測）。** base の設計文書で `folio build --dir design-intent --out <置き場> --write` を撃つと、出力は 30 file のまま、違うのは constitution.html の 1 file だけで、中身の差は機構がまだ無い 6 本の条の小窓の名札の字の 6 か所（各 12 byte・計 +72 byte・1,924,927 → 1,924,999 byte・名札の字を置き換えた base の constitution.html が便の後の出力と一致）だけである。部品目録の検査 `folio parts --check` は便の後の憲法・入口・要件書の面で合格（違反 0・まだ分からない 0）。判断の記録 ADR-23 の面の本文の「憲法の段の値は M1」の 1 か所は正本の字（来歴）なので残る。語彙の項 段 の断りの句（ADR-23 と憲法の機構の注と面の名札が言う「実在の段」「機構の段（の規則）」「段の値」）も、便の後の憲法の面（用語集の 2 か所）と要件書の面（用語集の 1 か所）に逐語で残る。便の後は面の名札が「段の値」を言わないので、同じ憲法の面の上で用語集の句が名札と食い違う。この句は次の天井の周の前に一括で直す（台帳の控え f2-648.209・(i) の 1）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 7 本とも印なし。書き換える 4 本 = `crates/folio/src/face_labels.rs`・`crates/folio/src/face.rs`・`crates/folio/tests/face_constitution.rs`・`tests/fixtures/face/expected.html`。本文不変の 3 本 = `crates/folio/tests/face.rs`・`crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`（verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_labels.rs` | 420 | 1,080 | 438（+18・起草役の歯の書き方による参考値） | 1,062 |
| `crates/folio/src/face.rs` | 1,111 | 389 | 1,111（不変・字だけ 4 行） | 389 |

   余地はどちらも S の見積 100 を超える。`face.rs` には歯 f87（`crates/folio/tests/face_labels.rs` の f87_face_is_split_and_under_the_cap）の上限 1,250 が在り、base で 1,111 行なので器が足せるのは 139 行までだが、本便は `face.rs` の行数を変えない。src の外の参考値は、`tests/face_constitution.rs` 759 → 818（行数の上限の歯は無い）・`tests/face.rs` 885・`tests/badge.rs` 958・`tests/site.rs` 750・`expected.html` 309 → 309。
3. **size は S。** 触る src は 2 本で、字 8 か所（表 4・針 4）と単体の歯 1 本。
4. **verify は 8 行**で、done の 8 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f141_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_constitution f141_` = (c) の 2・3（2 本）。
   3. `cargo nextest run -p folio --test face_constitution` = 憲法の面の歯の全部（期待の字を直した f132 と、名札の数え上げを機構の注の前に限った f84 を含む・参考値 19 本）。
   4. `cargo nextest run -p folio --bin folio face_labels_are_frozen_needles_for_the_string_tables` = 名札の表の凍結の針。
   5. `cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture` = 憲法の面の凍結 anchor との byte 一致。
   6. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   7. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   8. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   base e5b8930 では 1 と 2 が 0 件で終了コード 4（歯が無い）、3〜8 は緑（3 は 17 本）である。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_constitution・face・badge・site の 4 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f141_ を関数名に持つ src は `face_labels.rs`、face_labels_are_frozen_needles_for_the_string_tables を持つ src は `face.rs` で、どちらも write-set に在る。

### (g) 門と受付・判断の記録 ADR-5 の撤退条件 ②

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。周の引き金（ADR-18 の規範の欄）にも触れないので、本便の後に天井の周を回し直す要は無い。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測・(h) の 6）。
3. **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が同じ部品について上限を超えて続く）に当たらない。** 本便は名札の字の意味を直す便で、部品・色・余白・並び・名札の class・小窓の数を 1 つも変えない（`folio parts --check` が便の前後で合格）。直すのは、値が条の段と読める字と、値が予定と読めない字で、字の正しさ（P-6.1）である。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137・138・139 の契約と同じである。憲法の面の機構の小窓を直した便 84 と 132 も撤退条件 ② の数えに入れていない。撤退条件 ② の上限は規則の表の行 R-7 の値 2 回で、もし同じ小窓の名札の直し（便 84・132・141）を見た目の直しと数えれば、本便で 3 回になり上限を超える（持ち主に外部の部品へ移るかを問う時機に当たる）。それでも数えないのは、3 本とも直したのが名札の字の意味（便 84 は値の意味を足す・便 132 は機構がまだ無い条を動くと見せない・本便は条の段と読める字と予定と読めない字）で、見た目（部品・色・余白・並び・名札の class）は 3 本とも変えていないためである。撤退条件 ② が問うのは手書きの部品の見た目の往復が収まらないことで、字の意味の直しは部品を外部の部品に替えても残る。要件書の面の見た目の直しの数え（上限 2 に到達済み）は要件書の面の話で、本便は要件書の面を変えない。持ち主の walk（面の承認）は前置しない（章と部品を変えない・席の裁定＝着地後の面を持ち主への次の報告で見せる）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d141-draft.md`、模擬の差分と script は同じ dir の d141-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-141.py（実装 impl と、針・期待の字・凍結 anchor の pins）と apply-141-teeth.py（歯）を撃つ。その差分が c141.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との `diff -r`・`folio parts --check`・verify の 8 行（verify-141.sh）を撃つ。
2. RED: apply-141-teeth.py だけを base に当てると (e) の 3 のとおり（r141-teeth.patch）。apply-141.py impl だけを当てると (e) の 1 の 5 本が落ちる。
3. 突然変異: mut-141.sh（(e) の 4）。
4. 余地: lines-141.py と lines-141.awk（同じ式の 2 実装）を repo の根で write-set の src の file に当てる。
5. 凍結 anchor の要約値: python の hashlib と `sha256sum` の 2 実装。
6. 受付の先撃ち: 契約を commit した後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-141.md#en`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 語彙の項 段 の断りの字（面の名札が言う「段の値」の句は、本便の後の面に当たる字が無くなるが、憲法の面と要件書の面の用語集には便の後も逐語で出て、名札と食い違う・設計文書の正本なので次の天井の周の前に一括で直す・席が台帳の控え f2-648.209 に足した・(e) の 5・§5）。判断の記録 ADR-23 の字。憲法の正本の意味の表 mechanism_live_meaning と値域と各条の値（ADR-23 決定 (1) ⑤）。機構の注。folio check の標準エラーの 1 行（便 131・括弧の中は値だけ）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 次の周の読みやすさの観点が F-1 の面の側を解けたと読むかは周の結果でしか分からない。「実在の予定」の字が持ち主にとって十分に平易かは、面を見せるまで分からない。値 M1 が済んだ段階の名であることは、名札だけからは読めない（予定と読めるところまでが本便の範囲で、値を変えるのは ADR-23 の撤退条件と条 A-2 の側）。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 5 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の設計文書で組んだ `folio build` の出力が、着地の直前の main の binary の出力と file 数で違うか、憲法の面の機構がまだ無い条の小窓の名札の字（「憲法の段の値は」から「憲法が書く実在の予定は」への置き換え・base では 6 か所）の外で 1 byte でも違うか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、mechanism_live_meaning の 4 つの字・face.rs の針・f132 の期待の字・凍結 anchor の 2 か所・憲法の値域 mechanism_live のどれかが base と違えば、(a)(b)(e) を (h) の手順で数え直してから運ぶ（「段」の字が既に消えていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の mechanism_live_meaning の 4 つの字と単体の歯 1 本。`crates/folio/src/face.rs` の凍結の針 4 行。`crates/folio/tests/face_constitution.rs` の f132 の期待の字 1 行と歯 2 本。面の凍結 anchor `tests/fixtures/face/expected.html` の 2 か所。
- 入れない: mechanism_live_label と now の意味・ほかの名札・小窓の部品と class・機構の注・憲法の正本（意味の表・値域・各条の値）・語彙・判断の記録・folio check の 1 行・ほかの面・床・設計文書・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| meaning | 意味の名札 | `crates/folio/src/face_labels.rs` の mechanism_live_meaning の now でない 4 つの腕 |
| needle | 凍結の針 | `crates/folio/src/face.rs` の face_labels_are_frozen_needles_for_the_string_tables の表の 4 行 |
| expect | 既存の歯の期待の字 | `crates/folio/tests/face_constitution.rs` の f132 の 1 行 |
| anchor | 憲法の面の凍結 anchor | `tests/fixtures/face/expected.html` の 2 か所（P-1・N-1） |
| teeth | 歯 | 単体の f141_ 1 本と face_constitution の f141_ 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main e5b8930・便 139 と便 140 は着地済み・§0）。
- 並行の便: 無し（§0）。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.206）を閉じる。語彙の項 段 の断りの「面の名札が言う…「段の値」」の句が面に当たる字を失うので、次の一括で断りから面の名札を外す（ADR-23 と憲法の機構の注の字はそのまま残る・席が台帳の控え f2-648.209 に足す）。次の周の読みやすさの観点で F-1 と同じ所見が面の側に立たないかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "en"
title = "一括 20 の B-2（天井の 33 周目 読みやすさ F-1 の面の側・台帳 f2-648.206）: 憲法の面の機構の小窓の名札の意味 床は判定しない・憲法の段の値は <値> の 段 が条の段（絶対にやらない／確認してから／いつも守る）と読み違えられ、M1 が着地済みなのに値 M1 が残る（判断の記録 ADR-23 決定 (1) ⑤）ことが予定と読めない問題を直す。crates/folio/src/face_labels.rs の mechanism_live_meaning の now でない 4 つの腕（M0・delivery-0・M1・adr）の字を 床は判定しない・憲法が書く実在の予定は <値> にし（値は正本の live の字のまま・語彙の機構の項の いつ実在するか を名詞にした字）、同じ表を逐語で持つ crates/folio/src/face.rs の凍結の針 4 行と、crates/folio/tests/face_constitution.rs の f132 の期待の字 1 行と、憲法の面の凍結 anchor tests/fixtures/face/expected.html の 2 か所を直す。mechanism_live_label・now の意味・部品・class・機構の注・憲法の正本・語彙・判断の記録は変えない。歯は単体の f141_ 1 本と face_constitution の f141_ 2 本。設計文書の正本を書き換えないので門の対象外で、判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_labels.rs", "crates/folio/src/face.rs", "crates/folio/tests/face_constitution.rs", "tests/fixtures/face/expected.html", "crates/folio/tests/face.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --bin folio f141_", "cargo nextest run -p folio --test face_constitution f141_", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --bin folio face_labels_are_frozen_needles_for_the_string_tables", "cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f141_（値域の 5 値のうち now は 今の folio に在る、now でない 4 値は 床は判定しない・憲法が書く実在の予定は に正本の値の字を続けた字で、5 つの意味と名札 機構がまだ無い のどれも 段 を含まない）が緑、face_constitution の f141_ の 2 本（実の憲法の写しの live を M0・delivery-0・M1・adr の 4 値に並べた面で、値ごとに 機構がまだ無い（床は判定しない・憲法が書く実在の予定は <値>）の小窓がその値の条の数だけ在り、どの小窓の名札の部分も 段 を含まない・実の憲法の面と凍結 anchor expected.html に 憲法の段の値 の字が無く、凍結 anchor に M0 と delivery-0 の新しい名札がちょうど 1 つずつ在る）が緑、face_constitution の歯の全部（期待の字を直した f132 と f84 を含む）が緑、名札の表の凍結の針の単体の歯が §1 (b) の 1 の字で緑、§1 (b) の 5 の 2 か所を直した憲法の面の凍結 anchor との byte 一致の歯（face）が緑、名札の無い面の凍結 anchor 7 本との byte 一致の歯（badge）が緑、組み立ての出力の凍結 anchor との byte 一致の歯（site）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで、違いは憲法の面の機構がまだ無い条の小窓の名札の 憲法の段の値は から 憲法が書く実在の予定は への置き換えだけである"
<!-- contracts:end -->
