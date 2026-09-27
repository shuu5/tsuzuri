# 設計: 便 165 — 外の置き場で面だけが止まる 4 か所を面の側で寛容にする（要件書の counts を数えて導く・promise と scope / scope_m1 を任意に・憲法の改訂の散文を章の本文に）（台帳 f2-648.247・S）

- 要件: FR4（入口・憲法・要件書の 3 面を 1 つの生成器で出す）と FR22（骨格〔init〕の雛形）。規範文・確かめ方・受入基準は変えない（AC19 の閉じた 11 本も変えない）。
- 条: P-6.3 / P-6.4（同じ内容を 2 面に持たない・2 面を人が書いて一致を検査で強制しない＝counts）/ P-4.1 / P-4.2（在るのに形が違えば今どおり まだ分からない）/ P-1.2（面が中身を決めない＝無い節は描かない）。
- 出所: 台帳 **f2-648.247**（f2-648.245 の便 A1・席の裁定 2026-09-27 10:57 JST「調べの推奨を全部採る」）。材料は調べ facestops-study.md（§1〜§6）と probe.log（骨格からの 31 場面）。
- 置き場: 契約表は末尾の区間・審査の材料は行 `fl` が指す §1 だけ。write-set 8 本（新しい file 1・本文が変わらない歯の file 1）・新しい dir と fixture は無い。
- 門: 対象外。作業ツリー planner-d165 の一番上で base の binary に write-set を渡すと **0（通す・設計文書の正本を書き換えない便）**。base と本便の写しでも 0。
- 前の便: **base = main 8472b5c。便 163 は未着地（write-set は重ならない）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 並行の便: 便 166 と `crates/folio/src/face_srs.rs`、一括 30 と `crates/folio/tests/face_srs.rs` が重なる（どちらも別の区間・(g)）。
- 改訂: 改訂 a（2026-09-27・席）= 検証役（d165-verify・条件付きで通す・blocking 1）の歯の案（B-1 と N-1）を (c) に取り込み（歯 6・7 と 1〜4 の確かめ）、字の直し 2 つ（(b) の 3・(c) の 5）・(g) の数を今の値に・(i) に 5 と 6 を足した。write-set・verify の字・size・src・門は変えていない。

## 1. 設計

### (a) いま起きていること（参考値・base 8472b5c）

1. **面だけが要る欄が 4 つある。** 床（`crates/folio/src/check.rs` の check_srs・check_constitution）はどれも見ず、欄の決まり（生成区間）にも字が無い。だから床は合格のまま、面の導出だけが まだ分からない で止まり、`folio build` は全部か無しかで 1 file も書かない。

| # | 欄 | 面の読み手（base） | 面での使い道 |
| --- | --- | --- | --- |
| 1 | 要件書 meta.counts | `face_srs.rs` の derive → check_counts・`face_index_read.rs` の srs_card | 表示しない。数えた行の数との一致を確かめるだけ（表紙の件数と入口のカードは数えた数） |
| 2 | 要件書 meta.promise | `face_srs.rs` の cover（必須の欄の口 ef） | 表紙の要約の札「この文書が約束すること（1 文）」 |
| 3 | 憲法 amendment | `face_constitution.rs` の chapter_h2（章 06）と amendment_chapter | 表（declaration / steps / effective_step）だけを読み、図 1 と段の一覧を出す |
| 7 | 要件書 scope・scope_m1 | `face_srs.rs` の scope_chapter（必須・scope_m3 だけ任意〔便 118〕） | 章 02 の段の範囲の塊（M0・M1） |

2. **骨格からの道（調べの 31 場面・`folio init` の骨格に字を書き足して commit し、素の check と build --write）。** base と便 163 の写しの binary の答えは同じ（控え probe-base.log・probe-163.log・調べの probe.log と一致）。床は 6 場面とも骨格と同じ（違反 0・まだ分からない 2）で、面だけが止まる。

| 場面 | 書き足し | build（base） |
| --- | --- | --- |
| 02 | 要件・受入基準・目標を 1 つずつ書き counts を直さない（**誰でも最初に当たる**） | 「srs.yaml.meta.counts.fr: 0 だが数えた行は 1」・0 file |
| 03・04 | counts の行・promise の行を消す | 「meta: 欄 counts が無い」「meta: 欄 promise が無い」 |
| 05・06 | scope_m1 の行・scope の行を消す | 「欄 scope_m1 が無い」「欄 scope が無い」 |
| 25 | 憲法の amendment を散文の block に | 「constitution.yaml.amendment: 表でない」 |

