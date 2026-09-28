# 設計: 便 202 — 外の置き場で folio2 の条の番号を名指さない（下限を数えなかった知らせと下限を割った違反の字・--polarity の説明の字）

- 要件: FR29（止める仕掛けの一覧〔極性一覧〕を出し、その場で止める仕掛けの本数を下限と比べる・要件書 第 1.55 版）。本流の要件書に在る id で、規範文・確かめ方・受入基準は変えない（FR29 は知らせと違反の字の中身を定めない）。
- 条: P-6.3（外の置き場の字も生成区間と同じ関数で導く＝外の判定と番号の落とし方の 2 つ目の式を持たない）・P-4.2（数えなかったことは外でも 1 行出す）・P-10.1（期待の字は歯の側の手書き）。
- 出所: 台帳 f2-648.275.8（tsuzuri の写しで便 200 の binary を撃った実測・handoff-2026-09-28/tsuzuri-pin.md の 2 回目の節）。同じ種類の前例 = 台帳 f2-648.261（便 194・`docs/design/delivery-194.md`・外の置き場で folio2 の番号の跡を残さない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gw` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 6 本（src 4・歯の file 2）。縮む file・消す file・新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 6 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 3f9ec88**（便 200 の着地 a000dce・便 199 の着地の後）。この契約の数はすべて 3f9ec88 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d202`（commit **1842864**・親 3f9ec88）。`git diff 3f9ec88 1842864` が便の全体の差分（6 file・+194 −15・22,930 byte）。見本は 3 commit（0b706a6 が本体・7016401 は歯の file の頭の注を clippy の決まりに合わせて段落に分けただけ・1842864 は検証役の N-202-2 で説明の字の歯に assert を 1 行足しただけ）。**作業者は write-set の file をこの commit の中身にしてよい**。write-set の外は変えない。
- 並行の便との重なり: §1 (g) の表。
- 昇格条件との関係: 台帳の昇格条件（次の小さな直しの便の束・tsuzuri の道の上・S）。編集時の止めの便（f2-648.275.2 の 198〜200）は着地済みで、polarity.rs を触るほかの便は無い＝単独の便で運ぶ。

## 1. 設計

### (a) いま起きていること（base 3f9ec88 の実測・参考値）

1. **素の床の知らせ。** `crates/folio/src/polarity.rs` の定数 OFF「# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外・条 P-18.4）」を、`crates/folio/src/main.rs` が置き場の名に依らず素の床の標準エラーへ出す。外の置き場（骨格 `folio init`・tsuzuri の写し）でも 条 P-18.4 が出る。
2. **下限を割った違反の字。** `polarity.rs` の `check_floor` は、名札 P-18 と末尾の（P-18.4）を置き場に依らず積む。骨格に閾値の行 R-9（条 P-1・欄 key in-loop-min・値 1 本以上）を足した写しで、素の床と編集時の口（`folio check --proposed`）がどちらも「[P-18] 極性一覧の編集時（in-loop）の仕掛けが 0 本で、行 R-9 の下限 1 本以上を割る（P-18.4）」を出す（起草の記録の sk-202.log）。
3. **取り違え。** tsuzuri の条 P-18 は「設計文書の提示層は単一の生成器が所有する」で、P-18.4 は「部品・図の型・密度 profile は閉じた一覧（部品目録）で持ち…」。folio2 の条 P-18 に当たるのは tsuzuri の条 P-14（逸脱は編集の時点で止める（極性一覧））で、下限の行は tsuzuri の行 R-11（条 P-14・値 1 本 以上・欄 key なし）。tsuzuri の写し（tsuzuri-pin の基準 bb1d94b に base の `schema --write` を当てた写し）の素の床は標準エラーに 条 P-18.4 の 1 行を出し、行 R-11 に欄 key in-loop-min を付けて値を 8 本以上にした写しは「[P-18] …7 本で、行 R-11 の下限 8 本以上を割る（P-18.4）」を出す（tz-202.log）。
4. **--polarity の出力。** 各行の名は置き場の条と行と床の定数の仕掛けの名で、集計の行も folio2 の番号を持たない（骨格・tsuzuri の写しとも base と見本で byte で同じ）。ただし説明の字（`folio check --help` の --polarity）に 便 200 が在る。説明の字は置き場に依らず、`main.rs` の説明の字で便の番号を持つのはこの 1 つだけ。
5. **全数の表（polarity.rs と便 198〜200 の本流の commit ca8640a・a000dce・3f9ec88 が src に足した字のうち、利用者に見える字で folio2 の番号を持つもの）。**

