# 設計: 便 193 — 面の小さな直し（列挙の印が「。**」「。」」の後も項を分ける・表紙の日付が席の裁定の行を読む・裁定の枡の鎖の歯）

- 要件: FR16（判断の記録の面を正本から逐語で生成する）・FR9（設計ノートを 1 つの型で生成する）・FR4（入口・憲法・要件書の面を 1 つの生成器で出す）。本流の要件書に在る id で、字は変えない。
- 条: P-6.1 / P-6.2（面は正本から生成し、正本は変えない）・P-6.3（列挙の印の数えを判断の記録の面と設計ノートの面で 1 つの関数にする）・P-12.2（承認欄の行は役の字で持ち主の承認と席の裁定を分けたまま・面の承認欄の表は役の字をそのまま出す）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 台帳 f2-648.231（天井の 49〜51 周目の読みやすさ・判断の記録の面の列挙が分かれない）・f2-648.229（一括 27 の帰結・規則の表の行 D-17 の注「席の裁定の行は、台帳 f2-648.229 の便までは面の表紙の日付に読まれない」）・f2-648.244 の 2 つ目（一括 30 の検証の N6・裁定の枡の鎖で落ちない変異 R4・R5・R6・R9）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gn` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 11 本（src 6〔縮む 1〕・歯の file 5）。消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 11 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 6467915**（便 185 の着地の後）。この契約の数はすべて 6467915 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d193`（commit **fb069d6**・親 6467915）。`git diff 6467915 fb069d6` が便の全体の差分（11 file・+280 −52・28,170 byte）。**作業者は write-set の file をこの commit の中身にしてよい**。write-set の外は変えない。
- 並行の便との重なり: 起草の時点の見本の枝で、`face_note.rs` は便 187（impl/d187・床の穴）が触る（契約表の行の verify と done を在るときだけ出す塊・本便の消す関数とは別の塊・git merge-tree は衝突なし）。床の穴のレーンの残り（便 188・189）も面の生成器を触りうる。本便は 187 と 190 を待たずに先に受け付ける（席の順・2026-09-28）。後から着地する便の側で数え直し、本便より先に着地した便が在れば受付の時点の本流で数え直す（§1 (g)）。

## 1. 設計

### (a) いま起きていること（base 6467915 の実測・参考値）

1. **列挙の印が「。**」「。」」の後で分かれない。** 判断の記録の面の `item_marks`（`crates/folio/src/face_adr.rs`）は、項の印「(k) 」の直前（末尾の空白を除く）が句点「。」のときだけ項の頭と数える。前の項が太字の閉じ「。**」かかぎ括弧の閉じ「。」」で終わると、次の項が前の項の中に続けて出る。実の正本で 7 欄: ADR-13 文脈（項 1 → 本文の番号は 9）・ADR-13 決定（7 → 16）・ADR-17 決定（3 → 6）・ADR-18 文脈（3 → 5）・ADR-19 文脈（2 → 6）・ADR-26 決定（6 → 7）・ADR-27 決定（1 → 4・台帳の実例の一覧に無い 1 欄）。「(0)」から始まる欄の (0) は今も前置き。設計ノートの面（`face_note.rs`）は同じ関数の写しを別に持ち、同じ形で分けない（folio2 の設計ノートの散文には該当 0）。
2. **表紙の日付が席の裁定の行を読まない。** `face_labels.rs` の `last_approval` は承認欄の役「承認」の行だけを読む。規則の表の行 D-17 で、席の裁定で発効した版は承認欄に役「席の裁定」の行を記帳する。写しの要件書に席の裁定の行を 1 つ足すと、床は合格のまま・表紙の鮮度の札・版の札・表紙の状態・入口のカードの「更新」の日付は前の承認の日付のまま（今の本流の正本に席の裁定の行は 0）。
3. **裁定の枡の鎖の歯の穴。** 憲法の面の規則の表の裁定の枡（`face_constitution.rs` の `ruling`）は、過去の裁定を小窓へ畳むとき区切り「）・前の裁定 = 」で割った字に閉じ括弧を戻す。今の歯は各片を部分の字で探すので、過去の裁定の閉じ括弧を落とす・最後にも足す・最新の閉じ括弧を落とす・順を逆にする変異（一括 30 の検証の R4・R5・R6・R9）が落ちない。正本の前の裁定を持つ行は 16（2 件以上は 9）。
4. **台帳 f2-648.244 の 1 つ目（src の注 2 つ）は消えた。** 一括 30 は凍結で入らず（2026-09-27 の凍結の仕分け）、`face_constitution.rs` の「正本 rules.yaml で 1 つに揃っている」（前の裁定を持つ行 16）と `face_srs.rs` の「status_note は最初の「。」までが要旨・後ろが版ごとの来歴」（来歴 18,021 字）は今の正本で真。
5. **base の歯（参考値）。** workspace の nextest 1050 / 1050・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`git grep -n f193_ -- crates` は 0 件・行 id `gn` は 0 件。

