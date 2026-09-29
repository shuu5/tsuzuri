# 設計: 便 180 — `folio graph --print --summary`: 節点ごとに、id が書かれた行の番号・平易文・技術の要約を添えた 1 行の JSON（JSON Lines）を出す

- 要件: FR14（機械が読む id の索引を出す・要件書 第 1.52 版）。第 1.52 版は枝 docs/srs152 の commit 201a40a の下書きと、その改訂 a の eec0c31（受入基準の技術の要約は題の全文・席と tsuzuri の設計席の合意 2026-09-28）で、**持ち主の承認待ち**（席が同じ 1 回で問うている）。第 1.52 版の FR14 の規範文に足した 1 文は「求められたとき（--summary）は、表の代わりに、節点ごとに id と種類と所属 file と 1 行の題（表と同じ字）に、所属 file の中でその id が書かれた行の番号（1 始まり）と、平易文の欄の字（そのまま）と、技術の要約（規範文・本文・what・決定の欄のうち最初に在るものの字・条はその最初の規範文の字・受入基準は題の全文）を加え、1 節点 1 行の機械が読む形（JSON Lines・欄が無ければ空の値 null）で出す。」で、確かめ方に「求められたときに出す行の番号と文が、正本のその行と欄の字に一致することも見る」を足した。本便はこの字を実装に写す。契約表の行の req は FR14 の 1 つ（本流に在る id）。FR14 の verify.ac は空（受入基準の id は無い）。
- 条: P-4.1 / P-4.2（組めない索引から行を出さない＝今と同じ まだ分からない）/ P-6.1・P-6.3（要約の字は正本の欄の字を写すだけ＝使う側に正本の 2 つ目の読み手を作らせない）/ P-10.1・P-10.2（期待の字は歯の側の手書き＝凍結 anchor・生成物どうしの突き合わせにしない）/ P-5.1（技術の要約に使う欄の一覧は実装の型付きの定数）。
- 出所: tsuzuri（folio2 の唯一の外の利用者）の設計席の依頼（2026-09-28・席の経由）。tsuzuri の server は `folio graph --print` の索引から図（graph）を組むが、索引の節点に要約の字と行が無い。tsuzuri の要件 FR15（tsuzuri の写しの srs.yaml 237 行目）は「節点ごとの概要（非エンジニア向けとエンジニア向け）は設計文書の plain の欄と台帳の要約…」で、面は写すだけ。folio が出さないと tsuzuri の core に設計文書の正本の 2 つ目の読み手が要る（負債）。合意した形（tsuzuri が了承済み）＝1 節点 1 行の JSON Lines・欄は id・kind・file・line・title・plain・eng の 7 つ。既存の `--print`（タブ区切りの 2 つの表と要約の 1 行）と `--digest` は変えない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ga` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 3 本（src 2・新しい歯の file 1）。新しい file は 1 本（頭に `+`・既存の dir `crates/folio/tests/` の下）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。本便は設計文書の正本（`design-intent/` の下）を書き換えない。write-set 3 本を本流 866bb78 の組み立ての binary で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・`通す（設計文書の正本を書き換えない便）`）**（eec0c31 の上の worktree で撃った）。要件書の下書き（docs/srs152・`design-intent/srs.yaml` の 1 本）も同じ binary で 0（印の周 2026-09-27-round51・4 観点合格・印の後の変更は審査していない）。
- 前提: **base = 枝 docs/srs152 の要件書 第 1.52 版の下書き（commit 201a40a と改訂 a の eec0c31 = 本流 866bb78 に `design-intent/srs.yaml` だけを変えたもの）が本流に着地した後の main。この契約の数はすべて eec0c31 の写しの実測（参考値）である**（規則の表の行 D-13）。本便の歯と実装は要件書の字を読まないので、866bb78 の写しに本便を当てても f180_ の 6 本は緑（起草役の実測）。受付の時点の main が eec0c31 の中身と違えば、その main で数え直す。
- 実装の見本: origin の枝 `impl/d180`（commit d2fe440 = 見本 60236c4〔親 201a40a〕に枝 docs/srs152 の eec0c31 を merge した 8f82619 に、改訂 a〔受入基準の技術の要約は題の全文・a5454de〕と改訂 b〔歯から python の独立の実装を外し、手書きの fixture で `|` の塊と text / what の順を縛る〕を積んだもの）が本便の後の中身で、`git diff eec0c31 d2fe440` が便の全体の差分（3 file）。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout d2fe440 -- <write-set の file>`）。write-set の外は変えない。docs/srs152 が本流へ squash で入った後は、席が本流を見本へ merge する（便 178 と同じ）。
- 並行の便との重なり: 便 179（行 fz・枝 docs/cap・見本 impl/d179 の 21b44b3）の write-set 23 本と本便の write-set 3 本は**重ならない**。便 179 は歯の file の graph と hello と床の土台（`tests/fixtures/floor_base/design-intent/` の rules.yaml と constitution.yaml）と索引の凍結 anchor 3 本を書き換えるが、`crates/folio/src/graph.rs` と `crates/folio/src/main.rs` は書き換えない。本便の歯は土台の節点の数を固定せず、手で書いた行は便 179 が動かさない行を選び、索引の凍結 anchor は file から読む＝(c) の 2・5。見本どうしを重ねた写し（d2fe440 に 21b44b3 を merge・衝突なし）で workspace の nextest 1016 / 1016・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file（起草役の実測）＝**どちらの順で受け付けてもよい**（受付の順は席が決める）。ほかに graph.rs と main.rs を書き換える未着地の便の契約は base の時点で無い（便 172〜178 は着地済み）。

