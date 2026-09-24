# 設計: 便 125 — 骨格の命令 folio init --dir（新しい置き場に最初の文書一式を書く）と、憲法の正本が無い置き場で案内の 1 行が骨格の命令を名指す（ADR-16 決定 (7) の ④ の 2 便目）

- 要件: **FR22**（新しい置き場に最初の文書一式〔骨格〕を書く・版 B = 要件書 第 1.34 版で足した id・受入基準 AC19 / AC26 / AC27）/ FR3（気づかせる 1 行・注は版 B で「置き場に憲法の正本が無いときにこの 1 行が案内する先〔骨格の命令〕は FR22 が持つ」と書いた）/ FR19（床の定数から欄の決まりの file の生成区間へ写す＝骨格の生成区間は同じ導出）。契約表の行の req は FR22 と FR3 の 2 つ。版 B は main 2d90e8d に在る（FR22 は main の id）。
- 条: N-1.1（管理下の対象を回復不能に削除しない＝正本が 1 本でも在れば何も書かない・上書きしない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する＝雛形は folio2 の正本と床の定数から導出する）/ P-7.1（条と判断の記録の番号は利用者の番号空間の 1 から振り、folio2 の番号を利用者の file に写さない）/ P-12.2（承認欄の逐語）/ P-4.1・P-4.2（書けなかったものを書いたと言わない）/ P-1.2（採否を代行しない＝骨格は提案中の雛形で、発効は利用者の持ち主の承認）/ P-10.1（独立の凍結 anchor）
- 出所: 判断の記録 **ADR-16**（発効 2026-09-23 17:39 JST・持ち主の承認・対話面 R-8・逐語「裁定は両方承認する」・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (3)（骨格の命令）と (2)(オ)(カ)（最小の条と行・最初の判断の記録）と、決定 (7) の便の列の **④** の 2 便目。1 便目（解決先の式・生成区間の注）は `docs/design/delivery-123.md`（行 `dv`）。判断の記録 ADR-6 決定 (1)（相談窓口は置き場から読む）は変えない。行 D-11 は、この行と契約表の行の title が ADR-16 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `dx` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 8 本。**新しい file は 3 本で、先頭に `+` を付けて宣言する**（src 1・歯 1・凍結の一覧 1）。縮む file も消す file も無く、新しい dir も作らない（凍結の一覧は既に在る dir `tests/fixtures/schema/` に置く）。
- 門: 本便は設計文書の正本（design-intent の下）を 1 本も書き換えない。起草役が write-set 8 本を `folio ceiling --gate --write-set …` に渡すと **0（通す・設計文書の正本を書き換えない便）**（2026-09-24・base 2d90e8d の binary と便 123 の後の binary の両方）。受付の時点の main で撃ち直す。
- 前の便: **base = main 8066e4c（改訂 c で rebase した）。crates・tests・contracts は 2d90e8d から 1 字も変わらないので、この契約の数は base 2d90e8d の実測（参考値）をそのまま使い、受付の時点の main で数え直す**（規則の表の行 D-13）。**受付の順序 = 便 ②（`docs/design/delivery-121.md`〔と 124〕・列の根の表と始まりの凍結）→ 便 ③（`docs/design/delivery-122.md`・値域の部分集合の床）→ 便 123 → 一括 16 → 本便**（ADR-16 決定 (7) の ② → ③ → ④）。**席の裁定: 受付は一括 16（要件書 第 1.35 版・FR22 の規範文の書く file の列挙に graph.yaml を足す版）の発効の後**で、そのときに本文の要件 FR22 の注の引用〔第 1.34 版〕（§1 (b) の 書く file と §1 (d) の 1）を第 1.35 版の規範文の字に改訂してから受付ける。骨格の生成区間は既存の `folio schema --write` の口（schema.rs の run）を呼んで書くので、便 ② が判断の記録の欄の決まりの生成区間を置き場の憲法の名で決まる形にすると、骨格もそのまま同じ形になる（§1 (c) の 1）。
- 分割: ADR-16 決定 (7) の ④「1〜2 便」の 2 便目。本便 = 骨格の命令と案内の 1 行。案内の 1 行を骨格の命令と同じ便に置くのは、案内がまだ無い命令を名指す期間を作らないためである（便 123 の §0 の 分割）。
- 実測の断り: 起草役は本便の Rust の実装を組んでいない。§1 (c) の骨格の中身は、同じ中身を組む Python の試作（持ち主の home の `.local/share/folio2/handoff-2026-09-24/d125-draft-proto.py`・改訂 b で席の裁定 1〜4 の形に直した）で骨格を書き、base の binary・便 123 の後の binary・便 ② ③ と便 123 の起草の patch を全部当てた binary で床を撃った実測である。歯の RED は base に命令が無いことによる（§1 (e)）。変異の表は見込みで、測っていない（§1 (e) の末尾）。

## 1. 設計

### (a) いま起きていること（実測・base main 2d90e8d）