### (b) 直す先

1. **`crates/folio/src/face_adr.rs`。** 定数 `ITEM_CLOSERS`（印の直前の句点の後ろに来てよい閉じの字・強調の閉じ `**` と鉤括弧の閉じ 」 の 2 つ）を足し、`item_marks` は印の直前の末尾の空白を除いた後、閉じの字を末尾から剥がせるだけ剥がしてから句点を見る。番号が 1 から 1 ずつ増える決まりは変えない。`item_marks` を `pub(crate)` にする。
2. **`crates/folio/src/face_note.rs`（縮む）。** 同じ関数の写しを消し、散文の節は `face_adr.rs` の `item_marks` を呼ぶ（29 行減る）。
3. **`crates/folio/src/face_labels.rs`。** 定数 `DATED_ROLES`（日付を読む役の名・承認 と 席の裁定 の 2 つ）を足し、`last_approval` は承認欄の最後の、役がそのどちらかの行（行の順で最後・日付の大小でない）の日付を返す。日付の名（「承認」か「生成」）と draft の扱いは変えない。
4. **注の字だけ。** `face_labels.rs` の `shelf_updated` の注・`face_srs.rs` の 1 行・`face_index.rs` の 2 行・`face_index_read.rs` の 2 行の「最後の 承認 の行」を「最後の 承認 か 席の裁定 の行」に。
5. **変えないもの。** 面の字の決まり（強調の印・前置きの段落・(0) の扱い）・日付の名の値域・承認欄の表の役の字・憲法の面の裁定の枡の生成器（歯だけ足す）・設計文書の正本・床。`folio build` の出力は判断の記録の面 6 枚（adr-13・17・18・19・26・27）だけが変わる（§1 (e) 1）。

### (c) 歯（f193_・base で 0 件）

