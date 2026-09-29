# 設計: 便 75 — 床の穴: 要件の行（requirements / nonfunctional）の必須欄を folio check が数える（台帳 f2-648.67・FR5 / FR4）

- 要件: FR5（検査結果を必ず返す = 構造の床）/ FR4（3 枚を 1 つの生成器で出す = 面の生成器が読む欄）/ GOAL4（壊れにくい道具）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1（検査できなかった結果を異常なしとして扱わない）/ P-5.1（規則は型付きデータに置く）/ P-10.1（凍結 anchor を持つ）/ N-2.1・N-2.2（散文にしか無い規則を持ち込まない）
- 出所: 台帳 f2-648.67（2026-09-19 の実測）。要件書 v1.7 の起草中に、要件の行に figures 欄が無いまま folio check が 合格（違反 0）を返し、面の生成器だけが まだ分からない で倒れて面と束を読む歯 19 本が連鎖で落ちた。床が要件の行の必須欄を数えていない（憲法の条と判断の記録の欄は数えている）。
- 根拠の判断: 新しい判断は要らない。必須欄の一覧は面の生成器 face_srs.rs が実際に読む欄から導くだけで、床が面より厳しくも緩くもならないようにする。置き場は床の定数（要件書は欄の決まりの節を持たないので、便 34 が置いた図の行の定数 SRS_FIGURE_REQUIRED / SRS_FIGURE_OPTIONAL と同じ形）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bx が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規 file は無い）。
- 門: 本便は design-intent の下の file を 1 つも書き換えないので、天井の門（folio ceiling --gate）は 通す（設計文書の正本を書き換えない便）。

## 1. 目的と中身

(a) 今の床が数えている欄（実測）。crates/folio/src/check.rs の check_srs（532 行〜）は、要件書の節ごとに欄の非空を数える。548 行が requirements と nonfunctional の欄を id・title・shall・plain の 4 つだけに定めている（verdicts は id・name・tone・cond、ほかの節は id・title）。要件書の最上位の節の閉じた一覧は床の定数 SRS_TOP_LEVEL（41 行〜・16 節）、図の節の行の欄は SRS_FIGURE_REQUIRED（59 行・4 欄）と SRS_FIGURE_OPTIONAL（60 行・2 欄）で、行の欄の集合を閉じて数える形は check_srs_figure（571 行〜）が既に持つ。要件の行にはこの形が無い。

(b) 面の生成器が要件の行に要求する欄（実測）。crates/folio/src/face_srs.rs の items（288〜300 行）が id と title を、item_row（967〜1068 行・requirements と nonfunctional の両方がこの 1 本を通る。fr_chapter の 925 行と nfr_chapter の 961 行）が次を読む。f は必須の欄（無ければ Err・face.rs の 186 行）、g は任意の欄（face.rs の 195 行）。

- 必須・字の欄: pattern（971 行）・strength（973 行）・when（990 行）・shall（991 行）・plain（999 行）
- 必須・一覧の欄（seq を求める＝空の一覧は通る）: goals（1023 行）・basis（1027 行）・figures（1043 行）
- 必須・表の欄: verify（1002 行）。その中の method（1003 行）・how（1007 行）は字、ac（1011 行）は一覧
- 任意の欄: milestone（975 行・g）・rules（1031 行・g）・note（1039 行・g）

(c) 憲法と判断の記録が同じことをしている場所（実測）。憲法は check.rs の check_constitution（323 行〜）が条の欄を non_empty（333 行の id・title・statements、345 行の規範文の id・text）で数え、値域を持つ欄は check_article_enums（378 行〜・便 55）が数える。判断の記録は crates/folio/src/adr.rs の床の定数 RECORD（33 行・required 10 欄 / optional 8 欄）と OPTION・RETREAT・GRILL・APPROVAL（60〜78 行）で欄の集合を閉じて持ち、その写しが design-intent/adr/schema.yaml の生成区間（folio schema --write の導出物）に出る。要件書は欄の決まりの節も生成区間も持たないので、本便は床の定数に置くだけで design-intent を 1 file も触らない。

(d) 床に足す決まり。check.rs に定数 4 本と関数 2 本を足し、check_srs の節ごとの分岐（546〜556 行）で requirements と nonfunctional だけ新しい関数へ渡す。値は (b) の実測から導き、床は面より厳しくも緩くもしない（空の一覧は落とさない）。