1. **骨格を書く命令が無い。** `crates/folio/src/main.rs` の命令の一覧は 13 本（check・inject・parts・face・figure・build・derive・intake・hello・ceiling・schema・graph・serve）で、新しい置き場に文書一式を書く命令は無い。`crates/folio/tests/check.rs` の歯 p1_commands_closed_list が 13 本の閉じた一覧と全数で突き合わせる。
2. **案内の 1 行は相談窓口の命令を名指す。** `crates/folio/src/hello.rs` の未整備の 1 行（置き場に憲法の正本 constitution.yaml が無いとき）は字「AI に「folio intake」と頼むと相談が始まる」を持つ。相談窓口の命令は置き場の相談窓口の正本（intake.yaml）を読むので、正本の無い新しい repo では「まだ分からない」で止まる（行き止まり・ADR-16 決定 (3)）。`crates/folio/tests/hello.rs` の歯 hello_line_names_intake_and_the_quiet_file がこの字を固定している。整備済み（憲法の正本が在る）の 1 行は「folio: 設計文書 N 節点・M 辺。全体像は folio graph --digest」の形。
3. **生成区間を書く口は 1 つ在る。** `crates/folio/src/schema.rs` の run（pub・旗 Write と Check を受ける）が、9 file（adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml・index.yaml・srs.yaml・vocabulary.yaml・intake.yaml・graph.yaml）の印 2 本の間を床の木の導出で埋める（`folio schema --write` と同じ）。9 file が全部在り、各 file に印がちょうど 1 対在るときだけ合格を返す（1 本でも無い・印が 1 対でないなら「まだ分からない」）。印の字（BEGIN と END）も pub。
4. **骨格の最小の形（起草役の試作・改訂 b）。** 版管理（git の init と commit）の中の空の置き場（design-intent/folio2 と design-intent の 2 通り）に §1 (c) の中身を書き、`folio schema --write` で生成区間を埋めてから床を撃つと、便 123 の後の binary で `folio check` は **違反 0・まだ分からない 2**（凍結 anchor が 0 本〔A-2 / N-4 の差分検査〕と、id の一覧の baseline が無い）で、`folio schema --check` は 9 file 一致で 0、`folio hello --dir <置き場>` は整備済みの 1 行（5 節点・7 辺）、`folio intake --dir <置き場> --print` は質問 q1〜q5 を出す。便 ② ③ と便 123 の起草の patch を全部当てた binary でも同じ（違反 0・まだ分からない 2・便 ③ の値域の部分集合の床も通る）で、`folio check --freeze-start` は憲法の名 floor-declaration が列の根の表に無いので digest の全桁を出して凍結しない（便 ② の設計どおり）。base の binary では違反が 2 件（要件書と語彙の生成区間の注の「id N-3 が実在しない」）増える＝便 123 が先に要る。版管理の外の置き場では「版管理（git）が無いか読めない」の「まだ分からない」が 1 件増える（要件 FR22 の注のとおり）。
5. **判断の記録の日付は年-月-日でないと落ちる。** 試作で最初の判断の記録の date を「未記入」にすると床が「ADR-1.date「未記入」が年-月-日でない」の違反を 1 件出す。欄の決まりの file の meta の date・憲法の meta の generated・規則の表の行の ruling と ruled_at は「未記入」でも床は落とさない。
6. **folio2 の正本をそのまま写すと folio2 固有の id が骨格に載る。** 改訂 a の試作（入口・天井・相談窓口の人が書く節を字のまま写す）では、検証役の数えで folio2 の id が 21 か所に残った。とくに入口の lanes の行 `{doc: adr, at: ADR-8, …}` と `{doc: adr, at: ADR-11, …}` は判断の記録を型付きで名指す欄で、利用者の置き場には ADR-1 しか無く、利用者が後で自分の ADR-8 を書くと別の記録を指す（ADR-16 決定 (2)(オ)「folio2 の番号を予約しない」の向き）。天井の正本の documents の注（便 76・便 77）と忠実さの観点の問い（判断の記録 ADR-8 決定 (1)① の括弧）、相談窓口の sheet の節の文（FR2）にも在った。改訂 b の試作（§1 (c) の 3 の式）では、§1 (e) の歯 7 の数える範囲に 0 回、folio2 自身の design-intent の同じ範囲には 3,320 回（参考値）。
7. **AC26 の禁止字の全数（folio2 の正本の承認欄）。** folio2 の design-intent の全 yaml（preview の下を除く）で、鍵 approval の下の who と verbatim を集め、版付きのモデルの名（字 fable・opus・sonnet・haiku・mythos のどれかに空白と 数.数 が続く字）を足すと候補 30 通り（参考値・who 5・verbatim 24・モデルの名 1）。そのうち骨格の字（床の定数の写しの生成区間か雛形の文）に現れる 2 通り（持ち主・承認する）を除くと **28 通り**（§1 (e) の歯 3 の凍結の一覧）。改訂 b の試作の数える範囲に出た数は **0**、folio2 自身の design-intent の同じ数える範囲には **347**（参考値・歯が効くことの確かめ）。改訂 a の数えが拾っていた字 opus 2 は「opus 2 本」の一部でモデルの名ではないので、モデルの名は 数.数 の形に限る。
8. **相談窓口の正本は型付きの読み書きで往復できない。** 相談窓口の質問の行の鍵 yes と no は、型付きの読み手（`crates/folio/src/yaml.rs` の parse_typed・PyYAML と同じ型）では真偽の鍵になり、同じ file の書き手 write は字でない鍵を書けずに断る（起草役の実測: 書けない値〔文字列でないキー True〕）。憲法と規則の表は往復で値が一致する（起草役の実測）。
9. base の workspace の nextest は全部緑（参考値 796 本）。

### (b) 直す先 — 命令 folio init

**命令の口。**

```
folio init --dir <置き場>
```

- --dir は**既定なし**で必須（置き場は利用者が選ぶ・scribe2 は design-intent/folio2 を選ぶが、それは利用者の値で folio は既定を持たない・ADR-16 決定 (3)）。相対なら撃った場所からの相対。
- 書く file は次の **11 本**（置き場からの相対・この順に書く）: constitution.yaml・rules.yaml・vocabulary.yaml・srs.yaml・index.yaml・intake.yaml・ceiling.yaml（正本 7 file）、graph.yaml（索引の欄の決まり・要件 FR22 の注〔第 1.34 版〕の字）、adr/schema.yaml と adr/ADR-1.yaml（判断の記録の置き場と欄の決まりと最初の判断の記録）、design-note/schema.yaml（設計ノートの置き場と欄の決まり）。置き場と adr/ と design-note/ の dir が無ければ作る（置き場の親 dir が無ければ作らずに 2）。
- **断り（何も書かずに 1）。** 次のどれか 1 つでも在れば（通常の file・dir・symlink・壊れた symlink のどれでも）、1 byte も書かずに 1 で終わり、在ったものの名を全部 1 行ずつ標準エラーに出す: 正本 7 file と graph.yaml の 8 本・adr/・design-note/・anchors/（断りの名 11 個）。置き場に在るほかの file（例 README）は断りの理由にも書き換えの対象にもしない。
- **書く手順（2 段・同じ呼び出しの中）。** (1) 11 本を、各 file を新規作成でだけ開く形（既に在れば開かない）で書く。9 file の生成区間は、印の行 BEGIN と END だけを置いた空の区間で書く（区間の位置は §1 (c) の 1）。(2) 続けて `crates/folio/src/schema.rs` の run を旗 Write で呼び、9 file の空の区間を `folio schema --write` と同じ導出で埋める（schema.rs は 1 字も変えない）。(2) が書き換えるのは (1) で本命令が作った file の印の間だけである。
- **書けない（2）。** 置き場が symlink か dir でない file・置き場の親 dir が無い・書けない・(2) の run が合格を返さない・焼いた folio2 の字（include_str! で binary に入れた 5 本）が §1 (c) の 3 の式で読めない（型付きで読めない・行 R-16 がちょうど 1 つでない・写す節が無い）。焼いた字が読めないときは (1) の前に止めるので 1 byte も書かない。途中で書けなくなったら、そこで止めて 2 で終わり、書いた file と書かなかった file の名を全部出す（消さない・N-1.1・黙って半分を合格にしない・P-4.1）。run が返した理由の字も標準エラーに出す。
- **書いた（0）。** 1 file 1 行で「folio init: 書いた <path>（<byte> byte）」（byte は (2) の run が生成区間を埋めた後の file の大きさ）と、最後に次の手の 1 行を出す: 版管理に add して commit してから `folio check --dir <置き場>` を撃つこと（commit の前に撃つと「commit が 1 つも無い」の「まだ分からない」が 1 件足される＝検証役の実測）と、凍結の基準は後で凍結すること（始まりの凍結の命令の旗の名は便 ② が決めるので出さない・§1 (i) の 5）。
- 注入（folio inject・CLAUDE.md）には触れない。骨格は注入の対象にしない（ADR-16 決定 (3)・M3 の外）。置き場の外には 1 byte も書かない。

**案内の 1 行（FR3・AC27）。** `crates/folio/src/hello.rs` の未整備の 1 行を「folio: この project には設計文書（design-intent）がまだ無い。folio init --dir <置き場> で最初の文書一式（骨格）を書く（止めるには .folio-quiet を置く）」に替え、file の頭の注の相談窓口の命令を名指す字も同じ向きに直す。字 folio intake を名指さない。判定の順（止める設定 → 整備済み → 出した印 → 未整備）と、整備済みの 1 行と、印の置き場は 1 字も変えない。