## 1. 設計

### (a) いま起きていること（base eec0c31 の実測・参考値）

1. **索引の口の出力。** `crates/folio/src/graph.rs` の関数 run は、索引（Index）を組んだ後、行の逐語の読み手 Scan で節点の要約値を組み、Index の関数 render の字（節点の表〔1 行 = id / 種類 / file / 要約値 8 字 / 題 36 字・タブ区切り〕・辺の表・要約の 1 行）を出す（`--digest` なら関数 digest の 3 表と 2 行）。節点の行の番号と、平易文の欄の字と、技術の要約の字は出さない。`folio graph --print --summary` は clap の断り（未知の旗 --summary）で終了コード 2。
2. **行の番号の在り処。** Scan（便 99）は正本の file を行の逐語で切り分け、節点の頭の行（憲法・規則の表・要件書は字下げ 2 の `- id: ` か `- {id: ` の行、憲法の規範文は条の中の字下げ 6 の同じ形の行、判断の記録は最初の `id: ` の行）を見つけて要約値を組むが、行の番号は捨てている。索引を組む YAML の読み手（`crates/folio/src/yaml.rs` の型 Node）は行の印を持たない。Scan の節点と索引の節点の集合は Scan の関数 agree が突き合わせ、食い違えば表を出さずに まだ分からない（今の歯 f99 の 1 本がこれを縛る）。
3. **欄の実測（folio2 の置き場・eec0c31・234 節点）。** 平易文の欄 plain は条・要件・非機能要件・受入基準・判断の記録だけが持ち、技術の要約に使える欄（shall・text・what・decision）は下の表のとおり。受入基準（欄は id・title・plain・red_test・verifies）と登場人物と出力は 4 つのどれも持たない＝受入基準は題の欄の全文を技術の要約に使う（第 1.52 版 改訂 a）。受入基準 27 本のうち 18 本は題が 36 字を超え、索引の表の題は切れている。

| 種類 | 数 | plain | 技術の要約の欄 |
| --- | ---: | --- | --- |
| 条 | 27 | 27 | 条そのものは持たない（最初の規範文の text） |
| 規範文 | 67 | 0 | text 67 |
| 規則行 | 36 | 0 | what 36 |
| 目的 | 4 | 0 | text 4 |
| 要件 | 25 | 25 | shall 25 |
| 非機能要件 | 3 | 3 | shall 3 |
| 受入基準 | 27 | 27 | 4 つは無い（題 title の全文） |
| 制約 | 9 | 0 | text 9 |
| 登場人物 | 5 | 0 | 無い |
| 出力 | 3 | 0 | 無い |
| 判断の記録 | 28 | 28 | decision 28 |

   plain が無い節点 124・技術の要約が無い節点 8（登場人物 5・出力 3）。tsuzuri の写し（`t3v/tz3` を起草役の scratch へ git clone・HEAD b02c2ae）は 236 節点・plain が無い 146・技術の要約が無い 9（受入基準 17 本はどれも題の全文を持つ）。