- 定数 SRS_ITEM_TEXT = id・title・pattern・strength・when・shall・plain（7 欄・非空）
- 定数 SRS_ITEM_LIST = goals・basis・figures（3 欄・在ること。空の一覧は通る）
- 定数 SRS_ITEM_VERIFY_TEXT = method・how（2 欄・非空）。verify の ac は一覧（空でよい）
- 定数 SRS_ITEM_OPTIONAL = milestone・rules・note（3 欄）
- 関数 check_srs_item(section, row, report): 欄の集合が閉じた一覧（上の 4 本 + verify）に在るかを数え（種別 未知の欄・字面は check_srs_figure の 578 行と同じ形で srs.yaml: requirements の FR1 の未知の欄「extra」）、SRS_ITEM_TEXT を non_empty（種別 欄の非空・字面は今のまま srs.yaml: requirements の FR1 の pattern が空）、SRS_ITEM_LIST を (e) の関数、verify は表であることを見てから中の 2 欄を non_empty（場所は requirements の FR1 の verify）と ac を (e) の関数。verify が無い・表でないときは違反 1 件（種別 schema・srs.yaml: requirements の FR1 の verify が無い（表））を出して中は見ない＝1 つの欠けが 2 件に膨らまない。
- 関数 seq_field(file, at, row, key, report): 欄が一覧でなければ違反 1 件（種別 schema・srs.yaml: requirements の FR1 の figures が無い（一覧・空でよい））。欄が無いときも null のときも同じ字面（面はどちらでも倒れる）。

種別は今ある 3 つ（未知の欄・欄の非空・schema）だけを使い、新しい種別も旗も例外の口も足さない（N-3.1）。id・title・shall・plain の今の違反の字面は 1 字も変わらない（場所の組み立て方を変えないため）。

(e) 凍結の材料（tests/floor_cases.yaml・新しい節 cases_srs_item に 6 件）。凍結した土台 tests/fixtures/floor_base/design-intent/（実の置き場から独立・README のとおり取り直さない）に、手書きの要件の行 1 つを足す変異を当てて床を回す。土台の requirements は 19 行・nonfunctional は 3 行（実測）なので、path を requirements[19]・nonfunctional[3] にして add を true にすると既存の行を 1 つも動かさずに末尾へ足せる（runner の mutate・floor_cases.rs の 159〜194 行）。足す行の id は FR90 / NFR90 で、土台のどの id とも重ならない。本文（title・when・shall・plain）は日本語だけにする（語彙の検査 R-9 の母集団は要件の title・when・shall・plain＝vocab.rs の 140 行）。土台も正本も書き換えない（写しへ当てる）。

1. srs-item-full-row-passes（expect_rc 0）: 欄のそろった行（goals は GOAL1・basis は P-1・figures は 図1・verify は method test / how は日本語 1 行 / ac は AC1）を足す → 合格。床が新しい欄を要求しても、そろった行は落ちない（vacuous でない側の証拠）。
2. srs-item-empty-lists-pass（expect_rc 0）: 同じ行の goals・basis・figures・verify の ac を空の一覧にして足す → 合格。空の一覧を落とさない＝床が面より厳しくない（実の要件書は 20 行のうち 10 行が figures を空の一覧で持つ・実測）。
3. srs-item-no-figures（expect_rc 1・expect_n 1・expect_msg は の figures が無い）: 1 の行から figures だけを落として足す → 不合格 1 件。台帳 f2-648.67 が踏んだ穴そのもの。今の床ではこの case が 合格（rc 0）になる＝赤い歯。
4. srs-item-no-verify（expect_rc 1・expect_n 1・expect_msg は の verify が無い）: 1 の行から verify だけを落として足す → 不合格 1 件（中の 3 欄は数えない）。
5. srs-item-unknown-field（expect_rc 1・expect_n 1・expect_msg は 未知の欄）: 1 の行に閉じた一覧に無い欄 extra を 1 つ足して足す → 不合格 1 件。
6. srs-item-nfr-no-figures（expect_rc 1・expect_n 1・expect_msg は の figures が無い）: nonfunctional[3] へ figures を落とした行を足す → 不合格 1 件。requirements と nonfunctional が同じ 1 本の決まりを通ることの証拠。

tests/floor_cases.yaml の先頭の expected_cases を 135 から 141 に上げる（runner の floor_cases.rs 703 行が case の実数と突き合わせるので、上げ忘れは赤で出る）。既存の 135 件はどれも srs.yaml を触らない（grep で file: srs.yaml は 0 件・実測）ので、既存の case の期待は 1 件も変えない。

