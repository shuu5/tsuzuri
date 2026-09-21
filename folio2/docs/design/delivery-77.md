# 設計: 便 77 — 要件書・語彙・入口・相談窓口の最上位の節の閉じた一覧と、要件書の図の節の行の欄を、各 file の生成区間へ（ADR-11 決定 (4)⑤・FR19 / FR5）

- 要件: FR19（決まりの部分を床の定数から決定的に導出する・対象に 4 file を足す）/ FR5（床の 合格・不合格・まだ分からない）
- 条: P-5.1・P-5.6（実装の型付きの定数を正本とするあいだ、写しを設計文書の置き場へ導出する）/ P-6.2・P-6.3・P-6.4 / P-10.1（凍結 anchor）/ N-3.1 / P-4.2
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)「検査される file の側から緩められてはならない形の決まり（最上位の節の閉じた一覧・必須の欄の集合・値域・閉じた id の集合）で、その file に憲法と同じ強さの変更の門が無いものは、実装の型付きの定数が正本で、その file の中の生成区間に写しを導出する」と決定 (4)⑤「要件書・語彙・入口・相談窓口の最上位の節の閉じた一覧と図の欄の集合＝各 file の生成区間へ」。rules 行 D-11 の作法 = 本便の根拠は決定 (4)⑤。値は 1 つも変えない。
- 材料: 下調べ docs/design/adr-11-survey.md §2-a（最上位の節の閉じた一覧 6 本のうち 4 本が本便の対象・天井の正本は便 47 / 48 で済み・所見 file は生成物の置き場なので対象外）と §2-b（要件書の図の節の行の欄 = SRS_FIGURE_REQUIRED と SRS_FIGURE_OPTIONAL）。
- 位置: 着地済みの同型の便 47 / 48（天井の正本）・51 / 53（規則の表）と同じ型。**便 76（入口の棚の閉じた id の集合・ADR-11 決定 (4)④）の後**に受け付ける（§1 (g)）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bz が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は + を付けた 3 本）。
- 門: 本便は design-intent の下の正本 4 file を書き換える＝天井の門（folio ceiling --gate）が印を読む。受付の時点の印は 15 周目の 合格（§1 (h)）。

## 1. 目的と中身

要件書 design-intent/srs.yaml・語彙 vocabulary.yaml・入口 index.yaml・相談窓口 intake.yaml の 4 正本は、最上位に置ける節の閉じた一覧を file の側に一覧として持たず、床（crates/folio/src/check.rs の check_srs と check_vocabulary・entrance.rs の check_entrance・intake.rs の check_intake）の中の型付きの定数だけが持つ。要件書の図の節の行の欄（必須 4 つ・任意 2 つ）も同じで、写しがどこにも無い（下調べ §2-a・§2-b の類 B）。憲法 P-5.6 は、実装の型付きの定数を規則の正本にしているあいだ、その写しを人が読める型付きデータとして設計文書の置き場へ決定的に導出することを義務にする。本便はその写しを、着地済みの天井の正本・規則の表と同じ機構（印 2 本で挟んだ生成区間・命令 folio schema が床の木から導出して書く / 検査する）で 4 file に置く。値・床の判定・面の生成器・要件書の規範文は 1 つも変えない。

orchestrator 席の実測（2026-09-21・main 26fe980）: 対象の定数は crates/folio/src/check.rs の 42 行 SRS_TOP_LEVEL（16 語）・62 行 SRS_FIGURE_REQUIRED（4 語）・63 行 SRS_FIGURE_OPTIONAL（2 語）・66 行 VOCABULARY_TOP_LEVEL（3 語）、entrance.rs の 18 行 INDEX_TOP_LEVEL（5 語）、intake.rs の 18 行 INTAKE_TOP_LEVEL（5 語）。読み手は自分の module の unknown_sections の呼び出し（check.rs 503・534 行／entrance.rs 22 行／intake.rs 31 行）と check.rs 575・576・581 行の図の欄の判定だけで、module の外に読み手は無い。命令の側は schema.rs の 30 行 TARGETS（着地済みは 4 本・関数 run が順に見て標準出力へ 1 行ずつ足し、最初に合格でない file でそこで返す）と、印を探す関数 region_of（印は file に 1 対・位置は問わない）と、床の木を字面にする関数 derive（体裁の規則は schema.rs の先頭の注釈・1 行の幅 100 字）。生成区間を持つ file は、床の側では「schema という名の節を最上位に置いてよい」だけで足り、節が無くても床は黙る（規則の表と同じ扱い・写しの一致は folio schema が見る）。