**置き場（module）と層。** 新しい file `crates/folio/src/` の下の init.rs（層 3 導出する）に置く。持つもの = 書く file の閉じた一覧（上の 11 本の順）・断りの名の閉じた一覧・雛形の定数（§1 (c) の 2 の人が書く部分）・組み立て時に folio2 の正本から読む字（§1 (c) の 3）・雛形の組み立て・日付の字（UTC の年-月-日）・書く口と結果の型（3 値と標準出力と標準エラー）。名指すのは `crates/folio/src/schema.rs`（層 3・run と印の字）と `crates/folio/src/yaml.rs`（層 1・型付きの読み手と書き手）で、層 4 以上を名指さない。`crates/folio/src/main.rs` に命令 Init（旗 --dir だけ）と振り分けの枝と mod の 1 行を足す（命令 serve の前に置き、`crates/folio/tests/check.rs` の閉じた一覧も同じ順にする）。**schema.rs は write-set に入れない**（席の裁定 2026-09-24・既に在る口を呼ぶ）。

### (c) 骨格の中身（file ごとの出所）

**1. 生成区間（9 file）。** init は空の区間（BEGIN の行と END の行だけ）で書き、同じ呼び出しの中で `crates/folio/src/schema.rs` の run（Write）が埋める（§1 (b) の 書く手順）。区間の位置は folio2 の正本と同じ（規則の表は先頭の注釈の次・判断の記録と設計ノートの欄の決まりは meta の次・ほかは末尾）。表と導出の振り分けは schema.rs が持つので init.rs に写さない（P-6.3）。**便 ② の後の形。** 便 ②（行 `dt`・契約 §5）は `crates/folio/src/floor.rs` に名つきの導出の関数 derive_for（床の木と置き場の憲法の名を受け、名が無しなら今の導出と同じ字）を、`crates/folio/src/adr.rs` に置き場の constitution.yaml の meta.id を読む関数 place_name を足し、schema.rs の run は判断の記録の欄の決まり（adr/schema.yaml）だけを place_name で読んだ名で derive_for に渡す。init は (1) で骨格の憲法（meta.id は floor-declaration）を先に書いてから run を呼ぶので、run が同じ名を読み、`folio schema --write` と同じ写し（表に行の無い名なので列の根の表は空の表の 1 行）になる。init は derive_for も place_name も直接には呼ばない。書いた直後の `folio schema --dir <置き場> --check` が 0 になることを歯 1 が見る。

**2. 人が書く部分の雛形（init.rs の定数・利用者の本来の中身は利用者が書き換える）。** どの file も folio2 の口座名・席とモデルの名・folio2 の持ち主の逐語・folio2 の判断の記録と便と要件の id を持たない。日付の欄は §1 (c) の 4。

| file | 人が書く部分（要旨・値は字の値） |
| --- | --- |
| constitution.yaml | 頭の注釈 1 行（床の受付の宣言・folio init の雛形・名と本文は利用者が決める）。schema の節（§1 (c) の 3 の ア）。meta = id は floor-declaration（利用者が名を決めて書き換える）・version v1.0・status draft・binding false・baseline と author は 未記入・generated は日付・owner は 持ち主・counts は always 1 / ask-first 0 / never 0・approval は who と date と ruling と verbatim が 未記入 で surface が 規則の表の行 R-8 の対話面。north_star の 3 欄は 未記入。precedence = text（条どうしの順位は、利用者の本来の憲法の順位に従う。）・plain・binds both・mechanism（kind none・live now）・rationale 1 行（kind scribe2-article・本来の憲法の順位の節〔番号は利用者が書く〕）。articles は **条 P-1 の 1 本**（題 承認の受け方と散文の門の値・tier always・binds both・relations の rules は R-8 と R-16・規範文 P-1.1 は pattern ubiquitous・strength must・1 文で、本来の憲法の条〔番号は利用者が書く〕に従い、承認は規則の表の行 R-8 の対話面を通ったものだけを受け取り、散文の門は行 R-16 の値で数える、という字・rationale 1 行は kind scribe2-article で本来の憲法の条を名指す・mechanism は build-check・live now・stage post・polarity fail-closed）。rules_pointer・amendment（未記入）・glossary_pointer（行 R-9 を名指さない字）・sources は空の一覧 |
| rules.yaml | 頭の注釈 1 行と生成区間。その後に thresholds と discipline の 2 節（§1 (c) の 3 の イ の書き方）。thresholds は行 **R-8**（article P-1・what は 承認の対話面〔どの対話面を通った承認を受け取るか〕・value は 持ち主と作業する席の対話面〔記帳先 = 台帳の notes + 文書の承認欄〕・kind build-check・status 仮・ruling と ruled_at は 未記入・stage post）と行 **R-16**（article P-1・what と value は folio2 の行 R-16 の値・kind build-check・status 仮・ruling と ruled_at は 未記入・stage post・note と refs と population は写さない）の 2 行。discipline は空の一覧。行 R-17（散文の言及の歯）は入れない（ADR-16 決定 (2)(オ)） |
| vocabulary.yaml | terms と field_terms は空の一覧。identifiers は 1 群（骨格の雛形の語 = folio・ai・intake・rules・file・id・schema・repo・理由の字 1 つ）。雛形の文に出る英字の語が語彙に無い違反を 0 にする最小の群（試作の実測で 0） |
| srs.yaml | meta（id srs・title 要件書・version v0.1・status draft・generated は日付・approval は空の一覧）と生成区間だけ |
| index.yaml | meta（id index・title 設計文書の入口・同じ形）+ folio2 の入口の audience・shelf・lanes・intake の 4 節（§1 (c) の 3 の ウ）+ 生成区間 |
| ceiling.yaml | meta（id ceiling・title 天井の正本・同じ形）+ folio2 の天井の正本の weights・documents・viewpoints の 3 節の最小（§1 (c) の 3 の ウ）+ 生成区間 |
| intake.yaml | meta（id intake・title 相談窓口・同じ形・approval は空の一覧）+ folio2 の相談窓口の answers・targets・questions・sheet の 4 節の最小（§1 (c) の 3 の ウ）+ 生成区間 |
| graph.yaml | meta（id graph・title 索引の欄の決まり・同じ形）と生成区間だけ |
| adr/schema.yaml | meta（id adr-schema・version 1・date は日付・decided_by は ADR-1 の 1 つ・owner 持ち主・author 未記入）と生成区間 |
| design-note/schema.yaml | meta（id design-note-schema・version 1・date は日付・decided_by は ADR-1 の 1 つ・owner 持ち主・author 未記入）と生成区間 |
| adr/ADR-1.yaml | 最初の判断の記録（ADR-16 決定 (2)(カ)）: 題 判断の記録と設計ノートの欄の決まりに folio の床の定数を採る・status proposed・date は日付・plain と context と decision（欄の決まりは folio の床の定数の写し〔各置き場の schema.yaml の生成区間〕を採る）・options 2 つ（id a 採る adopted・id b 採らない rejected・理由は床が写しとの 1 字の違いを落とすため）・basis は P-1・retreat は kind ruling と condition 未記入。approval の欄は持たない（発効は利用者の持ち主の承認・P-1.2） |

