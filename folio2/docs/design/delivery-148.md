# 設計: 便 148 — 判断の記録の面に、ほかの記録から受けた改訂の逆向きの導線を描く（狭められた側・広げられた側の面から改訂した記録へ辿れる）

- 要件: FR16（判断の記録の面を正本から逐語で生成する・要件書 第 1.45 版）。FR16 の規範文は、面の中身を正本の欄（見出し・状態・日付・問題・決定・案の表・根拠の id・撤退条件・**改訂の一覧**・承認欄・平易文・帰結）から逐語で組み、廃止の記録には後継の id への導線を出すと書く。本便は、改訂の欄 revises（判断の記録どうしの改訂・便 101）を改訂する側の面だけでなく、改訂される側の面にも逆向きの導線として出す。行の字はどれも改訂する側の正本の欄の逐語で、規範文は変えない。契約表の行の req は FR16 の 1 つ（main に在る id）。
- 条: P-6.3（同じ内容を 2 つの面が持つときは一方を正本とし、他方は導出する＝正本の revises は改訂する側だけが持ち、改訂される側の面の行は生成のたびに走査で導出する）/ P-6.1（人が読むページは正本から逐語で生成する）/ P-2.4（部品の閉じた一覧＝部品も class も足さない）/ P-4.1（走査が読めなかった結果を「改訂なし」として扱わない）/ P-5.1（名札と読む状態の一覧は型付きの定数で持つ）/ P-10.1（期待の字は歯の側の手書きで持つ）。
- 出所: 天井の 40 周目（2026-09-26）の読みやすさの所見 **F-3**（重さ 直す・場所 ADR-18 の注）。ADR-18 の面を読む人は、決定 (5) が ADR-24 で狭められたことを注の地の文でしか知れず、見出しの下・要約の欄・章 05 改訂と帰結のどこにも案内が無い。ADR-24 の面から ADR-18 へは辿れるが、逆向きが切れている。台帳の控え **f2-648.222**。便 137 の契約 (d) の 4 は「改訂される側（相手の面）に来歴を出す」を採らず「要るなら別の便」とし、本便がその便に当たる。依頼は席から起草役へ（2026-09-26）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eu` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 4 本（書き換える 2 本 + 本文が変わらない verify の scope 2 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 改訂 b（2026-09-26・独立の検証 d148-verify.md〔支持・一致 25・blocking 0〕の非 blocking 3 点）: (e) の 6 の表紙の札の並びを変えた便の数え（便 33・71・137・本便）・(c) の 4 の歯に読めない記録と status の無い記録と target が id の形でない行の 3 通りを足した（(b) の 2 の Err の条件を全部縛る）・(e) の 5 の変異の表に M8a・M12・M13 と縛っていない M10。実装・write-set・verify の行・size・余地は変えない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 4 本を base の binary（写しの組み立て）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便を当てた写しの binary でも同じ。
- 前の便: 前提の着地は無い。**base = main c7cfa3a（一括 23 と ADR-24 の発効・40 周目の印の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草役は `git archive c7cfa3a` の写しを組み立てて測った（本流の作業ツリーの binary は使っていない）。
- 並行の便との重なり: base の時点で、開いた PR は無く、契約の枝のうち判断の記録の面（`crates/folio/src/face_adr.rs`）と歯の file `crates/folio/tests/face_adr.rs` を書き換える便も無い（起草役の実測）。共通の検証は同時に撃たない逐次を勧める。

## 1. 設計

### (a) いま起きていること（実測・base main c7cfa3a・数は参考値）

1. **欄の意味。** 判断の記録の欄の決まり（`design-intent/adr/schema.yaml` の revises_note）は、改訂の欄 revises を「発効した判断が生きたまま、その決定の範囲が別の判断で変わるとき」だけに使い、各行を target（相手の ADR-n）・decision（相手の決定の番号の字）・kind（narrow = 狭める／widen = 広げる）・summary（何をどう変えたかの 1 文）の 4 欄で持つと定める。改訂される側に来歴の欄は置かない（reverse の項・片側だけ）。ADR-21 の決定 (6) も同じ作法をとる。
2. **面は改訂する側だけに行を描く。** 判断の記録の面の生成器 `crates/folio/src/face_adr.rs` は、便 137 から章 05 に、その記録自身の revises の行（相手の面へのリンク・決定・向きの名札・summary）を描く。改訂される側の面は、自分を相手にするほかの記録の revises を読まない。表紙の札「判断の記録の改訂 n 件」も、その記録が改訂した件数だけを数える。
3. **実の正本の分布。** 判断の記録は 24 本で、どれも発効している。revises を持つのは 8 本（ADR-13・14・16・18・20・21・22・24）で、行は合わせて 17 行。改訂される側は 6 本である。

| 改訂される側 | 改訂する側と相手の決定（正本の順） | 行 |
| --- | --- | ---: |
| ADR-2 | ADR-16 (3) 広げる | 1 |
| ADR-3 | ADR-16 (7) 広げる・ADR-21 (7) 狭める・ADR-22 (8) 狭める | 3 |
| ADR-8 | ADR-13 (4) 狭める・ADR-18 (4) 狭める | 2 |
| ADR-13 | ADR-14 (1) 狭める・ADR-18 (8) 狭める・ADR-20 (8) 広げる | 3 |
| ADR-16 | ADR-21 (1)(3)(6)(7) 狭める・ADR-22 (1) 狭める | 5 |
| ADR-18 | ADR-20 (1) 広げる・ADR-20 (2) 狭める・ADR-24 (5) 狭める | 3 |

   改訂する側の 8 本は、どれも発効した記録である。
4. **所見の実例。** base の binary で組んだ ADR-18 の面は、adr-24.html へのリンクを 1 つも持たない。ADR-16 の面にも、ADR-21 と ADR-22 へのリンクが無い。
5. **既に全記録を読む口。** 面の骨格（frame）は前後の記録へのリンクを組むために、record_ids で `adr/` の file 名を数の順に並べ、record_title で各記録の題を読む。つまり、1 枚の面を組むたびに全記録を読んでいる。
6. **base の歯（参考値）。** workspace の nextest 938 / 938・clippy 0 警告・床 4 本 rc 0・`folio build --dir design-intent --out <置き場> --write` の出力 31 file（1,997,947 byte）。`git grep -n 'f148_' -- crates` は 0 件。歯の file `crates/folio/tests/face_adr.rs` は 39 本。

### (b) 直す先 — face_adr.rs の 1 file（部品も class も足さない）

1. **型付きの定数（β）。** 名札 REVISED_BY_LABEL を足す（字は **ほかの判断の記録による改訂**）。表紙の札と章 05 の h3 がこの名札を使う。読む改訂する側の状態の一覧 REVISED_BY_STATUS も足す（**accepted だけ**）。向きの名札は、便 137 の表 REVISE（narrow → 狭める・widen → 広げる）をそのまま使い、新しい表は作らない。
2. **走査の口 revised_by（dir と id を受ける）。** record_ids の順（改訂する側の id の数の順）に、自分を除く全記録を読む。改訂する側の状態が REVISED_BY_STATUS に無ければ、その記録の revises は読まない。発効していれば revises の各行を正本の順に見て、target がこの記録の id の行だけを集める。1 行の中身は、改訂する側の id・decision の逐語・向きの名札・summary の逐語である。記録が読めない・状態の欄が無い・revises が一覧でない・target が id の形でない・向きが表 REVISE の外なら、Err を返す。そのときこの記録の面を導出しない（まだ分からない・終了コード 2）。読めなかった記録がこの記録を改訂しているかは分からないので、「改訂されていない」と黙って出さない（P-4.1）。
3. **章 05 の一覧。** 集めた行が 1 つ以上なら、その記録自身の revises の一覧の後、帰結の見出し（この判断で変わること）の前に、h3「ほかの判断の記録による改訂」と一覧を置く。1 行は次の形で読める。
   - 改訂する側の面へのリンク（根拠の章と同じ xref のリンク・既存の link_text の口）
   - 字「がこの判断の決定」・decision の逐語・字「を」・向きの名札・コロン・summary の逐語
   - 例（ADR-18 の面の 3 行目）: ADR-24 がこの判断の決定 (5) を狭める: 「便の書き換える file の一覧に…」
   
   章 05 の頭の空の断り（条文の改訂なし・判断の記録の改訂なし）は変えない。断りはその記録の正本の欄についての字で、逆向きの行は正本の欄ではないからである。0 行なら h3 も一覧も出さない。
4. **表紙の札。** 集めた行が 1 つ以上のときだけ、札「判断の記録の改訂」の直後に札「ほかの判断の記録による改訂 n 件」を 1 つ足す（既存の札の部品 meta_span・図の札と同じ 1 つ以上のときだけの形）。既存の札「判断の記録の改訂」は、その記録が改訂した件数のまま数える（数え方は変えない）。**別の札を足す理由:** ADR-18 の表紙の札は「判断の記録の改訂 2 件」で、これは ADR-18 が改訂した件数である。ADR-24 に狭められたことは表紙に出ず、読み手は 2 件を「受けた改訂」と取り違えうる。F-3 は見出しの下と要約の欄にも案内が無いと書くので、表紙の段で向きの違う 2 つの札を並べる。0 件の面に札を出さないのは、改訂されていない 18 本の面を base と byte で同じに保つためである（(d) の 2）。
5. **変えないもの。** 機械のための面の一覧（脚の dl）には件数を足さない。脚の一覧は正本の欄（id・status・date・basis・amends・revises・figures）の写しで、逆向きの行は正本の欄ではないからである。ほかにも次のものは変えない。
   - 章の数と名と帯・目次・部品の一覧（PARTS の 9 種）・名札の class・様式の定義
   - 章 05 の頭の断り・その記録自身の amends と revises の一覧・置き換えの行・承認欄・図の章
   - 入口の面とほかの面の生成器・床・設計文書・凍結 anchor
   - 正本の欄の決まり（改訂される側に欄を足さない）

### (c) 歯（関数名 f148_・base で 0 件）

1. **f148_revised_by_label_and_status_are_frozen_needles（単体の歯・`crates/folio/src/face_adr.rs` の既存の tests の区間 face_adr_tests）。** 名札の字（ほかの判断の記録による改訂）と、読む状態の一覧（accepted だけ）を確かめる。名札が正本の 2 つの欄の名札（条文の改訂・判断の記録の改訂）の字を含まないことも確かめる。含むと表紙の札と h3 を字で取り違える。**base では定数が無く、組み立てが通らない＝RED。**
2. **f148_revised_by_rows_link_back_to_the_reviser（`crates/folio/tests/face_adr.rs`・binary 経由）。** 面の fixture の写しの ADR-1 を改訂する側にする。歯の中で状態を発効にし、revises を手書きで 3 行足す（ADR-2 の決定 (2) を狭める〔summary に山括弧〕・ADR-9 の決定 (1) を広げる・ADR-2 の決定 (1) を広げる）。そのうえで ADR-2 の面を組み、次を確かめる。
   - 終了コード 0。表紙の札「ほかの判断の記録による改訂 2 件」がちょうど 1 つ。
   - 正本の欄の札（条文の改訂 0 件・判断の記録の改訂 0 件）と、章 05 の断り（条文の改訂なし・判断の記録の改訂なし）と、機械のための面（amends 0・revises 0・逆向きの欄なし）は変わらない。
   - 章 05 の h3 の下に、ADR-1 の面へのリンクの 2 行が正本の順で逐語に並ぶ（summary は歯の側の escape）。2 行は帰結の見出しの前に在り、相手の違う行（ADR-9）は出ない。
   - 札の 1 行と h3 の一覧を除くと、改訂されていない写しの面と byte で同じ。
   **base では札も行も無い＝RED。**
3. **f148_only_an_accepted_reviser_is_read（同）。** 同じ 3 行を、ADR-1 の状態を 3 通り（発効・提案中・廃止）に替えて足す。発効なら札が 2 件で出る。提案中と廃止なら、ADR-2 の面は改訂されていない写しの面と byte で同じになる。**base では発効の写しに札が無い＝RED。**
4. **f148_unknown_when_a_reviser_row_cannot_be_read（同）。** 写しの ADR-1 を、(b) の 2 が Err と書く条件の 5 通りに書き換える。
   - 発効で、向きが表の外（shrink）の行を持つ。断りの字に「改訂の向き の表に無い値「shrink」」を含む。
   - 発効で、revises が一覧でない字。
   - 発効で、target が id の形でない行（adr two）を持つ。断りの字に `adr/ADR-1.yaml.revises[0].target` を含む。
   - 状態の欄が無く、ADR-2 を相手にする行を持つ。断りの字に「欄 status が無い」を含む。
   - yaml として読めない（状態の行が閉じない括弧）。断りの字に「adr/ADR-1.yaml: 読めない」を含む。
   
   どれも ADR-2 の面が終了コード 2（まだ分からない）になり、断りの字に `adr/ADR-1.yaml` を含み、面は書かない。**base ではほかの記録を読まずに 0 で書く＝RED。**
5. **f148_real_sources_draw_every_revised_by_row（同・実の正本の census）。** 実の判断の記録の全本（24 本）で次を確かめる。
   - 歯の側の手書きの読みで、逆向きの行を集める。yaml-rust2 で各記録を直に読み、発効の記録の revises を id の数の順・正本の順に集める。
   - 各面の表紙の逆向きの札が、行が在るときだけ n 件でちょうど 1 つ在る。章 05 の h3 も、行が在るときだけ 1 つ在る。
   - 各行が章 05 に逐語でちょうど 1 回、集めた順に在る。リンクの href・decision と summary の escape・向きの名札は、歯の側で手書きする。
   - 描いた行の総数が 0 でない（空回りしない）。
   **base では札も行も無い＝RED。**
6. **f148_adr18_and_adr16_link_back_to_their_revisers（同・所見の実例）。** 実の正本の ADR-18 の面の章 05 に、ADR-24 の面へのリンクと「がこの判断の決定 (5) を狭める:」の行が在る。ADR-16 の面の章 05 に、ADR-21 と ADR-22 の面へのリンクの行が在る。**base では無い＝RED。**

fixture は新しい file を足さない。写しの ADR-1（id と状態だけの最小の手書き・`tests/fixtures/face/adr/ADR-1.yaml`）を歯の中で書き換える。既存の変異の口 mutated が写しの ADR-2 を書き換えるのと同じ形である（helper は revised_face・reviser・revised_by_badge・revised_by_row の 4 つと、実の正本の読み real_revised_by を足す）。

**既存の歯の期待の直し（全数）。** 1 本である。`crates/folio/tests/face_adr.rs` の face_adr_census_on_the_real_sources_counts_and_verbatims は、根拠のリンク（xref）の数を basis と図の refs と revises の和で数える。li の頭のリンクの数も basis と revises の和で数える。本便の行も xref のリンクで li の頭に来るので、2 つの式にその面の逆向きの行の数（real_revised_by の手書きの読み）を足す。ほかの主張は変えない。凍結 anchor の手直しは 0 本（(e) の 1）。

### (d) 採らなかった形

1. **正本の改訂される側に欄（revised_by）を足す。** 欄の決まりは revises を片側と定め（revises_note の reverse・ADR-21 決定 (6) の作法）、双方向にしても床が確かめられるものは増えない。人が 2 か所に書いて一致を検査で強制する形は P-6.4 が採らない。面の導出で足りる。
2. **逆向きの札を 0 件の面にも出す（札の並びを 24 本で揃える）。** 便 137 は正本の 2 つの欄の札を 0 件でも出した（揃い）。逆向きの件数は正本の欄ではなく導出の数である。0 件でも出すと、改訂されていない 18 本の面と凍結 anchor 2 本（expected-adr.html・expected-site-adr-2.html）が動き、表紙に 0 件の札が 1 つ増える。図の札と同じく 1 つ以上のときだけ出す形を採る（席への問い 1）。
3. **発効かどうかを問わず全記録の revises を読む。** 提案中の記録の改訂の欄は、まだ効いていない。それを改訂される側の面に「この判断の決定を狭める」と出すと、読み手は効いている改訂と取り違える。廃止の記録の改訂は、置き換えた後継の記録が自分の revises で持ち直す形になる。発効だけを読む（席への問い 2）。
4. **読めない記録を飛ばして面を出す（record_title と同じ緩い形）。** 題は無くても面の意味が変わらないが、逆向きの行は欠けると「改訂されていない」と黙って読める（P-4.1）。床を通った置き場では起きないので、厳しい形でも folio2 自身の面は落ちない（席への問い 3）。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか・面の変化

1. **実装だけを base に当てた写し（歯の file は base のまま）で、workspace の nextest は 938 本のうち 1 本が落ちる（起草役の実測）。** 落ちるのは face_adr の census（(c) の既存の歯の直し）で、xref の数が 17 行の分だけ増える。凍結 anchor（`tests/fixtures/face/` の expected 7 本・天井の束の凍結 anchor・床の凍結の土台）はどれも動かない。面の fixture の ADR-1 と ADR-2 は提案中で revises を持たないので、ADR-2 の面に逆向きの行が出ないからである。
2. **本便の差分の全部を base に当てた写しで、** workspace の nextest は 944 / 944（base 938 + 単体 1 + face_adr 5）・clippy 0 警告・床 4 本 rc 0・verify の 6 行すべて rc 0。face_adr の歯の file は 44 / 44。
3. **RED（起草役の実測）。** binary の歯（census の式の直しを含む）だけを base に当てた写し（943 本）で、6 本が落ちる。落ちるのは (c) の 2〜6 の 5 本と census である。単体の歯だけを当てると組み立てが通らない（E0425 が 4 つ＝REVISED_BY_LABEL 3・REVISED_BY_STATUS 1）。
4. **folio2 自身の面の変化（起草役の実測）。** base と本便の写しで `folio build --dir design-intent --out <置き場> --write` の出力は 31 file のまま（1,997,947 → 2,006,244 byte）。違う file は判断の記録の面 6 枚（ADR-2・3・8・13・16・18）だけで、どの面も足した行だけで消した行は 0 行である。ほかの 25 file（改訂されていない 18 枚・入口・憲法・要件書・設計ノート・様式・script）は byte で一致する。変わった 6 枚は `folio parts --check --page adr=<path>` で合格する（違反 0・まだ分からない 0）。

| 面 | 足した行（札 1・h3 と一覧の枠 3・行） |
| --- | ---: |
| ADR-2 | 5（行 1） |
| ADR-3 | 7（行 3） |
| ADR-8 | 6（行 2） |
| ADR-13 | 7（行 3） |
| ADR-16 | 9（行 5） |
| ADR-18 | 7（行 3） |

5. **突然変異（起草役の実測）。** 本便を当てた写しの実装だけを 1 通りずつ変え、単体の f148_ と歯の file face_adr の全部（45 本）を撃った。11 通りのうち 10 通りで 1 本以上が落ちる。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 逆向きの一覧を描かない | (c) の 2・5・6 と census（4 本） |
| M2 状態を問わず読む | (c) の 3（1 本） |
| M3 行の順を逆にする | (c) の 2・5（2 本） |
| M4 逆向きの札を 0 件でも出す | (c) の 2・5 と凍結 anchor の歯 2 本（4 本） |
| M5 表の外の向きを 狭める に読み替える | (c) の 4（1 本） |
| M6 改訂する側をリンクにせず id の字だけにする | (c) の 2・5・6 と census（4 本） |
| M7 相手を問わず発効の記録の revises を全部出す | (c) の 2・3・5 と census（4 本） |
| M8a 読めない記録を黙って飛ばす | (c) の 4（1 本） |
| M12 target が id の形でない行を黙って飛ばす | (c) の 4（1 本） |
| M13 状態の欄が無い記録を黙って飛ばす | (c) の 4（1 本） |
| M10 走査で自分自身を除かない | 落ちない（縛っていない） |

   M10 は歯で縛っていない。床（`crates/folio/src/adr.rs` の check_revises）が自分の id を target に書いた行を落とすので、床を通った置き場では自分を相手にする行が無く、実害は無い。

6. **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便が、同じ部品について規則の表の行 R-7 の上限〔2 回〕を超えて続いたとき）: 当たらない。** 本便は面に導線（改訂する側へのリンクの行）という中身を足す便で、見た目の直しではない。部品・色・余白・並び・様式の定義・名札の class を 1 つも変えず、足す字面はどれも章 05 と根拠の章が既に使う形（h3・一覧・xref のリンク）である。表紙の札は既存の札の部品が 1 つ増えるだけである。見た目を部品・色・余白・並び・名札の class と読むのは、便 132・135・137〜139・141・144〜147 の契約と同じ読みである。もし中身の追加や正しさの直しも数えるなら、判断の記録の面を変えた便は少なくとも便 27（見た目の直し）・便 137（改訂の欄と表紙の札）・便 146（鮮度の札）で、本便は少なくとも 4 本目になり上限を超える。部品ごとに数えても、表紙の札の並び（cover-meta）を変えた便は便 33（図の札を 1 枚以上のときだけ足す・本便と同じ形）・便 71（撤退条件の札を 1 文に）・便 137（改訂の札を 2 つに分けた）で、本便は少なくとも 4 本目になる（起草役が `git show` で d98fba2・08d9dd0・66fd1e1 の差分を確かめた・便 27 は表紙の h1 と副題を変え、cover-meta は変えていない）。よって 当たらない の根拠は、本便が見た目でなく中身の追加であるという前段の読みだけである。席はこの読みを採るかを決める（持ち主への次の報告で明記し、持ち主はこの読みを覆せる）。面の型は増えない（撤退条件 ① にも当たらない）。
7. **持ち主の面の承認（walk）。** 判断の記録の面は 2026-09-18 に持ち主の walk の承認を得ている。本便は章と部品を変えず、章 05 に h3 と一覧を 1 組と表紙に札を 1 つ足すだけなので、起草役は walk の取り直しを要しないと読む（便 137 と同じ）。求めるかは席が決める。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 4 本とも印なし。書き換える 2 本は `crates/folio/src/face_adr.rs` と `crates/folio/tests/face_adr.rs`。本文不変の 2 本は `crates/folio/tests/badge.rs` と `crates/folio/tests/site.rs`（verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_adr.rs` | 916 | 584 | 1,011（+95・うち単体の歯 約 10） | 489 |

   余地は S の見積 100 を超える。席の依頼の参考値 583 と 1 違うが、どちらでも S に足りる。行数の上限の歯の対象の file（face.rs・face_srs.rs）は触らない。歯の file は src の外なので余地を測らない（参考値 wc -l で 1,411 行 → 1,648 行）。
