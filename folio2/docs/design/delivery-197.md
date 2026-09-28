# 設計: 便 197 — 相談窓口の回答の値を正本から読み、回答の読み方と固定の値を生成区間へ写す。規則の表の欄の決まりの除外を人の作業の時間と AI の費用に狭める（台帳 f2-648.74・f2-648.227・S）

- 要件: FR1（相談窓口）・FR19（欄の決まりの生成区間は床の定数から決定的に導出する）。規範文・確かめ方・受入基準は変えない。
- 条: P-6.3（同じ内容を 2 つの面が持つとき一方を正本にし他方は導出）・P-6.4（2 つの面を人が書き一致を検査で強制しない）・P-5.6・P-5.1。
- 出所: 判断の記録 ADR-11 決定 (3)(ア)・(4)⑥（相談窓口の回答の値は file が正本で、実装は実行時に読む・台帳 **f2-648.74**）と規則の表の行 D-11。同乗は台帳 **f2-648.227**（除外の字が機械の待ち時間まで除外に読める・天井の 46 周目の忠実さの気づき・席の依頼 2026-09-28 18:1x）。
- 承認: .74 は file の値に従う形へ実装を直し（folio2 と土台と tsuzuri では答えが 1 byte も変わらない）、その読み方の写しを足す＝行 D-17 の 4 つに当たらないと読み、席の裁定で受け付ける。.227 は行 D-17 の注（実装の型付きの定数で生成区間の規則を変える）に当たり、持ち主の承認（2026-09-28 の束・C）の後に受け付ける（検証役 B1）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾。審査の材料は行 `gr` が指す §1 だけ。write-set 9 本（src 3・歯の file 2・設計文書 2・fixture 2）・新しい file も dir も無い。
- 門: **0（通す）**。本流 44c14ac の組み立てに write-set 9 本を渡した `folio ceiling --gate --dir design-intent --write-set …` の答え。
- 前の便: **base = 本流 44c14ac**（便 185・192〜194 の着地の後）。数は base の写しの実測（参考値・行 D-13）。組み直す手順は控え `~/.local/share/folio2/handoff-2026-09-28/copies-scripts/d197/`。
- 見本: origin の枝 `impl/d197`（**92972b1**・本流 44c14ac を merge した後に検証役の N1 の歯 1 本〔(c) の 6〕を積んだ commit）。`git diff 44c14ac 92972b1` が便の全体の差分。作業者は write-set の file をこの commit の中身にしてよい。

## 1. 設計

### (a) いま起きていること（参考値・base 44c14ac）

1. **回答の値が 2 か所（.74）。** 相談窓口の正本 `design-intent/intake.yaml` は `answers: {values: [はい, いいえ], default: recommend}` を人が書き、床（`intake.rs`）はそれを値域として読む。ところが支度表の命令（`sheet.rs`）は定数 YES = はい・NO = いいえ を手で持ち、回答が YES なら質問の行の yes の行き先・NO なら no の行き先を選び、ほかの値は断る。file の側の値を変えると、床は合格なのに `folio intake --write` だけが「はい でも いいえ でもない」で止まる。
2. **固定の値の写しが無い。** `intake.rs` の INJECT_TARGET（行き先の値域に足す inject）と DEFAULT_RECOMMEND（answers.default が取れる recommend）は、判定に使う値なのに生成区間に写しが無い。相談窓口の正本の生成区間は最上位の節の閉じた一覧だけを持つ。
3. **除外の字が広い（.227）。** 規則の表の欄の決まりの生成区間 excluded（正本 = `rules.rs` の EXCLUDED_WHAT と why）は「時間」「費用」を凍結から外すと書き、字のうえでは道具が決定的に測れる機械の待ち時間まで除外に当たる。要件書 not_frozen が外しているのは人の作業の時間（60 分以内）と AI の費用（300k token 以下）である。
4. **base の歯。** workspace の nextest 1070 / 1070・clippy 0 警告・床 4 本 rc 0・`folio build --write` 37 file。`f197_` と行 id `gr` は 0 件。