(f) 歯（crates/folio/tests/check.rs・関数名は f75_ で始める。grep -rn 'fn f75_' crates/folio/tests は今 0 本・実測）。既存の Work（80〜148 行・実の design-intent の写しに字面の変異を 1 か所だけ当てて folio check を回す形・便 34 と便 55 が使う）と、新しい助けの関数 assert_srs_item_violation（assert_srs_figure_violation（252〜273 行）と同じ形・不合格 1 件で語を全部含む）を使う。当て先はどれも実の srs.yaml に 1 か所しか無いことを実測済み（mutate_file の 151 行が 1 か所でなければ落ちる）。

1. f75_srs_requirement_without_figures_fails: FR1 の figures の行（改行 + 4 字下げ + figures: [図2-1, 図2-2, 図1] + 改行・実測 1 か所）を改行 1 つに置き換える → 不合格・違反 1 件・語は requirements の FR1・figures・が無い。今の床では 合格（rc 0）になる＝赤い歯。
2. f75_srs_requirement_with_an_empty_figures_list_passes: 同じ行を figures: [] に置き換える → 合格・違反 0。空の一覧を落とさない。
3. f75_srs_requirement_without_verify_fails: FR1 の verify の行（4 字下げ + verify: {method: test, how: 決まった回答 5 つを入れ、支度表が期待どおりか比較する, ac: [AC1]} + 改行・実測 1 か所）を空の字に置き換える → 不合格・違反 1 件・語は requirements の FR1・verify・が無い。
4. f75_srs_requirement_with_an_empty_verify_how_fails: 同じ verify の行の how の値だけを空の字にした行に置き換える → 不合格・違反 1 件・語は FR1 の verify・how・が空。verify の中まで数えることの証拠。
5. f75_srs_requirement_with_an_unknown_field_fails: FR1 の figures の行を figures: [] と 4 字下げの extra: 1 の 2 行に置き換える → 不合格・違反 1 件・語は 未知の欄・extra。
6. f75_srs_nonfunctional_without_figures_fails: NFR3 の figures の行（改行 + 4 字下げ + figures: [全段] + 改行・実測 1 か所）を改行 1 つに置き換える → 不合格・違反 1 件・語は nonfunctional の NFR3・figures。
7. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/ の既存の歯すべて（凍結の場合 141 件・実の design-intent が今までどおり 合格 になる check_canonical_design_intent_passes を含む）。