4. **base の歯と出力（参考値）。** workspace の nextest 1003 / 1003・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file（全 file の sha256 の要約 ffe238314d34f598）。`--print` の出力は folio2 の置き場 49,777 byte・床の土台 31,277 byte（凍結 anchor graph-anchor.txt のとおり）・tsuzuri の写し 35,525 byte。`git grep -n f180_ -- crates` は 0 件・行 id `ga` は 0 件。

### (b) 直す先 — Scan が見つけた行の番号と、正本の欄の字を、節点ごとの 1 行の JSON で出す

1. **`graph.rs` の定数 ENG_FIELDS（新しい定数）と関数 eng（新しい関数）。** 技術の要約に使う欄の閉じた一覧 shall・text・what・decision（この順）。関数 eng は行の ENG_FIELDS のうち最初に字の値を持つ欄の字を返す（空の値の欄は飛ばす）。
2. **`graph.rs` の Index。** 欄 lines（id → 行の番号）と欄 texts（id → 平易文と技術の要約・無ければ無し）を足す。関数 texts（新しい関数）は、平易文を行の欄 plain の字（字の値だけ＝空の値 `~`・null・空と、一覧や表の値は無し）とし、技術の要約を渡された字とする。同じ id の 2 度目は最初を残す（関数 node と同じ）。呼ぶ所と渡す技術の要約は、関数 constitution（条 = その条の statements のうち最初に id を持つ行の eng・規範文 = その行の eng）・関数 rules（その行の eng）・関数 srs（受入基準〔種類 6〕は題の欄の全文＝表の 36 字で切らず空白も畳まない字、ほかの 6 節はその行の eng）・関数 adr（その記録の eng）。
3. **`graph.rs` の Scan。** 欄 lines を足し、関数 file の中で、条・規則の表の行・要件書の行は頭の行、規範文は字下げ 6 の頭の行、判断の記録は最初の `id: ` の行の番号（1 始まり）を覚える（要約値の式と切り方は変えない）。
4. **`graph.rs` の関数 run。** 引数に summary を足す（dir・digest・summary）。digest でなければ今どおり Scan と agree を撃ち、agree の前に Scan の行の番号を索引へ移す（agree が通った＝索引の全節点に行の番号が在る）。summary なら Index の関数 jsonl（新しい関数）の字を、でなければ今どおり render の字を出す。組めないときは今どおり、表も行も出さずに `まだ分からない（…）` で終了コード 2。
5. **`graph.rs` の関数 jsonl の形。** 節点の表と同じ順（id の byte 順）に 1 節点 1 行。1 行は JSON の表 1 つで、欄は id・kind・file・line・title・plain・eng の 7 つをこの順に、区切りの空白を挟まずに書き、改行 1 つで終える。id・kind・file・title は節点の表と同じ字（title は空白を畳み 36 字に切った題）、line は数、plain と eng は字か null。字の書き方は `crates/folio/src/yaml.rs` の関数 json_str（既に parts.rs と freeze.rs が使う・床の python の json.dumps の既定で非 ASCII を逃がさない形と同じ escape＝二重引用符と逆斜線の前に逆斜線・改行・復帰・タブ・後退・改頁は逆斜線と n・r・t・b・f・ほかの U+0000〜U+001F は逆斜線と u と 16 進 4 字・ほかの字はそのまま）。字の中の改行とタブは必ず逃がされるので、1 節点は必ず 1 行に収まる。
6. **`crates/folio/src/main.rs` の命令 graph。** 旗 --summary（bool）を足す。clap の属性 conflicts_with で --digest と一緒には撃てず、今の組 mode（--print か --digest のちょうど 1 つが要る）はそのまま＝--summary は --print と一緒のときだけ撃てる（--summary だけ・--digest --summary は clap の断りで 2）。graph の関数 run に summary を渡す。命令と旗の説明の字。
7. **頭の注。** graph.rs の file の頭の注に便 180 の 1 項（2 行）を足す。
8. **変えないもの。** `--print` の 2 つの表と要約の 1 行の字・`--digest` の字・Scan の要約値の式と切り方・関数 agree・床の口 check_index・`folio hello` が使う関数 counts・節点の種類と辺の型と辺の欄の閉じた一覧・索引の欄の決まりの生成区間（`design-intent/graph.yaml`・graph.rs の定数 FLOOR）・終了コードの意味・要件書と設計文書・fixture と凍結 anchor の file。folio2 自身の床の結果と `folio build` の出力（35 file）は base と byte で同じ（(e) の 1）。

