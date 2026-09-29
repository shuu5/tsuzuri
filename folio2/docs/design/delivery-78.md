# 設計: 便 78 — 要件書の生成区間の注の誤り（scope_m1 は実在する）を実装の定数と凍結 anchor の側で直す（天井 22 周目 実態 F-1・FR19 / FR5）

- 要件: FR19（決まりの部分を床の定数から決定的に導出する）/ FR5（床の 3 値）
- 条: P-5.6（実装の型付きの定数が正本のあいだ、写しは導出する）/ P-6.2（生成物を手で直さない）/ P-10.1（凍結 anchor）
- 出所: 天井の 22 周目・実態 F-1（反証 支持・2026-09-21）。便 77（main 133ce94）が置いた要件書の生成区間の注 top_level_note が「scope_m1 は名を空けてある節で、今の正本には無い」と書くが、正本 design-intent/srs.yaml は 105 行に scope_m1 の節を持つ。誤りは席が便 77 の契約 §1 (b) に書いた anchor の逐語に在り、作業者はそれを忠実に写した。注の正本は crates/folio/src/check.rs の SRS_FLOOR なので、直す先は実装の定数と凍結 anchor で、要件書の側は folio schema --write で導出し直す（手で直さない）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 ca が指す §1 だけ。write-set は手書き。
- 門: 本便は design-intent/srs.yaml の生成区間を書き換える＝天井の門が印を読む。印は便 77 の着地で 古い（22 周目は実態と忠実さが止めた）。規則の表 D-12 のとおり、席は持ち主の裁定を先に取ってから器へ出す（§1 (e)）。

## 1. 目的と中身

(a) 実装の定数。crates/folio/src/check.rs の 70 行からの SRS_FLOOR のうち、73〜76 行の top_level_note（Floor::Val・字は 75 行）の字を次の逐語に置き換える（scope_m1 の一文を落とすだけ・ほかの葉は変えない）: `最上位の節の閉じた一覧（ほかの節は床が落とす・N-3）。schema のほかの節は人が書き、schema は生成区間`。SRS_TOP_LEVEL（17 語・scope_m1 を含む）・SRS_FIGURE_REQUIRED・SRS_FIGURE_OPTIONAL・床の判定は変えない。

(b) 凍結 anchor。tests/fixtures/schema/srs-region.txt を、席が便 77 の anchor から scope_m1 の一文だけを落として独立に組んだ次の逐語に置き換える。23 行・708 byte・sha256 290e27043b7b0e01b7d78a1c2829f1e484da4af57c874bd7d0c54792b467c3aa。作業者は anchor を命令の出力から作らない。

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
```

(c) 要件書。design-intent/srs.yaml の生成区間（末尾・印 2 本の間）を `cargo run -p folio -- schema --write --dir design-intent` で導出し直す。区間の外の byte は 1 つも変えない（版・承認欄・人が書く節は不変）。

(d) 歯。crates/folio/tests/schema.rs の 89 行からの定数 F77_REGIONS の srs.yaml の行を 708 と `290e27043b7b0e01b7d78a1c2829f1e484da4af57c874bd7d0c54792b467c3aa` に直す（行数 23 は不変）。歯 f77_check_covers_the_three_files（1124 行）は F77_REGIONS の byte 数を読むので字は変えない。新しい歯 1 本を crates/folio/tests/schema.rs に足す: f78_srs_note_does_not_claim_scope_m1_is_absent = 写しに --check → 0 ∧ design-intent/srs.yaml の生成区間の字に「今の正本には無い」が含まれない ∧ 同じ正本の最上位に scope_m1 の節が在る（本便の前の main では 1 つ目の条件が偽で落ちる＝赤い歯）。回帰（期待不変）: tests/schema.rs の f77_ 5 本・tests/check.rs の f77_ 1 本と既存の歯すべて。

(e) 門と裁定。write-set に design-intent/srs.yaml が在るので folio ceiling --gate は 2（まだ分からない・印が古い）を返す。規則の表 D-12 は「止める・まだ分からない のときは出さず、所見の解消か持ち主の裁定を先に取る」と定める。本便は所見（22 周目 実態 F-1）を解消する唯一の経路で、書き換える設計文書は機械が書く写しの区間だけなので、席は持ち主の裁定（この便を門の外で器へ出してよいか）を取り、裁定の逐語と時刻を台帳 f2-648.116 の notes に記帳してから出す。裁定が無ければ出さない。

(f) 大きさ。src は check.rs だけ（正規化 714 行・余地 786・変更は 1 行の字）。歯は tests/schema.rs（正規化 1101・余地 399・+約 20 行）。size S。外部 crate は増やさない。main.rs・schema.rs・face・部品目録・様式・tests/floor_cases.yaml・ほかの fixture は触らない。

## 2. 範囲

- 入れる: 注の字の直し・anchor の置き換え・要件書の生成区間の導出し直し・歯の定数の更新と f78_ 1 本。
- 入れない: 閉じた一覧の値・床の判定・要件書の人が書く節・版と承認欄・ほかの 7 file の生成区間。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| note | 注 | check.rs SRS_FLOOR の top_level_note |
| anchor | 凍結 | tests/fixtures/schema/srs-region.txt（708 byte） |
| region | 生成区間 | design-intent/srs.yaml の末尾（folio schema --write） |
| teeth | 歯 | tests/schema.rs の F77_REGIONS と f78_ 1 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ca"
title = "要件書の生成区間の注の誤り（scope_m1 は実在する）を実装の定数 SRS_FLOOR と凍結 anchor の側で直し、要件書の区間は folio schema --write で導出し直す（値と床の判定は不変・22 周目 実態 F-1・持ち主の裁定の後に受付）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/check.rs", "tests/fixtures/schema/srs-region.txt", "design-intent/srs.yaml", "crates/folio/tests/schema.rs", "crates/folio/tests/check.rs"]
verify = ["cargo nextest run -p folio --test schema f78_", "cargo nextest run -p folio --test schema --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f78_ の歯 1 本（要件書の生成区間に「今の正本には無い」が無く scope_m1 の節が在る）が緑、tests/schema.rs と tests/check.rs の既存の歯（f77_ 6 本を含む）が全部緑、folio schema --check が 8 file とも一致、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