### (a) 床の木 4 本と、閉じた一覧に schema を足す

置き場は定数と同じ module（同じ一覧を 2 回書かず、木の葉は既存の定数を指す）。型は schema.rs の Floor。

1. check.rs に pub(crate) の SRS_FLOOR と VOCABULARY_FLOOR。
2. entrance.rs の床の木（便 76 が置く FLOOR）の**先頭に** top_level と top_level_note の 2 欄を足す（便 76 の欄はその後ろに残す・§1 (g)）。
3. intake.rs に pub(crate) の INTAKE_FLOOR。
4. SRS_TOP_LEVEL を 17 語（末尾に schema）・VOCABULARY_TOP_LEVEL を 4 語（末尾に schema）・INTAKE_TOP_LEVEL を 6 語（末尾に schema）にする。INDEX_TOP_LEVEL は便 76 が 6 語にしている。配列の長さの注釈（[&str; 16] など）も直す。
5. 木の葉は既存の定数を指す（Floor::Strs(&SRS_TOP_LEVEL) の形）。図の欄は Floor::Map の 2 欄 required と optional で、値は SRS_FIGURE_REQUIRED と SRS_FIGURE_OPTIONAL を指す。
6. 床の判定は変えない。schema の節が無くても違反にしない（天井の正本のように必須にはしない＝写しの fixture を 1 本も直さないため）。schema の節の中身を床が床の木と突き合わせることもしない（それは folio schema --check の仕事）。

### (b) 凍結 anchor 4 本（P-10.1）

anchor は orchestrator 席が Rust の実装とは独立に組んだ（Python で derive の体裁の規則を写し、着地済みの tests/fixtures/schema/rules-region.txt と byte 一致することで写しの正しさを確かめた・2026-09-21）。**作業者は anchor を生成物から作らない。**下の逐語をそのまま file に置き、生成物が合わなければ実装の側を直す（anchor を生成物に合わせない・合わせられないと判断したら席へ問う）。

1. tests/fixtures/schema/srs-region.txt（新規）= 23 行・777 byte・sha256 879ab87dc75a6c2545868a33cb96540cf679b0e6499222daa653f720da6563ef

```
schema:
  top_level:
    - meta
    - goals
    - scope
    - scope_m1
    - actors
    - outputs
    - rail
    - verdicts
    - requirements
    - nonfunctional
    - acceptance
    - not_frozen
    - constraints
    - sources
    - glossary_pointer
    - figures
    - schema
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。schema のほかの節は人が書き、schema は生成区間。scope_m1 は名を空けてある節で、今の正本には無い
  figures:
    entry: {required: [id, type, caption, spec], optional: [refs, note]}
  figures_note: 要件書の図の節の行の欄（判断の記録と設計ノートの欄の決まりの figures.entry と同じ形）。型（type）の値域は部品目録が持つ
```

2. tests/fixtures/schema/vocabulary-region.txt（新規）= 3 行・238 byte・sha256 2746b140abdf5bfafa2b3b907b2bce91c9ee21e3af5488ba09420006f0170161

```
schema:
  top_level: [terms, field_terms, identifiers, schema]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。terms と field_terms と identifiers は人が書き、schema は生成区間
```

3. tests/fixtures/schema/intake-region.txt（新規）= 3 行・262 byte・sha256 38fd3ff44e9aa6a398344d3385cb4281c77daf62d3839410e5da3814183aee8b

```
schema:
  top_level: [meta, answers, targets, questions, sheet, schema]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。meta と answers と targets と questions と sheet は人が書き、schema は生成区間
```

4. tests/fixtures/schema/index-region.txt（便 76 が置いた file を直す）= 1 行目の schema: の**直後に**次の 2 行を逐語で差し込む（+2 行・+246 byte）。ほかの行は 1 byte も変えない。

```
  top_level: [meta, audience, shelf, lanes, intake, schema]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。meta と audience と shelf と lanes と intake は人が書き、schema は生成区間
```

体裁は derive の規則どおりで、要件書の一覧だけ flow の 1 行が 100 字を超えるので block になる（17 項）。注の欄（_note で終わる欄）は床の突き合わせの外で、命令の導出だけが書く。