**3. 組み立て時に folio2 の正本から読む字（build の時に binary へ焼く・実行時に folio2 の repo を読まない）。** 読む file は folio2 の `design-intent/` の下の constitution.yaml・rules.yaml・index.yaml・ceiling.yaml・intake.yaml の 5 本（組み立て時に字として焼く・crate の外の file を焼くのは憲法の値域を焼く `crates/folio/build.rs` と同じ向き）。meta（承認欄を含む）と schema の生成区間と注釈は、どの file からも写さない。支度表（intake-sheet.yaml）は読まない。

   - **ア 憲法の schema の節。** folio2 の憲法を型付きの読み手 `crates/folio/src/yaml.rs` の parse_typed で読み、最上位の schema の値だけを 1 つの表（鍵 schema）に入れて、同じ file の書き手 write で書く（字は全部二重引用符・入れ子は字下げの block）。注釈は読み手が落とすので写らない（folio2 の条 id を名指す注釈〔例 未知の節は検査で落とす（N-3）〕も落ちる）。値は folio2 の憲法の schema の値と等しい＝値域（enums）は組み立てた値域そのもので、便 ③ の部分集合の床は必ず通る。値の説明の字（live の値域の delivery-0 の説明の 便 0・mechanism_live_rule の P-4.2）は値として残る（歯 7 の数える範囲の外・§1 (i) の 1）。
   - **イ 規則の表の行 R-16 の what と value（席の裁定 2 の式）。** folio2 の規則の表を parse_typed で読み、最上位の thresholds の一覧から欄 id の値が字 R-16 の行を 1 つ取り、その行の欄 what と value の 2 つの値を取り出す（value は入れ子の表で、字の引用符の中の 。 ） ・ 、 と null を持つが、型付きで読むので字の走査はしない）。骨格の規則の表の thresholds と discipline の 2 節は、行 R-8（雛形の定数）と行 R-16（id・article・what・value・kind・status・ruling・ruled_at・stage の順で、what と value だけが取り出した値）の 2 行の一覧と空の一覧を持つ 1 つの表を write で書いた字である。R-16 の行がちょうど 1 つでない・what か value が無いとき（焼いた字は include_str! で binary に入るので、組み立ての後に実行時に分かる）は、1 byte も書かずに 2 で終わる（§1 (b) の 書けない（2）の列挙の 1 つ・黙って空の行を書かない・P-4.1・build.rs で組み立て時に落とす形は §1 (d) の 7 で採らない）。
   - **ウ 入口・天井・相談窓口の節（席の裁定 1）。** この 3 本は字のまま行で写す（相談窓口の鍵 yes と no が型付きでは往復できない・(a) の 8）。式は次の順で、決定的に当てる。
     1. 行の走査で最上位の節を切る（行頭が空白でも # でもない行の鍵が節の始まり・次の節の始まりの手前まで）。行頭の空白の後が # の行（注釈の行）は落とす（この 3 本の写す節には行の途中の注釈が無い・試作の実測）。写す節は、入口 = audience・shelf・lanes・intake、天井 = weights・documents・viewpoints、相談窓口 = answers・targets・questions・sheet。
     2. **最小。** 天井の documents の行は欄 id と file だけを残し（床が要る欄・注の欄 note は写さない）、相談窓口の targets の行は欄 id と type と with だけを残し、相談窓口の sheet の節は欄 file と title と explain だけを残す（sections の一覧は写さない＝支度表の欄の決まりは支度表の便の持ち物）。観点 4 つ・文書 10 行・質問 5 つ・行き先 5 行は床が閉じた一覧で数えるので全部写す。1 行の表の行（行頭の - の後が波括弧）の欄は、括弧の深さが 0 の読点（,）で区切って欄の名で選ぶ。
     3. **folio2 固有の id を持つ括弧を落とす。** 写した字のうち、全角の括弧（ と ）で囲まれた部分（入れ子は数える）の中に、歯 7 の id の形（§1 (e) の 7）で骨格自身の番号（ADR-1・P-1・P-1.1・R-8・R-16）でないものが 1 つでも在れば、その括弧を中身ごと落とす（例 天井の忠実さの観点の問いの末尾の括弧・相談窓口の sheet の括弧）。
     4. **入口の lanes の行き先。** lanes の各行の stops のうち doc が adr の行は、その行の最初の位置に 1 行 `{doc: adr, at: ADR-1, label: なぜそう決めたか（最初の判断の記録）}` を置き、残りの doc が adr の行は落とす（骨格自身の最初の判断の記録・提案中を指す）。at が図の id（字 fig- で始まる）の行は、骨格の要件書に図が無いので落とす。
   - 骨格の本文（入口・天井・相談窓口の人が書く欄）に folio2 の判断の記録・便・要件と受入基準・条と規則の行の id を写さない（席の裁定 1・ADR-16 決定 (2)(オ)）。数えは歯 7。
   - 組み立てた後に folio2 の正本が変われば次の組み立てで骨格も変わる（1 つの組み立てが 1 つの骨格を持つ・ADR-16 決定 (2)）。

**4. 日付。** 日付の欄（憲法の meta の generated・srs と index と intake と ceiling と graph の meta の generated・判断の記録と設計ノートの欄の決まりの meta の date・最初の判断の記録の date）は、撃った時点の UTC の年-月-日（外部 crate を使わず std の時計から数える）。これが骨格の中で撃つたびに変わる唯一の字で、ほかは組み立てが同じなら byte 一致する（歯 4）。承認欄と裁定の欄（ruling・ruled_at・承認の date）は 未記入 のまま（承認は利用者の持ち主が行う）。

**5. 条の形（ADR-16 決定 (2)(オ)・(3)・帰結）。** 条は P-1 の 1 本（行 R-8 と R-16 を縛る）。例外の口を足さないことを定める条（folio2 の条 N-3 に当たる）は書かない（置くかどうかは骨格の承認の裁定・利用者の持ち主・§1 (i) の 3）。置くと裁定されたら、利用者が凍結の前に条 P-2 として足す（骨格の命令は変えない）。条の番号は利用者の番号空間の 1 から振り、folio2 の番号を予約しない。根拠の種別は値域の既存の値 scribe2-article で本来の条を名指す（ADR-16 決定 (3) の字のまま・ほかの利用者にも同じ値を書く）。

### (d) 採らなかった形