| # | 置き場所 | 字 | 本便 |
| --- | --- | --- | --- |
| 1 | `polarity.rs` 定数 OFF（素の床の標準エラー） | 条 P-18.4 | 拾う（外は項を落とす） |
| 2 | `polarity.rs` の `check_floor` の名札 | P-18（違反の行の頭 [P-18]） | 拾う（外は名札 polarity） |
| 3 | 同じ違反の字の末尾 | （P-18.4） | 拾う（外は落とす） |
| 4 | `main.rs` の --polarity の説明の字 | 便 200 | 拾う（置き場に依らず落とす） |
| 5 | `rules.rs` の欄 key の注（規則の表の生成区間） | 判断の記録 ADR-31 決定 (2)(ア)・ADR-33 決定 (5) | 拾わない（生成区間は外で `floor.rs` の `text_for` が落とす・tsuzuri-pin の実測で跡 0） |
| 6 | `adr.rs` の後継の輪の違反の字 | （P-7.2） | 拾わない（便 199 は字を動かしただけ・前からの族） |
| 7 | `proposed.rs` の単体の歯の path | adr/ADR-1.yaml | 拾わない（利用者に見えない） |

6. **同じ族の前からの字（本便の外・参考）。** 骨格の素の床の まだ分からない の 6 行のうち 5 行が folio2 の条の番号を持つ（凍結 anchor の行の A-2・N-4・P-10.3 と、骨格の印の 4 行の 条 P-17.3）。前からの違反の名札（A-2・N-4・P-7・P-8・R-3）と字（（P-8.1）・（P-7.2）ほか）も外で出る。行 R-17 の知らせの 行 R-17 は、道具が置き場の行を id で読む口の名で、番号の跡ではない。
7. **base の歯（参考値）。** workspace の nextest 1145 / 1145・clippy 0 警告・床 4 本 rc 0（folio2 の素の床は標準エラーに 1 の今の字の知らせを出す）・`folio build --write` 39 file。`git grep -n f202_ -- crates` は 0 件・行 id `gw` は 0 件。

### (b) 直す先

