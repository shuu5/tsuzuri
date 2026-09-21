# 設計: 便 85 — 規則の表の種別 deny の意味を、上限だけでなく下限と固定の値にも当たる字に直す（天井 16 周目 整合 F-4・FR19 / FR5）

- 要件: FR19（決まりの部分を床の定数から決定的に導出する）/ FR5（床の 3 値）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数が正本のあいだ、写しは導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（一方を正本とし他方は導出する）/ P-10.1（凍結 anchor）
- 出所: 一括 10 の仕分け C。天井の 16 周目 整合 F-4「種別 deny の意味と行 R-13 / R-14 の向きの食い違い。直す先は生成区間の正本＝実装の定数」。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ch が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き。
- 門: 本便は design-intent/rules.yaml の生成区間を書き換える＝天井の門（folio ceiling --gate）が印を読む。§1 (e) のとおり、席は規則の表 D-12 に従って先に印を付け直すか持ち主の裁定を取り、通す を実測してから器へ出す。

## 1. 目的と中身

規則の表 design-intent/rules.yaml の schema の節（生成区間）は、種別 deny の意味を「機械が測って、超過なら落とす」と書く。ところが deny の行は超過だけを見るわけではない。行 R-13 の値は「1 本以上」で、落とすのは下限に足りないときである。行 R-14 の値は「最上段（showcase）固定」で、落とすのは固定の値と違うときである。どちらも超過という向きを持たないので、種別の意味と行の実際が食い違う。値・種別・母集団は正しいので、直すのは意味の 1 行だけである。生成区間は機械が書く写しなので、直す先は正本＝実装の型付きの定数（crates/folio/src/rules.rs の FLOOR）と凍結 anchor で、file の側は folio schema --write で導出し直す（手で直さない・P-6.2）。行の value と kind と status と ruling は 1 字も変えない（P-17.4 の前置の検証は要らない＝閾値の値を緩めないため）。

orchestrator 席の実測（2026-09-22・main 81bc02c）:

- 意味の正本は crates/folio/src/rules.rs の FLOOR（133〜225 行）の中の kind_meaning の枝（169〜189 行）。deny の値は 172 行の Floor::Val で、字は 機械が測って、超過なら落とす（憲法の reject / build-check に対応）。鍵は RuleKind の値の名（Deny・BuildCheck・Detect・HumanReview）を呼んで作るので、鍵の字面の手書きの写しは無い。
- 生成区間の写しは design-intent/rules.yaml の先頭の注釈の次に 1 対の印で挟まれた 27 行で、凍結 anchor は tests/fixtures/schema/rules-region.txt（今 27 行・1764 byte・sha256 dcf207ced150b5ebd3d6ae03bcdb5bc42ef9a06d6f74ff3f830ff96729dafad2）。
- 導出の体裁は crates/folio/src/schema.rs の先頭（8〜13 行）。表の block の子は 字下げ + 鍵 + コロン + 値 の 1 行で、値が Floor::Val のときは幅に関わらず折らない（block_map の 1 つ目の枝）。だから本便の直しは行の数を変えず、その 1 行の byte 数だけを増やす。
- 命令 folio schema の対象は 8 file で、rules.yaml は 4 本目（schema.rs の TARGETS）。
- 行 R-13 の value は 1 本以上・kind は deny・stage は post。行 R-14 の value は 最上段（showcase）固定・kind は deny・stage は post。どちらも本便で触らない。deny の行はほかに R-1 から R-8 など（実測で閾値行 16 本のうち kind が deny の行が大半）。
- 憲法の面の章 05 の種別の凡例は face_constitution.rs の rules_chapter（722〜740 行）が、読んでいる rules.yaml の schema.kind_meaning から出す（便 70）。実の置き場では本便の直しがそのまま凡例に出る。凍結の写し tests/fixtures/face/rules.yaml は別の最小の手書きなので、凍結の写し 7 本は 1 byte も変わらない。

### (a) 実装の定数

rules.rs の FLOOR の kind_meaning の deny の値（172 行の Floor::Val）の字を、次の逐語に置き換える。ほかの 3 つの種別の値・鍵・kind_map_to_constitution・enums・そのほかの枝は 1 字も変えない。

`機械が測って、値域の外なら落とす（上限の超過・下限の不足・固定の値との違い。憲法の reject / build-check に対応）`

### (b) 凍結 anchor（P-10.1）