1. **graph.yaml を書かない（要件 FR22 の規範文の 7 file + 2 置き場だけ）。** `folio schema --check` と schema.rs の run は 9 file が全部無いと合格を返さず、骨格の生成区間を埋める口も確かめる口も止まる。要件 FR22 の注（第 1.34 版・改訂 g）が「生成区間の導出は索引の欄の決まりの file graph.yaml も書く＝骨格は正本 7 file と 2 つの置き場に加えて graph.yaml を持つ」と書いた（規範文の列挙には無く、注で持つ）。採らない。
2. **生成区間を init.rs から床の導出の関数で直に書く（改訂 a の形）。** schema.rs の 9 file の表を crate の中で見えるようにし、判断の記録の欄の決まりだけ便 ② の名つきの導出を呼ぶ振り分けを init.rs にも持つことになる（同じ振り分けを 2 か所に持つ・便 ② と schema.rs で重なる）。既に在る口 run を呼べば 1 か所で済む（席の裁定 3・検証役の別案）。採らない。
3. **入口・天井・相談窓口の節を字のまま全部写す（改訂 a の形）。** folio2 の判断の記録を型付きで名指す欄（lanes の ADR-8・ADR-11）と、便・要件の id を持つ注と括弧が利用者の file に載る（(a) の 6）。採らない（席の裁定 1）。
4. **入口・天井・相談窓口の雛形を init.rs に全部手書きで持つ。** folio2 の正本と同じ字を 2 か所に持つ（P-6.3）。要件 FR22 の注は相談窓口の質問を組み立て時に folio2 の相談窓口の正本から導出すると書く。採らない。
5. **行 R-16 を行の字の走査（括弧の深さと引用符を数える）で切り出す。** 1 行の flow の表の中の入れ子で、引用符の中に括弧と読点が在る。型付きの読み手と書き手が既に在り、往復で値が一致する（(a) の 8）。採らない（席の裁定 2）。
6. **入口・天井・相談窓口も型付きで読み書きする。** 相談窓口の鍵 yes と no が真偽の鍵になり、書き手が断る（(a) の 8）。採らない。
7. **組み立て時の導出を build.rs に置く。** 読むのは字の切り出しと型付きの読みだけで、型を作らない（憲法の値域を型に焼く `crates/folio/build.rs` とは違う）。build.rs を変えると全部の組み立ての入力が増える。採らない。
8. **--dir に既定を置く（design-intent）。** 利用者の置き場の名は利用者が選ぶ（ADR-16 決定 (3)）。採らない。
9. **在る file を飛ばして無い file だけを書く。** 半分の骨格ができ、どれが雛形でどれが利用者の file か分からなくなる。1 本でも在れば断る（ADR-16 決定 (3)）。採らない。
10. **日付を旗で渡す・日付を書かない。** 最初の判断の記録の date は年-月-日でないと床が落とす（(a) の 5）。旗は要件 FR22 の口に無い。撃った日の UTC を書く。採らない。
11. **支度表（intake-sheet.yaml）の雛形も書く。** 支度表は回答と承認の記録で、雛形として写さない（ADR-16 決定 (3)）。採らない。
12. **骨格の憲法に folio2 の前文の字をそのまま写す。** 前文は順位の規範で、指す先の無い規範を骨格が書くことになる（ADR-16 帰結）。本来の憲法の順位に従う 1 文にする。採らない。

### (e) 歯（新しく 7 本・置き場は新しい file `crates/folio/tests/` の下の init.rs と、直す既存の歯の file）

関数名は f125_ で始める（base で `git grep -n 'fn f125_'` は 0 件）。**凍結 anchor（P-10.1）** は 2 つで、どちらも生成器から独立した手書き: (1) 新しい file `tests/fixtures/schema/` の下の init-forbidden.txt（AC26 の禁止字の閉じた一覧・1 行 1 字列・下の 28 行・凍結の時点の全数）、(2) 歯の中に字で持つ書く file 11 本の閉じた一覧。

init-forbidden.txt の 28 行（起草役の数え・(a) の 7・この順で書き、各行の前後に空白を足さない）:

```
Aですすめてよい
aで
fable 5.1
orchestrator 席（AI・fable 5.1）
orchestrator席に直してよい
planner 席（AI・fable 5.1）
scribe2がある程度形になったので開発をどんどん進めていきたい。いずれも承認する
いずれも推奨で進めて
すべて承認する
すべて推奨で進めて
それらの制限はすべて取っ払うので今のscribe2を使ってfolio2の実装を進めろ。
どちらも承認する
どちらも推奨で
両方推奨で承認する
全部承認する
全部推奨で良い
各ページの語彙の項目だがconstitutionとSRSでスタイルが異なる。これに関してはconstitution側に統一したほうが良いのではと思う
問いは全部推奨で
問い１−４は推奨で進めて
図はこれでよい。
承認する。
承認するから進めて
持ち主（shuu5）
敵対レビュー（ad-hoc WF wf_5e659fc9-0ed・7 観点・反証 24 + 追加反証 34）
確認した　大体これで良い
裁定はすべて承認する
裁定は両方承認する
１．推奨で
```

**歯の土台。** 一時 dir の根で git の init をし、置き場を根の下の design-intent/folio2 に取る（利用者の形・版管理の外の「まだ分からない」を混ぜない＝要件 FR22 の注）。印の置き場（hello の --state）も一時 dir の下に取る。落ちても消す。環境変数 GIT_* は継承しない（`crates/folio/tests/check.rs` の写しと同じ形）。YAML を読む歯は、src の読み手 `crates/folio/src/yaml.rs` を歯の file に取り込んで読む（`crates/folio/tests/floor_cases.rs` と同じ形）。