1. **`crates/folio/src/floor.rs`。** 外の置き場の判定 `abroad`（名が在って folio2 の置き場の名でない）と置き場へ写す値の字 `val_for`（folio2 の置き場と名の無い口は定数のまま・外は `text_for` で folio2 の番号の印を持つ括弧の項と文を落とす・注の外で何も残らなければ空の字）を、crate の中から呼べるようにする（可視性と注だけ・式は変えない）。
2. **`crates/folio/src/polarity.rs`。** 置き場へ出す字を 1 つの関数にする: 定数の字を `val_for` の注の外として通す。素の床の知らせは、置き場の名を `adr::place_name`（`folio schema` と床の欄の決まりの突き合わせが使う読み口）で読んで、この関数で出す（外は「…（床の判定の外）」）。`check_floor` は置き場の名を引数に取り、下限を割った違反の名札を `abroad` なら polarity・さもなくば P-18 にし、字の末尾の（P-18.4）は同じ関数に通す（外は落ちる）。定数 OFF は module の中だけで使う。
3. **`crates/folio/src/check.rs`。** `check_dir` が `adr::place_name(dir)` で置き場の名を読み、`check_floor` へ渡す（1 行増える）。編集時の口（`folio check --proposed`）は置き場を写した dir で同じ `check_dir` を撃つので、同じ名で同じ字になる。
4. **`crates/folio/src/main.rs`。** 素の床の知らせを置き場の dir から組んだ字で出す。--polarity の説明の字から「便 200・」を落とす（「（正本が読めなければ まだ分からない 2）」）。
5. **外の置き場の字（見本の binary の実測）。** 知らせ =「# 欄 key が in-loop-min の閾値の行が規則の表に無い＝編集時の止めの本数の下限は数えていない（床の判定の外）」。違反 =「[polarity] 極性一覧の編集時（in-loop）の仕掛けが <数> 本で、行 <置き場の行の id> の下限 <数> 本以上を割る」。
6. **変えないもの。** folio2 の置き場と名の無い口（置き場の名を読めない）の字（名札 P-18・末尾の（P-18.4）・条 P-18.4 の知らせ）・判定（下限の数え・違反かどうか・まだ分からない の字・--polarity の出力）・`floor.rs` の外の判定と番号の落とし方の式・床の定数の字・生成区間。folio2 の `folio check --help` は 1 行変わる（--polarity の説明の字から 便 200 を落とす・説明の字は置き場に依らない・検証役の N-202-1）。

### (c) 歯（f202_・base で 0 件・2 本と既存の歯 1 本の字の期待）

1. **単体 f202_notice_and_violation_follow_the_place_name（`polarity.rs` に tests の区間を新しく置く）。** 手書きの規則の表（欄 key in-loop-min の行 R-9・値 1 本以上）と空の憲法で、名が無い・folio2-constitution は名札 P-18 と末尾の（P-18.4）・知らせは今の字、tsuzuri-constitution・folio2・未記入（骨格の名）は名札 polarity と末尾なし・知らせは外の字（歯の側の手書き）。下限の行が無い規則の表は数えなかった真を返して違反 0。
2. **`tests/polarity.rs` f202_abroad_place_names_no_folio2_article。** 外の置き場の最小の写し = 骨格（git init の後に `folio init`・名は 未記入）に閾値の行 R-9（条 P-1・値 1 本以上・条 P-1 の relations に R-9）を足して 1 commit。(a) 欄 key 無しの写しの素の床: 標準エラーに外の知らせがちょうど 1 行・今の字の知らせは 0 行・P-18 を持つ行が標準出力と標準エラーに 0（骨格の憲法に条 P-18 は無い）。(b) 欄 key in-loop-min の写しの素の床: 違反の行はちょうど「[polarity] 極性一覧の編集時（in-loop）の仕掛けが 0 本で、行 R-9 の下限 1 本以上を割る」・知らせ 0 行・P-18 0。編集時の口: 欄 key 無しの写しに欄 key を付けた rules.yaml を渡すと終了 1・違反の行は同じ 1 行・P-18 0。(c) --polarity: 終了 0・集計の行「folio check --polarity: 仕掛け 10（in-loop 0・post 10）・下限 1 本以上（行 R-9）に足りない」・R-9 の行が在る・P-18 0。説明の字: `folio check --help` が「（正本が読めなければ まだ分からない 2）」を持ち、便 200 と P-18 を持たない（P-18 は検証役の N-202-2）。folio2 の置き場（床の土台・名は folio2-constitution）: 行 R-13 に欄 key を付けて値 99 本以上の写しの違反は「[P-18] …行 R-13 の下限 99 本以上を割る（P-18.4）」、欄 key 無しの写しの知らせは今の字 1 行・外の字 0 行。
3. **既存の歯の字の期待 1 か所。** `tests/mechanism_live.rs` の f156_a_place_without_r17_says_the_mentions_are_not_counted は骨格（外）の標準エラーの末尾 2 行を見る。その 2 行目の期待を外の字にする（定数 IN_LOOP_OFF_ABROAD を足す・判定と件数は同じ）。床の土台（folio2 の名）を見る f131_ の 2 本の期待（今の字 IN_LOOP_OFF）は変えない。
4. **RED の実測。** 歯の file 2 本だけ（見本の tests の字）を base に当てると、tests/polarity.rs の f202_ の 1 本（8 本のうち 1 本・外の知らせの行が 0 行で期待の 1 行と違う）と tests/mechanism_live.rs の f156_ の 1 本（6 本のうち 1 本・骨格の標準エラーの末尾が今の字）が落ちる（nextest の rc 100・起草の記録の red-202-polarity.log・red-202-mechanism_live.log・1842864 の歯の file で撃ち直して同じ）。base の `--bin folio f202_` は 0 本（単体の歯は src の中・nextest の rc 4）。