3. **size は S。**
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f148_` = (c) の 1。
   2. `cargo nextest run -p folio --test face_adr f148_` = (c) の 2〜6（5 本）。
   3. `cargo nextest run -p folio --test face_adr` = 判断の記録の面の歯の全部（凍結 anchor との byte 一致・census の直し・便 137 の改訂の欄・便 146 の鮮度の札を含む・参考値 44 本）。
   4. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致（本便で動かないこと）。
   5. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor（expected-site-adr-2.html を含む）との byte 一致（同）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_adr・badge・site の 3 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 f148_ を関数名に持つ src は `crates/folio/src/face_adr.rs` だけで write-set に在る。base では verify の 1・2 が 0 本の実行で rc 4、3〜6 が rc 0（起草役の実測）。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-26/d148-draft.md`、模擬の差分と script は同じ dir の d148-scripts（repo には入れない）。

1. 模擬: base の写しの根で apply-148.py（実装）と apply-148-teeth.py（歯）を撃つ。その差分が c148.patch。続けて次を撃つ。
   - workspace の nextest・clippy・床 4 本（floor4.sh）
   - `folio build --write` の出力の file 数と、base との `diff -rq`（build-diff.txt）
   - 変わった 6 枚の `folio parts --check`
   - verify の 6 行（verify-148.sh）
