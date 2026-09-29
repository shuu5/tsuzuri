# 設計: 便 194 — 外の置き場で folio2 の番号の跡を残さない（生成区間の字の壊れ・相談窓口の上限の違反の字）

- 要件: FR19（文書の決まりの部分を機械の中の決まりから各 file の生成区間へ写す・schema）・FR1（相談窓口・質問の数の上限の違反の字）。本流の要件書に在る id で、字は変えない。
- 条: P-5.6（床の定数の写しを人が読める型付きデータとして決定的に導く）・P-6.3（folio2 の字も外の字も同じ関数で導く）・P-4.1（外の注が出所の無い裁定を名指さない）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 台帳 f2-648.261（便 174 の独立の検証 d174-verify の非 blocking の提案 1〜3・字の壊れ 4 件と歯の生き残り V1・V7）と、台帳 f2-648.75 の付け足し（持ち主に問う 4 件の材料 asks-draft.md §1・席の依頼 2026-09-28）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `go` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 5 本（src 2・歯の file 3〔本文不変 1〕）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 5 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 6467915**（便 185 の着地の後）。この契約の数はすべて 6467915 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d194`（commit **898cf8e**・親 6467915）。`git diff 6467915 898cf8e` が便の全体の差分（4 file・+157 −7・15,376 byte）。**作業者は write-set の file をこの commit の中身にしてよい**。write-set の外は変えない。
- 並行の便との重なり: 起草の時点の見本の枝（親はどれも 6467915）で、`tests/schema.rs`（本便は本文不変）は便 190（impl/d190・検査の信頼）と便 196（impl/d196・写しの負債）が字の期待を直し、便 190 は床の定数 `floor_adr.rs` の注の字を、便 196 は `floor_note.rs` の注の欄を変える（外の生成区間の字が動きうる）。`crates/folio/src/intake.rs` は便 197（impl/d197・写しの負債）が頭の床の木と定数を触る（本便は違反の字だけ・別の塊で git merge-tree は衝突なし）。本便は導出の関数（`floor.rs` の `text_for` まわり）と違反の字だけで、定数の字は変えない。**本便は 187・190 と写しの負債の便より先に受け付ける（席の順・2026-09-28）**。後から着地する便（190・196・197）の側で、tests/place_name.rs の f194_ の歯と骨格の生成区間の差を撃ち直す（起草の時点では、本便に 190 と 196 を重ねた写しと 197 を重ねた写しで、binary と tests/place_name.rs・schema.rs・intake.rs の歯が全部緑＝mall-194-190-196.log・mall-194-197.log）。本便より先に着地した便が在れば、受付の時点の本流で数え直す（§1 (g)）。
- 昇格条件との関係: 台帳の昇格条件（外の置き場の天井の周がこの注を場所にした所見を出す）は未到来。直しは導出の関数だけで安く、外の利用者が自分で直せない字（生成区間）なので、所見を待たずに運ぶ（席の判断）。

## 1. 設計

### (a) いま起きていること（base 6467915 の実測・参考値）

1. **外の置き場の生成区間の字の壊れ 3 件。** folio2 の置き場の名（憲法 meta.id が `folio2-constitution`）でない置き場では、床の定数の字を `crates/folio/src/floor.rs` の `text_for` に通し、folio2 の番号の印を持つ括弧の項と文を落とす（便 174）。骨格（`folio init`）の 9 本の生成区間で、folio2 の同じ欄と比べて外にだけ在る壊れ（独立の数え abroad-261.py）:
   - 詰まり: `adr/schema.yaml` の id_pattern_note「folio2 の他の **idと**同じく」（括弧「（P-1・R-7・FR1）」を丸ごと落として英字と和字が接する）。
   - 宙に浮く指す語: `design-note/schema.yaml` の derived_note「…のはそのため。**どちらも**面の生成器も…」（指し先の文「導出物を組む口（folio derive --write）と差分を数える口（folio derive --check）は便 119 で入った。」が便の番号で丸ごと落ちる）。
   - 出所の抜け: 同じ file の figures_note「図の対（**持ち主の裁定 2026-09-19**）」（同じ括弧の台帳の id の項「f2-648 notes」だけが落ち、外では出所の無い持ち主の裁定として読める）。
   - 検証の記録が挙げた 4 件目「idを」（ruling_pattern_note）は、便 181 で注の字が書き換わって消えた。
2. **歯の生き残り（便 174 の検証）。** V1（名の判定を「folio2 を含む」にする）と V7（注で何も残らない欄を空の字で書く）は今の歯で落ちない。
3. **tsuzuri の写し（参考値）。** HEAD 0918967（pin 6467915）の生成区間は骨格と同じ壊れを持つ（独立の数え abroad-261.py で詰まり 1・出所の抜け 1・derived_note の「どちらも」1）。base の binary で床 3 本と build は 0。
4. **相談窓口の上限の違反の字が folio2 の要件の id を名指す。** `crates/folio/src/intake.rs` は質問が上限（定数 MAX_QUESTIONS = 5）を超えると「intake.yaml: questions が FR1 の上限を超える」を出す。外の置き場（tsuzuri）の FR1 は別の意味（面の数）で、6 問目を足した利用者は原因を辿れない（asks-draft.md §1 の ②）。tsuzuri の写しの質問は 5 問ちょうど（今は出ない）。上限の値そのものを規則の表の行へ移す件（行 R-18 の追加）は同じ台帳の本体で、席が持ち主へ問う（本便は運ばない）。
5. **base の歯（参考値）。** workspace の nextest 1050 / 1050・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`git grep -n f194_ -- crates` は 0 件・行 id `go` は 0 件。

### (b) 直す先 — `crates/folio/src/floor.rs` と `crates/folio/src/intake.rs`

1. **詰まり。** `drop_marked_items` で括弧ごと落としたとき、前の字と後ろの字が英数字と和字（平仮名・片仮名の字と長音・漢字）か英数字どうしで空白なしに接するなら、空白を 1 つ置く（関数 `wa` と `joins`）。読点・中黒・句点とは接しても置かない。
2. **宙に浮く指す語。** `text_for` の文の段で、落とした文の直後の文が、語が文の主の指す語（定数 `POINTERS` = これ・その）で始まるなら続けて落とす（続く限り）。抜いても文が立つ指す語（定数 `POINTER_ADVERBS` = どちらも）で始まるなら、その語だけを落として文の中身は残す（derived_note の「面の生成器も様式の file も呼ばず、導出物の置き場（--out）は消費側が宣言する（既定なし）。」は外にも残る・独立の検証の N3）。指す語の前の空白は除いて見る。落としていない文の後ろの指す語の文は、語も文も残す。
3. **出所の抜け。** `drop_marked_items` の項の段で、同じ括弧の中に台帳の id の項（`has_ruling`）が在れば、「持ち主の裁定」（定数 `OWNER_RULING`）を含む項も落とす。台帳の id の無い括弧の「持ち主の裁定」の項は残す。
4. **`crates/folio/src/intake.rs`。** 上限の違反の字を「intake.yaml: questions が上限 5 問を超える（<数えた問の数> 問）」にする（上限は定数 MAX_QUESTIONS から・数えた数は質問の行の数）。判定（5 問を超えたら違反）と種別 intake と定数は変えない。
5. **変えないもの。** folio2 の置き場と名の無い口の字（定数のまま・folio2 の 9 本の生成区間と凍結の写しは 1 byte も変わらない）・印の決まり（`marked`）・床の定数の字・欄を書くかの決まり・突き合わせの規則（`floor_diff_for` は同じ関数を通る）。外の置き場で変わるのは (a) 1 の 3 つの注の字だけ（骨格で実測・(e) 3）。相談窓口の上限の違反の字は、置き場の名に依らず folio2 でも外でも同じ字になる（folio2 の正本は 5 問ちょうどで出ない）。

### (c) 歯（f194_・base で 0 件）

1. **単体 f194_text_for_leaves_no_broken_joins（`floor.rs` の tests の区間）。** 手書きの 16 通り（英数字と和字の両向き・英数字と数字・和字どうし・読点と中黒の前・指す語の続けての落とし〔3 文の鎖〕・どちらも は語だけを落とす〔続けて落とした文の後ろでも〕・指す語の前の空白・落としていない文の後ろの指す語は残す・台帳の id の在る括弧と無い括弧の「持ち主の裁定」）と、定数 `POINTERS`・`POINTER_ADVERBS`・`OWNER_RULING` の字の針と、`abroad` が名の等しさで決まる（`folio2x-constitution` も `folio2` も外・検証の V1）。
2. **`tests/place_name.rs` f194_abroad_regions_leave_no_broken_joins。** 外の置き場（骨格を tsuzuri の名に替えて `folio schema --write`）の 9 本の生成区間で、英数字と和字が接する 2 字の並びは folio2 の同じ file の生成区間に在る並びだけ。id_pattern_note は「id と同じく」・adr/schema.yaml に `grill_note:` が無い（検証の V7）・derived_note の行に「どちらも」が無く（ほかの注の括弧の中の「どちらも」は指す語でないので見ない）、区間に「のはそのため。面の生成器も様式の file も呼ばず、導出物の置き場（--out）は消費側が宣言する（既定なし）。値に二重引用符」と続く・figures_note は「図の対＝設計ノートの図は」で「（持ち主の裁定 2026-09-19）」が無い。
3. **`tests/intake.rs` f194_intake_limit_names_the_number_not_fr1。** folio2 の正本の写し（5 問ちょうど）は合格、7 問にすると違反はちょうど 1 行「[intake] intake.yaml: questions が上限 5 問を超える（7 問）」で、出力に FR1 の字が無い。既存の歯 intake_over_the_question_limit_fails の字の期待を「questions が上限 5 問を超える（6 問）」に直す（判定と件数は同じ）。
4. **RED の実測。** 歯の file だけ（見本の tests の字）を base に当てると tests/place_name.rs の f194_ の 1 本と tests/intake.rs の f194_ の 1 本が落ち（red-194.log・nextest の rc 100）、tests/intake.rs の全部では字の期待を直した intake_over_the_question_limit_fails も落ちる（15 本のうち 2 本・red-194-intake.log）。base の `--bin folio f194_` は 0 本（定数と関数が無い・nextest の rc 4）。

### (d) 採らなかった形

1. **床の定数の字そのものを外でも壊れない字に書き直す。** folio2 の生成区間と凍結の写し（`tests/fixtures/schema/*-region.txt`）が変わり、写しの負債のレーンの定数と重なる。導出の関数で直せば folio2 の字は変わらない。
2. **指す語の文を落とさず、指し先の文を残す（番号だけ落とす）。** 文の中の番号を消すと文の形が崩れる（「口は で入った」）。便 174 の「印を持つ文は丸ごと落とす」を保った。
3. **括弧を落とした跡の空白を常に置く。** 和字どうし（「4 桁（ADR-0047）は」→「4 桁は」）や読点の前に余計な空白が入る。
4. **指す語の文を常に文ごと落とす（付け足しの前の見本 74cecb6 の形）。** 外の読み手にも真の字（導出物の置き場は消費側が宣言し既定は無い・面の生成器を呼ばない）まで消える（独立の検証の N3）。
5. **指す語をすべて語だけ落とす。** 語が文の主の指す語は、抜くと助詞が頭に残る（「これは同じ。」→「は同じ。」）か、別の語が崩れる（「そのため足す。」→「ため足す。」）。語だけを落とすのは、抜いても文が立つ どちらも に限った。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写しで workspace の nextest 1053 / 1053（base + f194_ の 3 本）・clippy 0 警告・床 4 本 rc 0（folio2 の `schema --check` 一致）・`folio build --write` 37 file（base と全 file が byte で同じ）。字の期待を直した既存の歯は (c) 3 の 1 本（intake_over_the_question_limit_fails）だけ。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f194_ と便 174 の f174_ と tests/intake.rs を撃つ）。** 17 通りとも落ちる（生き残り 0・見本 f0a31b7 で全部を撃ち直した mut-194c.log と、place_name の歯を直した 898cf8e で place_name が拾う 6 通りを撃ち直した mut-194d.log・定数を変える変異は配列の長さを保つ形）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 括弧を落とした跡に空白を置かない | 単体・place_name |
| M2 和字の後の英数字には空白を置かない | 単体 |
| M3 和字を ASCII でない字全部にする（読点の前にも空白） | 単体 |
| M4 指す語の文を続けて落とさない | 単体 |
| M5 一度落としたら後ろを全部落とす | 単体・place_name・便 174 の単体 f174_text_for… |
| M6 台帳の id が無くても持ち主の裁定の項を落とす | 単体 |
| M7 持ち主の裁定の項を落とさない | 単体・place_name |
| M8 名の判定を folio2 を含むかにする（便 174 の検証の V1） | 単体 |
| M9 注で何も残らない欄を空の字で書く（便 174 の検証の V7） | place_name |
| M10 文ごと落とす指す語を これ だけにする | 単体 |
| M11 上限の違反の字を FR1 の上限を超える に戻す | intake の f194_・字の期待を直した既存の歯 |
| M12 5 問ちょうども違反にする | intake の f194_ と既存の歯 10 本（正本の写しが合格しない） |
| M13 数えた問の数の代わりに上限を出す | intake の f194_・字の期待を直した既存の歯 |
| M14 どちらも の文を語だけでなく文ごと落とす（74cecb6 の形・独立の検証の N3） | 単体・place_name |
| M15 落としていない文の後ろでも どちらも を落とす | 単体 |
| M16 指す語の前の空白を除かない（独立の検証の V2） | 単体 |
| M17 語だけを落とす指す語を その にする | 単体・place_name |
3. **外の置き場。** 骨格（`folio init`）の 9 本の生成区間は base と見本で (a) 1 の 3 つの注の字だけが違う（sk-diff-194.log と見本 f0a31b7 の sk-diff-194c.log〔898cf8e は歯の file だけが違う〕・独立の数え abroad-261.py の 3 形とも 0 件）。tsuzuri の写し（HEAD 0918967）では見本の binary の `folio schema --dir design-intent --check` が 1（adr/schema.yaml の生成区間が 1 byte 違う）で、`folio schema --write` は 2 file（adr/schema.yaml・design-note/schema.yaml）の 3 つの注だけを書き直す（ほかの 7 本は変わらない）。check・derive・build（45 file）は base の binary と同じ答えと byte。tsuzuri の写しの相談窓口は 5 問ちょうどで違反は出ない。写しに 6 問目を足すと、base の binary は「questions が FR1 の上限を超える」、見本の binary は「questions が上限 5 問を超える（6 問）」を出す（tz-intake6-194.log・写しの file は scratch の別の dir に写して撃った）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** `crates/folio/tests/schema.rs` は本文を変えない（verify の `--test` の scope）。
2. **余地（CapHeadroom）。** write-set の src の 2 本（python と awk の 2 実装で一致・cap-small.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/floor.rs` | 716 | 784 | 792（+76・単体の歯を含む） | 708 |
| `crates/folio/src/intake.rs` | 252 | 1248 | 256（+4） | 1244 |

3. **size は S。** src の増分は +80 で S の見積 100 の内。余地の最小（floor.rs の base 784）は S の 100 を超える。
4. **verify は 7 行**で、done の 7 つの塊と 1 対 1 に揃える。見本の写しで 7 行とも rc 0。
   1. `cargo nextest run -p folio --bin folio f194_` = (c) 1（1 本）。
   2. `cargo nextest run -p folio --test place_name f194_` = (c) 2（1 本）。
   3. `cargo nextest run -p folio --bin folio floor::tests` = 便 174 の単体の歯（外の字の 3 段の落とし）を含む床の機械の歯。
   4. `cargo nextest run -p folio --test place_name` = 便 154・174・175・177 の置き場の名と生成区間の歯。
   5. `cargo nextest run -p folio --test schema` = folio2 の 9 本の生成区間と凍結の写しの byte 一致（folio2 の字が変わらない）。
   6. `cargo nextest run -p folio --test intake` = (c) 3（f194_ 1 本と字の期待を直した 1 本）と相談窓口の床の歯の全部。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/small の本契約で 契約に起因する断り 0（preflight ok・起草の記録の precheck-194.log）。見本の写しで verify の 7 行とも rc 0。
3. **数え直し。** 受付の時点の本流で `floor.rs` か、床の定数（`floor_*.rs`）の注の字が base と違えば、見本を本流に取り込み、骨格の生成区間の差（(e) 3）と verify を数え直してから運ぶ。
4. **着地の後（外の置き場）。** 新しい binary を入れた外の置き場は、生成区間の字が変わる file で `folio schema --dir design-intent --check` が 1 になる。席は tsuzuri へ「`folio schema --dir design-intent --write` → commit」を 1 行で返す（便 174 と同じ運び）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/small-draft.md`、script と log は同じ dir の small-scripts。独立の数え abroad-261.py（外と folio2 の生成区間の注の欄を対にし、外にだけ在る詰まり・宙に浮く指す語・出所の抜けを数える）。ほかは便 192 の (h) と同じ（run-small.sh・build-cmp.sh・red-small.sh・`mut-small.py <見本の写し> 194`・cap-small.sh・prep-tz.sh と tz-small.sh）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 床の定数の字・相談窓口の上限の値とその置き場（行 R-18 の件）・folio2 の生成区間・印の決まり・外の置き場の生成区間の書き直し（利用者の `schema --write`）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 指す語は 3 つの字（文ごと落とす これ・その と、語だけを落とす どちらも）で始まる文だけを見る（「それ」「この」で始まる文は今の定数に指し先の落ちる例が無く、足していない）。英数字と和字の詰まりは括弧を落とした跡だけを直し、定数の字にもともと在る詰まりは変えない（folio2 の同じ file に在る並びは歯も許す）。
3. **撤退条件。** (1) 床の定数の字か要件書・判断の記録・憲法の字を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、folio2 の 9 本の生成区間か、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `floor.rs` の `wa`・`joins`・`POINTERS`・`OWNER_RULING` と `drop_marked_items`・`text_for` の直し・`intake.rs` の上限の違反の字・歯の f194_ の 3 本と intake の歯 1 本の字の期待。
- 入れない: 床の定数の字・上限の値とその置き場（行 R-18 の件は席が持ち主へ問う）・folio2 の生成区間・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| join | 詰まりの空白 | `floor.rs` の `wa`・`joins`（括弧ごと落とした跡） |
| pointer | 指す語 | `floor.rs` の `POINTERS`（落とした文の直後の文も落とす） |
| owner | 出所 | `floor.rs` の `OWNER_RULING`（台帳の id の在る括弧の持ち主の裁定の項） |
| limit | 上限の字 | `intake.rs` の質問の数の上限の違反の字（数で言う） |
| teeth | 歯 | 単体の f194_ 1 本・`tests/place_name.rs` の f194_ 1 本・`tests/intake.rs` の f194_ 1 本と字の期待 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 185（本流 6467915）。
- 本便の着地の後に席が見ること: 台帳 f2-648.75 の付け足し（違反の字）を済みと記帳する（行 R-18 の本体は残る）。台帳 f2-648.261 の本文の 4 件と V1・V7 を閉じる（同じ台帳の notes の追記〔graph.yaml の注・門の見張りの字ほか〕は運ばない＝残す）。本流の `target/debug/folio` を組み直す。tsuzuri へ (g) 4 の 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "go"
title = "外の置き場で folio2 の番号の跡を残さない（台帳 f2-648.261・便 174 の検証の非 blocking の提案・台帳 f2-648.75 の付け足し）: crates/folio/src/floor.rs の drop_marked_items は、括弧ごと落として前後の字が英数字と和字（か英数字どうし）で接するなら空白を 1 つ置き（関数 wa と joins）、同じ括弧に台帳の id の項が在れば 持ち主の裁定 を含む項も落とす（定数 OWNER_RULING）。text_for は、落とした文の直後の文が語が文の主の指す語（定数 POINTERS = これ・その）で始まれば続けて落とし、抜いても文が立つ指す語（定数 POINTER_ADVERBS = どちらも）で始まればその語だけを落として文の中身は残す。folio2 の置き場と名の無い口の字・床の定数の字・印の決まり・folio2 の 9 本の生成区間は変えず、外の置き場の骨格では 3 つの注の字だけが変わる。crates/folio/src/intake.rs は質問の数の上限の違反の字を questions が上限 5 問を超える（数えた数 問）にして folio2 の要件の id を名指さない（判定と定数は変えない）。歯は floor.rs の単体の f194_ 1 本と tests/place_name.rs の f194_ 1 本と tests/intake.rs の f194_ 1 本と既存の歯 1 本の字の期待（tests/schema.rs は本文不変）。実装の見本は origin の枝 impl/d194 の commit 898cf8e（親 6467915）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 6467915"
req = ["FR19", "FR1"]
section = "1"
write-set = ["crates/folio/src/floor.rs", "crates/folio/src/intake.rs", "crates/folio/tests/place_name.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/intake.rs"]
verify = ["cargo nextest run -p folio --bin folio f194_", "cargo nextest run -p folio --test place_name f194_", "cargo nextest run -p folio --bin folio floor::tests", "cargo nextest run -p folio --test place_name", "cargo nextest run -p folio --test schema", "cargo nextest run -p folio --test intake", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "binary の単体の f194_ の 1 本（外の字の詰まりに空白・指す語の文の続けての落とし・どちらも は語だけを落とす・台帳の id の在る括弧の持ち主の裁定・名の等しさで外を決める）が緑、tests/place_name.rs の f194_ の 1 本（外の 9 本の生成区間の詰まりが folio2 の同じ file に在る並びだけ・grill_note が無い・どちらも が無く導出物の置き場は消費側が宣言する（既定なし）の文は残る・図の対 の出所の抜けが無い）が緑、binary の単体の floor::tests の全部が緑、tests/place_name.rs の歯の全部が緑、tests/schema.rs の歯の全部（folio2 の生成区間と凍結の写しが byte で同じ）が緑、tests/intake.rs の歯の全部（上限の違反の字が 上限 5 問を超える で FR1 を名指さない・5 問ちょうどは合格）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