### (c) 歯（f180_・base で 0 件）

歯の file は新しい `crates/folio/tests/graph_summary.rs`（5 本・どれも binary 経由）。期待の字はどれも歯の側の手書き（凍結 anchor）で、歯は python も外部ライブラリも使わない（repo に PyYAML を持ち込まない＝憲法 A-3.1・席の裁定 2026-09-28）。

1. **f180_each_node_of_the_print_has_one_line（実の正本 `design-intent`）。** `--print` と `--print --summary` を撃ち、行の数が節点の表の行の数と同じで、同じ順の各行が表の id・種類・file の字で始まり、続く line の数の後に表の題の字が来て、最後が eng の欄と閉じ括弧。line の数の行（1 始まり）には `id: <その id>` が書かれていて、その後は `,`・`}`・空白・行の終わりのどれか。出力は改行で終わり、タブと復帰を含まない。**base は --summary を読めず 2 ＝RED。**
2. **f180_the_frozen_base_lines_are_the_hand_written_ones（床の土台 `tests/fixtures/floor_base/design-intent`）。** 7 節点の行が歯の側に手で書いた字と byte で一致する（凍結 anchor・P-10.1）: 条 N-3（constitution.yaml 463 行・eng は規範文 N-3.1 の字）・規範文 P-6.2（201 行・plain は null）・規則行 R-5（rules.yaml 33 行・eng は what）・要件 FR8（srs.yaml 195 行・eng は shall）・受入基準 AC11（406 行・題は 37 字なので表の題は 36 字で切れ、eng は 37 字の全文）・受入基準 AC12（409 行・eng は題の字）・登場人物 folio-v2（92 行・plain も eng も null）。判断の記録 ADR-9 は、行の頭が id・種類・file・line 5（file の頭の注の後の `id: ` の行）・題・平易文の頭の字で、eng が決定の欄の頭の字で始まる。便 179 が土台に足す行 R-19（rules.yaml の 44 行目の後に 1 行）と条 P-2 の relations の直し（同じ行の中）は、選んだ行の番号も字も動かさない（重ねた写しで緑）。**base は 2 ＝RED。**
3. **f180_text_fields_are_escaped_into_one_json_line（土台の写し・手で 6 か所を変える）。** (i) 要件 FR8 の平易文を二重引用符の YAML の字（タブ・改行・二重引用符・逆斜線・U+0001 を escape で持つ）にし、shall の後に本文の欄 text を足す。(ii) 登場人物 folio-v2 の流れの形の行に shall（空の値）・decision・what（二重引用符を含む字）・plain（空の値）を足す。(iii) 条 N-3 の規範文 N-3.1 の前に id の無い行（text だけ）を足す。(iv) 受入基準 AC12 の題を二重引用符の YAML の字（二重の空白と改行を持つ 48 字）にする。(v) 条 N-3 の平易文を `|` の塊（2 行・末尾の改行）にする。(vi) 規則行 R-5 の流れの形の行に、what の前へ本文の欄 text を足す。行の数は節点の数のままでタブを含まず、FR8 の行は平易文が逆斜線の t・n・二重引用符・逆斜線・u0001 の escape で手書きの字と一致し eng は shall の字（text より先）、folio-v2 の行は plain が null で eng が what の字（空の値の shall を飛ばし、decision より先）、N-3 の eng は N-3.1 の字（id の無い行は規範文でない）、AC12 の行は題が空白を畳んで 36 字に切れ、eng が逆斜線の n を含む 48 字の全文（畳まず切らない）。N-3 の行は全体が手書きの字と一致し、plain が塊の 2 行と末尾の改行を逆斜線の n で持つ（eng は N-3.1 の字）。R-5 の行は全体が手書きの字と一致し、eng が text の字（what より先・題は what の字のまま）。**base は 2 ＝RED。**
4. **f180_an_unbuildable_index_prints_no_line。** 土台の写しから srs.yaml を消すと 2 で行を 1 つも出さず、`まだ分からない` と `srs.yaml を読めない` を出す。規範文 P-1.1 の頭の行を行の逐語で切れない形に崩す（YAML としては読める・今の歯 f99 と同じ崩し方）と 2 で行を出さず `食い違う` を出す。`--summary` だけと `--digest --summary` は 2 で標準出力が空。旗の順（`--summary --print`）を替えても出力は同じ。**base は 1 つ目の場合の断りが clap の字（まだ分からない を含まない）＝RED。**
5. **f180_the_summary_leaves_the_print_and_the_digest_on_their_anchors。** 土台で `--print --summary` が節点の行を出し、`--print` の出力の sha256 が凍結 anchor `tests/fixtures/schema/graph-anchor.txt` の出力全体の値と、`--digest` の出力が `tests/fixtures/schema/graph-digest-anchor.txt` と byte で一致する。anchor は file から読む（便 179 が anchor を組み直しても、この歯は本文を変えずに緑）。**base は --summary で 2 ＝RED。**
6. **RED の実測。** 歯だけの差分（r180-teeth-revb.patch = 歯の file・16,388 byte）を base に当てると f180_ の 5 本とも落ちる（本文は revb-red.log・どれも `unexpected argument '--summary' found` の断りで落ちる）。
7. **既存の歯は本文も fixture も変えない。** 歯の file の graph（便 94・96・99・136 の歯）と hello は変えずに通る（共通の検証の workspace の nextest で撃つ・(f) の 5）。