### (d) 採らなかった形

1. **置き場の憲法から下限を持つ条を引いて名指す（台帳の候補 2）。** 下限の行の欄 article を読む新しい読み口が要り、行の無い知らせ（数えなかった 1 行）には引く条が無い。外の条の番号が置き場ごとに正しくても、folio2 の置き場で名札が行の条に依って動く（今の字と判定を変えない、に触れる）。番号を落とす形は便 194 と同じ関数で済む。
2. **字を置き場に依らず番号なしにする。** folio2 の置き場の字が変わる（依頼の変えてはいけないもの）。folio2 の歯（tests/polarity.rs の f200_ の 2 本・tests/mechanism_live.rs の f131_ の 2 本）の期待も書き換わる。
3. **違反の字を丸ごと生成区間の落とし方に通す。** 字の中の 行 R-n（置き場の自分の行の id）が folio2 の番号の印に数えられ、文ごと落ちて空の字になる（便 174 の印の決まり）。落とすのは folio2 の条を名指す末尾の 1 項だけにした。
4. **外の置き場の名札を空の字にするか、置き場の条を名札にする。** 空の字は [] の行になり、素の床の違反の行の形（[名札] 字）が崩れる。条を名札にするのは 1 と同じ読み口が要る。名札は口の名（--polarity）と同じ polarity にした（ほかの名札も adr・schema・note・intake のように検査の名）。
5. **--polarity の説明の字を残す。** 説明の字は置き場に依らず外の利用者にも出る。main.rs の説明の字で便の番号を持つのはこの 1 つだけで、ほかの説明の字の形に揃える。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写しで workspace の nextest 1147 / 1147（base 1145 + f202_ の 2 本・0b706a6 で撃った・後の 2 commit は歯の file の注と assert 1 行だけ違う）・clippy 0 警告（1842864）・床 4 本 rc 0・`folio build --write` 39 file が base と全 file で byte で同じ。folio2 の素の床の標準出力と標準エラーは base と同じ（知らせは今の字）。字の期待を直した既存の歯は (c) 3 の 1 か所だけ。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、単体の f202_ と floor::tests・tests/polarity.rs・tests/mechanism_live.rs を撃つ）。** 15 通りとも落ちる（生き残り 0・起草の記録の mut-202.log〔M1〜M14・0b706a6〕と mut-202-M10-M15.log〔1842864〕・落ちた歯の本文は mut-202-M*.log）。注の中か外かの旗を変える形（注の中として通す）は、名札でない 2 つの字では同じ答えになる等価な変異なので数えない。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 素の床の知らせを定数のまま出す | tests/polarity.rs の f202_・mechanism_live の f156_ |
| M2 違反の字の末尾の条を落とさない | 単体・tests/polarity.rs の f202_ |
| M3 名札を常に P-18 | 単体・tests/polarity.rs の f202_ |
| M4 名札を常に polarity（folio2 の字が変わる） | 単体・f202_・f200_ の下限を割る歯 |
| M5 folio2 の置き場でも番号を落とす | 単体・f202_・f200_ の 2 本・mechanism_live の f131_ の 2 本 |
| M6 床が置き場の名を渡さない | tests/polarity.rs の f202_ |
| M7 知らせの名を読まない | tests/polarity.rs の f202_・mechanism_live の f156_ |
| M8 外の判定を folio2 を含むかにする（便 174 の検証の V1） | 単体・floor の f194_ |
| M9 名札の分岐を逆にする | 単体・f202_・f200_ の下限を割る歯 |
| M10 説明の字に 便 200 を戻す | tests/polarity.rs の f202_ |
| M11 外の名札を空の字にする | 単体・tests/polarity.rs の f202_ |
| M12 名が読めない口も外と扱う | 単体・floor の f174_ |
| M13 床の名を folio2 の置き場の名に固定する | tests/polarity.rs の f202_ |
| M14 口が今の知らせの字を直に出す | tests/polarity.rs の f202_・mechanism_live の f156_ |
| M15 説明の字に 条 P-18.3 を足す（検証役の N-202-2） | tests/polarity.rs の f202_ |