### (c) 実の 4 file の生成区間

4 file の**末尾**（最後の節の後ろに空行 1 つ）に、人が書く注釈 1 行 + 印 begin + 生成区間 + 印 end を置く。印の字面は既に在る 4 file と 1 字も違わない（schema.rs の定数 BEGIN と END）。注釈の字面は file ごとに次のとおり（下の 〈module〉 の所に入れる名 = 要件書と語彙は crates/folio/src/check.rs・入口は crates/folio/src/entrance.rs・相談窓口は crates/folio/src/intake.rs）。

`# 決まりの部分（生成区間）。最上位の節の閉じた一覧〔と図の節の行の欄〕。正本は実装の型付きの定数（〈module〉）で、ここはその人が読む写し。手で直さず folio schema --write が書く（判断の記録 ADR-11 決定 (3)(イ)・(4)⑤・憲法 P-5.6）。ほかの節は人が書く`

〔と図の節の行の欄〕は要件書の行だけに入れる。中身は印を置いた後に `cargo run -p folio -- schema --write --dir design-intent` で書く（人が打たない）。区間の外の byte は 1 つも変わらない（meta の版・承認欄・人が書く節・先頭の注釈は不変＝本便は 4 file の版を上げない・§1 (i)）。入口 index.yaml は便 76 が置いた区間がそのまま 2 行増える。

### (d) 命令 folio schema の対象に 3 本足す

1. schema.rs の TARGETS に 3 行足す。順は 6 本目 srs.yaml（check.rs の SRS_FLOOR）・7 本目 vocabulary.yaml（check.rs の VOCABULARY_FLOOR）・8 本目 intake.yaml（intake.rs の INTAKE_FLOOR）。5 本目 index.yaml は便 76 が置く。関数 run・run_one・derive・region_of は変えない。
2. main.rs の Schema の --dir の説明の字面を、8 本の file を名指す形に直す: `設計文書の置き場（生成区間を持つ file 8 本 = adr/schema.yaml・design-note/schema.yaml・ceiling.yaml・rules.yaml・index.yaml・srs.yaml・vocabulary.yaml・intake.yaml を読む）`。
3. これに伴い crates/folio/tests/check.rs の歯 r11_schema_help_names_the_four_schema_files の名を r11_schema_help_names_the_schema_files にし（名に数を持たせない）、期待の字面を 2 の 8 本の列にする。判定の向きは変えない。
4. 命令は 1 本も増やさない（tests/check.rs の p1_commands_closed_list が凍結する 11 本は不変）。新しい旗も足さない。

### (e) 歯（関数名は f77_ で始める・`grep -rn 'fn f77_' crates/folio/tests` は今 0 本＝席が 2026-09-21 に実測）

crates/folio/tests/schema.rs（実の設計文書の置き場の写し = 一時 dir・git の 1 commit に命令を当てる既存の作り方をそのまま使う）。

1. f77_check_covers_the_four_files: 写しに --check → 0 ∧ 標準出力が 8 行 ∧ 5〜8 行目が順に index.yaml・srs.yaml・vocabulary.yaml・intake.yaml を含む ∧ srs.yaml の行が 777 byte・vocabulary.yaml の行が 238 byte・intake.yaml の行が 262 byte を含む ∧ 4 行とも byte 数が対応する anchor file の byte 長と同じ（入口の byte 数は便 76 が決めるので anchor から取る）。
2. f77_regions_match_the_frozen_anchors: 写しの 4 file の生成区間（印の間）が (b) の 4 本の anchor と byte 一致 ∧ anchor 自身の自己検査 = srs-region.txt が 23 行・777 byte・sha256 879ab87d…、vocabulary-region.txt が 3 行・238 byte・sha256 2746b140…、intake-region.txt が 3 行・262 byte・sha256 38fd3ff4…、index-region.txt が (b)4 の 2 行を逐語で 2 行目と 3 行目に持つ。要約値は sha256 を測る既存の口で測り、測れなければ歯を落とす（まだ分からないで素通りさせない）。
3. f77_drift_in_each_region_fails: 4 file それぞれについて、写しの生成区間の 1 byte を書き換えて --check → 1 ∧ 標準エラーにその file 名と「≠ 導出」。
4. f77_missing_marker_in_each_file_is_unknown: 4 file それぞれについて、begin の印の行を消して --check → 2 ∧「印が 1 対でない」とその file 名。
5. f77_write_restores_the_four_regions: 3 でずらした写しに --write → 0 ∧ 4 file 全体が元と byte 一致（人が書く節・meta・承認欄・先頭と末尾の注釈も不変）∧ もう 1 度 --write → 0 ∧ 8 行とも「変わらない」。

