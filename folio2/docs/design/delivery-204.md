# 設計: 便 204 — 床が規則の表の行の裁定の時刻（ruled_at）の在ることと形を全行で数え、閾値の行に human-review を置かせない

- 要件: FR26（決定の欄の裁定 id の形を床で数える・同じ歩き手に時刻の数えを足す）・FR19（規則の表の生成区間へ床の定数から導出して書く）。本流の要件書に在る id で、規範文・確かめ方・受入基準は変えない（FR26 の規範文は裁定 id の形だけを名指し、時刻の形を定めない。時刻を行に付ける決まりは憲法の条 P-17.1 が持ち、その形を本便が規則の表の生成区間の型付きの字に置く）。
- 条: P-17.1（rules 表の行の追加・変更に裁定 id と時刻を付ける）・P-17.2（裁定 id の無い変更を検査が落とす）・P-17.3（骨格の印は裁定の前）・P-6.3（時刻の数えは決定の欄と同じ歩き手の出力を読む＝2 つ目の歩き手を持たない・年-月-日の形は判断の記録の日付と同じ関数）・P-5.6（時刻の形の字面を規則の表の生成区間へ写す）・P-10.1（期待の字は歯の側の手書き・凍結 anchor は前の anchor に手で足した写し）。
- 出所: 台帳 f2-648.276.1（asks-draft の変異・handoff-2026-09-28/asks-draft.md）。裁定 = 台帳 f2-648.276.1 notes 2026-09-29 07:16 JST（tsuzuri の答え「要る・t3 の規則の表も数えてほしい」を受けて席が便 204 として起こした・規則の表の行 D-11 の名指し）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gy` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 23 本（src 3・歯の file 2・凍結 anchor 1・設計文書の生成区間 1・床の歯の fixture 16）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: **0（通す）**。本流の binary（main root の `target/debug/folio`・2bb2195 の組み立て）に write-set の 23 本を渡した `folio ceiling --gate --dir design-intent --write-set …` の答え「folio ceiling: 通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）」（起草の記録の gate-d204b.log）。
- 前提: **base = 本流 2bb2195**（便 203 の着地・台帳 f2-648.275.10）。この契約の数はすべて 2bb2195 と、その上に作り直した見本の写しの実測（参考値・規則の表の行 D-13）。改訂 b（2026-09-29 11:1x〜12:0x）: 前の版は base を 1c7f927 に置き、見本の中身へ write-set を置き換えてよいと書いていたので、便 203 の着地の後の本流では 203 の変更を消す読み方ができた（契約の審査 FAIL・literal-mismatch）。本版は base と見本を 2bb2195 の上に作り直し、数を全部撃ち直した。
- 実装の見本: origin の枝 `impl/d204b`（commit **95b1b6e**・親 = 本流 2bb2195・1 commit）。前の見本 `impl/d204` 1bc273b（親 1c7f927）の差分を 2bb2195 の上に `git merge --squash` で当てた（衝突 0・中身の字は 1bc273b の差分と同じ）。`git diff 2bb2195 95b1b6e` が便の全体の差分（23 file・+299 −74・48,932 byte）。**作業者は、本流 2bb2195 に対する見本の差分（`git diff 2bb2195 95b1b6e`）を write-set の file に当てる**（file を丸ごと見本の中身に置き換えない・受付の時の本流の write-set の file が 2bb2195 と違えば、差分を当てた後に §1 (e)(f) の数を数え直す）。write-set の外は変えない。`design-intent/rules.yaml` は手で書かず、見本の binary の `folio schema --dir design-intent --write` で書く。
- 並行の便との重なり: 便 203 は本流 2bb2195 に着地済みで、見本はその上で作り直した（§1 (g) 3）。

## 1. 設計

### (a) いま起きていること（base 2bb2195 の実測・参考値）

1. **決定の欄の床は裁定の時刻を見ない。** 決定の欄の歩き手（`crates/folio/src/ruling.rs` の `sites`）は規則の表の各行の `ruling` を拾い、`crates/folio/src/check.rs` の `check_rulings` が裁定 id を 1 つ以上切り出せるかだけを数える。規則の表の床（`check_rules`）は行の `id`・`article`・`what` の非空と閉じた欄と `refs`・`key` を数え、`rules.rs` の必須の欄の一覧（`THRESHOLD_REQUIRED`・`DISCIPLINE_REQUIRED`・どちらも `ruled_at` を持つ）は「置いてよい欄」としてだけ使う。
2. **欠けは面の床が拾うことがあり、形は誰も拾わない。** 便 187 の面の床（`site.rs` の `check_faces`・床のほかの段が何も数えていないときだけ面を組む）は、憲法の面が行の `ruled_at` を読めないと `[面]` の違反を 1 件出す。folio2 の正本の写しで 1 か所ずつ変えた素の床（起草の記録の probe-f2-2bb.log・base 2bb2195 の binary・1c7f927 の binary の probe-f2.log と同じ答え）:

| 変えた所 | base の床 |
| --- | --- |
| 行 D-18 の `ruled_at` を外す | 終了 1・`[面] rules.yaml.discipline[17]: 欄 ruled_at が無い` |
| 行 R-22 の `ruled_at` を外す | 終了 1・`[面] …thresholds[21]: 欄 ruled_at が無い` |
| 行 D-18 を null・一覧 | 終了 1・`[面] …ruled_at: 文字列でない` |
| 行 D-18 を「2026/09/28」・空の字・「2026-09-28 07:18 JST」・未記入 | **終了 0（落とさない）** |
| 行 D-18 を「2026-09-28T01:55Z」（UTC の分） | 終了 0 |
| 閾値の行 R-18 の kind を human-review | **終了 0（落とさない）** |
| 開発規律の行 D-18 の kind を deny | 終了 0（本便の外・(i) 2） |

   面の床は、ほかの段が何かを数えている置き場では回らない（同じ原因を 2 度数えない・便 187 の決まり）。台帳の観測 (1)（asks の写しで `ruled_at` を消して合格 0）は、便 187 の着地の前の binary の答えで、今の binary でもその写しは別の違反（設計ノートの欄の決まりの生成区間の差）が先に在るので欠けを数えない。**穴は、欠けが面の床だけに頼っていることと、形を誰も数えないこと**。
3. **置き場の今の形。** folio2 の規則の表は 40 行とも年-月-日（例 2026-09-28・引用符なし）。tsuzuri の規則の表（写し 6ffa67a・前の版の 947fb4a と 79c8608 も同じ）は 29 行とも UTC の分（例「2026-09-24T22:39Z」・引用符つき）。骨格（`folio init`）の行は `ruling` と `ruled_at` がどちらも 未記入。床の歯の fixture のうち `ruling` を持つ最小の規則の表 16 本（便 181 が `ruling` を足した組）は `ruled_at` を持たない（面の床に届かないので今は通る）。
4. **(2) 閾値の行の human-review は欄の決まりの字で許されない。** 規則の表の生成区間の `kind_meaning.human-review` は「人が守る作法（憲法の human-review に対応・D 行）」、`kind_map_to_constitution` は「R 行にだけ適用する（D 行は作法＝条の機構とは別）」で左辺は deny・build-check・detect の 3 つ（human-review の対応は無い）。閾値の行（機械が測って止める値）の種別は憲法の機構に対応しなければならず、human-review の閾値の行は対応を持てない。値域 `enums.kind` は 2 節で共通なので値域の床（面）は通す。folio2 と tsuzuri の閾値の行に human-review は 0 行（開発規律の行は両方とも全部 human-review）。
5. **base の歯（参考値）。** workspace の nextest 1153 / 1153（便 203 の着地の後）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 39 file。`git grep -n f204_ -- crates` は 0 件。

### (b) 直す先

1. **`crates/folio/src/ruling.rs`（歩き手）。** 拾った欄 `Site` に `row`（欄を持つ規則の表の行・ほかの欄は None）を足し、`sites` は規則の表の 2 節の各行の `ruling` の欄に行を添える。拾う欄・順・値・書き出し（`--emit-rulings`）は変えない。
2. **`crates/folio/src/rules.rs`（規則の表の欄の決まり）。**
   - 行の欄の字 `ROW_TIME`（ruled_at）・時刻の形の字面 `TIME_FORMAT`（`^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$`）・判定 `is_time`（先頭 10 字を判断の記録の日付と同じ `adr::is_date` で見て、後ろが無いか「T」時 2 桁「:」分 2 桁「Z」なら真・字の境でない位置では偽・暦に在る日かは見ない）。
   - 閾値の行の種別の床 `kind_violations`: 種別が値域の中で、床の木 `FLOOR` の `kind_map_to_constitution` の左辺に無い閾値の行を字にする（human-review だけが当たる・値域の外は面の床の持ち分で数えない・開発規律の行は数えない）。左辺は同じ `FLOOR` の木から引く（2 つ目の一覧を持たない）。
   - 床の木 `FLOOR`（規則の表の生成区間の正本）: `refs_note` の後に `ruled_at_format`（`TIME_FORMAT`）と `ruled_at_note` の 2 欄、`kind_map_to_constitution_note` の末尾に 1 文（閾値の行の kind は左辺のどれかで、human-review を置けば床が落とす）。値域・必須の欄・ほかの注は変えない。
3. **`crates/folio/src/check.rs`（床）。** 新しい関数 `check_times`: 歩き手の出力のうち行を添えた欄ごとに、裁定の欄が骨格の印の行（裁定の前・`check_rulings` が まだ分からない にする）を除き、`ruled_at` が無い・null・字でない（一覧・表）・形の違う（空の字・骨格の印を含む）を種別 裁定 id の違反にする。`check_dir` は `--emit-rulings` の段の後で呼ぶ。`check_rules` は `key_violations` に続けて `kind_violations` を種別 schema の違反にする。`check_rulings` の字は変えない。違反の字（歯の側の手書き）:
   - `[裁定 id] rules.yaml: 行 <id> の ruled_at が無い＝裁定の時刻が無い`（欄が無い・null）
   - `[裁定 id] rules.yaml: 行 <id> の ruled_at が字でない（一覧か表）＝裁定の時刻が無い`
   - `[裁定 id] rules.yaml: 行 <id> の ruled_at「<値>」が裁定の時刻の形でない（年-月-日か UTC の分・形は rules.yaml の ruled_at_format）`
   - `[schema] rules.yaml: 行 <id> の kind「human-review」は閾値の行に置けない（kind_map_to_constitution の左辺に無い＝開発規律の行の作法）`
   どの字も folio2 の条の番号を持たない（外の置き場にもそのまま出る）。
4. **`design-intent/rules.yaml`（生成区間）。** 見本の binary の `folio schema --dir design-intent --write` で書き直す（3 行増・1 行減）。区間の外は 1 byte も変えない。外の置き場では同じ導出が注の括弧の中の folio2 の条の番号（条 P-17.1・条 P-17.3）を落とす（便 174・194 の `floor.rs` の決まり・tsuzuri の写しで実測）。
5. **`tests/fixtures/schema/rules-region.txt`（凍結 anchor）。** 前の anchor に同じ字（2 行と 1 文）を手で足した写し（生成器の出力を写さない・P-10.1）。
6. **床の歯の fixture 16 本。** `tests/fixtures/{adr,anchor,check,link,refs,vocab}/*/rules.yaml` のうち `ruling` を持つ 16 本（便 181 が `ruling: f2-648.2 notes 2026-09-12` を足した組）の各行に `ruled_at: 2026-09-12` を足す（流れの形は `ruling` の直後・段の形は `ruling` の次の行＝既存の歯が見る行の番号は変わらない）。各 fixture の変異（重複キー・空の欄・未知の節ほか）は変えない。
7. **変えないもの。** 裁定 id の文法と決定の欄の閉じた一覧（判断の記録の欄の決まり `adr/schema.yaml` の生成区間は変わらない）・`check_rulings` の判定と字・`--emit-rulings` の出力・値域（kind・status・stage・key）・必須と任意の欄の一覧・面の床・開発規律の行の種別・folio2 と tsuzuri の正本の行・骨格の出力（骨格の行は `ruling` が骨格の印なので時刻を数えない）。

### (c) 歯（f204_・base で 0 件・6 本）

1. **単体 f204_the_time_is_the_date_or_the_utc_minute（`rules.rs` の tests）。** `TIME_FORMAT` と `ROW_TIME` が手書きの字と同じ。年-月-日・UTC の分（「2026-09-24T22:39Z」）は真、「/」区切り・月が 1 桁・JST の時刻付き・Z の無い分・時が 1 桁・秒付き・後ろに字・小字の t と z・時分が英字・前後の空白・未記入・全角の数字（字の境でない位置を含む）は偽。
2. **単体 f204_a_threshold_row_takes_only_a_kind_mapped_to_the_constitution（同）。** 手書きの規則の表で、閾値の行の deny・build-check・detect・値域の外（bogus）と開発規律の行の human-review は字 0、閾値の行の human-review はちょうど 1 つの字（手書き）。`kind_map_to_constitution` の左辺として真になる種別は deny・build-check・detect の 3 つ。
3. **`tests/ruling.rs` f204_a_rules_row_without_a_time_is_one_violation。** 床の土台の写し（`tests/fixtures/floor_base/design-intent/`）の閾値の行 R-10 と開発規律の行 D-8 から `ruled_at` を外すと、種別 裁定 id の違反がちょうど「…行 <id> の ruled_at が無い＝裁定の時刻が無い」の 1 つ。base は面の床にも届かない（土台は版管理が無く まだ分からない が在る）ので 裁定 id の行が 0 で **RED**。
4. **同 f204_a_time_out_of_form_is_one_violation。** 行 R-10 の `ruled_at` を「2026/09/12」・空の字・「2026-09-12 20:25 JST」・「2026-09-12T1:25Z」・「2026-09-12T11:25:00Z」・「2026-09-12T11:25」にすると形の違反がちょうど 1 つ、一覧・表・null はそれぞれの字の違反がちょうど 1 つ（表は検証役の非 blocking 1）。base は形の 6 通りで 0 行（**RED**）。
5. **同 f204_the_two_forms_pass_and_the_mark_waits_only_with_the_ruling。** 年-月-日と UTC の分は違反 0。`ruling` と `ruled_at` をどちらも骨格の印にすると違反 0・まだ分からない は `ruling` の 1 行だけ増える。`ruling` が埋まったまま `ruled_at` だけ骨格の印にすると形の違反がちょうど 1 つ（base は 0 行で **RED**）。
6. **同 f204_a_threshold_row_of_human_review_is_one_violation。** 行 R-10 の kind を human-review にすると標準出力の違反の行がちょうど `[schema] rules.yaml: 行 R-10 の kind「human-review」は…` の 1 行、deny・build-check・detect は 0 行（base は human-review で 0 行・**RED**）。
7. **既存の歯の期待の直し。** `tests/schema_docs.rs` の規則の表の生成区間の凍結 anchor の行数・byte 数・sha256 の 3 つの定数（anchor を手で直した後に sha256sum と wc で測り直した値・注に便 204 の 1 文）。`rules.rs` の単体の `rules_floor_derives_the_frozen_anchor_byte_for_byte` は本文を変えず、新しい anchor と床の木の導出が byte で一致する。
8. **RED の実測。** 歯の file だけ（`tests/ruling.rs`・`tests/schema_docs.rs`・`tests/fixtures/schema/rules-region.txt`）を base に当てると、tests/ruling.rs の f204_ の 4 本（11 本のうち）・tests/schema_docs.rs の規則の表の生成区間を見る 4 本（28 本のうち・実の正本と anchor の byte 一致・1 byte の差・書き直しの冪等・f85_ の anchor 一致）・単体の anchor の byte 一致の 1 本（rules::tests の 7 本のうち）が落ちる（nextest の rc 100・base 2bb2195 に 95b1b6e の歯の file を当てた・起草の記録の red-ruling.log・red-schema_docs.log・red-unit.log）。単体の f204_ の 2 本は実装の関数を呼ぶので base では組めない（0 本）。

### (d) 採らなかった形

1. **時刻の欄を決定の欄の閉じた一覧（`ruling.rs` の `FIELDS`）に足す。** 一覧は判断の記録 ADR-31 決定 (1) の裁定 id の欄の一覧で、判断の記録の欄の決まりの生成区間（`ruling_fields`）と書き出しの対象に写る。時刻は裁定 id を持たないので一覧の意味が変わり、書き出しと生成区間も動く。歩き手に行を添え、床の関数を分けた。
2. **時刻の骨格の印をいつも まだ分からない にする。** 骨格の各行で まだ分からない が 2 行ずつになり（同じ原因を 2 度数える）、骨格と外の置き場の出力と既存の骨格の歯の期待が変わる。`ruling` が骨格の印の行は時刻を数えず、`ruling` が埋まった行の時刻の骨格の印は違反（時刻の無い裁定）にした。
3. **規則の表の床で必須の欄の全部の非空を数える。** 欠けと空は拾えるが形は拾えず、kind・status・stage・value の欠けが面の床と 2 度数えになる。時刻は P-17 の決まりなので決定の欄の歩き手に足した。
4. **時刻の形を `ruling.rs` に置く。** 歩き手の module は層 1（読む）で、年-月-日の関数（`adr.rs`・層 2）を呼べない（`tests/modules.rs` の層の辺の凍結一覧・最初の見本 6635999 で落ちた）。規則の表の欄の決まりの module（`rules.rs`・層 2）に置いた。
5. **JST の時刻付きや暦の妥当さまで数える。** folio2 と tsuzuri の今の形（年-月-日と UTC の分）に要らず、裁定 id の日時との一致は台帳の持ち分（器と人）。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本 95b1b6e の写しで workspace の nextest 1159 / 1159（base 2bb2195 の 1153 + f204_ の 6 本）・clippy 0 警告・床 4 本 rc 0（見本の binary の `folio schema --dir design-intent --write` は何も書き換えない＝生成区間は見本の字のまま・folio2 の素の床の標準出力と標準エラーと `--emit-rulings` の出力は base と byte で同じ）・`folio build --write` 39 file が base と全 file で byte で同じ（起草の記録の base-2bb2195-*.log・impl-95b1b6e-*.log・impl-95b1b6e-schema-write.log）。字の期待を直した既存の歯は (c) 7 の 3 つの定数だけ。fixture 16 本に時刻を足さないと、その fixture を使う 25 本（check 3・gitcheck 8・anchor 2・freeze_root 1・link 3・adr 3・refs 3・vocab 2）が時刻の欠けの違反で落ちる（最初の見本 6635999 の実測・impl-nextest.log）。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、単体の f204_ と rules::tests と ruling::tests・tests/ruling.rs・tests/schema_docs.rs・tests/check.rs を撃つ）。** 21 通りとも落ちる（生き残り 0・95b1b6e で撃った・起草の記録の mut.log・落ちた歯の本文は mut-M*.log）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 床が時刻を数えない | tests/ruling.rs の f204_ の 3 本（欠け・形・骨格の印） |
| M2 UTC の分を通さない | 単体の形・f204_ の 2 つの形 |
| M3 年-月-日の後ろに何が在っても通す | 単体の形・f204_ の形の違う |
| M4 時と分が数字かを見ない | 単体の形 |
| M5 年-月-日の形を見ない | 単体の形・f204_ の形の違う |
| M6 骨格の印の行でも時刻を数える | f204_ の 2 つの形と骨格の印 |
| M7 欄が無い・null を数えない | f204_ の欠け・形の違う（null） |
| M8 一覧・表の時刻を数えない | f204_ の形の違う（一覧） |
| M9 歩き手が行を添えない | f204_ の 3 本 |
| M10 歩き手が開発規律の行だけ添えない | f204_ の欠け（行 D-8） |
| M11 時刻の欄の名を違える | tests/check.rs の 44 本・tests/ruling.rs の 10 本・単体 1 本（55 本） |
| M12 値域の外の種別も閾値の行の違反にする | 単体の種別 |
| M13 種別の対応を kind_meaning から引く | 単体の種別・f204_ の human-review |
| M14 床が閾値の行の種別を数えない | f204_ の human-review |
| M15 開発規律の行の種別も数える | tests/check.rs の 41 本・tests/ruling.rs の 2 本・単体 1 本（44 本・正本の開発規律の行が違反になる） |
| M16 生成区間の注の字を変える | tests/schema_docs.rs の 15 本と単体の anchor の byte 一致 |
| M17 形の字面の写しを変える | tests/schema_docs.rs の 15 本と単体 2 本（anchor の byte 一致と形の字面） |
| M18 字の境で止まらない | 単体の形（全角の数字で panic） |
| M19 違反の在り処を裁定の欄の字にする | f204_ の 3 本 |
| M20 時刻の骨格の印を通す | f204_ の 2 つの形と骨格の印 |
| M21 表の時刻だけを数えない（検証役の Y4） | f204_ の形の違う（表の 1 通り） |

3. **外の置き場（tsuzuri の写し・起草の記録の tz6ffa.log と probe-tz6ffa.log・本物の tsuzuri では何も撃たない）。** 写しは本物の `.git` だけを写して clone した木（hook なし）で、本物の main 6ffa67a（2026-09-29 11:11・規則の表は 947fb4a と同じ）で撃った（前の版の 947fb4a と 79c8608 の写しでも同じ答え）。
   - 書き直す前: base 2bb2195 の床・`folio schema --check`・derive --check（一致 62）はどれも 0・build 87 file。見本の素の床は合格 0（29 行とも UTC の分）・`folio schema --check` は rc 1（`rules.yaml: 生成区間 … byte ≠ 導出 … byte`）・derive --check 0・build 87 file が base と byte で同じ。
   - 見本の `folio schema --write` を写しの中で撃つと rules.yaml の生成区間だけが 3 行増・1 行減。`ruled_at_note` は括弧の中の 条 P-17.1 と 条 P-17.3 が落ちて文が立つ。書いた後: 床 0・schema --check 0・build 87 file が base と byte で同じ。base の binary の素の床も書いた後の写しで 0。
   - 写しの変異（行 R-1 と D-1）: 見本は `ruled_at` の欠け（R-1・D-1）・「2026/09/24」・空の字・「2026-09-24T22:39」を 裁定 id の違反 1 件に、kind human-review を schema の違反 1 件にし、年-月-日「2026-09-24」は通す。base は欠けを `[面]`、ほかを合格 0 にする。
4. **骨格。** 骨格の行は `ruling` と `ruled_at` がどちらも骨格の印なので、素の床の まだ分からない・違反・終了コードは base と同じ（既存の骨格の歯 init・place_name・polarity・mechanism_live が通る）。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・縮む file・消す file・新しい dir は無い。`crates/folio/src/rules.rs` は単体の歯（f204_ の 2 本と anchor の byte 一致）を持つので verify の `--bin folio` の scope。
2. **余地（CapHeadroom）。** write-set の src の 3 本（base 2bb2195 と見本 95b1b6e・python と awk の 2 実装で一致・起草の記録の cap-95b1b6e.log・cap.sh）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/check.rs` | 1180 | 320 | 1206（+26） | 294 |
| `crates/folio/src/rules.rs` | 549 | 951 | 644（+95・単体の歯 2 本を含む） | 856 |
| `crates/folio/src/ruling.rs` | 515 | 985 | 521（+6） | 979 |

3. **size は S。** file ごとの増分の最大は rules.rs の +95（単体の歯を含む）で S の見積 100 の内。余地の最小（check.rs の base 320・本便の後 294）は S の 100 を超える（M の 300 は割る）。
4. **verify は 6 行**で、done の 6 つの塊と 1 対 1 に揃える。見本 95b1b6e で 6 行とも rc 0（2・4・11・28・9 本と clippy・起草の記録の verify-impl-95b1b6e.log）。
   1. `cargo nextest run -p folio --bin folio f204_` = (c) 1・2（2 本）。
   2. `cargo nextest run -p folio --test ruling f204_` = (c) 3〜6（4 本）。
   3. `cargo nextest run -p folio --test ruling` = 便 181・182 の決定の欄の歯（裁定 id の違反と まだ分からない の字と数が変わらない）と (c) 3〜6。
   4. `cargo nextest run -p folio --test schema_docs` = 規則の表の生成区間が新しい凍結 anchor と byte で一致し、書き直しが冪等（(c) 7）。
   5. `cargo nextest run -p folio --bin folio rules::tests` = 床の木の導出と凍結 anchor の byte 一致・値域と欄 key の歯・便 203 の名札の歯（(c) 7）。
   6. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   fixture 16 本を使う歯（check・gitcheck・anchor・freeze_root・link・adr・refs・vocab）は common-verify の workspace の nextest が撃つ。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり 0（通す）。write-set に `design-intent/rules.yaml`（生成区間だけ）が在るので門の対象（規則の表の行 D-12）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d204b の本契約で PRECHECK（起草の記録の precheck-d204b.log）。見本 95b1b6e で verify の 6 行とも rc 0。
3. **便 203 は着地済み（本流 2bb2195・台帳 f2-648.275.10・器の作業者の実装）。見本は本流の上で作り直した。数は新しい見本の実測。** 前の見本 impl/d204 1bc273b（親 1c7f927）の差分を 2bb2195 の上に `git merge --squash` で当てると衝突 0 で、中身の字は前の見本の差分と同じ（新しい見本 impl/d204b 95b1b6e）。便 203 と同じ file を書き換えるのは `crates/folio/src/check.rs`（203 = `check_dir` の置き場の名・`check_rulings` の名の引数と骨格の印の行の字・`place_range` の字／本便 = `check_times` を `check_rulings` の後に足す・呼ぶ所は `--emit-rulings` の段の後・`check_rules` の 1 行）と `crates/folio/src/rules.rs`（203 = 頭の use・名札の一覧・tests の末尾／本便 = `ROW_TIME`〜`is_time`・`kind_violations`・床の木の 3 か所・tests の中ほど）の 2 本で、違う行。`check_rulings` の字は変えない（203 が書き換えた骨格の印の行の字をそのまま残す）。数（nextest・RED・変異・余地・verify・tsuzuri の写し）は §1 (a)(c)(e)(f) のとおり 95b1b6e と 2bb2195 の実測で、前の版の 1c7f927 の数から動いたのは nextest の本数（base 1147 → 1153・見本 1153 → 1159）・rules::tests の本数（8 → 9）・余地（check.rs 328 → 320 と 302 → 294・rules.rs 1002 → 951 と 907 → 856）・tsuzuri の build の file 数（72 → 87・tsuzuri の側の増え）だけ。受付の時の本流の write-set の file が 2bb2195 と同じなら、数え直しは要らない。取り消されたレーンの枝 impl/d201（`ruling.rs`）とも衝突 0。
4. **着地の後（外の置き場）。** tsuzuri は pin を上げるときに `folio schema --write` を 1 回撃って commit する（rules.yaml の生成区間の 3 行増・1 行減・(e) 3）。規則の表の行は書き直さなくても床が通る（29 行とも UTC の分）。席は tsuzuri へこの 1 行を返す。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/ruledat-draft.md`、script と log は同じ dir の `ruledat-scripts/`。
1. `run.sh <木> <印>`: 組み立て・workspace の nextest（--test-threads 2）・clippy・床 4 本・`folio build --write` の file 数と sha。
2. `red.sh <base の木> <見本の rev>`: 歯の file 3 本だけを base に当てて RED（落ちた歯の本文を log に残す）。
3. `mut.py <見本の clone>`: 変異 21 通り。
4. `cap.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk）。
5. `verify.sh <木> <印>`: 契約の verify の 6 行。
6. `probe.sh <folio> <木> <印>`・`probe-tz.sh <folio> <印>`: 規則の表の変異の写しで素の床の答え（(a) 2 と (e) 3）。`tz.sh <base の folio> <見本の folio> <rev…>`: tsuzuri の写し（`prep-tz.sh` で作る・hook なし）で base と見本の床・生成区間・build を比べる。本版の写しは `prep-tz-n.sh <名>`（写しの名を引数に）・`tz-n.sh`（写しの名を環境変数 TZNAME で）・`probe-tz-n.sh` で作って撃った（tz6ffa）。
7. `chain3.sh`: 本流 2bb2195 の上の見本 95b1b6e で 1〜5 を順に撃つ（1 本ずつ・重い cargo は flock で 1 つ）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書・憲法・判断の記録・値域・必須と任意の欄の一覧・決定の欄の閉じた一覧・folio2 と外の置き場の規則の表の行・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 本便は時刻の在ることと形だけを数え、時刻が裁定 id の日時と合うか、行を変えた commit で時刻も変えたか（条 P-17.1 の「変更に付ける」）は数えない（差分を数える口は無い）。開発規律の行に human-review でない種別（例 deny）を置いた写しも床は通す（欄の決まりの字は「D 行は作法」と書くが、開発規律の行の種別を human-review に限る字は無い・席へ返す）。憲法の条 P-17 の機構の注の「床が数えるのは、今在る各行の裁定の欄（ruling）に台帳の裁定 id が 1 つ以上在る形までである」は、本便の後は時刻の在ることと形も数える（憲法は本便で変えない・席へ返す）。本便の「暦に在る日かは見ない」は `check_times` の話で、引用符の無い暦に無い日（例 2026-13-45）は、型付きの読み（面の段）が正規化できない値として床全体を まだ分からない にする（base も同じ・引用符付きなら通す）。
   **骨格の印の判定は 2 か所に残す。** 「骨格が書く欄の値が骨格の印」の判定は `check_rulings` の腕と `check_times` の条件の 2 か所に同じ関数の組（`Site::skeleton` と `adr::unfilled`）で在る。起草の時に 1 つの関数に寄せると `check_rulings` の腕の条件の行が変わり、便 203 がすぐ下の 2 行（骨格の印の行の字）を書き換えるので、便 203 の見本 impl/d203 d41ba90 と `git merge-tree` で衝突した（試した差分は起草の記録の unify-attempt.patch）。本版も `check_rulings` の字は 2bb2195 のまま変えず、寄せるのは本便の着地の後の小さな直し（台帳 f2-648.276.7・席が起こした memo）。
3. **撤退条件。** (1) 要件書・判断の記録・憲法・値域を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に §1 (c) 7 の 3 つの定数と §1 (b) 6 の fixture 16 本のほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、folio2 の素の床の標準出力と標準エラーか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。(4) tsuzuri の写しで書き直した後の床が合格 0 でなければ、止めて席へ返す。

## 2. 範囲

- 入れる: 歩き手が規則の表の行を添えること（`ruling.rs`）・時刻の形と閾値の行の種別の床と生成区間の 2 欄と 1 文（`rules.rs`）・床の関数 `check_times` と種別の違反の出し方（`check.rs`）・生成区間の書き直し（`design-intent/rules.yaml`）・凍結 anchor・歯（単体 2 本・`tests/ruling.rs` 4 本・`tests/schema_docs.rs` の定数 3 つ）・床の歯の fixture 16 本の時刻。
- 入れない: 要件書・憲法・判断の記録・値域・欄の一覧・決定の欄の一覧・書き出し・面の床・正本の規則の表の行・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| row | 行を添える | `ruling.rs` の歩き手が規則の表の行の欄に行を添える |
| time | 時刻の形 | `rules.rs` の `ROW_TIME`・`TIME_FORMAT`・`is_time`（年-月-日は `adr::is_date`） |
| floor | 時刻の床 | `check.rs` の `check_times`（裁定の前の行を除き、欠け・字でない・形の違うを違反に） |
| kind | 種別の床 | `rules.rs` の `kind_violations`（閾値の行の human-review）と `check_rules` の 1 行 |
| region | 生成区間 | `FLOOR` の `ruled_at_format`・`ruled_at_note`・対応の注の 1 文と `schema --write`・凍結 anchor |
| teeth | 歯 | 単体 2 本・`tests/ruling.rs` 4 本・`tests/schema_docs.rs` の定数・fixture 16 本の時刻 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 203（本流 2bb2195）。
- 本便の着地の後に席が見ること: 台帳 f2-648.276.1 を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) 4 の 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gy"
title = "床が規則の表の行の裁定の時刻 ruled_at の在ることと形を全行で数え、閾値の行に human-review を置かせない（台帳 f2-648.276.1・裁定 f2-648.276.1 notes 2026-09-29 07:16 JST・条 P-17.1）: crates/folio/src/ruling.rs の決定の欄の歩き手が規則の表の行の欄に行を添え、crates/folio/src/check.rs の新しい関数 check_times が同じ歩き手の出力で、裁定の欄が骨格の印の行を除いて ruled_at の欠け・null・字でない・形の違う（空の字と骨格の印を含む）を種別 裁定 id の違反にする。形は年-月-日か UTC の分で、crates/folio/src/rules.rs の ROW_TIME・TIME_FORMAT・is_time（年-月-日は adr::is_date）が持ち、床の木 FLOOR の ruled_at_format と ruled_at_note として規則の表の生成区間 design-intent/rules.yaml へ folio schema --write で写す（凍結 anchor tests/fixtures/schema/rules-region.txt は前の anchor に同じ字を手で足す）。rules.rs の kind_violations は閾値の行の種別が FLOOR の kind_map_to_constitution の左辺に無い（human-review）行を種別 schema の違反にし、対応の注に 1 文を足す。check_rulings の字・決定の欄の一覧・書き出し・値域は変えない。床の歯の fixture 16 本の各行に ruled_at: 2026-09-12 を足す。歯は rules.rs の単体の f204_ 2 本と tests/ruling.rs の f204_ 4 本と tests/schema_docs.rs の anchor の定数 3 つ。実装の見本は origin の枝 impl/d204b の commit 95b1b6e（親 2bb2195）で、作業者は本流 2bb2195 に対する見本の差分を write-set の file に当て、write-set の外は変えない。base = 本流 2bb2195"
req = ["FR26", "FR19"]
section = "1"
write-set = ["crates/folio/src/ruling.rs", "crates/folio/src/rules.rs", "crates/folio/src/check.rs", "crates/folio/tests/ruling.rs", "crates/folio/tests/schema_docs.rs", "tests/fixtures/schema/rules-region.txt", "design-intent/rules.yaml", "tests/fixtures/adr/effective-no-approval/rules.yaml", "tests/fixtures/adr/schema-drift/rules.yaml", "tests/fixtures/adr/two-adopted/rules.yaml", "tests/fixtures/anchor/no-anchor/rules.yaml", "tests/fixtures/anchor/root-digest-drift/rules.yaml", "tests/fixtures/check/dup-key/rules.yaml", "tests/fixtures/check/empty-field/rules.yaml", "tests/fixtures/check/unknown-section/rules.yaml", "tests/fixtures/link/adr-id-missing/rules.yaml", "tests/fixtures/link/amended-by-orphan/rules.yaml", "tests/fixtures/link/retreat-kind-drift/rules.yaml", "tests/fixtures/refs/bad-counts/rules.yaml", "tests/fixtures/refs/dangling-id/rules.yaml", "tests/fixtures/refs/orphan-rule/rules.yaml", "tests/fixtures/vocab/exemptions/rules.yaml", "tests/fixtures/vocab/unknown-word/rules.yaml"]
verify = ["cargo nextest run -p folio --bin folio f204_", "cargo nextest run -p folio --test ruling f204_", "cargo nextest run -p folio --test ruling", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --bin folio rules::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "binary の単体の f204_ の 2 本（時刻の形は年-月-日と UTC の分だけが真で字の境でない位置でも止まらない・閾値の行の human-review だけが種別の字になり左辺は deny と build-check と detect）が緑、tests/ruling.rs の f204_ の 4 本（床の土台の写しで ruled_at の欠け・形の違う 6 通り・一覧と表と null・裁定の欄が埋まった行の時刻の骨格の印・閾値の行の human-review がそれぞれ違反ちょうど 1 つ、年-月-日と UTC の分と両方の骨格の印は違反 0）が緑、tests/ruling.rs の歯の全部（便 181・182 の決定の欄の違反と まだ分からない が変わらない）が緑、tests/schema_docs.rs の歯の全部（規則の表の生成区間が新しい凍結 anchor と byte で一致）が緑、binary の単体の rules::tests の全部（床の木の導出と凍結 anchor の byte 一致）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