3. **tsuzuri の写し**（design-intent と contracts を cp・tsuzuri の repo では何も書かず git も撃たない・写しの根で schema --write → commit）。素の check は合格 0/0。base の binary の build は設計ノートの承認欄（便 163 が塞ぐ）で止まり、便 163 の写しの binary では台帳 f2-648.245 の 9 か所（1〜9）で止まる（控え tsuzuri-rerun-*.log・d163 の記録と一致）。tsuzuri の srs.yaml は promise・counts・scope_m1 を持たず、憲法の amendment は 3 行の散文。
4. **base の歯。** workspace の nextest 985 / 985・clippy 0 警告・床 4 本 rc 0・folio2 自身の `folio build --write` 34 file（2146076 byte）。`--test face_srs` 25 本・`--test face_srs_body` 18 本・`--test init` 21 本。`f165_` と行 id `fl` は 0 件。

### (b) 直す先

1. **`crates/folio/src/face_srs.rs`。**
   - derive から check_counts の呼び出しと関数を外す（面は counts を読まない）。表紙の件数と章の見出しの数は今も数えた数（range・count_word）。
   - cover: promise を任意の欄の口（g・無い・null は無し）で読み、在れば今と同じ要約の札、無ければ札を出さない。在るのに字でなければ今どおり まだ分からない。
   - scope_chapter: 段の範囲の節 3 つ（scope = M0・scope_m1 = M1・scope_m3 = M3）をどれも任意の節として読み、在る節だけ塊を出す（塊の字と注の読み方は変えない）。1 つも無ければ小見出し「作るもの / 作らないもの」も出さない。
2. **`crates/folio/src/face_index_read.rs` の srs_card。** counts を読まず、要件・非機能要件・受入基準の 3 節の行を数えてカードに出す（字の並び「機能 n · 非機能 n · 受入基準 n」は同じ）。
3. **`crates/folio/src/face_constitution.rs`。** amendment が字（scalar）なら、章 06 の帯を h2「変えるときの手続き」（段の数を付けない・lead なし）で出し、本文に字の各行（空白だけの行を除く）を escape して `<p>` 1 つずつで逐語に出す（面の全体の ADR-n のリンク付けは他の章と同じに掛かる）。図 1（fig-amend-flow）・段の一覧（stepper）は出さない。目次の h2 も同じ字。改訂の例（条の supersedes_v1・amended_by）と版ごとの変更点（meta の changes_from_）は形に依らず今どおり出す。表なら今の読み（amendment_flow に切り出すだけで字は 1 字も変えない）。一覧なら今どおり まだ分からない。
4. **`crates/folio/src/init.rs`。** 雛形の要件書から `counts: {fr: 0, nfr: 0, ac: 0, con: 0}` の 1 行を外す（書く 11 本・断りの名・生成区間・ほかの欄は変えない）。
5. **変えないもの。** 床（check.rs）・憲法の meta.counts（床 refs.rs が数える・面も今どおり一致を確かめる）・部品・欄の決まりと生成区間・設計文書の正本（folio2 の srs.yaml の counts を含む）・folio2 自身と骨格の面の出力。

### (c) 歯（関数名 f165_）

新しい file `crates/folio/tests/outside_faces.rs`（binary 経由）。fixture の dir を足さず、口 Place が一時 dir の根で git init → `folio init --dir <根>/design-intent` の骨格を書き、字を当てて commit し、素の check（rc 2・違反 0・まだ分からない 2 = 骨格と同じ床）と build --write（rc 2・6 file を書く）を確かめる（便 163 の f163_signed と同じ口の形）。止まる側は口 Place::stop（rc 2・配信先に 0 file）。