### (d) 採らなかった形

1. **`--print` の節点の表に列を足す（行の番号・平易文・技術の要約をタブ区切りの 6〜8 列目に）。** 既存の `--print` の byte と凍結 anchor 3 本と、tsuzuri の今の読み手が変わる。平易文の改行とタブを表に入れるには独自の escape が要る。合意した形は JSON Lines。
2. **YAML の読み手の型 Node に行の印を持たせる。** 床と面の全部の読み手が使う型を変える大きな差分になる。Scan が同じ頭の行を既に見つけていて、agree が索引と同じ集合であることを確かめているので、行の番号はそこから取る。
3. **1 行に要約値（digest）と辺を入れる。** 合意した欄は 7 つ。要約値と辺は `--print` に在る（tsuzuri は `--print` から辺を組む）。
4. **歯を今の歯の file の graph に足す。** 便 179 の write-set（歯の file の graph と hello）と重なり、受付の順が縛られる。新しい歯の file にして、verify から今の graph の歯の行を外した（今の歯は共通の検証で撃つ・(c) の 6 が `--print` と `--digest` の不変を f180_ の中で縛る）。
5. **folio に依らない独立の実装（python と PyYAML）を repo に置き、歯から撃って全行を byte で突き合わせる。** 外部ライブラリ PyYAML を repo に持ち込む＝憲法 A-3.1 の持ち主の確認が要る（検証役の B1・席の裁定 2026-09-28 で採らない）。同じ性質（`|` の塊の末尾の改行・欄の順）は歯 3 の手書きの行で縛り、全行の突き合わせは起草と検証の記録（repo の外の道具）に残した。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写し（見本 d2fe440）で、workspace の nextest **1008 / 1008**（1003 + f180_ の 5 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file（base と全 file の sha256 の要約が同じ ffe238314d34f598）。`--print` と `--digest` の出力は、folio2 の置き場・床の土台・tsuzuri の写しの 3 つとも base の binary と byte で同じ（reva-tsuzuri.log）。
2. **突然変異（見本の写しの graph.rs だけを 1 通りずつ変え、f180_ の 5 本を撃つ）。** 17 通りとも f180_ のどれかが落ちる（revb-mut.log・各変異の本文は mut-M<n>.out）。M15〜M17 は検証役の変異で改訂 a の歯を生き残った 3 つ（text と what の順・`|` の塊の末尾の改行）。落ちる歯の略: 各行 = each_node_of_the_print_has_one_line・手書き = the_frozen_base_lines_are_the_hand_written_ones・escape = text_fields_are_escaped_into_one_json_line・anchor = the_summary_leaves_the_print_and_the_digest_on_their_anchors。