### (b) 直す先

1. **`sheet.rs`。** YES / NO の定数を外し、intake.yaml の answers.values を読んで、1 つ目の値の回答は質問の行の yes の行き先、2 つ目の値の回答は no の行き先を選ぶ（質問の行の欄は `intake.rs` の新しい定数 ANSWER_BRANCHES = [yes, no] の順に読む）。values がちょうど 2 つでなければ、`--write` は 1 byte も書かずに既存の断りの形（`folio intake: まだ分からない: <理由>`・終了 2）で断る。`--print` は問いと推奨の字だけを出し行き先を出さないので、断りは `--write` に置く（base と同じ 0 のまま）。単体の歯 2 本は values を渡す形に直す。
2. **`intake.rs` の INTAKE_FLOOR（相談窓口の正本の欄の決まり）。** `answers: {values_count: 2, branches: [yes, no], default_fixed: recommend}` と `targets: {fixed: [inject]}` と注 2 つを足す。値は ANSWER_BRANCHES（数は長さ）・DEFAULT_RECOMMEND・INJECT_TARGET を引く。answers_note は、値の言葉の正本は人が書く answers.values で folio intake が毎回読むこと・位置で行き先を選ぶこと・数が違えば `--write` が断り床（folio check）は数えないこと・正本は実装の定数でこの節はその写しであることを書く。
3. **`rules.rs`（.227）。** EXCLUDED_WHAT を「人の作業の時間（「60 分以内」）」「AI の費用（「300k token 以下」）」に、excluded.why を「人の作業の時間と AI の費用は測る仕組みが別で凍結できない。M1 の実地試験（非エンジニア 1 回）で初めて数値を決める（要件書 not_frozen）。道具が決定的に測れる機械の待ち時間は除外に当たらず、規則の表の行に載せてよい。」にする。床はこの節を突き合わせない＝判定は変わらない。
4. **設計文書と fixture。** `design-intent/intake.yaml` と `design-intent/rules.yaml` の生成区間は便の binary の `folio schema --dir design-intent --write` で書き直す（区間の中の +4 行と 2 行の置き換えだけ・区間の外は 1 byte も変えない）。凍結 anchor `tests/fixtures/schema/intake-region.txt` と `rules-region.txt` は導出を使わない script（控えの anchor-197.py・anchor-197-rules.py）で組み、`tests/schema_docs.rs` の F77_REGIONS の intake の組と RULES_REGION_BYTES / RULES_REGION_SHA256 を写す。
5. **変えないもの。** 床（folio check）の判定・相談窓口の正本の人が書く節・支度表の形と凍結 anchor・folio2 自身の intake.yaml での `folio intake` の出力（`--print` と `--write` の 7 case・支度表 19 組が base と byte で同じ）・規則の表の行・folio2 の床 4 本の答え・`folio build` の出力。

### (c) 歯（f197_・base で 0 件・どれも `tests/sheet.rs`）