1. **f165_srs_without_counts_promise_and_scope_builds。** 骨格の要件書に counts の行が無い（init.rs の直し）。骨格の要件書から promise・scope・scope_m1 の行を消す → srs.html に要約の札・範囲の小見出し・「で作る」「では作らない」の card が 0、機能要件は「0 件」、index.html の要件書のカードは「機能 0 · 非機能 0 · 受入基準 0」。緑の対: 骨格のままなら要約の札・小見出し・M0 と M1 の card が 1 つずつ。**base では build が「欄 counts が無い」で 0 file＝RED。**
2. **f165_counts_are_counted_not_read。** 1 つの置き場で要件・受入基準・目標を 1 つずつ書き、counts が無い / 4 つの鍵とも行とずれる（fr 0・nfr 5・ac 0・con 5）/ 行と合う、の順に commit して build → 3 つの srs.html と index.html がそれぞれ byte で同じ、表紙に「1 件（FR1–FR1）」「1 件（AC1–AC1）」、カードに「機能 1 · 非機能 0 · 受入基準 1」。**base では counts の無い段で止まる＝RED。**
3. **f165_prose_amendment_is_the_chapter_body。** 骨格の憲法の amendment の表を、空行を挟む 2 行の散文（「<」「&」と ASCII の二重引用符を含む）に替える → constitution.html の章 06 に各行が `<p>` の中に逐語（5 字の escape）で 1 回ずつ、章 06 の p は 2 つで最後の行の p の直後で閉じる、h2 と目次の字は「変えるときの手続き」、fig-amend-flow・stepper・「変えるときの手続き — 」が 0。緑の対: 骨格のまま（表・段 0）なら図 1 と h2「変えるときの手続き — 0 段」。**base では「amendment: 表でない」で止まる＝RED。**
4. **f165_face_srs_does_not_read_counts（`crates/folio/tests/face_srs_body.rs`・既存の歯の書き換え）。** 面の fixture（tests/fixtures/face/）の写しで counts を行とずらす（fr 3）か行ごと消す （4 つの鍵とも）→ `folio face --face srs --write` が rc 0 で凍結 fixture expected-srs.html と byte 一致（P-10.1 の anchor で測る）。旧 face_srs_unknown_when_counts_differ_from_the_rows（ずらすと まだ分からない）を置き換える。頭の注「導出できない入力 8 つ」を 7 つに。**base では まだ分からない＝RED。**
5. **f165_a_missing_scope_or_scope_m1_drops_only_its_block（`crates/folio/tests/face_srs.rs`）。** 面の fixture の写しから scope か scope_m1 の節を落とす → rc 0・章 02 の塊が 1 つ・面が凍結 fixture からその塊（M0 か M1）を 1 つ除いた字と一致。既存の f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required の後半（落とすと 2）をこの向きに替え、前半（scope_m3 が null なら凍結 fixture と一致）は本体の字を変えずに（名から _and_scope_m1_stays_required を外して）f118_null_scope_m3_is_unchanged として残す（頭の注も直す）。**base では「欄 scope が無い」＝RED。**
6. **f165_a_present_field_in_a_wrong_shape_still_stops（outside_faces.rs・保つ歯）。** 骨格で promise を一覧・scope を字・scope_m1 を一覧・憲法の amendment を一覧に替えると、build が rc 2 で「まだ分からない: <欄>: 文字列でない / 表でない」を出し、配信先に 1 file も書かない（P-4.2）。**base でも緑**（本便の後も残ることを縛る）。
7. **f165_prose_amendment_keeps_the_examples_and_changes（outside_faces.rs）。** 面の fixture（改訂の例 1 つと版ごとの変更点 1 つを持つ）の amendment を散文に替えて `folio face --face constitution` → 章 06 の散文の行の後が凍結 fixture expected.html の段の一覧の後と byte で同じ、章 06 の外は目次の 06 の行（「 — 2 段」の有無）のほかは expected.html と同じ。**base では「amendment: 表でない」＝RED。**

### (d) 採らなかった形