| 変異 | 落ちる f180_ |
| --- | --- |
| M1 技術の要約の欄の順を text → shall にする | escape |
| M2 条の技術の要約を条そのものの欄から取る | 手書き・escape |
| M3 条の技術の要約を id の無い行も含む最初の行から取る | escape |
| M4 空の値の欄を飛ばさない | escape |
| M5 条・規則行・要件書の行の番号を 0 始まりにする | 各行・手書き・escape |
| M6 判断の記録の行を file の 1 行目にする | 各行・手書き |
| M7 規範文の行を条の頭の行にする | 各行・手書き |
| M8 無い欄を null でなく空の字で書く | 手書き・escape |
| M9 平易文を題と同じく畳む | 手書き・escape |
| M10 平易文と技術の要約を escape せずに書く | 各行・escape |
| M11 --summary を読まずに索引の表を出す | 各行・手書き・escape・anchor |
| M12 Scan の行の番号を索引へ渡さない（line が 0） | 各行・手書き・escape |
| M13 受入基準の技術の要約を表と同じ切った題にする | 手書き・escape |
| M14 受入基準も 4 つの欄から取る（題を使わず null） | 手書き・escape |
| M15 技術の要約の欄の順を shall → what → text にする | escape |
| M16 技術の要約の欄の順を what を先頭にする | escape |
| M17 平易文の末尾の改行を落とす | escape |

3. **外の置き場（tsuzuri の写し）。** 見本の binary の `folio graph --print --summary` は 236 行・終了コード 0・起草役の確かめの道具（repo の外の python の独立の実装）の出力と byte で一致・python の json.loads で全行が読め、line の行に id が在る（revb-tsuzuri の段・revb-chain.txt）。tsuzuri の要件 FR15 の行は line 237・kind 要件・plain と eng がその要件の欄の字。受入基準 17 本の eng はどれも題の全文。
4. **2 つの見本を重ねた写し。** d2fe440 に便 179 の見本 21b44b3 を merge した写しで、workspace の nextest 1016 / 1016・clippy 0・床 4 本 rc 0・build 35 file・f180_ 5 本とも緑・folio2 の置き場の 235 行（行 R-19 を含む）が repo の外の独立の実装と byte で一致（revb-chain.txt）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** `crates/folio/src/graph.rs` と `crates/folio/src/main.rs` は印なし（どちらも増える）、歯の file（crates/folio/tests/graph_summary.rs）は `+`（新しい file・既存の dir の下）。差分 29,903 byte（`git diff eec0c31 d2fe440 | wc -c`・3 file・+356 −13）。
2. **余地（CapHeadroom）。** 測るのは write-set の src の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した（revb-chain.txt の余地の段）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/graph.rs` | 805 | 695 | 868（+63） | 632 |
| `crates/folio/src/main.rs` | 682 | 818 | 686（+4） | 814 |

   歯の file は src の外なので余地を測らない（参考値 276 行）。
3. **size は S。** src の増分は +67（1 本あたり最大 +63）で S の見積 100 の内、余地の最小 632 は S の 100 を超える。
4. **verify は 2 行**で、done の 2 つの塊と 1 対 1 に揃える。便の後の写しで 2 行とも rc 0（verify-180.sh）。base では 1 が rc 101（歯の file が無いので cargo が試験の的 graph_summary が無いと断る）、2 は rc 0。
   1. `cargo nextest run -p folio --test graph_summary f180_` = (c) の 1〜5（5 本）。
   2. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は graph_summary の 1 本（新しい file）で write-set に在る。今の graph と hello の歯（`--print` と `--digest` の凍結 anchor と節点の数を縛る）は verify に置かず、共通の検証（`.vessel.toml` の common-verify = workspace 全体の nextest と clippy）で撃つ＝write-set を便 179 と重ねない（(d) の 4）。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0。
2. **受付。** 本便は要件書 第 1.52 版（docs/srs152）の着地の後に受け付ける（FR14 の字が本流に無いうちは受け付けない）。便 179 とは write-set が重ならず、順はどちらでもよい。受付の先撃ち（precheck）は起草の記録に在る（契約に起因する断りの数）。共通の検証は同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-28/d180-draft.md`、差分と script は同じ dir の d180-scripts（repo には入れない）。

