# 設計: 便 86 — 要件の行の欄の閉じた一覧（便 75 の SRS_ITEM_*）を要件書の生成区間へ導出する（台帳 f2-648.115・ADR-11 決定 (3)(イ) の類 B・FR19 / FR5）

- 要件: FR19（決まりの部分を床の定数から決定的に導出する）/ FR5（床の 合格・不合格・まだ分からない）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を正本とするあいだ、写しを設計文書の置き場へ導出する）/ P-6.2・P-6.3・P-6.4 / P-10.1（凍結 anchor）/ N-3.1（例外機構を足さない）
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)「検査される file の側から緩められてはならない形の決まり（最上位の節の閉じた一覧・必須の欄の集合・値域・閉じた id の集合）で、その file に憲法と同じ強さの変更の門が無いものは、実装の型付きの定数が正本で、その file の中の生成区間に写しを導出する」。本便が運ぶのは「必須の欄の集合」の類（類 B）。
- 出所: 台帳 f2-648.115。便 75（f2-648.110）が crates/folio/src/check.rs に置いた要件の行の欄の一覧（SRS_ITEM_TEXT 7 語・SRS_ITEM_LIST 3 語・SRS_ITEM_VERIFY_TEXT 2 語・SRS_ITEM_OPTIONAL 3 語）は写しがどこにも無い。便 77（決定 (4)⑤）の字面の外だったので便 77 では運ばなかった。
- 位置: 便 77（main 133ce94）と便 78（main 81bc02c）の後。どちらも着地済みで、凍結 anchor tests/fixtures/schema/srs-region.txt は今 23 行・708 byte・sha256 290e27043b7b0e01b7d78a1c2829f1e484da4af57c874bd7d0c54792b467c3aa（実測）。本便はその末尾に足す形なので、既存の 23 行は 1 byte も変わらない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ci が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き。
- 門: 本便は design-intent/srs.yaml の生成区間を書き換える＝天井の門（folio ceiling --gate）が印を読む。§1 (e) のとおり、席は規則の表 D-12 に従って先に印を付け直すか持ち主の裁定を取り、通す を実測してから器へ出す。

## 1. 目的と中身

要件書 design-intent/srs.yaml の要件の行（requirements と nonfunctional）の欄は、閉じた一覧として床（crates/folio/src/check.rs の check_srs_item）の中の型付きの定数だけが持つ。file の側にはその写しが無いので、正本を書く人からは「どの欄が要るのか・どの欄まで書いてよいのか」が file から読めない。憲法 P-5.6 は、実装の型付きの定数を規則の正本にしているあいだ、その写しを人が読める型付きデータとして設計文書の置き場へ決定的に導出することを義務にする。本便はその写しを、着地済みの同型の便（47 / 48 天井の正本・51 / 53 規則の表・76 入口・77 要件書と語彙と相談窓口の最上位の節・78 注の直し）と同じ機構（印 2 本で挟んだ生成区間・命令 folio schema が床の木から導出して書く / 検査する）で要件書の生成区間の末尾に足す。値・床の判定・面の生成器・要件書の規範文は 1 つも変えない。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- 対象の定数は crates/folio/src/check.rs の 98〜103 行。SRS_ITEM_TEXT（7 語 = id・title・pattern・strength・when・shall・plain）／SRS_ITEM_LIST（3 語 = goals・basis・figures）／SRS_ITEM_VERIFY_TEXT（2 語 = method・how）／SRS_ITEM_OPTIONAL（3 語 = milestone・rules・note）。どれも module の中だけの const。
- 読み手は同じ file の check_srs_item（625〜651 行）だけ。欄の集合の判定（630〜637 行）は SRS_ITEM_TEXT・SRS_ITEM_LIST・SRS_ITEM_OPTIONAL のどれにも無く、かつ verify でもない鍵を 未知の欄 の違反にする。verify の中は 646 行で SRS_ITEM_VERIFY_TEXT の非空を見て、647 行で ac が一覧であることを見る。
- 閉じた集合のうち、定数になっていない字面が 2 つ在る。634 行と 645 行と 647 行の中の verify と、647 行の中の ac である。本便はこの 2 つも定数にして、生成区間が集合の全部を覆うようにする（写しに穴を作らない）。
- 要件書の生成区間の床の木は同じ file の SRS_FLOOR（69〜94 行）。枝は top_level・top_level_note・figures・figures_note の 4 つで、末尾は figures_note。
- 命令 folio schema の対象は 8 file で、srs.yaml は 6 本目（crates/folio/src/schema.rs の TARGETS・床の木は check::SRS_FLOOR）。命令・旗・関数（run・run_one・derive・region_of）は本便で触らない。
- 導出の体裁は schema.rs の先頭（8〜13 行）。幅は 100 字。表の子が全部 文字列か その一覧 で 1 行が 100 字以内なら flow、さもなくば block。Floor::Val は幅に関わらず折らない。
- 床は生成区間の中身を床の木と突き合わせない（写しの一致を見るのは folio schema --check）。写しの fixture（tests/fixtures/floor_base/ など）は schema の節を持たないままで床を通る（便 77 と同じ扱い）。