1. **counts が在れば一致を確かめる（無ければ読まない）。** folio2 の srs.yaml（counts を持ち行と合う）の答えはどちらの形でも変わらない。だがこれは P-6.4 の「2 面を人が書き、一致を検査で強制する」をそのまま残し、骨格に counts を書いた利用者は要件を足すたびに counts を手で直すことになる（場面 02 がそのまま残る）。P-4.1 に照らしても、読まない形は検査を飛ばして「異常なし」と言うものではない: 面は counts を表示も保証もせず（数えた数だけを出す）、counts は面の検査の対象から外れる。変異 M1・M4 がこの形を落とす。**採らない＝counts は在っても読まない。**
2. **counts が在れば まだ分からない（読まない欄を断る）。** folio2 の srs.yaml が counts を持つので folio2 の面が止まる（撤退条件 (2)）。counts を外すのは一括（(i) の 2）。
3. **字の amendment を段 1 つの図にする／空行で段落に分け行の中の改行を空白に畳む。** 前者は正本に無い段と担当を面が作る（P-1.2）。後者は字を変える（逐語でない）。
4. **scope と scope_m1 を必須のまま利用者が 1 行足す（便 118 の (d) の 3）。** 席の裁定で A（面が寛容）。無い節は描かないだけで、在る節の形が違えば今どおり まだ分からない（P-4.2 の向きは保つ）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけを当てると 2 本が落ちる**＝(c) の 4・5 が置き換える歯（face_srs_body の face_srs_unknown_when_counts_differ_from_the_rows・face_srs の f118_null_scope_m3_is_unchanged_and_scope_m1_stays_required・本文は控え sim0-nextest.log）。本便の全部で workspace の nextest 991 / 991・clippy 0 警告・床 4 本 rc 0。
2. **RED。** 歯だけを base に当てると、4 つの歯の file（outside_faces・face_srs・face_srs_body・init）の 70 本のうち f165_ の 6 本だけが落ちる（(c) の 6 は保つ歯で base でも緑・控え red-a.log・落ちた本文も）。
3. **突然変異。** 写しの src 4 本だけを 1 通りずつ変え、4 つの歯の file の全部を撃った。**M1〜M7（起草役）と V1〜V10・V4b（検証役）の 18 通りは全部落ちる**（控え mut-a.log・検証役の vmut.py の写し）。

| 変異 | 落ちる歯（(c) の番号） |
| --- | --- |
| M1 要件書の面が counts を在れば読む / V8 counts.con だけを読む | 2・4 |
| M2 promise が無くても空の要約の札を出す / M6 範囲の節が 1 つも無くても小見出しを出す / V7 雛形に counts の行を戻す | 1 |
| M3 字の amendment の行を出さない / V4 散文の枝で改訂の例と変更点を出さず章を閉じない / V10 目次の字を帯の h2 と違える | 3・7 |
| M7 行を escape しない / V5 空白だけの行を落とさない / V6 escape を < だけにする | 3 |
| M4 入口のカードが counts を在れば読む / V9 counts.nfr だけを読む | 2 |
| M5 scope_m1 を必須に戻す | 1・5 |
| V1 promise が字でなければ黙って札を消す / V2 範囲の節が表でなければ黙って塊を消す / V3 amendment が一覧なら黙って空の散文にする | 6 |
| V4b 散文の枝で改訂の例と変更点を出さない（章は閉じる） | 7 |

### (f) 大きさ・verify と done の対応

1. **write-set（8 本）。** src 4（`face_srs.rs`・`face_index_read.rs`・`face_constitution.rs`・`init.rs`）・tests 4（`+crates/folio/tests/outside_faces.rs` は新規・`face_srs.rs`・`face_srs_body.rs`・`init.rs`〔本文は変えない・verify の `--test init` の scope〕）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致（控え lines.log）。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_srs.rs` | 1068 | 432 | 1055（−13） | 445 |
| `crates/folio/src/face_index_read.rs` | 638 | 862 | 631（−7） | 869 |
| `crates/folio/src/face_constitution.rs` | 1191 | 309 | 1223（+32） | 277 |
| `crates/folio/src/init.rs` | 802 | 698 | 802（±0） | 698 |

3. **size は S。**
4. **verify は 7 行**で、done の 7 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test outside_faces f165_` = (c) の 1〜3・6・7（base は file が無く組み立てが rc 101）。
   2. `cargo nextest run -p folio --test face_srs f165_` = (c) の 5（base は 0 件で rc 4）。
   3. `cargo nextest run -p folio --test face_srs_body f165_` = (c) の 4（base は 0 件で rc 4）。
   4. `cargo nextest run -p folio --test face_srs` = 要件書の面の歯の全部（f118 の 4 本と f165_ を含む・参考値 26 本）。
   5. `cargo nextest run -p folio --test face_srs_body` = 要件書の面の本体の歯の全部（凍結 fixture の byte 一致を含む・18 本）。
   6. `cargo nextest run -p folio --test init` = 骨格の歯の全部（AC19 の 11 本・f152 の通し・21 本）。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門・受付・並行の便