tests/fixtures/schema/rules-region.txt を次の逐語に置き換える。席が今の anchor から deny の 1 行だけを (a) の字に替えて独立に組んだもの（行の数は 27 のまま・幅の折り返しは起きない）。**作業者は anchor を命令の出力から作らない。**下の逐語をそのまま file に置き、生成物が合わなければ実装の側を直す（anchor を生成物に合わせない・合わせられないと判断したら席へ問う）。27 行・1833 byte・sha256 54580596905e2d70d834c34553d3ad9000dd356473a0df0f60e8afe8fafe652c。

```
schema:
  version: 1
  top_level: [schema, thresholds, discipline]
  top_level_note: 最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。thresholds と discipline は人が書き、schema は生成区間
  threshold_row:
    required: [id, article, what, value, kind, status, ruling, ruled_at, stage]
    optional: [basis, projection, same_failure, population, note]
  discipline_row: {required: [id, article, what, kind, status, ruling, ruled_at], optional: [note]}
  enums:
    kind: [deny, build-check, detect, human-review]
    status: [仮, 凍結, 未定]
    stage: [in-loop, post]
  enums_note: stage は憲法の値域 stage と同じ値（実装は憲法から導出した名の列を使う）
  kind_meaning:
    deny: 機械が測って、値域の外なら落とす（上限の超過・下限の不足・固定の値との違い。憲法の reject / build-check に対応）
    build-check: 生成時の検査で数え、違反なら落とす（憲法の build-check に対応）
    detect: 記録・起票のみ・止めない（憲法の none に対応）
    human-review: 人が守る作法（憲法の human-review に対応・D 行）
  kind_map_to_constitution:
    deny: [reject, build-check]
    build-check: [build-check]
    detect: [none]
  kind_map_to_constitution_note: R 行にだけ適用する（D 行は作法＝条の機構とは別）。右辺は憲法の値域 mechanism_kind の値
  excluded:
    what: [時間（「60 分以内」）, 費用（「300k token 以下」）]
    why: 測る仕組みが別で凍結できない。M1 の実地試験（非エンジニア 1 回）で初めて数値を決める（要件書 not_frozen）。
  reverse_reference: 各行は article 欄で条を指し、その条の relations.rules に行 id が載る（R-4 の双方向）。
```

### (c) 規則の表の生成区間

design-intent/rules.yaml の生成区間（印 2 本の間）を `cargo run -p folio -- schema --write --dir design-intent` で導出し直す。区間の外の byte は 1 つも変えない（閾値行と開発規律行・先頭の注釈は不変）。ほかの 7 file の生成区間も 1 byte も変わらない。

### (d) 歯（関数名は f85_ で始める。`grep -rn 'fn f85_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

crates/folio/tests/schema.rs に 2 本。

1. f85_rules_region_matches_the_new_anchor: 写しに --check → 0 ∧ 標準出力が 8 行 ∧ rules.yaml の行が 1833 byte を含む ∧ 写しの rules.yaml の生成区間が (b) の anchor と byte 一致 ∧ anchor 自身の自己検査 = 27 行・1833 byte・sha256 54580596905e2d70d834c34553d3ad9000dd356473a0df0f60e8afe8fafe652c。要約値は sha256 を測る既存の口で測り、測れなければ歯を落とす（まだ分からないで素通りさせない）。
2. f85_deny_meaning_names_the_lower_bound_and_the_fixed_value: 写しの rules.yaml の生成区間の deny の行に 値域の外・上限の超過・下限の不足・固定の値との違い の 4 つが在り、超過なら落とす が 0 回 ∧ 同じ写しの閾値行の R-13 の value が 1 本以上 で kind が deny、R-14 の value が 最上段（showcase）固定 で kind が deny のままである（意味だけを直し、値と種別は動かしていない）。本便の前の main では 1 つ目の条件が偽なので赤い歯。

crates/folio/tests/face.rs に 1 本。