### (a) 定数を 2 つ足し、字面の写しを閉じる

check.rs の 98〜103 行の並びに次の 2 本を足し、読み手の字面の写しを置き換える。値も判定も変えない。

1. SRS_ITEM_VERIFY: &str = verify の字。634 行の鍵の比較と 645 行の row.get と、(b) の床の木の鍵に使う。
2. SRS_ITEM_VERIFY_LIST: [&str; 1] = ac の字を 1 つ持つ配列。647 行の一覧の欄の判定をこの配列の各項について回す形にする（今と同じ 1 回の判定になる）。断りの字面（「{at} の verify の ac が無い（一覧・空でよい）」）は 1 字も変えない。

### (b) 床の木に要件の行の枝を足す

check.rs の SRS_FLOOR の末尾（figures_note の後ろ）に 2 つの枝を足す。葉は既存の定数を指す（同じ一覧を 2 回書かない）。

1. requirement_row = Floor::Map の 4 欄。required_text は Floor::Strs(&SRS_ITEM_TEXT)／required_list は Floor::Strs(&SRS_ITEM_LIST)／optional は Floor::Strs(&SRS_ITEM_OPTIONAL)／SRS_ITEM_VERIFY を鍵にした Floor::Map の 2 欄（required_text = Floor::Strs(&SRS_ITEM_VERIFY_TEXT)・required_list = Floor::Strs(&SRS_ITEM_VERIFY_LIST)）。欄の順は上のとおり。
2. requirement_row_note = Floor::Val で次の逐語。

`要件の行（requirements と nonfunctional）の欄の閉じた一覧。required_text は非空の字・required_list は在ること（空の一覧でよい）・optional は任意・verify は表`

3. 床の判定は変えない。最上位の節の閉じた一覧（SRS_TOP_LEVEL・17 語）も変えない。schema の節の中身を床が床の木と突き合わせることもしない。

### (c) 凍結 anchor（P-10.1）