crates/folio/tests/check.rs。

6. f77_top_level_is_closed_on_the_four_files: (c) を当てた実の置き場に folio check → 0 ∧ 種別「未知の節」の違反が 0 件（本便の前の main では、生成区間を置いた時点で 4 件の未知の節が出て落ちる＝赤い歯）。続けて、写しの 4 file それぞれの最上位に節 extras を足す → 不合格 ∧ 未知の節がちょうど 1 件でその file 名と extras を含む（閉じた一覧は定数が持ち、file の側の schema.top_level を書き換えても緩められない・N-3.1）。

7. 回帰（期待不変・verify の 2 行目）: tests/schema.rs（着地済み 4 file の一致・要約値・行数の歯を含む）・tests/check.rs（check_canonical_design_intent_passes・check_srs_figure_* の 5 本・check_rules_* の 2 本・p1_commands_closed_list・r11_parts_help_*）・tests/entrance.rs・tests/intake.rs の既存の歯すべて。共通の検証（.vessel.toml の common-verify）が workspace 全部を回すので、面と束と凍結の場合 134 件もそこで見る。

### (f) 大きさと接続

新規 file は 3 本（anchor・§1 (b)）。src の実測（正規化 = 空行を除き幅 120 で折る・上限 1500・main 26fe980＝便 76 の前）と見積: check.rs 625 行・余地 875（+約 25 行）／entrance.rs 239 行・余地 1261（+約 6 行）／intake.rs 214 行・余地 1286（+約 12 行）／schema.rs 438 行・余地 1062（+約 6 行）／main.rs 534 行・余地 966（+1 行）。歯の側は tests/schema.rs 826 行（+約 130 行）・tests/check.rs 637 行（+約 45 行）。size **S**（src の各 file の余地はどれも 100 以上・src の足し合わせは 50 行に届かない）。便 76 が entrance.rs と schema.rs と main.rs に足す分（20 行に満たない見込み）を引いても余地は動かない。外部 crate は増やさない。部品目録・様式・面の生成器・design-intent/preview の下・tests/fixtures の写し（floor_base の 24 file を含む）・tests/floor_cases.yaml・CI の yml は触らない（写しは schema の節を持たないままで床を通る＝直す差分が無い）。

### (g) 便 76 との重なり（前提）

入口 index.yaml・crates/folio/src/entrance.rs・schema.rs の TARGETS・main.rs の説明の字面・tests/fixtures/schema/index-region.txt の 5 か所が便 76（ADR-11 決定 (4)④）と重なる。**本便は便 76 の着地の後に受け付ける。**前提は 2 つで、どちらも受付の前に席が実測して確かめる: ① 便 76 が index.yaml に生成区間を置き、INDEX_TOP_LEVEL に schema を足し、TARGETS の 5 本目に index.yaml を置き、anchor を tests/fixtures/schema/index-region.txt に置いていること。② その anchor の 1 行目が schema: であること（(b)4 の差し込みの位置）。前提が崩れていたら、席が受付の前に §1 (b)4 と (d) の順と字面を直す（作業者は判断しない）。便 76 が入口の最上位の節の一覧まで運んでいたら、本便からは入口の分（(a)2・(b)4・(d) の 5 本目・歯の入口の場合）を外す。

### (h) 天井の門と印の運用

受付の時点の印 design-intent/preview/ceiling-stamp.yaml は 15 周目・4 観点とも 合格・止める 0（main 6de202c で導出・2026-09-21）で、正本の要約値は今の main と同じなので、門は 通す を返す。本便は設計文書の正本 4 file を書き換えるので、着地した時点で印の sources の要約値が変わり、印は 古い になる（次に設計文書を書き換える便の門は まだ分からない を返す）。付け直しは次の天井の周の後に folio ceiling --stamp で導出し直す（便 74 の後の付け直しと同じ手順・印は生成物なので手で直さない）。本便は印そのものを書き換えない。

### (i) 本便が運ばないもの（席が別に持ち主へ問う）