2. RED: r148-teeth.patch（binary の歯と census の直し）を base に当てると (e) の 3 のとおり。実装だけを当てると census の 1 本が落ちる。
3. 突然変異: mut-148.sh（mut-148.py の 11 通り・(e) の 5・終わりに写しの binary を組み直す）。
4. 余地: lines-148.py と lines-148.awk（同じ式の 2 実装）を write-set の src の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 次のものは本便で運ばない。
   - 正本の欄の決まりの変更（改訂される側に欄を足すこと）と、ADR-18 の平易文や注の字。平易文が狭める前の形で説明していることは、F-3 の注が別に指す。
   - 機械のための面の逆向きの件数と、入口の面の棚の判断の記録のカード。
   - 条文の改訂（amends と amended_by）の逆向き。憲法の面が既に来歴を持つ。
   - 凍結 anchor・台帳への記帳（席）・外部 crate。
2. **言えないこと。** 向き（narrow / widen）が本当かは、床も面も確かめない（欄の決まりの kind の項＝人が読む）。改訂する側が発効かだけを見て、改訂の中身が今も効いているか（後の記録がさらに改訂したか）は読まない。廃止の記録の改訂は出さないので、後継の記録が revises を持ち直さなければ、改訂される側の面から辿れなくなる。1 枚の面が全記録の正本を読むので、どれか 1 本が読めなければ判断の記録の面は全部 まだ分からない になる（床を通った置き場では起きない）。面の見た目が持ち主に受け入れられるかは walk でしか分からない。次の周で読みやすさの観点が F-3 を解けたと読むかは、周の結果でしか分からない。
3. **撤退条件。** 次のどれかが起きたら、止めて席へ返す。
   - (1) 本便の後に、(c) の census のほかに既存の歯が 1 本でも落ちた。そのときは、その歯の本文も fixture も直さない。
   - (2) 着地の直前の main の設計文書で組んだ `folio build` の出力を、着地の直前の main の binary の出力と比べて、次のどれかが起きた。
     - 逆向きの行を持つ判断の記録の面のほかの file が 1 byte でも違う。
     - 変わった面に消えた行が在る。
     - folio2 自身の床 4 本の結果が変わった。
   - (3) 受付の時点の main で、revises の 4 欄か向きの値域（narrow・widen）が base と違う。
   
   受付の時点の main で、改訂される側の本数や行の数だけが base と違うときは止めない。(a) の 3 と (e) の 4 の表を数え直してから運ぶ。