1. **空の置き場に書くと骨格が書かれ、床は凍結の基準の不在だけを「まだ分からない」にする（AC19 の前半）。** init → 0・書かれた file の集合が 11 本の一覧とちょうど一致・根の下に置き場の外の file が 1 本も無い（根の .git の下は数えない・CLAUDE.md も無い）・標準出力の最後の行が字 commit と字 folio check を持つ・git の add と commit の後の `folio check --dir <置き場>` が 2 で、違反 0・まだ分からないの行がちょうど 2 本で、それぞれ字「凍結 anchor が 0 本」と「anchors/ids-*.yaml）が無い」を持つ・`folio schema --dir <置き場> --check` が 0・`folio intake --dir <置き場> --print` が 0 で q1〜q5 を出す・`folio hello --dir <置き場> --state <一時>` が整備済みの 1 行（字 節点・辺）。**base では命令が無く落ちる＝RED。**
2. **正本が 1 本でも在れば何も書かずに断る（AC19 の後半）。** 断りの名 11 個（正本 7・graph.yaml・adr/・design-note/・anchors/）について 1 つずつ、それだけを置いた置き場（file は 1 字の中身・dir は空・加えて constitution.yaml を壊れた symlink にした形を 1 つ）で init → 1・標準エラーにその名・置き場の全 file の名と中身の要約値（歯の側で sha256 を数える）が撃つ前と一致。置き場にほかの file（README）だけが在る形は 0 で、README は byte 不変。置き場の親 dir が無い → 2 で何も作らない。**base では落ちる＝RED。**
3. **禁止字が数える範囲に 1 回も出ない（AC26）。** 凍結の一覧を読み、自己検査を 2 つ見る: (i) 各行が、folio2 の repo の design-intent の yaml（preview の下を除く）の、鍵 who か verbatim を持つ行（鍵は引用符付きも数える）に 1 回以上在る。(ii) 各行が init の雛形の定数 `crates/folio/src/init.rs` の字に 1 回も無い（床の定数の全体や src の全体には求めない＝`crates/folio/src/stamp.rs` のモデルの名は天井の印の書き手の字で雛形ではない・席の裁定 4）。そのうえで、骨格の 11 本の数える範囲（生成区間の印の間と、intake.yaml の answers・targets・questions・sheet の節を除いた字）に一覧のどの行も 0 回。あわせて folio2 自身の design-intent の同じ数える範囲に当てると 1 回以上出る（数えが効くことの確かめ）。**base では命令が無く落ちる＝RED。**
4. **撃つたびに違うのは日付だけ。** 2 つの置き場に撃ち、11 本の名と中身が、日付の字（撃った日の UTC の年-月-日と前後 1 日のどれか）を固定の字に置き換えた後で byte 一致する。日付の字が在る欄は §1 (c) の 4 の欄だけ。**base では落ちる＝RED。**
5. **雛形の出所が folio2 の正本と揃う。** 骨格の憲法の schema の値が folio2 の憲法の schema の値と等しく、骨格の憲法の schema の節の字に # が 0 個（注釈の落とし損ねを拾う）。骨格の規則の表の行 R-16 の what と value が folio2 の行 R-16 と値で等しい。骨格の相談窓口の answers と questions の値が folio2 と等しく、targets の各行の id・type・with が folio2 と等しく、sheet が file・title・explain だけで folio2 と等しい。骨格の天井の documents の各行の id と file の組が folio2 と等しく、viewpoints の各行の id・name・reader・reads が folio2 と等しい。meta を持つ 5 file（srs・index・intake・ceiling・graph）の meta の approval が空の一覧。骨格の憲法の条は 1 本で id が P-1、その relations の rules が R-8 と R-16。adr/schema.yaml と design-note/schema.yaml の meta の decided_by が ADR-1 だけで、adr/ADR-1.yaml の status が proposed。**base では落ちる＝RED。**
6. **案内の 1 行（AC27）。** 版管理の中の憲法の正本が無い置き場で `folio hello --dir <置き場> --state <一時>` → 1 行が字 folio init を持ち、字 folio intake を持たない。同じ置き場に init を撃った後は、1 行が整備済みの形（字 節点・辺・folio graph --digest）で、未整備の 1 行ではない。**base では 1 行が folio intake を名指して落ちる＝RED。**
7. **folio2 固有の id を写さない（席の裁定 1）。** 骨格の 11 本の id の数える範囲（生成区間の印の間と、憲法の schema の節〔組み立てた値域の写し・(c) の 3 の ア〕を除いた字）で、id の形に当たる字（ADR- と数・便 と数〔間の空白は任意〕・FR と NFR と AC と CON のどれかに続く数・P- と N- と A- のどれかに数〔. と数が続いてもよい〕・R- と D- のどれかに数・どれも前の字が英字でないもの＝字 SHA-256 の A-256 などは数えない）のうち、骨格自身の番号（ADR-1・P-1・P-1.1・R-8・R-16）でないものが 0 個。とくに字 ADR- と数は ADR-1 だけで、入口の lanes の stops のうち doc が adr の行の at は ADR-1 だけ。あわせて folio2 自身の design-intent の同じ数える範囲に当てると 1 個以上（数えが効くことの確かめ）。**base では命令が無く落ちる＝RED。**

歯の効き（**見込み・測っていない**・実装の後に実装役か検証役が変異で測る）:

| 当てる形 | 落ちるはずの歯 |
| --- | --- |
| 在る file を飛ばして残りを書く | 2 |
| 断りを 0 で返す・1 本書いてから断る | 2 |
| 空の区間で書いた後に run を呼ばない | 1（schema --check が 1） |
| graph.yaml を書かない | 1（run と schema --check が 2） |
| 相談窓口の承認欄を写す | 3・5 |
| 憲法の schema の節を字で写して注釈を落とさない | 5（字 # が在る）・7（注釈の N-3 などが数える範囲に出る） |
| 入口の lanes の ADR-8・ADR-11 の行をそのまま写す | 7 |
| 天井の documents の注か忠実さの観点の問いの括弧を落とさない | 5（documents の欄）・7（便・判断の記録の id） |
| 行 R-16 の value の入れ子を切り損ねる | 1（床か schema の違反）・5 |
| 日付を書かない・固定の日付を書く | 1（ADR-1.date の違反）・4 |
| 案内の 1 行に folio intake を残す | 6 |

### (f) 既存の歯のうち落ちるもの・直し方

| 歯（file・名） | 落ちる理由 | 直し方 |
| --- | --- | --- |
| `crates/folio/tests/check.rs` の p1_commands_closed_list | 命令の一覧が 14 本になる | 閉じた一覧に init を足す（13 → 14・置く位置は main.rs の命令の順と揃える）。採否を決める口の禁止の語の検査は変えない |
| `crates/folio/tests/modules.rs` の p106_layers_cover_every_module | 新しい file が層の表に無い | 表 LAYERS に init を層 3 で 1 行（init は層 3 の schema と層 1 の yaml を名指し、層 4 以上を名指さない） |
| `crates/folio/tests/hello.rs` の hello_line_names_intake_and_the_quiet_file | 1 行が folio intake を名指さない | 名を hello_line_names_init_and_the_quiet_file に改め、字 folio init と .folio-quiet を持ち folio intake を持たないことを見る形に。頭の注に便 125 の 1 行 |

- 凍結 anchor は 1 本も動かない（design-intent と tests/fixtures の既存の file を書き換えない）。
- 本便の後の木で workspace の nextest は全部緑（**受付の時点の main の本数 + 本便の新しい歯 7**・起草役の参考値は base 2d90e8d で 796、便 123 の後で 803、便 ② ③ と便 123 の起草の patch を全部当てた木で 816〔検証役の再現〕）。

### (g) 門

設計文書の正本を書き換えない便なので、門は 0（通す）で、起草役の実測も 0（§0 の 門）。受付の時点の main で write-set 8 本を `folio ceiling --gate --write-set` に渡して撃ち直す。

### (h) 大きさ・verify と done の対応

1. **write-set の印。** 新しい file 3 本に `+`（`crates/folio/src/` の下の init.rs・`crates/folio/tests/` の下の init.rs・`tests/fixtures/schema/` の下の init-forbidden.txt）。書き換える 5 本は印なし（main.rs・hello.rs・歯の file の check.rs と modules.rs と hello.rs）。消す印と縮める印は当たらない。新しい dir は作らない。
2. **言えないこと。** 骨格の中身の字面の全部を固定する凍結 anchor は置かない（最小の手書きの fixture の作法・11 本の全字は数百行で、手書きの写しは生成物の写しになる）。字面は歯 1（床と schema --check が通る）・歯 3（禁止字）・歯 5（出所が揃う）・歯 4（決定的）・歯 7（folio2 固有の id）で囲む。雛形の平易な字の良し悪しは床と歯では言えない（天井か持ち主の読み）。
3. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の印なしの .rs と新しい .rs（幅 120 の正規化・§1 (j)）。size M の見積は 1 file あたり 300。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 |
| --- | --- | --- |
| `crates/folio/src/main.rs` | 632 | 868 |
| `crates/folio/src/hello.rs` | 128 | 1,372 |
| 新しい init.rs | 0 | 1,500 |

   新しい init.rs の見積は 350〜450 行（雛形の定数 約 120・節の切り出しと 4 つの式 約 120・型付きの 2 つ 約 40・書く口と断りと run の呼び出し 約 100・単体の歯なし）。