要件書 FR19 の規範文と受入基準 AC17 は、対象を「判断の記録と設計ノートの欄の決まりの file と天井の正本と規則の表」と名指す。本便で対象が 8 本になるので、FR19 と AC17 の対象に 入口・要件書・語彙・相談窓口 を足す版上げが要る（天井の正本を足した v1.9・規則の表を足した v1.10 と同じ形）。これは要件書の規範の側の変更で、持ち主の承認を要するので本便では運ばない（write-set に入れるのは srs.yaml の生成区間の分だけ）。**席が 🔴 で持ち主へ問い、その版が発効してから本便を受け付ける**（v1.9 → 便 48・v1.10 → 便 53 と同じ順）。便 76 の分（入口）と 1 回の承認にまとめるのが推奨。4 正本の meta.version と承認欄を動かすかどうかも同じ問いに含める（本便の既定は動かさない＝生成区間は機械が書く写しで、人が書いた中身は 1 byte も変わらないため）。

## 2. 範囲

- 入れる: 床の木 4 本（うち入口は 2 欄の追加）・閉じた一覧 3 本に schema を足すこと・実の 4 file の生成区間と注釈 1 行・凍結 anchor 4 本・命令の対象 3 本・説明の字面・歯 6 本。
- 入れない: 値と床の判定の変更・schema の節を必須にすること・床が生成区間の中身を床の木と突き合わせること・要件書 FR19 と AC17 の版上げ（§1 (i)）・4 正本の版と承認欄・下調べ §2-b の残りの定数（参照 id の形と節の一覧・配信の接続先の判定の式・実装にだけ在る閾値 = ADR-11 決定 (4)⑦⑧ の別の便）・所見 file の欄の一覧（生成物の置き場なので対象外）・面の生成器・部品目録・様式・写しの fixture。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の木 | check.rs の SRS_FLOOR と VOCABULARY_FLOOR・entrance.rs の 2 欄・intake.rs の INTAKE_FLOOR |
| closed | 閉じた一覧 | SRS_TOP_LEVEL・VOCABULARY_TOP_LEVEL・INTAKE_TOP_LEVEL に schema を足す |
| anchor | 凍結 | tests/fixtures/schema の 4 本（新規 3・入口は 2 行の差し込み） |
| region | 生成区間 | 実の 4 正本の末尾（注釈 1 行 + 印 2 本 + 導出） |
| target | 対象 | schema.rs の TARGETS に 3 行・main.rs の説明の字面 |
| teeth | 歯 | tests/schema.rs に 5 本・tests/check.rs に 1 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bz"
title = "要件書・語彙・入口・相談窓口の最上位の節の閉じた一覧と、要件書の図の節の行の欄を、各 file の生成区間へ導出する（床の木 4 本・閉じた一覧に schema・命令 folio schema の対象を 8 本に・実の 4 正本は凍結 anchor と byte 一致・命令は 1 本も増やさない・値と床の判定と要件書の規範文は不変・ADR-11 決定 (4)⑤・便 76 の後）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/src/entrance.rs", "crates/folio/src/intake.rs", "crates/folio/src/schema.rs", "crates/folio/src/main.rs", "design-intent/srs.yaml", "design-intent/vocabulary.yaml", "design-intent/index.yaml", "design-intent/intake.yaml", "+tests/fixtures/schema/srs-region.txt", "+tests/fixtures/schema/vocabulary-region.txt", "+tests/fixtures/schema/intake-region.txt", "tests/fixtures/schema/index-region.txt", "crates/folio/tests/schema.rs", "crates/folio/tests/check.rs", "crates/folio/tests/entrance.rs", "crates/folio/tests/intake.rs"]
verify = ["cargo nextest run -p folio --test schema --test check f77_", "cargo nextest run -p folio --test schema --test check --test entrance --test intake", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f77_ の歯 6 本（8 行の一致と byte 数・4 本の凍結 anchor との byte 一致と anchor の自己検査・4 file のずれ・4 file の印・書き直しと冪等・未知の節が 0 件で file の側から緩められない）が緑、tests/schema.rs・tests/check.rs・tests/entrance.rs・tests/intake.rs の既存の歯が全部緑（実の設計文書の置き場に folio check を当てる歯と、命令 11 本の閉じた一覧の歯を含む）、folio schema --check が 8 行とも一致、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