門は 0（控え gate.log）。precheck の契約に起因する断りは 0。重なりは `git -C <worktree> diff --stat origin/main` と各契約の区間で数えた（2026-09-27）。
- 便 162（41 本）・便 163（5 本）・便 164（7 本）・便 168（4 本）・一括 31（design-intent の 6 本）: 0（改訂 a の時点で数え直した）。
- 便 166（docs/d166・3 本）: `crates/folio/src/face_srs.rs` が重なる。本便は derive の check_counts の 1 行、便 166 は同じ関数の 6 行下の呼び出し 2 行で、便 166 の起草役がどちらの順でも当たることを確かめた（delivery-166.md (g) の 2）。
- 一括 30（docs/batch29・ddf8b44）: `crates/folio/tests/face_srs.rs` が重なる（一括 30 は f81 の歯の区間・本便は頭の注と f118 の歯の区間）。改訂 a の c165.patch は一括 30 の枝にそのまま当たり、当てた写しで outside_faces・face_srs・face_srs_body・init・face_constitution の 96 本が緑（控え b30-a.log）。
- 後続の便 B1（f2-648.249）は本便の後。

### (h) 今の置き場の床と面（撤退条件 (2) の実測）

1. **folio2 自身。** 床 4 本・凍結の 4 つの旗の出力（標準出力・標準エラー・rc・書いた file）・build（34 file・2146076 byte）が base と本便で 1 byte も違わない（counts は行と合い、promise・scope・scope_m1 と表の amendment を持つ）。
2. **骨格。** base と本便で、素の check rc 2（まだ分からない 2）・build は 6 file（130777 byte）を書き面は diff -r 一致。骨格の file の差は srs.yaml の counts の 1 行だけ。便 165 の前の骨格（counts を持つ）に本便の binary を当てると、場面 02・03・04・05・06・25 が 6 file を書くようになり、ほかの 25 場面の答えは床も面も変わらない（控え probe-sim.log）。
3. **tsuzuri の写し（便 163 と本便の後）。** 止まる所は台帳 f2-648.245 の **4（道具の役が 0）・5（入れる側の帯 5）・6（出る側の帯 5）・8（basis に ADR-7）・9（設計ノート surface の章 21）の 5 か所だけ**。この 5 か所を写しの上でだけ避けると素の check 合格 0/0・build rc 0（15 file・byte 数は避け方による参考値）。章 06 は散文 3 行が `<p>` 3 つで出る。便 166 は adrs を描くだけで basis の判定を変えないので 8 は残る（tsuzuri の basis に ADR-n を持つ行は 20・adrs が指す ADR-3・5・7・8 は置き場に在る）。5 か所は利用者が直す側で、便 B1 / B2 で床が面と同じ関数で落とす。

### (i) 運ばないもの・言えないこと・撤退条件

1. **4・5・6・8・9 は運ばない**（(h) の 3・便 B1 / B2 と tsuzuri の手番）。
2. **folio2 の srs.yaml の counts は外さない。** 設計文書の便（一括）で外す。外すまで counts は床も面も読まない欄として残る。
3. **憲法の amendment の表の形（declaration / steps / effective_step・段 ≤ 7）を床が数えるのは便 B1。**
4. **借り（行 D-11）。** 要件書の任意の欄（promise・scope・scope_m1・scope_m3）と、面が counts を読まないことの字は実装にだけ在り、欄の決まりの生成区間には無い。写しの置き場は便 B1 が面の要る欄を生成区間へ写すときに決める（本便は design-intent を書かない）。
5. **空の字の amendment。** 空か空白だけの字（`amendment: ''`）は、章 06 に h2 だけが出て本文が空になる（骨格の段 0 の表と同じく空の章）。まだ分からない にするかは、憲法の amendment の形を床が数える便 B1 で床と一緒に決める。
6. **counts の後始末。** 一括で外すまで、folio2 の srs.yaml の counts が壊れても行とずれても床と面は黙る。counts を知らない欄として床が落とすか残すかは、便 B1（面の要る欄を欄の決まりへ写す便）の手番に足す。
7. **撤退条件。**
   - (1) (e) の 1 の 2 本のほかに既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。
   - (2) 着地の後の main で、folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前の main と 1 byte でも違ったら止めて席へ返す。
   - (3) 受付の時点の main で (b) の関数・init.rs の雛形の要件書・tests/face_srs.rs の f118 の歯と helper（fixture_srs・drop_section・callouts・frozen_srs）が base と違えば数え直してから運ぶ（手順は控え d165-draft.md）。