tests/fixtures/schema/srs-region.txt を次の逐語に置き換える。席が今の anchor（便 78 が置いた 23 行・708 byte）の末尾に (b) の 6 行を足して独立に組んだもの。体裁の規則は schema.rs の先頭のとおりで、requirement_row は子に表を持つので block、その 4 つの子はどれも 1 行が 100 字以内なので flow になる（実測の字数は順に 69・42・38・63）。**作業者は anchor を命令の出力から作らない。**下の逐語をそのまま file に置き、生成物が合わなければ実装の側を直す（anchor を生成物に合わせない・合わせられないと判断したら席へ問う）。29 行・1168 byte・sha256 3937340f77713b087b33ca43b8fe866ca310b97cd04e5e1f4ce9967e19b724e1。

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
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。schema のほかの節は人が書き、schema は生成区間
  figures:
    entry: {required: [id, type, caption, spec], optional: [refs, note]}
  figures_note: 要件書の図の節の行の欄（判断の記録と設計ノートの欄の決まりの figures.entry と同じ形）。型（type）の値域は部品目録が持つ
  requirement_row:
    required_text: [id, title, pattern, strength, when, shall, plain]
    required_list: [goals, basis, figures]
    optional: [milestone, rules, note]
    verify: {required_text: [method, how], required_list: [ac]}
  requirement_row_note: 要件の行（requirements と nonfunctional）の欄の閉じた一覧。required_text は非空の字・required_list は在ること（空の一覧でよい）・optional は任意・verify は表
```

### (d) 要件書の生成区間

design-intent/srs.yaml の生成区間（末尾・印 2 本の間）を `cargo run -p folio -- schema --write --dir design-intent` で導出し直す。区間の外の byte は 1 つも変えない（版・承認欄・人が書く節は不変＝本便は要件書の版を上げない）。ほかの 7 file の生成区間も 1 byte も変わらない。

### (e) 門と裁定

write-set に design-intent/srs.yaml が在るので、受付の事前読みで天井の門（folio ceiling --gate）が印 design-intent/preview/ceiling-stamp.yaml を読む。規則の表 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。席は受付の直前に門を撃って 3 値を実測し、通す でなければ印を付け直す（周を回して folio ceiling --stamp）か、持ち主の裁定（この便を門の外で器へ出してよいか）を取り、その逐語と時刻を台帳 f2-648.115 の notes に記帳してから出す。裁定も印も無ければ出さない。

### (f) 歯（関数名は f86_ で始める。`grep -rn 'fn f86_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

crates/folio/tests/schema.rs に 2 本。

1. f86_srs_region_matches_the_new_anchor: 写しに --check → 0 ∧ 標準出力が 8 行 ∧ srs.yaml の行が 1168 byte を含む ∧ 写しの srs.yaml の生成区間が (c) の anchor と byte 一致 ∧ anchor 自身の自己検査 = 29 行・1168 byte・sha256 3937340f77713b087b33ca43b8fe866ca310b97cd04e5e1f4ce9967e19b724e1。要約値は sha256 を測る既存の口で測り、測れなければ歯を落とす（まだ分からないで素通りさせない）。本便の前の main では byte 数が 708 なので赤い歯。
2. f86_region_lists_every_group_of_the_row: 写しの srs.yaml の生成区間の requirement_row の枝に、7 語・3 語・3 語と、verify の中の 2 語と 1 語が、実装の定数の並びと同じ順で在る（歯の側は期待を (c) の anchor から取らず、正本の要件の行に実際に現れる欄の集合〔design-intent/srs.yaml の requirements と nonfunctional の全行の鍵の和集合〕が生成区間の 4 群 + verify の中に過不足なく収まることを見る）。

crates/folio/tests/check.rs に 2 本。

3. f86_unknown_field_cannot_be_loosened_from_the_file: (d) を当てた実の置き場の写しに folio check → 0 ∧ 違反 0。続けて、写しの要件の行 1 つに欄 extras を足す → 不合格 ∧ 未知の欄 がちょうど 1 件で extras を含む。さらに写しの schema.requirement_row の optional に extras を足しても不合格のまま（閉じた一覧は実装の定数が持ち、file の側から緩められない・N-3.1）。
4. f86_verify_inner_list_is_still_checked: 写しの要件の行の verify から ac を落とす → 不合格 ∧ 断りの字が 「verify の ac が無い（一覧・空でよい）」 のまま（定数化で断りの字面が変わっていない）。
5. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/schema.rs の f76_ から f78_ を含む既存の歯すべて・tests/check.rs の f75_ 5 本と f77_ と r11_ と床の歯すべて。共通の検証（.vessel.toml の common-verify）が workspace 全部を回すので、面と束と凍結の場合 134 件もそこで見る。