4. **size は M。** 変える src は 3 本（うち新しい 1 本が大半）。
5. **verify は 5 行**で、done の 5 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test init f125_` = (e) の新しい歯 7 本。
   2. `cargo nextest run -p folio --test hello` = 案内の 1 行の歯（直した 1 本を含む全部）。
   3. `cargo nextest run -p folio --test check p1_commands_closed_list` = 命令の閉じた一覧の歯。
   4. `cargo nextest run -p folio --test modules` = 層の割り当てと辺の歯。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
6. **verify の歯の file と write-set。** verify が `--test` で名指す歯の file 4 本（init・hello・check・modules）はすべて write-set に在る。`--bin folio` は使わない。

### (i) 本便が運ばないもの・言えないこと・まだ分からない点・撤退条件

1. **運ばないもの。** 始まりの凍結の命令と列の根の表（便 ②）・値域の部分集合の床（便 ③）・器の導出 file の解決先と生成区間の注の N-3（便 123）。生成区間のほかの注に残る folio2 の id（便 123 の §1 (c) の 2・床の定数の字で、骨格は run の導出のまま書く）と、憲法の schema の値の説明の字に在る folio2 の字（便 0・P-4.2・値域の写しで歯 5 が値の一致を見る・直すなら folio2 の憲法の schema の改訂）＝どちらも歯 7 の数える範囲の外。利用者の置き場への実の撃ち（利用者の側の便・判定点 ①）。注入。面・配信・天井。要件書・判断の記録・憲法・規則の表・語彙の字（FR22 の注の「口は未実装である」の字の直しは着地の後の要件書の版）。外部 crate。台帳への記帳。
2. **folio2 自身は変わらない。** 本便は folio2 の design-intent を 1 byte も書かない。folio2 の置き場で init を撃つと、正本が在るので断る（歯 2 の形）。
3. **まだ分からない — 利用者の持ち主の裁定 2 つ。** (1) 例外の口を足さない条（folio2 の条 N-3 に当たる）を骨格に置くか（ADR-16 帰結・scribe2 は置かない〔条 1 本〕を推奨と表明済み・裁定は scribe2 の持ち主が便 ④ の前に行う）。本便の骨格は条 1 本で、置くと裁定されても骨格の命令は変えずに利用者が足す（§1 (c) の 5）。(2) 骨格が縛る範囲を folio の置き場の運用に限ることを受けるか（ADR-16 帰結・今の縛る相手の値域では型付きの値にできない）。どちらも骨格の第 1.0 版が列の根の表に載ると変えられない（ADR-16 帰結）ので、利用者は始まりの凍結の前に決める。
4. **言えないこと — folio2 の正本との結びつき。** init は folio2 の 5 本を組み立て時に読み、歯 5 と歯 7 は folio2 の今の正本と突き合わせる。後の便で folio2 の入口・天井・相談窓口の写す節に、全角の括弧の外で folio2 の id を書くと、その便で歯 7 が落ちる（写しの漏れを拾う意図どおり・その便が式か字を直す）。全角の括弧の中に書くと括弧ごと落ちるので歯 7 は通るが、天井の観点の reads などの値が変わる場所なら歯 5 の値の一致が落ちる（落ちた便が字の置き場を直す）。
5. **言えないこと — 禁止字の一覧の古さ。** 凍結の一覧は 2026-09-24 の全数（28 行）で、後に folio2 の承認欄に足される逐語は一覧に入らない（歯は一覧の古さを拾わない・自己検査 (i) は一覧 ⊆ 承認欄の字だけを見る）。足すのは受入基準 AC26 の固定の材料を直す要件書の版か後の便。
6. **まだ分からない — 便 ② ③ の着地版の上での骨格の床。** 起草中の便 ② ③ の patch を当てた binary では違反 0・まだ分からない 2 のままだった（(a) の 4）。着地版の上で撃ち直す。
7. **撤退条件。** (1) 受付の時点の main で、骨格の試作の床が違反 0・まだ分からない 2（凍結の基準の不在だけ）に届かなければ、足りない字を契約の改訂で足してから運ぶ（雛形の字を実装役が考えない）。(2) 受付の時点の main で、schema.rs の run の形（pub・旗 Write・9 file の表・判断の記録の欄の決まりだけ置き場の名で導く振り分け）か印の字が §1 (b)(c) の 1 と違えば、書く手順の名指しを契約の改訂で直してから運ぶ。(3) 本便の後に (f) の 3 本のほかに既存の歯が 1 本でも落ちたら、その歯の本文を直さずに止めて席へ返す。(4) 利用者の持ち主の裁定（(i) の 3）が「骨格の命令が条を 2 本書く」に決まったら、雛形の定数と歯 5 の期待を直す改訂を先に起こす。(5) 型付きの読み手と書き手で憲法の schema か行 R-16 が往復で値一致しなければ（(a) の 8 の前提が崩れたら）、止めて席へ返す（書き方を実装役が考えない）。

### (j) 数え直す手順（規則の表の行 D-13）

起草の記録は持ち主の home の `.local/share/folio2/handoff-2026-09-24/` の d123-draft.md（本便の分を含む）。骨格の試作の script は d125-draft-proto.py（引数 = folio2 の design-intent・書く置き場・改訂 b）、禁止字と id の数えは d125-draft-forbidden.py（引数 = folio2 の design-intent・骨格の置き場・書き出す一覧の path〔任意〕）。改訂 a の script は同じ名の末尾に -a を付けて残した。

1. 試作: 空の一時 dir の下の design-intent/folio2 に `python3 d125-draft-proto.py <folio2>/design-intent <置き場>` → `folio schema --dir <置き場> --write`（init の run の呼び出しと同じ）→ 一時 dir の根で git の init と add と commit → `folio check --dir <置き場>`（便 123 の後の binary で 違反 0・まだ分からない 2）。
2. 禁止字と id: `python3 d125-draft-forbidden.py <folio2>/design-intent <置き場>`（禁止字 28・数える範囲に出た数 0・folio2 自身 347・folio2 固有の id 0・folio2 自身 3,320）。
3. 余地: `python3 d119-draft-lines.py <file…>`。
4. 門: `folio ceiling --gate --write-set <write-set の 8 本（接頭辞を剥がす）>`。

## 2. 範囲

- 入れる: 新しい file `crates/folio/src/` の下の init.rs（命令の本体・雛形の定数・組み立て時に読む folio2 の正本の字と 4 つの式・run の呼び出し）。`crates/folio/src/main.rs` の命令 Init と振り分けの枝と mod の 1 行（命令の一覧に init を足す行だけ）。`crates/folio/src/hello.rs` の未整備の 1 行の字と頭の注。新しい歯の file（f125_ の 7 本）と凍結の一覧 1 本。`crates/folio/tests/check.rs` の閉じた一覧 1 語・`crates/folio/tests/modules.rs` の表 1 行・`crates/folio/tests/hello.rs` の歯 1 本の名と本文と頭の注。
- 入れない: 床の判定の式と断りの字。`crates/folio/src/schema.rs`（呼ぶだけで 1 字も変えない）。folio2 の design-intent の全 file。始まりの凍結と列の根の表（便 ②）・値域の床（便 ③）・解決先の式（便 123）。注入・面・配信・天井。新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| cmd | 骨格の命令 | 新しい file `crates/folio/src/` の下の init.rs と `crates/folio/src/main.rs` の命令 Init（--dir だけ） |
| tpl | 雛形 | init.rs の人が書く部分の定数と、組み立て時に folio2 の正本（憲法の schema の節・規則の表の行 R-16・入口・天井・相談窓口）から読む字と 4 つの式 |
| region | 生成区間 | 空の区間で書いてから `crates/folio/src/schema.rs` の run（Write）が埋める（folio schema --write と同じ） |
| hello | 案内の 1 行 | `crates/folio/src/hello.rs` の未整備の 1 行（folio init を名指す） |
| teeth | 歯 | 新しい歯の file の f125_ の 7 本と凍結の一覧 `tests/fixtures/schema/` の下の init-forbidden.txt と、直す既存の歯 3 本 |

## 4. 検査（歯）

§1 (e)(f) と (h) の 5 のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない（日付は std の時計から数える・YAML の読み書きは crate の中の読み手と書き手）。新しい dir は無い。
- 前提の着地: 版 B（要件書 第 1.34 版・main 2d90e8d に在る）・便 ②・便 ③・便 123・一括 16（要件書 第 1.35 版・FR22 の規範文に graph.yaml・§0 の 前の便）。
- 並行の便: 便 ②（行 `dt`）は `crates/folio/src/main.rs` の命令 check に旗 1 つと振り分けの 1 枝を足し、`crates/folio/src/schema.rs` の命令の本体を名を渡す形に変える。本便は schema.rs を変えず run を呼ぶだけなので、便 ② と file が重なるのは main.rs だけ（本便の main.rs の変更は命令 Init の宣言と振り分けの枝と mod の 1 行で hunk は重ならない）。便 ③（行 `du`）とは file が重ならない。便 123 は main.rs の命令 derive の説明の 1 行を変える。
- 本便の着地の後に席が見ること: 要件 FR22 の注の「口は未実装である」の字と、受入基準 AC19 / AC26 / AC27 の固定の材料の字（置き場は本便で決まった `tests/fixtures/schema/` の下の init-forbidden.txt と歯の file）を次の要件書の版で直す。
- **利用者（scribe2）への連絡の欄**（席が本便の着地の後に送る）: (1) 骨格の命令の着地と、骨格の承認の前に決める 2 つの裁定（§1 (i) の 3）。(2) **案内の 1 行と置き場の名。** 命令 folio hello の --dir の既定は design-intent なので、置き場 design-intent/folio2 に骨格を書いた後も、既定のまま撃つ hello は design-intent に憲法の正本が無いと見て未整備の 1 行を 1 回出す。セッションの始まりに hello を結ぶ仕掛け（hook）は利用者の設定（要件 FR3 の注）なので、利用者の hook が hello に `--dir design-intent/folio2` を渡すこと（folio は既定を変えない・§1 (d) の 8）。(3) 骨格を書いた後は、add と commit をしてから床を撃つこと（§1 (b) の 書いた）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dx"
title = "判断の記録 ADR-16 決定 (3)・(2)(オ)(カ) と (7) の ④ の 2 便目（裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）: 骨格の命令 folio init --dir を足す（新しい file crates/folio/src/init.rs・層 3）。--dir は既定なし。置き場に正本 7 file・graph.yaml・adr/・design-note/・anchors/ のどれか 1 つでも在れば 1 byte も書かずに 1 で断り、無ければ 11 本（正本 7 file と索引の欄の決まり graph.yaml と判断の記録の欄の決まり adr/schema.yaml と最初の判断の記録 adr/ADR-1.yaml〔folio の欄の決まりを採る・提案中〕と設計ノートの欄の決まり design-note/schema.yaml）を新規作成でだけ書き、生成区間は空の印で書いてから既存の crates/folio/src/schema.rs の run（Write）を同じ呼び出しの中で呼んで埋めて 0（schema.rs は変えない・途中で書けなければ止めて 2・消さない）。人が書く部分は init.rs の雛形（条は P-1 の 1 本で行 R-8 と R-16 を縛り、本来の憲法の条を番号で指す字・根拠の種別は scribe2-article・行 R-17 は入れない）と、組み立て時に folio2 の正本から読む字（憲法の schema の値と行 R-16 の what と value は型付きの読み手で読み書き手で書く・入口と天井と相談窓口の節は行で写して最小の欄に絞り、folio2 固有の id を持つ全角の括弧を落とし、入口の lanes の判断の記録の行き先は ADR-1 の 1 行にする）で、folio2 の口座名・席とモデルの名・持ち主の逐語と承認欄と支度表と folio2 の判断の記録や便や要件の id を写さない。日付の欄は撃った日の UTC。注入の対象にしない。案内の 1 行（crates/folio/src/hello.rs）は憲法の正本が無い置き場で folio init を名指し folio intake を名指さない。歯は新しい crates/folio/tests/init.rs に 7 本と手書きの禁止字の一覧 tests/fixtures/schema/init-forbidden.txt（28 行・凍結の時点の全数）で、落ちる既存の歯 3 本（命令の閉じた一覧・層の表・案内の 1 行）を直す。受付は便 ② ③ と便 123 の着地と一括 16（要件書 第 1.35 版）の発効の後"
req = ["FR22", "FR3"]
section = "1"
write-set = ["+crates/folio/src/init.rs", "crates/folio/src/main.rs", "crates/folio/src/hello.rs", "+crates/folio/tests/init.rs", "crates/folio/tests/hello.rs", "crates/folio/tests/check.rs", "crates/folio/tests/modules.rs", "+tests/fixtures/schema/init-forbidden.txt"]
verify = ["cargo nextest run -p folio --test init f125_", "cargo nextest run -p folio --test hello", "cargo nextest run -p folio --test check p1_commands_closed_list", "cargo nextest run -p folio --test modules", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f125_ の歯 7 本（版管理の中の空の置き場に撃つと 11 本がちょうど書かれ置き場の外に何も書かず次の手の行が commit と folio check を名指し床が違反 0 で まだ分からない は凍結 anchor が 0 本と id の一覧の baseline の不在の 2 本だけで schema の check が 0 で intake の print が質問 5 つで hello が整備済みの 1 行／断りの名 11 個のどれか 1 つが在る置き場と壊れた symlink の置き場で 1 を返し置き場の名と中身の要約値が不変でほかの file だけの置き場は 0 で置き場の親が無いと 2／禁止字の一覧 28 行が自己検査〔承認欄の行に在り init.rs に無い〕を通り骨格の数える範囲に 0 回で folio2 自身には 1 回以上／2 回撃った骨格が日付の字を除いて byte 一致／憲法の schema の値と行 R-16 の what と value と相談窓口の answers と questions と targets と sheet と天井の documents と viewpoints が folio2 の正本と揃い憲法の schema の節に # が無く承認欄が空で条は P-1 の 1 本で欄の決まりの decided_by が ADR-1 で ADR-1 が提案中／憲法の正本が無い置き場で hello の 1 行が folio init を名指し folio intake を名指さず init の後は整備済みの 1 行／骨格の id の数える範囲に骨格自身の番号でない id が 0 個で folio2 自身には 1 個以上）が緑、案内の 1 行の歯（hello の全部）が緑、命令の閉じた一覧の歯が 14 本で緑、層の割り当てと辺の歯が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通る"
<!-- contracts:end -->