1. 要件書の下書き: srs152.py（eec0c31 と同じ字を当てる・201a40a の上でも 866bb78 の上でも同じ結果・2 回撃っても同じ）。
2. 模擬: 差分 c180-revb.patch を base に当て、run-180.sh（組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と要約）。
3. RED: red-180.sh（歯の file だけを base の写しへ写して f180_ を撃ち、消す）。
4. 突然変異: mut-180.py（(e) の 2 の 17 通り・最後に graph.rs を戻して組み直す）。改訂 b の撃ち直しは chain-b.sh（1〜6 を逐次に）。
5. 外の置き場と folio2 自身の置き場: tsuzuri-180.sh（t3v の tz3 を scratch へ clone し、base と見本の binary で `--print`・`--digest` の byte 比べと、`--print --summary` と repo の外の独立の実装 node-summary.py〔PyYAML と json・起草の記録の確かめの道具で歯ではない〕の比べ）。
6. 余地と差分の byte: lines-180.sh（lines.py と lines.awk）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書の字（第 1.52 版は docs/srs152 が運ぶ）・索引の欄の決まりの生成区間（graph.yaml に --summary の欄 7 つを写すなら生成区間の規則の変更＝行 D-17 の別の承認）・`--print` と `--digest` の字・床・面・fixture と凍結 anchor の file・tsuzuri の側の読み手（tsuzuri の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) plain と eng は YAML の字の値（折り返しと escape を解いた後の字）だけを写す。一覧や表の値の欄は null（今の folio2 と tsuzuri の置き場には無い）。数や真偽の字面は字として写す（12 は字の 12）。(2) 条の eng は最初に id を持つ規範文の欄で見て、条そのものの欄は見ない。受入基準は 4 つの欄のどれも持たないので eng は題の欄の全文（受入基準の red_test は使わない）、登場人物と出力は eng が null。受入基準の行が 4 つの欄のどれかを持っても、eng は題の全文（種類で決める・条と同じ扱い）。(3) line は id が書かれた頭の行で、Scan が索引と同じ集合を切れたときだけ出る（引用符つきや注釈つきの id は今どおり まだ分からない）。(4) 歯は手書きの字で縛るので、手書きの行に無い形（YAML の書き方の組み合わせ）は各行の歯 1 の頭と題と line の突き合わせだけで縛る。folio に依らない独立の実装との全行の突き合わせは起草役と検証役の確かめ（repo の外）で、folio2・床の土台・tsuzuri の写しの 3 つで byte 一致を記録に残した。
3. **撤退条件。** (1) 要件書 第 1.52 版の FR14 の字が本流に無いか違えば、受け付けずに席へ返す。(2) 受付の時点で本流の graph.rs の関数 run・Scan の関数 file・Index、または main.rs の命令 graph が base（eec0c31）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に `--print` か `--digest` の出力（folio2 の置き場・床の土台）、folio2 自身の床 4 本の結果、`folio build` の出力（35 file）が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/graph.rs` の定数 ENG_FIELDS と関数 eng・Index の欄 2 つと関数 texts と関数 jsonl・Scan の欄 lines と行の番号の記録 3 か所・run の引数 summary と出力の分かれ・4 つの読み口から texts を呼ぶ 5 か所・頭の注。`crates/folio/src/main.rs` の旗 --summary と run への受け渡しと説明の字。新しい歯の file（5 本）。
- 入れない: `--print` と `--digest` の字・Scan の要約値の式・agree・check_index・counts・閉じた一覧・生成区間・設計文書・fixture と凍結 anchor の file・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| lines | 行の番号 | `graph.rs` の Scan が節点の頭の行の番号を覚え、run が索引へ渡す |
| texts | 平易文と技術の要約 | `graph.rs` の Index の関数 texts と関数 eng（plain と、ENG_FIELDS のうち最初に字を持つ欄・条は最初の規範文・受入基準は題の全文） |
| jsonl | 1 節点 1 行 | `graph.rs` の Index の関数 jsonl（欄 7 つ・yaml.rs の json_str の escape） |
| flag | 旗 | `main.rs` の --summary（--print と一緒のときだけ） |
| teeth | 歯 | 新しい歯の file の f180_ 5 本（期待の字は手書き） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない（歯は python を使わない・PyYAML を repo に持ち込まない＝憲法 A-3.1）。新しい dir は無い。
- 前提の着地: 要件書 第 1.52 版（docs/srs152）。便 94（索引）・便 99（Scan と要約値）は本流に在る。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ `folio graph --print --summary` の着地を返す（欄 7 つ・(e) の 3）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ga"
title = "索引の口の節点ごとの要約（要件書 第 1.52 版の FR14）: folio graph --print --summary は、表の代わりに、節点ごとに id・種類・所属 file・1 行の題（表と同じ字）に、所属 file の中でその id が書かれた行の番号（1 始まり）と平易文の欄 plain の字と技術の要約（shall・text・what・decision のうち最初に字を持つ欄の字・条はその最初の規範文の字・受入基準は題の全文）を加えた 1 行の JSON（欄 7 つ id kind file line title plain eng をこの順に空白なしで・無い欄は null・字は yaml.rs の json_str の escape）を、節点の表と同じ順に出す。crates/folio/src/graph.rs に定数 ENG_FIELDS と関数 eng と Index の関数 texts と関数 jsonl を足し、行の逐語の読み手 Scan が節点の頭の行の番号を覚えて run が agree の前に索引へ渡し、run は引数 summary で jsonl を出す。crates/folio/src/main.rs の命令 graph に旗 --summary（--digest とは一緒に撃てない・--print と一緒のときだけ）。組めなければ今どおり行を出さずに まだ分からない（2）。既存の --print と --digest の字・要約値の式・床・閉じた一覧・生成区間・設計文書は変えない。歯は新しい歯の file crates/folio/tests/graph_summary.rs の f180_ の 5 本（実の正本の各行と表の突き合わせと line の行の id・床の土台の 7 節点の手書きの行と ADR-9・タブと改行と引用符と逆斜線と制御の字と | の塊の escape と欄の順〔text が what より先〕と空の値と id の無い行と受入基準の題の全文・組めない置き場と旗の組の 2・--print と --digest の凍結 anchor）で、期待の字はどれも手書き（歯は python も外部ライブラリも使わない）。実装の見本は origin の枝 impl/d180 の commit d2fe440（60236c4 に docs/srs152 の eec0c31 を merge し改訂 a と改訂 b を積んだもの）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。設計文書の正本を書き換えないので門の対象外。base = 枝 docs/srs152 の要件書 第 1.52 版（201a40a と改訂 a の eec0c31）が本流に着地した後の main"
req = ["FR14"]
section = "1"
write-set = ["crates/folio/src/graph.rs", "crates/folio/src/main.rs", "+crates/folio/tests/graph_summary.rs"]
verify = ["cargo nextest run -p folio --test graph_summary f180_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "crates/folio/tests/graph_summary.rs の f180_ の 5 本（実の正本で節点の表の各行に 1 行ずつ同じ順・同じ id と種類と file と題で出て line の行に id: <id> が書かれ、床の土台の N-3 と P-6.2 と R-5 と FR8 と AC11 と AC12 と folio-v2 の行が手書きの字と一致し〔受入基準の eng は題の全文で AC11 は 36 字で切れた題と 37 字の eng〕 ADR-9 は line 5 で eng が決定の欄の字、タブと改行と二重引用符と逆斜線と U+0001 を持つ平易文と | の塊の 2 行と末尾の改行の平易文が JSON の escape で 1 行に収まり eng は shall が text より先・text が what より先で空の値の欄を飛ばし条は id を持つ最初の規範文の字で、二重の空白と改行を持つ受入基準の題は表では畳んで 36 字・eng では全文、srs.yaml の無い置き場と行の逐語で切れない置き場では 2 で行を出さず まだ分からない、--summary だけと --digest --summary は 2、--print と --digest の出力は凍結 anchor のまま）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