1. **f197_answer_words_come_from_the_values_in_the_file。** 一時の写しの values を [yes, no]（引用符付き）・推奨を yes にし、回答 q1 = no で `--write` が 0・持つ文書 4・はい / いいえ の字が無い。**base では「はい でも いいえ でもない」で断る＝RED。**
2. **f197_values_other_than_two_make_no_sheet。** values が 3 つ・1 つ × 支度表なし / 既に在る の 4 case で `--write` が 2・標準出力は空・支度表は出来ない / byte で変わらない。**base では書いて 0＝RED。**
3. **f197_real_region_copies_the_answer_rule。** 実の intake.yaml の生成区間の answers と targets の値と注の字、人が書く欄と食い違わないこと（values が 2 つ・default = recommend・各質問に yes と no）。**base では区間に無い＝RED。**
4. **f197_no_source_file_spells_the_answer_words。** src の各 file の最初の `#[cfg(test)]` より前に引用符付きの はい / いいえ が無い。**base では sheet.rs の 2 つ＝RED。**
5. **f197_rules_excluded_names_only_human_time_and_ai_cost。** 実の rules.yaml の生成区間の excluded.what がちょうど 2 語で、why が機械の待ち時間を除外に入れない字を持つ。**base では旧い字＝RED。**
6. **f197_only_intake_spells_the_copied_values。** src の各 file の最初の `#[cfg(test)]` より前で、行き先の固定の値 inject と回答の欄の一覧 [yes, no] の引用符付きの字は `intake.rs` の定数の宣言に 1 回ずつだけ在り（空白を除いて数える）、床の木 INTAKE_FLOOR の 4 欄は定数の名を引く（recommend は質問の行の欄の名と同じ字なので数えず、名を引くことを見る）。**base では INTAKE_FLOOR に 4 欄が無い＝RED。**
- 一時の写しは既存の歯の口（一時 dir に case と process id・Drop で消す）に揃え、固定の /tmp の名を使わない。歯だけを base に当てると、落ちるのは 1〜6 と、anchor と区間の数が変わる既存の歯 8 本（F77・F85・rules の区間の歯・単体 rules_floor_derives_the_frozen_anchor_byte_for_byte）だけ。

### (d) 採らなかった形

1. **values の言葉を実装の定数のまま、床に「はい と いいえ の 2 つ」を課す。** 手書きの写しと一致の検査の組になる（P-6.4）。ADR-11 決定 (3)(ア) の向きの逆。
2. **`--print` でも値の数で断る。** 判定を 1 つ増やす（base は 0）。`--print` は行き先を出さないので、断りは書く口に置いた。
3. **床に values の数を数えさせる。** 新しい判定（違反）を足す＝本便の範囲（判定を変えない）の外。要るなら床の穴の束で起こす。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は無い**（anchor と区間の数の値は (b) の 4 のとおり同じ便で直す）。便の全部を当てた写しで workspace の nextest 1076 本（+6 = f197_・8a0abe9 の全体の run が 1075 / 1075 で、積んだ歯 1 本は verify の行で緑）・clippy 0 警告・床 4 本 rc 0・`folio build` 37 file は base と byte で同じ。
2. **突然変異 17 通り・生き残り 0**（見本の src だけを 1 通りずつ変え、verify の歯の束を撃った・base 44c14ac の見本 8a0abe9 で撃ち直し・落ちる歯の数は前の base と同じ）。検証役の変異のうち V3（INTAKE_FLOOR が inject を字で持つ）は (c) の 6 が落とす（92972b1 で実測）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 値の数の検査を「多すぎる」だけに / M2 値の数の検査を外す / M10 断りの字から値の数を落とす / M13 3 つ以上なら断らない | 2 |
| M3 値の順と欄の順を逆に対にする / M4 欄を逆の順に読む | 1・支度表の anchor・引き継ぎ |
| M5 ANSWER_BRANCHES を [no, yes] | 1・anchor・区間の数 |
| M6 写しの values_count を 3 / M7 注から「この節はその写しである」を落とす / M8 targets.fixed を空に / M12 default_fixed を別の字に / M14 写しの branches を手書き | 3・区間の数と anchor |
| M9 真偽に倒れたキー（引用符なしの yes / no）を引かない | sheet の既存の歯 8 本・1・2 |
| M11 実装に const YES = はい を戻す | 4 |
| M15 除外の what を元の字に / M16 why を元の字に / M17 why から機械の待ち時間の文だけ落とす | 5・rules の区間の数と anchor・単体 rules_floor_derives |

### (f) 大きさ・余地・verify と done

1. **write-set 9 本**（印なし）: src 3（sheet・intake・rules）・歯の file 2（`tests/sheet.rs`・`tests/schema_docs.rs`）・設計文書 2（intake.yaml と rules.yaml の生成区間）・fixture 2（intake-region.txt・rules-region.txt）。
2. **余地（参考値）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: |
| sheet.rs | 577 | 598 | 902 |
| intake.rs | 256 | 290 | 1210 |
| rules.rs | 473 | 475 | 1025 |