(g) 最小の写し 17 本の手直し（tests/fixtures/*/srs.yaml）。床の穴を塞ぐと、要件の行を最小の形だけで持つ写し 17 本が新しい欄を欠いて落ち、それを読む歯（違反はちょうど 1 件と数える形・adr.rs 169 行・anchor.rs 105 行・link.rs 45 行・refs.rs 44 行・vocab.rs 44 行・check.rs 204 行）が全部赤くなる。写しは手書きなので、同じ便で最小の差分だけ足す（生成し直さない）。当て先の 17 本は find で全数を列挙した（tests/fixtures の下の srs.yaml は 20 本で、残る 3 本 = face/srs.yaml・ceiling/bundle/source/srs.yaml・floor_base/design-intent/srs.yaml は既に欄がそろっているので触らない・実測）。

足すのは 1 行だけ在る要件の行（FR1 か FR5）への 6 行で、値は床を通るいちばん小さい形にする: pattern は ubiquitous・strength は must・when は つねに・verify は {method: test, how: 歯で見る, ac: []}・goals は空の一覧・figures は空の一覧。basis は 13 本が既に持つ（P-1 か P-1, P-99）ので、持たない 4 本（check/ の 4 本）にだけ空の一覧で足す。値を空の一覧と日本語だけにするのは、参照 id の解決（refs.rs の 65 行が srs.yaml の全欄の字を歩く）と語彙の検査（vocab.rs の 140 行が要件の title・when・shall・plain を読む）に 1 件も違反を足さないため＝どの写しも「違反はちょうど 1 件」のままになる。tests/fixtures/check/missing-file/srs.yaml だけは憲法の写しが無くて床が要件書まで届かない（check.rs の load_all の 148〜157 行が先に まだ分からない で止まる）ので中身は変わらないが、形をそろえるために同じ 6 行を足して write-set に入れる。

(h) 大きさと接続。新規 file は無い・新しい dir も無い。src は check.rs 1 本だけ（生の行 654・正規化 625〔空行を除き幅 120 で折る〕・余地 875）で、足すのは定数 4 本と関数 2 本と分岐の組み替え＝約 +65 行（正規化で 625 → 約 690・余地 875 → 約 810）。ほかの src は 1 本も触らない。歯は crates/folio/tests/check.rs に約 +110 行、凍結の場合は tests/floor_cases.yaml に約 +80 行、写しは 17 本に各 6 行。size は M（触る file が 20 本なので予算は M・src の余地 875 は M の見積 300 を上回る）。外部 crate は増やさない。部品目録 design-intent/preview/parts.json にも様式 folio.css にも足さない（面を 1 行も変えない）。design-intent は 1 file も触らない。先行の便は無い（便 74 行 bw は main 26fe980 に着地済みで、触るのは face 側の 4 本＝本便の check.rs と重ならない）。

(i) 数えないと決めたこと（この便の外）。値域（pattern が型の一覧に・strength が強度の一覧に・verify の method が手段の一覧に在るか）は数えない＝在る値の妥当性は面の生成器と便 55 の形で別に塞ぐ。figures の各項の行き先（図2-<n>・図1・図3・全段 の形かと、その段が実在するか）も数えない＝面が まだ分からない で表す。requirements と nonfunctional 以外の節の行（goals・acceptance・constraints・actors・outputs・rail・verdicts）の欄は今のまま（同じ種類の穴が残るので、着地後に台帳へ 1 件起こす）。要件書の欄の決まりを design-intent の生成区間へ写すのは後続の f2-648.73（ADR-11 決定 (4)⑤）で、本便が置く定数 4 本はそのとき SRS_TOP_LEVEL・SRS_FIGURE_REQUIRED と一緒に運ぶ。

## 2. 範囲

- 入れる: 床の定数 4 本と関数 2 本・check_srs の分岐の組み替え・凍結の場合 6 件と件数の更新・f75_ の歯 6 本・最小の写し 17 本の手直し。
- 入れない: 欄の値域の検査・図の参照の行き先の検査・requirements と nonfunctional 以外の節の行の欄・design-intent への欄の決まりの追加・面の生成器と様式と部品目録の変更・新しい違反の種別。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| consts | 欄の一覧 | check.rs の SRS_ITEM_TEXT / SRS_ITEM_LIST / SRS_ITEM_VERIFY_TEXT / SRS_ITEM_OPTIONAL |
| item | 要件の行 | check.rs の check_srs_item（閉じた一覧・非空・verify の表） |
| seq | 一覧の欄 | check.rs の seq_field（在ること・空でよい） |
| frozen | 凍結の場合 | tests/floor_cases.yaml の cases_srs_item 6 件（土台は floor_base） |
| teeth | 歯 | crates/folio/tests/check.rs に f75_ の 6 本 |
| copies | 最小の写し | tests/fixtures の下の srs.yaml 17 本に 6 行ずつ |

## 4. 検査（歯）

§1 (e)(f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bx"
title = "床の穴 f2-648.67 — folio check が要件書の要件の行（requirements / nonfunctional）の必須欄を、面の生成器 face_srs.rs が読む欄から導いた床の定数で数える（字の 7 欄は非空・一覧の 3 欄と verify の ac は在ること〔空の一覧は通る〕・verify は表で中の 2 欄は非空・欄の集合は閉じた一覧）。凍結の材料は floor_base へ手書きの行を足す 6 件で、figures を欠いた行が 不合格 になる"
req = ["FR5", "FR4"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/tests/check.rs", "tests/floor_cases.yaml", "tests/fixtures/check/dup-key/srs.yaml", "tests/fixtures/check/empty-field/srs.yaml", "tests/fixtures/check/unknown-section/srs.yaml", "tests/fixtures/check/missing-file/srs.yaml", "tests/fixtures/adr/effective-no-approval/srs.yaml", "tests/fixtures/adr/schema-drift/srs.yaml", "tests/fixtures/adr/two-adopted/srs.yaml", "tests/fixtures/anchor/no-anchor/srs.yaml", "tests/fixtures/anchor/root-digest-drift/srs.yaml", "tests/fixtures/link/adr-id-missing/srs.yaml", "tests/fixtures/link/amended-by-orphan/srs.yaml", "tests/fixtures/link/retreat-kind-drift/srs.yaml", "tests/fixtures/refs/bad-counts/srs.yaml", "tests/fixtures/refs/dangling-id/srs.yaml", "tests/fixtures/refs/orphan-rule/srs.yaml", "tests/fixtures/vocab/exemptions/srs.yaml", "tests/fixtures/vocab/unknown-word/srs.yaml"]
verify = ["cargo nextest run -p folio --test check f75_", "cargo nextest run -p folio", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f75_ の歯 6 本（要件の行の figures 欠け・空の figures は通る・verify 欠け・verify の how が空・未知の欄・nonfunctional の figures 欠け）が緑、tests/floor_cases.yaml の凍結の場合 141 件（新しい 6 件を含む）と crates/folio/tests の既存の歯が全部緑（実の design-intent が今までどおり 合格 になる歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