3. f85_constitution_legend_shows_the_new_meaning: 実の置き場の写しで憲法の面を書く → 0 ∧ 章 05 の種別の凡例に 値域の外なら落とす が在り、超過なら落とす が 0 回（凡例は読んでいる file の kind_meaning から出るので、直しが読み手まで届いていることを見る）。
4. 既存の歯の定数の更新: crates/folio/tests/schema.rs の 81〜84 行の定数 RULES_REGION_LINES（27 のまま）・RULES_REGION_BYTES（1764 → 1833）・RULES_REGION_SHA256（dcf207ce… → 54580596905e2d70d834c34553d3ad9000dd356473a0df0f60e8afe8fafe652c）を (b) の anchor に合わせて直す。同 file の 23 行の注釈の 1764 byte の字も 1833 に直す。この 3 定数を読む既存の歯 7 か所（810・819・824・828・860・900・914 行）は字を変えない（定数の更新で緑に戻る）。
5. 回帰（4 の定数の更新の後は期待不変・verify の 2 行目）: crates/folio/tests/schema.rs の f76_ から f78_ を含む既存の歯すべて・tests/check.rs の f77_ と r11_ と床の歯すべて・tests/face.rs の既存の歯すべて（実の置き場の逐語と件数の census を含む。census が種別の凡例の字を凍結しているときは、歯の側の期待を正本から数え直す）。

### (e) 門と裁定

write-set に design-intent/rules.yaml が在るので、受付の事前読みで天井の門（folio ceiling --gate）が印 design-intent/preview/ceiling-stamp.yaml を読む。今の印は 21 周目のもの（4 観点とも 合格）だが、印を書いた後に設計文書が動いていれば 印が古い で まだ分からない を返す。規則の表 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。席は受付の直前に門を撃って 3 値を実測し、通す でなければ印を付け直す（周を回して folio ceiling --stamp）か、持ち主の裁定（この便を門の外で器へ出してよいか）を取り、その逐語と時刻を台帳の notes に記帳してから出す。裁定も印も無ければ出さない。

### (f) 大きさ

src は crates/folio/src/rules.rs 1 本だけ（幅 120 で正規化して 259 行・余地 1241・変更は 1 行の字）。歯は tests/schema.rs（正規化 1118 行・余地 382・+ 約 35 行）と tests/face.rs（正規化 1637 行・+ 約 20 行）。size **S**。外部 crate は増やさない。main.rs・schema.rs・check.rs・面の生成器・部品目録・様式・tests/floor_cases.yaml・tests/fixtures/face/ の写し・ほかの 7 file の生成区間・CI の yml は触らない。

### (g) 本便が運ばないもの

行 R-13 と R-14 の value・kind・status・ruling・note（値と種別は正しいので動かさない）。ほかの 3 つの種別の意味。憲法の値域 mechanism_kind とその対応表 kind_map_to_constitution。規則の表の裁定の欄（この直しは意味の言い直しで、値の改訂ではないので行に裁定 id を足す必要は無い。行の値も種別も母集団も変わらないため P-17.1 の対象外）。凍結の写し tests/fixtures/face/ の 7 本と写しの rules.yaml。

## 2. 範囲

- 入れる: 実装の定数の 1 行・凍結 anchor の置き換え・規則の表の生成区間の導出し直し・歯 3 本。
- 入れない: 行の値と種別と裁定・ほかの種別の意味・ほかの 7 file の生成区間・床の判定・面の生成器・部品目録・様式・凍結の写し。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| const | 実装の定数 | rules.rs の FLOOR の kind_meaning の deny |
| anchor | 凍結 | tests/fixtures/schema/rules-region.txt（1833 byte） |
| region | 生成区間 | design-intent/rules.yaml の先頭の印の間（folio schema --write） |
| teeth | 歯 | tests/schema.rs に f85_ 2 本・tests/face.rs に f85_ 1 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ch"
title = "規則の表の種別 deny の意味を、上限の超過だけでなく下限の不足と固定の値との違いにも当たる字へ、実装の定数 rules.rs の FLOOR と凍結 anchor の側で直し、規則の表の生成区間は folio schema --write で導出し直す（行の値と種別と裁定は不変・天井 16 周目 整合 F-4・門の 3 値を実測してから受付）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/rules.rs", "tests/fixtures/schema/rules-region.txt", "design-intent/rules.yaml", "crates/folio/tests/schema.rs", "crates/folio/tests/face.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test schema --test face f85_", "cargo nextest run -p folio --test schema --test face --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f85_ の歯 3 本（規則の表の生成区間が新しい凍結 anchor と byte 一致し anchor の自己検査が通る・deny の意味が下限と固定の値を名指し超過の字が消えている・R-13 と R-14 の値と種別が不変・憲法の面の種別の凡例に新しい字が出る）が緑、tests/schema.rs の凍結 anchor の定数 3 つ（行数・byte 数・要約値）を新しい anchor に合わせた上で tests/schema.rs と tests/check.rs と tests/face.rs の既存の歯が全部緑、folio schema --check が 8 file とも一致、folio check が合格（違反 0・まだ分からない 0）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