3. **外の置き場。** 骨格の写し（(c) 2 と同じ編集・起草の記録の sk-202.log）と tsuzuri の写し（(a) 3・tz-202.log）で、base と見本の答えの違いは次の 3 か所だけで、ほかの標準出力・標準エラー・終了コード・--polarity の出力・`folio build` の出力（tsuzuri 72 file）は byte で同じ。
   - 素の床の標準エラーの知らせ: 「…（床の判定の外・条 P-18.4）」→「…（床の判定の外）」。
   - 下限を割った素の床の違反の行: 「[P-18] …行 R-11 の下限 8 本以上を割る（P-18.4）」→「[polarity] …行 R-11 の下限 8 本以上を割る」（tsuzuri の写しの行 R-11 に欄 key を付けて値を 8 本以上にした写し・終了 1 は同じ。値を 7 本以上にすると足りて合格 0 も同じ）。
   - 編集時の口（欄 key の無い写しに付けた後の rules.yaml を渡す）の止める行: 同じ字の違い（終了 1 は同じ）。
   - 見本の出力で P-18 を持つ行は、tsuzuri の --polarity の自分の条 P-18 の 1 行（出所 憲法の条の機構）だけ。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・縮む file・消す file は無い。`crates/folio/src/polarity.rs` は単体の歯（f202_）を持つので verify の `--bin folio f202_` の scope。