### (g) 大きさ

src は crates/folio/src/check.rs 1 本だけ（幅 120 で正規化して 714 行・余地 786・見積は + 約 25 行）。歯は tests/schema.rs（正規化 1118 行・余地 382・+ 約 45 行）と tests/check.rs（正規化 750 行・余地 750・+ 約 40 行）。size **S**。外部 crate は増やさない。main.rs・schema.rs・intake.rs・entrance.rs・面の生成器・部品目録・様式・tests/floor_cases.yaml・tests/fixtures/floor_base/ の写し・ほかの 7 file の生成区間・CI の yml は触らない。

### (h) 本便が運ばないもの

要件書の版と承認欄（生成区間は機械が書く写しで、人が書いた中身は 1 byte も変わらないため）。要件 FR19 の規範文と受入基準 AC17（第 1.22 版で対象に要件書を足してある）。判断の記録 ADR-11 の決定 (4) の列への追記（発効済みの欄なので便では触らない。決定 (4) に足す注が要るか、判断の記録の注記で足りるかは席が判断し、別の PR で運ぶ・台帳 f2-648.115 の断り）。下調べ docs/design/adr-11-survey.md §2-b の残りの定数（参照 id の形と節の一覧・配信の接続先の判定の式・実装にだけ在る閾値 = 決定 (4)⑦⑧ の別の便）。所見 file の欄の一覧（生成物の置き場なので対象外）。語彙と相談窓口と入口と規則の表と天井の正本と判断の記録と設計ノートの生成区間。

## 2. 範囲

- 入れる: 定数 2 本の追加と字面の写しの置き換え・床の木の枝 2 つ・凍結 anchor の置き換え・要件書の生成区間の導出し直し・歯 4 本。
- 入れない: 値と床の判定と断りの字面・要件書の版と承認欄と人が書く節・ほかの 7 file の生成区間・判断の記録の改訂と注の追記・命令と旗の増減・面の生成器・部品目録・様式・写しの fixture。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| const | 定数 | check.rs の SRS_ITEM_VERIFY と SRS_ITEM_VERIFY_LIST（字面の写しを閉じる） |
| floor | 床の木 | SRS_FLOOR の requirement_row と requirement_row_note |
| anchor | 凍結 | tests/fixtures/schema/srs-region.txt（1168 byte） |
| region | 生成区間 | design-intent/srs.yaml の末尾（folio schema --write） |
| teeth | 歯 | tests/schema.rs に f86_ 2 本・tests/check.rs に f86_ 2 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ci"
title = "要件の行（requirements と nonfunctional）の欄の閉じた一覧を、実装の定数 SRS_ITEM_* から要件書 srs.yaml の生成区間の末尾へ導出し（verify と ac の字面も定数にして写しの穴をふさぐ）、凍結 anchor と byte 一致させる（値と床の判定と断りの字面と要件書の版は不変・file の側から緩められない・台帳 f2-648.115 / ADR-11 決定 (3)(イ)・門の 3 値を実測してから受付）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/check.rs", "tests/fixtures/schema/srs-region.txt", "design-intent/srs.yaml", "crates/folio/tests/schema.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test schema --test check f86_", "cargo nextest run -p folio --test schema --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f86_ の歯 4 本（要件書の生成区間が新しい凍結 anchor と byte 一致し anchor の自己検査が通る・正本の要件の行に現れる欄の集合が生成区間の群に過不足なく収まる・知らない欄が 1 件で file の側から緩められない・verify の ac の断りの字面が不変）が緑、tests/schema.rs と tests/check.rs の既存の歯が全部緑（f75_ 5 本と f77_ と f78_ を含む）、folio schema --check が 8 file とも一致、folio check が合格（違反 0・まだ分からない 0）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