3. **size は S**（最小の余地は sheet.rs の 902 ≥ 100・src の差は小さい）。
4. **verify は 5 行**で、done の塊と 1 対 1: `--test sheet f197_`・`--test sheet`（支度表の既存の歯と凍結 anchor）・`--test schema_docs`（2 区間の anchor と数・歯の file の上限）・`--bin folio sheet_resolve sheet_map rules_floor_`（sheet.rs の単体の歯 2 本と rules.rs の床の木の単体の歯 3 本）・clippy。base では f197_ の行が 0 件で終了コード 4、ほかは緑。

### (g) 受付・並行の便

- **重なり**（本流 44c14ac の上の見本どうしの `git merge-tree` の実測・2026-09-28 20:0x・読むだけ）: 便 187（床の穴・先に着地・impl/d187 430c9c7）・便 190（検査の信頼・持ち主の承認待ち・83f049d）・便 198（編集時の止め・後・15b235a）とは write-set が重ならない。便 194 は base に入った（両方が書いた `intake.rs` は merge で衝突なし・194 の歯 f194_ と本便の歯が同じ木で緑）。便 162（未着地・枝 docs/d162 336b11f・base 8472b5c）の契約の write-set とは `rules.rs`・`rules.yaml`・`rules-region.txt`・`tests/schema_docs.rs` が重なり、RULES_REGION_* の 2 行は両方が書く＝後に着地する側が数え直す。
- **同じレーン**: 便 195・196 とは `tests/schema_docs.rs` の頭の来歴の行が隣り合う所だけが当たる（後に着地する側が両方の行を残す）。3 本を本流 44c14ac の上に 195 → 196 → 197 の順で重ねた写し（控え copies-scripts/stack44/・commit を作らない git apply -3）では、字の衝突はその頭の注の 2 塊だけ（両方の行を残して解いた）で、9 本の生成区間は重ねたまま schema --check が一致し（--write は何も書き直さない）、workspace の nextest 1083 / 1083（base 1070 + 5 + 3 + 5）・clippy 0 警告・床 4 本 rc 0・`folio build` は本流と byte で同じ・3 本の verify 17 行が全部 rc 0、`tests/schema_docs.rs` は器の式で 1177 行（上限 1200）・`tests/schema.rs` は 700 行（上限 700）。着地の順は 187 → 190 → 195 → 196 → 197 を既定とし、190 は持ち主の承認待ちなので 195〜197 が先に着地してもよい（187 → 190 → 195 → 196 → 197 と 187 → 195 → 196 → 197 → 190 の 2 つの順で 44c14ac の上に重ねた写しは、最後の木が byte で同じで、workspace の nextest 1107 / 1107・clippy 0 警告・床 4 本 rc 0・歯 f89 が緑）。
- **受付の時点の main が 44c14ac と違えば**、2 区間・anchor 2 本・区間の数・余地を数え直してから運ぶ（便 162 が先に着地したら RULES_REGION_* と rules-region.txt は必ず）。

### (h) 今の置き場の床が変わらないこと

1. **folio2 自身。** 床 4 本の答えは同じ（schema --check の intake.yaml と rules.yaml の byte 数の 2 行だけが違う）。`folio build` の出力は byte で同じ。`folio intake` の出力と支度表は base と byte で同じ。
2. **tsuzuri**（写しへ cp・本物では何も撃たない・写し 59af37e・values は [はい, いいえ]・本流 44c14ac の binary と比べた）。便の binary の `folio schema --write` が本流の binary の書き直し（写しが本流より古い分の adr/schema.yaml と design-note/schema.yaml）のほかに書き直すのは intake.yaml と rules.yaml の 2 本だけで、その後の check・schema --check・derive・`folio intake --print` / `--write` は 0、出力と支度表は本流の binary の答えと byte で同じ。