2. **余地（CapHeadroom）。** write-set の src の 4 本（python と awk の 2 実装で一致・起草の記録の cap-202.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/polarity.rs` | 120 | 1380 | 179（+59・単体の歯を含む） | 1321 |
| `crates/folio/src/floor.rs` | 792 | 708 | 794（+2） | 706 |
| `crates/folio/src/check.rs` | 1171 | 329 | 1172（+1） | 328 |
| `crates/folio/src/main.rs` | 778 | 722 | 778（±0） | 722 |

3. **size は S。** src の増分は +62 で S の見積 100 の内。余地の最小（check.rs の base 329）は S の 100 を超える。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。見本の 1842864 で 6 行とも rc 0（1・1・8・6・6 本と clippy 0 警告・verify-impl3.log）。
   1. `cargo nextest run -p folio --bin folio f202_` = (c) 1（1 本）。
   2. `cargo nextest run -p folio --test polarity f202_` = (c) 2（1 本）。
   3. `cargo nextest run -p folio --test polarity` = 便 200 の f200_ の 7 本（folio2 の置き場の字と判定が変わらない）と (c) 2。
   4. `cargo nextest run -p folio --test mechanism_live` = (c) 3 と床の土台の今の知らせ（f131_）。
   5. `cargo nextest run -p folio --bin folio floor::tests` = 便 174・194 の外の字の落とし方の単体の歯（可視性だけ変えた関数の式が変わらない）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり対象外・0（通す）。本流の binary（`target/debug/folio`・05:12 の組み立て）に write-set の 6 本を渡した答え「folio ceiling: 通す（設計文書の正本を書き換えない便）」（gate-202.log）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d202 の本契約で 契約に起因する断り 0（起草の記録の precheck-202.log）。見本の写しで verify の 6 行とも rc 0。
3. **ほかのレーンとの重なり。** 起草の時点（2026-09-29 05:4x）の origin の impl/* の枝で write-set の file を書き換え、本流に着地していないのは impl/d201（`crates/folio/src/main.rs`・地図の節点の裁定 id のレーン・取り下げ済み）だけで、`git merge-tree` は衝突 0。ほかの枝（impl/d172〜d200）は本流に squash で着地済み。床の穴の 188・189 は便 187 に畳まれ、検査の信頼の 191 は見本の枝が無い。本便の write-set の file を書き換える便がほかに先に着地したら、本便は余地と歯の数を数え直してから運ぶ。
4. **数え直し。** 受付の時点の本流で write-set の file か、`floor.rs` の `abroad`・`val_for`・`text_for` が base と違えば、見本を本流に取り込み、verify と骨格と tsuzuri の写しの答え（(e) 3）を数え直してから運ぶ。
5. **着地の後（外の置き場）。** 生成区間も床の判定も変わらないので、外の置き場の file の書き直しは要らない。新しい binary では、素の床の標準エラーの知らせが「…（床の判定の外）」になり、下限の行を置いて割ったときの違反の行は [polarity] で始まって末尾の条を持たない。席は tsuzuri へこの 1 行を返す。参考（本便は運ばない）: tsuzuri の行 R-11 の値「1 本 以上」は欄 key in-loop-min を付けると値の形（<正の整数> 本以上）が違い、素の床と --polarity が まだ分からない になる（見本の binary で実測）。tsuzuri が欄 key を付けるなら値を「1 本以上」に直す要がある。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/p184-draft.md`、script と log は同じ dir の `p184-scripts/`。
1. `run-202.sh <木> <印>`: 組み立て・workspace の nextest（--test-threads 2）・clippy・床 4 本・`folio build --write` の file 数と sha（base と見本で撃って比べる）。
2. `red-202.sh <base の木> <見本の rev>`: 歯の file 2 本だけを base に当てて RED（落ちた歯の本文を log に残す）。
3. `mut-202.py <見本の clone>`: 変異 14 通り（1 通りずつ src を変え、単体の f202_ と floor::tests・tests/polarity.rs・tests/mechanism_live.rs を撃つ）。
4. `cap-202.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk の 2 実装）。
5. `verify-202.sh <木> <印>`: 契約の verify の 6 行。
6. `tz-202.sh <base の folio> <見本の folio>`: tsuzuri の写し（tsuzuri-pin の基準 tzc の clone・hook なし）で base と見本の答えを比べる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** folio2 の置き場の字と判定・`floor.rs` の外の判定と番号の落とし方の式・床の定数の字・生成区間・要件書・規則の表・外の置き場の file（tsuzuri の行 R-11 の値の形を含む）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 本便が拾うのは polarity.rs と便 198〜200 の足した字（§1 (a) 5 の表）だけで、前からの床の字が folio2 の条の番号を名札と字に持つ族（名札 A-2・N-4・P-7・P-8・R-3、字の（P-8.1）・（P-7.2）・（P-10.3）・条 P-17.3 など）は外でも出る（骨格の素の床で まだ分からない の 3 種 4 行・§1 (a) 6）。同じ族の全数の直しは本便の外（席へ返す）。
3. **撤退条件。** (1) 要件書・判断の記録・憲法・床の定数の字を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に §1 (c) 3 の 1 か所のほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、folio2 の素の床の標準出力と標準エラーか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/polarity.rs` の置き場へ出す字（知らせ・違反の名札・末尾の条）と単体の歯・`crates/folio/src/check.rs` が置き場の名を下限の数えへ渡す 1 行・`crates/folio/src/main.rs` の知らせの出し方と --polarity の説明の字・`crates/folio/src/floor.rs` の 2 つの関数の可視性・歯 `tests/polarity.rs` の f202_ 1 本と `tests/mechanism_live.rs` の骨格の知らせの字の期待 1 か所。
- 入れない: folio2 の置き場の字と判定・`floor.rs` の外の判定と番号の落とし方の式・床の定数の字・生成区間・要件書・規則の表・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| notice | 知らせの字 | `polarity.rs` が置き場の名で下限を数えなかった知らせを出す（外は条の番号の項を落とす） |
| short | 違反の字 | `polarity.rs` の下限を割った違反の名札と末尾の条（外は名札 polarity・末尾なし） |
| name | 置き場の名 | `check.rs` と `main.rs` が `adr::place_name` で読んで渡す |
| help | 説明の字 | `main.rs` の --polarity の説明の字から便の番号を落とす |
| teeth | 歯 | 単体の f202_ 1 本・`tests/polarity.rs` の f202_ 1 本・`tests/mechanism_live.rs` の字の期待 1 か所 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 200（a000dce）と便 199（本流 3f9ec88）。
- 本便の着地の後に席が見ること: 台帳 f2-648.275.8 を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) の 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gw"
title = "外の置き場で folio2 の条の番号を名指さない（台帳 f2-648.275.8・便 200 の後・同じ種類の前例は便 194）: crates/folio/src/polarity.rs は、素の床の下限を数えなかった知らせと下限を割った違反の字を置き場の名で出し分ける。置き場の名は adr::place_name で読み（crates/folio/src/check.rs が下限の数えへ渡し、crates/folio/src/main.rs が知らせを出す）、外の置き場（名が folio2 の置き場の名でない）では生成区間と同じ crates/folio/src/floor.rs の abroad と val_for（可視性だけ変える）で、知らせから 条 P-18.4 の項を落とし、違反の名札を polarity にして末尾の（P-18.4）を落とす。folio2 の置き場と名の無い口の字と判定と --polarity の出力は変えない。main.rs の --polarity の説明の字から 便 200 を落とす。歯は polarity.rs の単体の f202_ 1 本と tests/polarity.rs の f202_ 1 本（骨格 folio init を外の置き場の最小の写しにし、素の床・編集時の口・--polarity・説明の字に P-18 が無く、床の土台は今の字）と tests/mechanism_live.rs の骨格の知らせの字の期待 1 か所。実装の見本は origin の枝 impl/d202 の commit 1842864（親 3f9ec88）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 3f9ec88"
req = ["FR29"]
section = "1"
write-set = ["crates/folio/src/polarity.rs", "crates/folio/src/floor.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/tests/polarity.rs", "crates/folio/tests/mechanism_live.rs"]
verify = ["cargo nextest run -p folio --bin folio f202_", "cargo nextest run -p folio --test polarity f202_", "cargo nextest run -p folio --test polarity", "cargo nextest run -p folio --test mechanism_live", "cargo nextest run -p folio --bin folio floor::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "binary の単体の f202_ の 1 本（名の無い口と folio2 の置き場は名札 P-18 と末尾の（P-18.4）と今の知らせ、名の等しくない置き場は名札 polarity と末尾なしと外の知らせ、下限の行が無ければ違反 0）が緑、tests/polarity.rs の f202_ の 1 本（骨格の素の床の知らせ・下限を割った違反の字・編集時の口・--polarity の出力・説明の字に folio2 の条の番号 P-18 が無く、床の土台は今の字）が緑、tests/polarity.rs の歯の全部（f200_ の 7 本の folio2 の置き場の字と判定）が緑、tests/mechanism_live.rs の歯の全部（床の土台の今の知らせ・骨格の外の知らせ）が緑、binary の単体の floor::tests の全部（外の字の落とし方の式）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