## 2. 範囲

- 入れる: §1 (b) の 1〜4、(c) の歯 5 本と口（Place・one_requirement・drop_lines・page）と既存の歯の書き換え 2 本。
- 入れない: 床（check.rs）・憲法の counts・ほかの面・部品目録・欄の決まりと生成区間・設計文書の正本・fixture と新しい dir・(i) の 1〜4・tsuzuri の repo・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| counted | 数えた数 | counts を読まずに行を数える（要件書の面と入口のカード） |
| optional | 任意の節 | promise の札と段の範囲の塊を在るときだけ出す |
| prose | 散文の改訂 | 章 06 で字の各行を逐語に出す（表なら amendment_flow） |
| teeth | 歯 | f165_ 5 本と置き換え 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（便 163 と独立）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .247 を閉じ、(i) の 2・4 を一括に束ねる。tsuzuri の設計席へ、(h) の 3 の 5 か所（利用者が直す側）を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fl"
title = "台帳 f2-648.247（f2-648.245 の便 A1）: 外の置き場で床は合格なのに面だけが止まる 4 か所を面の側で寛容にする。face_srs.rs は要件書の meta.counts を読まず（check_counts を外す・表紙は今も数えた数）、promise を任意の欄に（無ければ要約の札を出さない）、scope と scope_m1 も scope_m3 と同じ任意の節に（在る節だけ塊を出し 1 つも無ければ小見出しも出さない）。face_index_read.rs の要件書のカードも counts を読まず行を数える。face_constitution.rs は憲法の amendment が字なら章 06 の本文に行ごとに逐語で出し図 1 と段の一覧と段の数を出さない（表なら今の読み）。init.rs の雛形の要件書から counts の行を外す。在るのに形が違う欄は今どおり まだ分からない。床・憲法の counts・部品・欄の決まり・folio2 自身の出力・骨格の面は変えない。歯は f165_ 7 本（新しい tests/outside_faces.rs の 5 本〔歯の中で folio init の骨格を一時 dir に書く・在るのに形が違えば止まる保つ歯と散文でも改訂の例が出る歯を含む〕）と、counts と scope_m1 を必須と見ていた既存の歯 2 本の置き換え。門の対象外。base = main 8472b5c"
req = ["FR4", "FR22"]
section = "1"
write-set = ["crates/folio/src/face_srs.rs", "crates/folio/src/face_index_read.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/init.rs", "+crates/folio/tests/outside_faces.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_srs_body.rs", "crates/folio/tests/init.rs"]
verify = ["cargo nextest run -p folio --test outside_faces f165_", "cargo nextest run -p folio --test face_srs f165_", "cargo nextest run -p folio --test face_srs_body f165_", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_srs_body", "cargo nextest run -p folio --test init", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/outside_faces.rs の f165_ の 5 本（folio init の骨格は要件書に counts の行を持たず、promise・scope・scope_m1 を消しても素の check は骨格と同じで build --write が 6 file を書き要約の札と範囲の塊と小見出しが無い / counts が無い・4 つの鍵ともずれる・合うの 3 つで要件書の面と入口の面が byte で同じで数えた数を出す / 憲法の amendment が字なら章 06 に各行が 5 字の escape の逐語の p で出て空行を落とし最後の p で閉じ、図 1 と段の一覧と段の数が無く目次も同じ字・表なら図 1 と 0 段 / 在るのに形が違う promise・scope・scope_m1・amendment では build が rc 2 で まだ分からない を出し 1 file も書かない / 面の fixture の amendment を字にしても改訂の例と版ごとの変更点が凍結 fixture と同じ）が緑、tests/face_srs.rs の f165_（scope か scope_m1 を落とすと面が凍結 fixture からその塊だけを除いた字と一致）が緑、tests/face_srs_body.rs の f165_（counts を 4 つの鍵ともずらすか消しても面が凍結 fixture と byte 一致）が緑、tests/face_srs.rs の歯の全部（本体の字を変えずに名だけ短くした f118_null_scope_m3_is_unchanged を含む）が緑、tests/face_srs_body.rs の歯の全部が緑、tests/init.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