1. **単体 f193_item_marks_step_over_the_closers（`face_adr.rs` の tests の区間）。** 「。**」「。」」「。」**」「。**」」と空行を挟む形で印 2 つ・句点の無い閉じの字と読点の後は印 1 つ・閉じの字は 2 つだけ（全角の丸括弧の閉じ「。）」の後は項の頭にしない）。
2. **単体 f193_last_approval_reads_the_seat_ruling_rows（`face_labels.rs` の tests の区間）。** 承認 → 席の裁定・席の裁定 → 承認の順で最後の行・作成とレビューは読まない・一覧に無い役は読まない・draft は読まない。
3. **`tests/face_adr.rs`。** f193_items_split_after_a_bold_or_quote_closer（写しの ADR-2 の文脈で 4 つの項が li 4 つ・句点の無い閉じの字の後は段落のまま 2 通り）・f193_real_records_split_every_numbered_item（実の正本の 7 欄の li の数が本文の番号の数 9・16・6・5・6・7・4 と等しい・発効した記録は封で本文が変わらない）。
4. **`tests/face_note.rs`。** f193_note_prose_splits_after_a_bold_or_quote_closer（写しの設計ノートの散文で「。**」「。」」の後の項が分かれる）。
5. **`tests/face_srs.rs`。** f193_srs_cover_dates_read_the_seat_ruling_rows（写しの要件書の承認の行の後ろに席の裁定の行・承認と席の裁定の順を変えた 3 通りで鮮度の札・版の札・表紙の状態が最後の行の日付・レビューの行は動かさない）。既存の歯 f145_real_srs_cover_dates_follow_the_last_approval と f146_real_srs_figure_captions_and_foot_follow_the_last_approval の歯の側の手書きの読みを「承認 か 席の裁定」に直す（今の正本では同じ答え）。
6. **`tests/face_index.rs`。** f193_index_reads_the_seat_ruling_rows（要件書のカードの「更新」と入口そのものの札が席の裁定の行を読む）。既存の歯 f144_real_cards_follow_the_approvals と f146_real_index_stamp_shelf_and_foot_follow_the_last_approval の手書きの読みを同じく直す。
7. **`tests/face_constitution.rs`。** f193_rules_ruling_chain_rebuilds_the_source（実の正本の前の裁定を持つ行の全部で、枡の見える字と小窓の段落を区切りで繋ぎ直すと正本の裁定の欄と字で等しい）・f193_rules_ruling_chain_of_three_is_exact（写しの行 R-6 を前の裁定 3 件の鎖にして、見える字「甲（1）（2026-09-12） 」と小窓「前の裁定 3 件」と本体の 3 段落が手書きの字のとおり）。
8. **RED の実測。** 歯の file だけ（見本の tests の字）を base に当てると binary の f193_ の 5 本（face_adr の 2・face_note・face_srs・face_index）が落ちる（red-193.log・nextest の rc 100）。face_constitution の f193_ の 2 本は base でも緑（今の正しい振る舞いを縛る歯で、変異 R4〜R9 で落ちる＝(e) 2）。手書きの読みを直した既存の歯 4 本も base で緑（今の正本では同じ答え）。base の `--bin folio f193_` は 0 本（関数と定数が無い・nextest の rc 4）。

### (d) 採らなかった形

