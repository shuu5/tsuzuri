# 設計: 便 137 — 判断の記録の面に改訂の欄（revises）の行を描く（表紙の札を条文の改訂と判断の記録の改訂に分けて数える）

- 要件: FR16（判断の記録の面を正本から逐語で生成する・要件書 第 1.42 版）。FR16 の規範文は、面の中身を正本の欄（見出し・状態・日付・問題・決定・案の表・根拠の id・撤退条件・**改訂の一覧**・承認欄・平易文・帰結）から逐語で組むと書く。本便は、判断の記録の欄の決まりが便 101（ADR-13 決定 (14)）で足した任意の改訂の欄 revises を、面の改訂の一覧に入れる。規範文は変えない。契約表の行の req は FR16 の 1 つ（main に在る id）。
- 条: P-6.1（人が読むページは正本から逐語で生成する＝正本に在る欄を面が黙って落とさない）/ P-2.4（部品の閉じた一覧＝新しい部品を足さず、章 05 と根拠の章が既に使う字面だけで組む）/ P-4.2（行き先の無い相手は「まだ分からない」の印で表に出す）/ P-5.1（改訂の向きの名札は型付きの表で持つ）/ P-6.2（生成物を手で直さない＝面の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（凍結 anchor は歯の側の手書きの字で持つ）。
- 出所: 天井の 34 周目（2026-09-25・一括 20 の枝の上）の読みやすさの所見 **F-1**（重さ 直す・場所 ADR-22 の平易文）。平易文は「この判断の改訂の欄」へ案内するが、面の表紙の札は「改訂 0 件」、05 章の頭は「条文の改訂なし」と出し、改訂の欄の行はどこにも描かれない。同じ形は revises を持つ ADR-13・14・16・18・20・21・22 の面に共通。一括 20 の仕分け（`docs/design/batch20-triage.md`）の便の候補 **B-4**。台帳の控え **f2-648.207**。改訂の欄を足した便 101 の設計ノートは「id の改訂を面へ出すかは見た目の裁定を伴うので面の便へ回す」と書いており（delivery-101.md の範囲の節）、本便がその面の便に当たる。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ej` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 6 本（書き換える 4 本 + 本文が変わらない verify の scope 2 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-25・独立の検証 d137-verify.md〔条件付き支持・blocking 0・数値は全部一致〕の文面 6 点）: (e) の 4 の build の命令の字・(f) の余地の表に便の後の余地・§0 の並行の便との重なりを今の事実に・(e) の 5 に ADR-7・(c) の 1 の「数える」を「確かめる」・(e) の 4 に平易文の字と見出しの字の不一致の断り。数値・write-set・verify・done は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 6 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は一括 20 の取り込み（本流 06cd100）。**base = main 06cd100。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 135（枝 docs/d135・1f8e0ac）と共に持つ file は `crates/folio/tests/badge.rs` と `crates/folio/tests/site.rs` だけで、どちらの便も本文を変えない（verify の scope）。便 136（枝 docs/d136・7477f64）と共に持つ file は `crates/folio/tests/site.rs` だけで、便 136 はそこへ歯を 1 本足す。どちらの便も判断の記録の面（`crates/folio/src/face_adr.rs` と面の凍結 anchor 2 本）は変えない。共通の検証は同時に撃たない逐次で受け付ける（`crates/folio/tests/sheet.rs` の一時 dir の衝突も避ける）。

## 1. 設計

### (a) いま起きていること（実測・base 06cd100）

1. **欄の意味（欄の決まりの実測）。** 判断の記録の欄の決まり（`design-intent/adr/schema.yaml`・生成区間の正は床の定数 `crates/folio/src/floor_adr.rs`）は、改訂に当たる任意の欄を 3 種に分ける。
   - **amends = 条文の改訂。** 憲法の条の欄を改める判断が持ち、凍結 anchor との差分と 1 対 1 に消し込まれ、条の側の amended_by と双方向。
   - **revises = 判断の記録どうしの改訂。** 発効した判断が生きたまま、その決定の範囲が別の判断で変わるときだけに使う（欄の決まりの revises_note の target の項）。各行は target（相手の ADR-n）・decision（相手の決定の番号の字・例 (8)）・kind（向き・narrow = 狭める／widen = 広げる・床は値域だけを見る）・summary（何をどう変えたかの 1 文）の 4 欄で、改訂する側だけが持つ（片側・revises_note の reverse の項）。床は便 101 の check_revises（`crates/folio/src/adr.rs`）で欄の集合・非空・target の形・kind の値域・対の一意を見る。
   - **supersedes / superseded_by = 丸ごとの置き換え。** 廃止の記録と後継の双方向。面は既に章 05 の置き換えた判断・後継の判断の行と、廃止の状態の行で描いている（本便は変えない）。
2. **面は revises を読まない。** 判断の記録の面の生成器 `crates/folio/src/face_adr.rs` は、字 revises を 1 か所も持たない（起草役の `grep` の実測）。表紙の札は名札「改訂」で amends の件数だけを数え、章 05 は amends が空なら「条文の改訂なし」の段落 1 行を出し、機械のための面の一覧は amends の件数だけを持つ。便 101 の設計ノートが書いたとおり、面は自分が読む欄だけを描くので、欄が増えても落ちない代わりに黙って描かない。
3. **実の正本の分布（参考値）。** 判断の記録 23 本のうち、revises を持つのは 7 本で行は通して 16 行、amends が空でないのは 4 本で行は通して 12 行、どちらも空は 12 本。両方を持つ記録は 0 本。

| 形 | 記録 | 行 |
| --- | --- | ---: |
| revises だけ | ADR-13（1）・14（1）・16（2）・18（2）・20（3）・21（5）・22（2） | 16 |
| amends だけ | ADR-11（5）・12（3）・17（2）・23（2） | 12 |
| どちらも空 | ADR-1〜10・15・19 | 0 |

4. **所見の実例。** base の binary で組んだ ADR-22 の面は、表紙の札が「改訂 0 件」、章 05 の頭が「条文の改訂なし」で、改訂の欄の 2 行（ADR-3 の決定 (8) を狭める・ADR-16 の決定 (1) を狭める）は面のどこにも無い。平易文の案内の先が面に無い。
5. **base の歯（参考値）。** workspace の nextest 868 / 868・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file。`git grep -n 'f137_' -- crates` は 0 件。歯の file `crates/folio/tests/face_adr.rs` は 32 本。

### (b) 直す先 — face_adr.rs の 4 か所（部品は足さない）

1. **名札の表（β）。** 改訂の向きの表 REVISE（narrow → 狭める・widen → 広げる・床の定数 REVISE_KIND〔`crates/folio/src/floor_adr.rs`〕の値域と同じ順）と、2 つの欄の名札の定数（AMENDS_LABEL = 条文の改訂・REVISES_LABEL = 判断の記録の改訂）を足す。表に無い向きは既存の表引き（案の判定と同じ口）が Err を返し、面を導出しない（まだ分からない・終了コード 2）。任意の一覧の欄を読む小さな関数 entries を足し、amends と revises の両方をそれで読む。
2. **表紙の札を 2 つに分ける。** 名札「改訂」の札 1 つを、名札「条文の改訂」（amends の件数）と名札「判断の記録の改訂」（revises の件数）の 2 つの札にする。札の部品と並びの位置（根拠の札の後・撤退条件の札の前）は変えない。0 件でも 2 つとも出す（23 本の面で札の並びを揃える）。
3. **章 05 の頭。** 空の欄の断りは 1 段落にまとめて章の頭に置く＝両方が空なら「条文の改訂なし・判断の記録の改訂なし」、片方だけが空ならその片方の断り（例「判断の記録の改訂なし」）。中身の在る欄は、名札と同じ字の h3 見出し（条文の改訂・判断の記録の改訂）の下に一覧で並べる。amends の一覧の行の字は 1 字も変えない（見出しが 1 つ付くだけ）。revises の一覧は amends の一覧の後・帰結の見出しの前に置く。
4. **revises の 1 行。** 相手の id の面へのリンク（根拠の章と同じ id_link の口・リンクの class は既存の xref・行き先の file が無い相手は「ADR-n（まだ分からない）」の印）・字「の決定」・decision の逐語・字「を」・向きの名札・コロン・summary の逐語（escape する）。ADR-22 の 1 行目は「ADR-3 の決定 (8) を狭める: 「folio2 の M1 で…」」と読める。行の順は正本の順。
5. **機械のための面。** 脚の一覧（amends の件数）の後に revises の件数を足す（0 件でも出す）。
6. **変えないもの。** 章の数と名と帯・目次・部品の一覧（PARTS の 9 種）・名札の class・根拠の章・置き換えの行・承認欄・図の章・ほかの面の生成器・床・設計文書。改訂される側（相手の記録の面）に来歴を出すことはしない（欄の決まりが片側と定める・(d) の 4）。

### (c) 歯（関数名 f137_・base で 0 件）

1. **f137_revise_kind_labels_follow_the_floor_enum（単体の歯・`crates/folio/src/face_adr.rs` の既存の tests の区間）。** 表 REVISE の鍵の並びが床の定数 REVISE_KIND（`crates/folio/src/floor_adr.rs`）と等しいこと、表の 2 行と 2 つの名札の字が (b) の 1 の字と等しいこと（凍結の針・歯の中の手で写した字）、表に無い向き shrink の表引きの Err の字を確かめる。**base では表と定数が無く、組み立てが通らない＝RED。**
2. **f137_revises_rows_show_the_link_decision_kind_and_summary（`crates/folio/tests/face_adr.rs`・binary 経由）。** fixture の写しの ADR-2 に、歯の中で手書きした改訂の欄 2 行（相手 ADR-1・決定 (2)・狭める・summary に山括弧を含む字／相手 ADR-9〔写しに無い〕・決定 (1)・広げる）を足して面を書く。終了コード 0・表紙の札が条文の改訂 0 件と判断の記録の改訂 2 件で名札「改訂」だけの札が無い・章 05 の頭の断りは「条文の改訂なし」がちょうど 1 つで判断の記録の改訂の断りが無い・h3「判断の記録の改訂」がちょうど 1 つ・1 行目は ADR-1 の面へのリンクと決定と向きと escape した summary の逐語・2 行目は「ADR-9（まだ分からない）」の印・2 行は正本の順で章 05 の中（承認欄の前）・機械のための面が amends 0 と revises 2 を数える。**base では行も札も無い＝RED。**
3. **f137_a_record_without_revisions_says_both_are_none（同）。** 変異なしの写し（amends は空・revises は無い）の面で、2 つの札が 0 件・章 05 の断りが「条文の改訂なし・判断の記録の改訂なし」のちょうど 1 段落・h3 が無い・機械のための面が 0 と 0。**base では断りが「条文の改訂なし」だけ＝RED。**
4. **f137_amends_go_under_their_own_heading（同）。** 写しの amends を 1 行にした面で、札が条文の改訂 1 件・判断の記録の改訂 0 件・断りが「判断の記録の改訂なし」・h3「条文の改訂」の後に amends の行が逐語で在る。**base では h3 と札が無い＝RED。**
5. **f137_unknown_when_a_revise_kind_is_outside_the_table（同）。** 向きを値域の外（shrink）にした写しで、終了コード 2・まだ分からない・字「改訂の向き の表に無い値「shrink」」・面を書かない。**base では欄を読まずに 0 で書く＝RED。**
6. **f137_real_sources_draw_every_revises_row（同・実の正本の census）。** 実の判断の記録の全本（23 本）について、yaml-rust2 で正本を直に読み、2 つの札の件数・h3「判断の記録の改訂」の有無・断りの有無・revises の各行の逐語（相手の面への href・decision と summary は歯の側の escape・向きは歯の側の手で写した名札）がちょうど 1 回在ることを数え、描いた行の総数が 0 でないこと（空回りしない）を数える。**base では札も行も無い＝RED。**

fixture は新しい file を足さず、既存の写しの ADR-2 を歯の中で書き換える（歯の file の既存の変異の口 mutated と同じ形）。

### (d) 採らなかった形

1. **札を 1 つのまま両方を数える（例 札「改訂」に「条文 n 件・判断の記録 m 件」）。** 23 本の面が変わるのは同じだが、札の名札と章 05 の h3 の字が揃わず、平易文が「改訂の欄」へ案内する先を札の名札から章の見出しへ字で辿れない。2 つの欄に 1 つずつの名札を持つほうが、欄の決まりの 2 つの欄と 1 対 1 になる。
2. **revises が 1 件以上の面だけに札を足す（図の札と同じ形）。** revises を持たない 16 本の面は byte で変わらずに済むが、名札「改訂」の札が面によって意味（amends だけか）と並びが変わり、揃いを崩す（憲法の前文の順位「網羅より揃い」）。0 件の面でも 2 つの欄のどちらも無いことを読み手が確かめられるほうを採る。
3. **改訂の欄に新しい部品（行の部品）を足す。** 部品目録の閉じた一覧を広げると判断の記録が前置になる（P-2.4）。章 05 が既に使う h3・一覧・段落と、根拠の章が既に使うリンク（xref）で足りる。
4. **改訂される側（相手の面）に「この判断を改めた記録」を出す。** 欄の決まりは revises を片側と定め（revises_note の reverse の項）、相手の面に出すには全記録の走査が要る。本便の所見（案内の先が改訂する側の面に無い）は改訂する側だけで解ける。要るなら別の便。
5. **歯の fixture を新しい file（`tests/fixtures/face/adr/` の下の 3 本目）で持つ。** 写しの集合を変えると凍結 anchor の組み立ての入力が増える。歯の file の既存の変異の口（mutated・写しの ADR-2 を書き換える）で最小の手書きの 2 行を持てる。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **落ちる既存の歯は 5 本で、どれも本便の write-set で直す（起草役の実測）。** 実装だけを base に当てた写し（歯の file と凍結 anchor は base のまま）で、workspace の nextest は 869 本のうち 5 本が落ちる。

| 歯 | 落ちる理由 | 直し方 |
| --- | --- | --- |
| face_adr の face_adr_write_matches_the_frozen_fixture | 凍結 anchor expected-adr.html と byte が違う | anchor の 3 か所（(e) の 2） |
| face_adr の face_adr_figure_chapter_is_the_only_difference_from_the_figureless_face | 同じ anchor を読む | 同上（本文は変えない） |
| site の site_write_matches_the_frozen_fixture | 凍結 anchor expected-site-adr-2.html と byte が違う | anchor の 3 か所（(e) の 2） |
| badge の badge_faces_without_the_mark_match_the_seven_frozen_fixtures | 上の 2 本の anchor を読む | 同上（本文は変えない） |
| face_adr の face_adr_census_on_the_real_sources_counts_and_verbatims | リンクの数を根拠と図の refs だけで数える | 数える式に revises の行数を足す（2 か所・ほかの主張は変えない） |

   本便の差分の全部を base に当てた写しで、workspace の nextest は 874 / 874（base 868 + 単体 1 + binary 5）・clippy 0 警告・床 4 本 rc 0・`crates/folio/tests/face_adr.rs` 37 / 37・site 12 / 12・badge 16 / 16。
2. **動く凍結 anchor は 2 本で、どちらも同じ 3 か所だけ。** `tests/fixtures/face/expected-adr.html` と `tests/fixtures/face/expected-site-adr-2.html`（写しの ADR-2 は amends が空で revises が無い）の、表紙の札の 1 行（札「改訂 0 件」を札「条文の改訂 0 件」と札「判断の記録の改訂 0 件」の 2 行に）・章 05 の断りの 1 行（「条文の改訂なし」を「条文の改訂なし・判断の記録の改訂なし」に）・機械のための面の一覧（amends 0 の後に revises 0）。起草役は anchor を手で書き換え、生成器の出力と byte で一致した。ほかの面の凍結 anchor・床の凍結の土台・天井の束の凍結 anchor は 1 byte も変えない。
3. **突然変異（起草役の実測・本便を当てた写しの face_adr.rs の実装だけを 1 通りずつ変える）。**

| 変異 | 単体の歯 f137_ | binary の f137_（5 本） |
| --- | --- | --- |
| M1 revises の行を描かない | 緑 | 3 本落ちる（行・向きの表の外・実の正本） |
| M2 表紙の札を 1 つ（改訂 = amends）に戻す | 緑 | 4 本落ちる |
| M3 向きの名札を入れ替える | 落ちる | 2 本落ちる（行・実の正本） |
| M4 相手をリンクにせず id の字だけにする | 緑 | 2 本落ちる（行・実の正本） |
| M5 空の断りを条文の改訂だけに戻す | 緑 | 3 本落ちる（断り・amends の見出し・実の正本） |

4. **folio2 自身の面の変化（起草役の実測）。** `folio build --dir design-intent --out <置き場> --write`（起草役は写しの外の一時 dir を置き場にした）の出力は 30 file のまま。変わるのは判断の記録の面 23 本（adr-1〜adr-23）で、ほかの 7 file は 1 byte も変わらない。

| 面 | 変わる行（base との diff の行数・参考値） |
| --- | --- |
| どちらも空の 12 本 | 札 1 → 2 行・断りの 1 行・機械のための面 1 行（7） |
| amends だけの 4 本 | 札 1 → 2 行・断り「判断の記録の改訂なし」と h3「条文の改訂」の 2 行を足す・機械のための面 1 行（7） |
| revises だけの 7 本 | 札 1 → 2 行・h3 と一覧の枠と行（行数 + 3）を足す・機械のための面 1 行（9〜13） |

   ADR-22 の平易文は案内の先を「この判断の改訂の欄」と呼ぶが、面の見出しと札の字は「判断の記録の改訂」で、字は一致しない（章 05「改訂と帰結」の中の見出しとして辿れる）。正本の平易文は本便では触らず、字を揃えるかは次の天井の周の読みやすさの観点が読む。
   章 05 の頭の「条文の改訂なし」は、revises だけの 7 本では字が変わらない（条文の改訂は本当に無い）。その下に判断の記録の改訂の見出しと行が付く。天井の束の面（faces）の要約値は 23 本の面の分だけ変わるが、面は周の引き金ではない（引き金は設計文書の正本の規範の欄）。
5. **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便の数え）に当たらない。** 本便は正本に在る欄（revises）を面が黙って落としていたのを、要件 FR16 の「改訂の一覧を逐語で」と、判断の記録の面を定めた ADR-7（決定 (2) が面の節に改訂の一覧を挙げ、決定 (3) が中身を正本の欄から逐語で組むと定める）に合わせて描く忠実さの直しで、見た目（部品・色・余白・名札の class・章の並び）は直さない。足す字面はどれも章 05 と根拠の章が既に使う形（h3・一覧・段落・xref のリンク）で、表紙の札は既存の札の部品の数が 1 つ増えるだけである。仮に表紙の札を分けたことを見た目の直しと数えても、判断の記録の面の見た目の直しは便 27 の 1 本目に次ぐ 2 本目で、上限（規則の表の行 R-7 = 2 回）を超えないので問いの時機に当たらない。面の型も増えない（撤退条件 ① にも当たらない）。
6. **持ち主の面の承認（walk）。** 判断の記録の面は 2026-09-18 に持ち主の walk の承認を得ている。本便は章と部品を変えず、欄の行を足すだけなので、起草役は walk の取り直しを要しないと読む。便 101 の設計ノートは「id の改訂を面へ出すかは見た目の裁定を伴う」と書いたので、walk を求めるかは席が決める（(i) の 2）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 6 本とも印なし（書き換える 4 本 = `crates/folio/src/face_adr.rs`・`crates/folio/tests/face_adr.rs`・`tests/fixtures/face/expected-adr.html`・`tests/fixtures/face/expected-site-adr-2.html`／本文不変の 2 本 = `crates/folio/tests/badge.rs`・`crates/folio/tests/site.rs`・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_adr.rs` | 856 | 644 | 914（+58） | 586 |

   余地は S の見積 100 を超える。歯の file は src の外なので余地を測らない（参考値 1,102 行 → 1,263 行）。