### (i) 連絡・運ばないもの・撤退条件

1. **利用者への連絡**（便 195 の (i) の 1 と同じ 1 回）: 本便の分は intake.yaml と rules.yaml。folio intake は answers.values の 1 つ目を yes の行き先・2 つ目を no の行き先の回答として読み、values が 2 つでなければ `--write` が断る（values の順が意味を持つ）。
2. **運ばないもの。** intake.yaml の区間の外の注釈（「決まりの部分（生成区間）。最上位の節の閉じた一覧。」＝区間が読み方も持つ後は言い足りない・席の一括）・土台の写し `tests/fixtures/floor_base/design-intent/rules.yaml` の旧い除外の字（床は突き合わせない入力の写し）・行 R-20（asks のレーンが本便の着地の後に足す）。
3. **撤退条件。** (1) (b) の 4 の値の直しのほかで既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で、folio2 の床 4 本か `folio build` の出力か `folio intake` の出力が (h) の 1 の差のほかで着地の直前と違えば止めて席へ返す。

## 2. 範囲

- 入れる: §1 (b) の 1〜4 と (c) の歯 6 本と anchor・区間の数の直し。
- 入れない: §1 (b) の 5・(i) の 2・命令の旗・床の新しい判定・新しい file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| U1 | 回答の値の読み | sheet.rs が answers.values を位置で読む・値の数の断り |
| U2 | 回答の読み方と固定の値の写し | intake.rs の ANSWER_BRANCHES と INTAKE_FLOOR の answers・targets と注 |
| U3 | 除外の字 | rules.rs の EXCLUDED_WHAT と excluded.why |
| U4 | 写しの data | 2 区間・anchor 2 本・区間の数の値 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は便 185（base に入る）。受付の前提は .227 の持ち主の承認（2026-09-28 の束・C）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .74 と .227 を閉じ、行 R-20 の便へ知らせる。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gr"
title = "台帳 f2-648.74 と f2-648.227: 支度表の命令 sheet.rs が回答の値 はい / いいえ を定数で持つのをやめ、相談窓口の正本 intake.yaml の answers.values を読んで 1 つ目を yes・2 つ目を no の行き先の回答とし、values が 2 つでなければ --write が断る（判断の記録 ADR-11 決定 (3)(ア)(4)⑥）。その読み方と固定の値 inject・recommend を intake.rs の床の木 INTAKE_FLOOR から相談窓口の正本の生成区間へ写し（行 D-11）、規則の表の欄の決まりの除外 EXCLUDED_WHAT と why を人の作業の時間と AI の費用に狭める。2 file は folio schema --write で書く。folio2 と tsuzuri の答えは変えない。歯は f197_ 6 本。base = main 44c14ac"
req = ["FR1", "FR19"]
section = "1"
write-set = ["crates/folio/src/sheet.rs", "crates/folio/src/intake.rs", "crates/folio/src/rules.rs", "crates/folio/tests/sheet.rs", "crates/folio/tests/schema_docs.rs", "design-intent/intake.yaml", "design-intent/rules.yaml", "tests/fixtures/schema/intake-region.txt", "tests/fixtures/schema/rules-region.txt"]
verify = ["cargo nextest run -p folio --test sheet f197_", "cargo nextest run -p folio --test sheet", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --bin folio sheet_resolve sheet_map rules_floor_", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/sheet.rs の f197_ の歯 6 本が緑、tests/sheet.rs の歯の全部（支度表の凍結 anchor を含む）が緑、tests/schema_docs.rs の歯の全部（相談窓口の正本と規則の表の生成区間の凍結 anchor・行数・byte 数・要約値と歯の file の上限）が緑、sheet.rs の単体の歯 2 本と rules.rs の床の木の単体の歯 3 本が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が 9 file とも一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と byte で同じ"
<!-- contracts:end -->