## 2. 範囲

- 入れる: `crates/folio/src/face_adr.rs` の名札 REVISED_BY_LABEL・読む状態 REVISED_BY_STATUS・走査の口 revised_by・表紙の札・章 05 の逆向きの一覧・単体の歯 1 本・頭の注。`crates/folio/tests/face_adr.rs` の歯 5 本と census の数える式 2 か所と頭の注。
- 入れない: 部品目録・名札の class・様式の定義・章の数と帯・目次・章 05 の頭の断り・機械のための面・ほかの面・床・設計文書・正本の欄の決まり・凍結 anchor・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| labels | 名札と読む状態 | `crates/folio/src/face_adr.rs` の REVISED_BY_LABEL・REVISED_BY_STATUS（向きは既存の REVISE） |
| scan | 走査の口 | face_adr.rs の revised_by（全記録の revises からこの記録を相手にする行） |
| cover | 表紙の札 | ほかの判断の記録による改訂 n 件（1 つ以上のときだけ） |
| chapter | 章 05 の逆向きの一覧 | h3 と一覧（改訂する側の面へのリンク・決定・向き・summary） |
| teeth | 歯 | 単体の f148_ 1 本・face_adr.rs の f148_ 5 本と census の式 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main c7cfa3a）。
- 並行の便: 無し（§0）。受付は逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件（f2-648.222）を閉じる。持ち主の walk を求めるかを決める（§1 (e) の 7）。次の周で、読みやすさの観点が F-3 を解けたと読むかを見る。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eu"
title = "天井の 40 周目 読みやすさ F-3（台帳 f2-648.222）: 判断の記録の面は自分の改訂の欄 revises の相手へリンクする（便 137）が、狭められた側・広げられた側の面には逆向きの導線が無い（ADR-18 の面に ADR-24 への導線が無い・ADR-16 の面に ADR-21 と ADR-22 の行が無い）問題を直す。正本は片側（改訂する側）のまま欄を足さず、面の生成器（crates/folio/src/face_adr.rs）が adr/ の全記録の revises を走査して逆向きの行を導出する（P-6.3）。走査は改訂する側の id の数の順・正本の順で、改訂する側が発効（型付きの定数 REVISED_BY_STATUS = accepted）の記録だけを読み、記録が読めない・欄の形が違う・向きが表 REVISE の外なら面を導出しない（まだ分からない）。章 05 の、その記録自身の revises の一覧の後・帰結の前に、h3 ほかの判断の記録による改訂（型付きの定数 REVISED_BY_LABEL）と一覧を置き、各行は改訂する側の面への xref のリンク・がこの判断の決定・decision の逐語・を・向きの名札（既存の表 REVISE）・summary の逐語。表紙には行が 1 つ以上のときだけ札 ほかの判断の記録による改訂 n 件を足し、既存の札 判断の記録の改訂 の数え方・章 05 の頭の断り・機械のための面・部品・class・ほかの面・床・設計文書・凍結 anchor は変えない（folio2 自身の面で変わるのは ADR-2・3・8・13・16・18 の 6 枚で、足した行だけ）。歯は単体の f148_ 1 本と face_adr の f148_ 5 本と census の数える式 2 か所。判断の記録 ADR-5 の撤退条件 ②（見た目だけの直し）には当たらない。base = main c7cfa3a・受付の時点の main で数え直す"
req = ["FR16"]
section = "1"
write-set = ["crates/folio/src/face_adr.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --bin folio f148_", "cargo nextest run -p folio --test face_adr f148_", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f148_（名札 REVISED_BY_LABEL の字と読む状態 REVISED_BY_STATUS が accepted だけで、名札が正本の 2 つの欄の名札の字を含まない）が緑、face_adr の f148_ の 5 本（写しの発効の ADR-1 の revises が ADR-2 の面の章 05 に改訂する側へのリンクと決定と向きと escape した summary の逐語で正本の順に並び表紙の札が 2 件で、正本の欄の札・断り・機械のための面は変わらず、札と一覧のほかは改訂されていない面と byte で同じ・提案中と廃止の改訂する側は読まない・向きが表の外・revises が一覧でない・target が id の形でない・状態の欄が無い・yaml として読めない改訂する側の 5 通りで終了コード 2 で面を書かない・実の正本の全本で逆向きの札と h3 の有無と全行の逐語と順・ADR-18 の面に ADR-24 への行と ADR-16 の面に ADR-21 と ADR-22 への行）が緑、face_adr の歯の全部（凍結 anchor expected-adr.html との byte 一致と census の直しを含む）が緑、badge の名札の無い面の凍結 anchor 7 本との byte 一致が緑、site の組み立ての出力の凍結 anchor との byte 一致が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数が同じで、逆向きの行を持つ判断の記録の面のほかは byte 一致・変わった面は足した行だけ"
<!-- contracts:end -->