1. **閉じの字に全角の丸括弧の閉じ「）」と二重かぎ括弧の閉じ「』」も入れる。** 実の正本に該当 0（独立の数え marks-231.py で `**` と `」` だけで 7 欄が全部揃う）。入れても歯で縛れない字が増える。
2. **席の裁定で発効した版の日付の名を「席の裁定」にする。** 判断の記録と設計ノートの面は、席の裁定で発効した記録の承認欄の日付をすでに名「承認」で出す（行 D-17 の注の形）。名の値域を 3 つにすると面の 5 種と歯の全部が変わる。役の区別は承認欄の表が役の字のまま出す。
3. **設計ノートの面の関数の写しを残して両方を直す。** 同じ式を 2 か所に置く形のまま（P-6.3）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯と面。** 見本の写しで workspace の nextest 1059 / 1059（base + f193_ の 9 本）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file で、base と違う file は 判断の記録の面 6 枚（adr-13・adr-17・adr-18・adr-19・adr-26・adr-27.html）だけ（build-cmp.sh）。字の期待を直した既存の歯は (c) 5・6 の 4 本（手書きの読みの役の条件だけ）。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、f193_ を撃つ）。** 12 通りとも落ちる（生き残り 0・mut-193.log と mut-rerun.log）。M1・M2・M4・M6 は最初に定数の配列の長さを変える形で書き、単体の歯の針（配列の字）で組み立てが通らなかったので、長さを保つ形に書き直して撃ち直した。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 閉じの字からかぎ括弧の閉じを抜く | face_adr の 2 本・face_note・単体の item_marks |
| M2 閉じの字から太字の閉じを抜く | face_adr の 2 本・face_note・単体の item_marks |
| M3 閉じの字を 1 つだけ剥がす | face_adr の items_split・単体の item_marks |
| M4 丸括弧の閉じも剥がす | 単体の item_marks |
| M5 設計ノートの面だけ前の数えのまま | face_note |
| M6 席の裁定の行を読まない | face_srs・face_index・単体の last_approval |
| M7 最初の行を読む | face_srs・face_index・単体の last_approval |
| M8 作成のほかの役を全部読む | face_srs・単体の last_approval |
| M9 R4 過去の裁定の閉じ括弧を落とす | face_constitution の 2 本 |
| M10 R5 最後の過去の裁定にも閉じ括弧 | face_constitution の 2 本 |
| M11 R6 最新の裁定の閉じ括弧を落とす | face_constitution の 2 本 |
| M12 R9 過去の裁定の順を逆に | face_constitution の 2 本 |
3. **外の置き場（tsuzuri の写し・参考値）。** HEAD 0918967（pin 6467915）で、見本の binary の床 3 本と build（45 file）は base の binary と byte で同じ（判断の記録の文脈と決定に閉じの字の後の印は 0・承認欄に席の裁定の行は 0＝独立の数え marks-231.py と grep）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 縮む file は `crates/folio/src/face_note.rs`（write-set では頭に - の印）。新しい file は無い。本文を変えない write-set は無い。
2. **余地（CapHeadroom）。** write-set の src の 6 本（python と awk の 2 実装で一致・cap-small.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/face_adr.rs` | 955 | 545 | 986（+31） | 514 |
| `crates/folio/src/face_note.rs` | 1063 | 437 | 1034（−29） | 466 |
| `crates/folio/src/face_labels.rs` | 723 | 777 | 750（+27） | 750 |
| `crates/folio/src/face_srs.rs` | 1056 | 444 | 1056（0） | 444 |
| `crates/folio/src/face_index.rs` | 799 | 701 | 799（0） | 701 |
| `crates/folio/src/face_index_read.rs` | 631 | 869 | 631（0） | 869 |

3. **size は S。** src の増分は +58（単体の歯を含む）で S の見積 100 の内。余地の最小（face_note.rs の base 437）は S の 100 を超える。
4. **verify は 7 行**で、done の 7 つの塊と 1 対 1 に揃える。見本の写しで 7 行とも rc 0。
   1. `cargo nextest run -p folio --bin folio f193_` = (c) 1・2（2 本）。
   2. `cargo nextest run -p folio --test face_adr` = (c) 3（2 本）と判断の記録の面の歯の全部。
   3. `cargo nextest run -p folio --test face_note` = (c) 4（1 本）と設計ノートの面の歯の全部。
   4. `cargo nextest run -p folio --test face_srs` = (c) 5（1 本）と要件書の面の歯の全部。
   5. `cargo nextest run -p folio --test face_index` = (c) 6（1 本）と入口の面の歯の全部。
   6. `cargo nextest run -p folio --test face_constitution` = (c) 7（2 本）と憲法の面の歯の全部。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0（通す）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/small の本契約で 契約に起因する断り 0（preflight ok・起草の記録の precheck-193.log）。見本の写しで verify の 7 行とも rc 0。
3. **数え直し。** 受付の時点の本流で write-set の src のどれかが base と違えば、見本を本流に取り込み、余地と verify を数え直してから運ぶ。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/small-draft.md`、script と log は同じ dir の small-scripts。独立の数え marks-231.py（判断の記録の文脈・決定と設計ノートの散文の段落で、印の数を今の規則と直しの規則で数える・PyYAML で読む）。ほかは便 192 の (h) と同じ（run-small.sh・build-cmp.sh・red-small.sh・`mut-small.py <見本の写し> 193`・cap-small.sh・tz-small.sh）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 正本の字（判断の記録の本文の番号の書き方）・日付の名の値域・承認欄の表の役の字・一括 30 の刈り込み・設計文書の正本・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 閉じの字の後の印は、句点が閉じの字の中に在るときだけ項の頭になる（「**あ**。 (2)」の形は今までどおり句点の後で分かれる）。閉じの字が 2 つ以外（丸括弧・二重かぎ括弧）の後の印は分けない（(d) 1）。
3. **撤退条件。** (1) 要件書・判断の記録・憲法の字を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に (c) 5・6 のほかの既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果が変わるか、`folio build` の出力が (e) 1 の 6 file のほかで 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `item_marks` の閉じの字と共有・`last_approval` の役の一覧・注の字 4 か所・歯の f193_ の 9 本と既存の歯 4 本の手書きの読みの役の条件。
- 入れない: 面の字の決まりのほか・日付の名・床・設計文書の正本・外の置き場・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| marks | 列挙の印 | `face_adr.rs` の `ITEM_CLOSERS`・`item_marks`（設計ノートの面も呼ぶ） |
| dated | 表紙の日付 | `face_labels.rs` の `DATED_ROLES`・`last_approval` |
| teeth | 歯 | 単体 2 本・面の歯の file 5 本の f193_ 7 本・手書きの読み 4 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 185（本流 6467915）。
- 本便の着地の後に席が見ること: 台帳 f2-648.231・f2-648.229・f2-648.244 を閉じる（.244 の 1 つ目は §1 (a) 4 の理由）。本流の `target/debug/folio` を組み直す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gn"
title = "面の小さな直し（台帳 f2-648.231・f2-648.229・f2-648.244）: crates/folio/src/face_adr.rs は定数 ITEM_CLOSERS（** と 」）を足し、列挙の印の直前の句点の後ろの閉じの字を剥がしてから項の頭と数え、関数 item_marks を設計ノートの面と共有する（crates/folio/src/face_note.rs の写しは消して呼ぶ）。crates/folio/src/face_labels.rs は定数 DATED_ROLES（承認 と 席の裁定）を足し、last_approval は承認欄の最後のそのどちらかの役の行の日付を返す（日付の名と draft の扱いは変えない）。face_labels.rs・face_srs.rs・face_index.rs・face_index_read.rs の注の字を同じ向きに。歯は単体の f193_ 2 本と tests/face_adr.rs・face_note.rs・face_srs.rs・face_index.rs・face_constitution.rs の f193_ 7 本（裁定の枡の鎖の閉じ括弧と順を含む）と既存の歯 4 本の手書きの読みの役の条件。設計文書の正本と床は変えず、folio build の出力は判断の記録の面 6 枚だけが変わる。実装の見本は origin の枝 impl/d193 の commit fb069d6（親 6467915）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 6467915"
req = ["FR16", "FR9", "FR4"]
section = "1"
write-set = ["crates/folio/src/face_adr.rs", "-crates/folio/src/face_note.rs", "crates/folio/src/face_labels.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_index.rs", "crates/folio/src/face_index_read.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/face_constitution.rs"]
verify = ["cargo nextest run -p folio --bin folio f193_", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test face_note", "cargo nextest run -p folio --test face_srs", "cargo nextest run -p folio --test face_index", "cargo nextest run -p folio --test face_constitution", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "binary の単体の f193_ の 2 本（閉じの字 ** と 」を剥がして句点を見る・承認欄の最後の 承認 か 席の裁定 の行）が緑、tests/face_adr.rs の歯の全部（写しで 4 つの項が li 4 つ・実の正本の 7 欄の li の数が本文の番号の数）が緑、tests/face_note.rs の歯の全部（設計ノートの散文も同じ関数で分ける）が緑、tests/face_srs.rs の歯の全部（席の裁定の行の日付が鮮度の札・版の札・表紙の状態に出る）が緑、tests/face_index.rs の歯の全部（要件書のカードの 更新 と入口の札）が緑、tests/face_constitution.rs の歯の全部（裁定の枡の字から正本の裁定の欄を組み直せる・前の裁定 3 件の鎖が手書きの字のとおり）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