3. **size は S。**
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f137_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_adr f137_` = (c) の 2〜6（5 本）。
   3. `cargo nextest run -p folio --test face_adr` = 判断の記録の面の歯の全部（凍結 anchor との byte 一致・実の正本の census・図の章の差を含む・参考値 37 本）。
   4. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   5. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_adr・badge・site の 3 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f137_ を関数名に持つ src は `crates/folio/src/face_adr.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。面の生成器の便 135・136 とは逐次に受け付ける（§0 の 並行の便との重なり）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d137-draft.md`、模擬の差分と script は同じ dir の d137-scripts（repo には入れない）。

1. 模擬: 差分 c137.patch を base に当て、run-137.sh（workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数・f137_ の単体と binary・face_adr と site と badge の全部）。base との面の差は、base と写しの build の出力を `diff -r` で比べる。
2. RED: 歯だけの差分 r137-teeth.patch（単体の歯の 1 本と歯の file の差分）を base に当てると、単体の歯は組み立てが通らず、binary の f137_ の 5 本と census が落ちる。実装だけ（歯の file と凍結 anchor を除く）を当てると (e) の 1 の 5 本が落ちる。
3. 突然変異: mut-137.sh（(e) の 3）。
4. 余地: lines-137.py と lines-137.awk（同じ式の 2 実装）を repo の根で write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 改訂される側の面の来歴（(d) の 4）。入口の棚の判断の記録のカード（便の候補 B-1）。平易文と設計文書の字（ADR-22 と ADR-21 の平易文の案内の字は変えない・案内の先が面に出来る）。ほかの面の生成器。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 向き（narrow / widen）が本当かは床も面も確かめない（欄の決まりの revises_note の kind の項＝人が読む）。面の見た目が持ち主に受け入れられるかは walk でしか分からない（(e) の 6）。天井の次の周で、読みやすさの観点が F-1 を解けたと読むかは周の結果でしか分からない。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 5 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に `folio build` の出力で判断の記録の面 23 本のほかの file が 1 byte でも変わるか、folio2 自身の床 4 本の結果が変わったら、止めて席へ返す。(3) 受付の時点の main で、revises を持つ記録の本数か、欄の決まりの revises の 4 欄と向きの値域（narrow・widen）が base と違えば、(e) の 4 の表と歯の名札を数え直してから運ぶ（値域が変わったら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/face_adr.rs` の名札の表と 2 つの名札・entries・表紙の札・章 05 の頭と 2 つの一覧・機械のための面の revises の件数・単体の歯 1 本・頭の注。`crates/folio/tests/face_adr.rs` の歯 5 本と census の数える式 2 か所と頭の注。凍結 anchor 2 本の 3 か所ずつ。
- 入れない: 部品目録・名札の class・章の数と帯・目次・根拠の章・置き換えの行・承認欄・図の章・ほかの面・床・設計文書・改訂される側の面・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| labels | 改訂の向きの表と 2 つの名札 | `crates/folio/src/face_adr.rs` の REVISE・AMENDS_LABEL・REVISES_LABEL |
| cover | 表紙の札 | 条文の改訂 n 件・判断の記録の改訂 m 件 |
| chapter | 章 05 の改訂の一覧 | 空の断りの 1 段落・h3 の下の amends と revises の一覧 |
| anchor | 面の凍結 anchor | `tests/fixtures/face/expected-adr.html`・`tests/fixtures/face/expected-site-adr-2.html` の 3 か所ずつ |
| teeth | 歯 | 単体の f137_ 1 本・face_adr.rs の f137_ 5 本と census の式 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 一括 20 の取り込み（本流 06cd100）。
- 並行の便: 便 135・136（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.207）を閉じる。持ち主の walk を求めるかを決める（§1 (e) の 6）。次の天井の周で読みやすさの F-1 が解けたかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ej"
title = "一括 20 の B-4（天井の 34 周目 読みやすさ F-1・台帳 f2-648.207）: 判断の記録の面の生成器（crates/folio/src/face_adr.rs）が、正本の任意の改訂の欄 revises（判断の記録どうしの改訂・便 101）を描かず表紙の札が改訂 0 件・章 05 が条文の改訂なしとだけ出す問題を直す。表紙の札を条文の改訂（amends の件数）と判断の記録の改訂（revises の件数）の 2 つに分け（0 件でも出す）、章 05 の頭に空の欄の断りを 1 段落（両方が空なら 条文の改訂なし・判断の記録の改訂なし）にまとめ、中身の在る欄は名札と同じ字の h3 の下に並べ、revises の各行に相手の面へのリンク（行き先が無ければ まだ分からない の印）・決定の番号・向きの名札（narrow 狭める・widen 広げる・表の外は面を導出しない）・summary の逐語を出し、機械のための面に revises の件数を足す。部品・名札の class・章・ほかの面・床・設計文書は変えない。凍結 anchor 2 本（expected-adr.html・expected-site-adr-2.html）の 3 か所ずつを手で直す。歯は単体の f137_ 1 本と face_adr の f137_ 5 本と census の数える式"
req = ["FR16"]
section = "1"
write-set = ["crates/folio/src/face_adr.rs", "crates/folio/tests/face_adr.rs", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-site-adr-2.html", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --bin folio f137_", "cargo nextest run -p folio --test face_adr f137_", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f137_（改訂の向きの表の鍵の並びが床の定数 REVISE_KIND と等しく、表と 2 つの名札の字が §1 (b) の 1 のとおりで、表の外の向きの表引きが Err）が緑、face_adr の f137_ の 5 本（写しに足した revises 2 行が相手の面へのリンクか まだ分からない の印と決定と向きと escape した summary の逐語で章 05 に正本の順で並び表紙の札が条文の改訂 0 件と判断の記録の改訂 2 件・改訂の無い記録の断りが 条文の改訂なし・判断の記録の改訂なし の 1 段落・amends の行が h3 条文の改訂 の下・向きが表の外なら終了コード 2 で面を書かない・実の正本の全本で札の件数と revises の全行の逐語）が緑、face_adr の歯の全部（凍結 anchor expected-adr.html との byte 一致と実の正本の census を含む）が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor（expected-site-adr-2.html を含む）との byte 一致が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返す"
<!-- contracts:end -->
