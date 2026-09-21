# 設計: 便 69 — 判断の記録の欄の決まりの注が言う rules 行 R-9 の母集団を 4 文書（語彙を含む）に直す（天井の 12 周目の整合 F-4 の是正・FR19 / FR5）

- 要件: FR19（欄の決まりの生成区間は床の定数から導出する）/ FR5（床の合格と不合格と まだ分からない）
- 条: P-5.6 / P-6.2 / P-6.3
- 出所: 天井の 12 周目（2026-09-21・main bbd2a05）の整合 F-4（直す）= 判断の記録の欄の決まり design-intent/adr/schema.yaml の生成区間の注 prose_note が「rules 行 R-9 の行と母集団〔憲法・rules・要件書〕は変えない」と書くが、規則の表の行 R-9 の母集団は語彙を含む 4 文書。生成区間の正本は crates/folio/src/adr.rs の床の木 FLOOR なので、直す先は実装の側（P-6.2）。
- 根拠の判断: 判断は無い（写しを行の字に合わせる）。行 R-9 の値と母集団は変えない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 br が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

(a) 床の木。crates/folio/src/adr.rs（正規化 約 1,195 行・余地 約 305）の FLOOR の prose_note の文字列の中の「〔憲法・rules・要件書〕」を「〔憲法・rules・要件書・語彙〕」にする（その 1 か所だけ・+3 字 = +9 byte）。

(b) 実の生成区間。folio schema --write を design-intent に当て、design-intent/adr/schema.yaml の生成区間の prose_note の行だけが変わる（ほかの 3 本の欄の決まりの file は 1 byte も変わらない）。orchestrator 席が独立に組んだ凍結 anchor の値 = 生成区間は 117 行・22,263 byte・sha256 = a5e1efca970d561966f78b598351afda2eda428fd4255b4a18a46f652ec6d389（席が 2026-09-21 に今の anchor の写しの同じ 1 か所を置き換えて測った）。tests/fixtures/schema/adr-region.txt の同じ 1 か所を同じ字にし（その結果が上の byte 数と要約値に一致することを歯が確かめる）、crates/folio/tests/schema.rs（正規化 約 800 行）の定数 REGION_BYTES を 22263・REGION_SHA256 を上の値にし、その定数の注釈と file 先頭の注釈の 22254 を 22263 に直す。生成物が anchor に合わないときは実装の側を直す（anchor を生成物に合わせない）。

(c) 写し 17 本。tests/fixtures の下の判断の記録の欄の決まりの写し（floor_base の 1 本と場合ごとの置き場の 16 本・write-set に全数）の prose_note の同じ 1 か所を同じ字にする（各 file その 1 行だけ・注は床の突き合わせの外なので歯は落ちないが、写しを実の file と同じ byte に保つ）。

(d) 歯（crates/folio/tests/schema.rs・関数名は r9_population_ で始める・今この語で始まる歯は無い）。
1. r9_population_names_the_vocabulary: design-intent の写しに folio schema --check を当てると 0 で終わり、design-intent/adr/schema.yaml の生成区間の prose_note の行に「憲法・rules・要件書・語彙〕」が在り「憲法・rules・要件書〕」が無い。
2. r9_population_anchor_holds: tests/fixtures/schema/adr-region.txt の byte 数が 22263・sha256 が (b) の値・行数 117（anchor 自身の自己検査）。
3. 回帰（期待不変・verify の 2 行目）: tests/schema.rs の既存の歯すべて（判断の記録の側の byte 数と要約値の歯は (b) の定数で緑になる）・tests/adr.rs・tests/floor_cases.rs（写しの注は突き合わせの外）。

(e) 大きさと接続。新規 file は無い。adr.rs（1 行）・adr/schema.yaml（1 行・--write）・adr-region.txt（1 行）・tests/schema.rs（定数 2 つ + 注釈 + 歯 2 本 約 25 行）・写し 17 本（各 1 行）。size S。外部 crate は増やさない。

## 2. 範囲

- 入れる: 注の 1 か所の字の直し・実の生成区間の再生成・anchor の定数・写し 17 本の同じ行・歯 2 本。
- 入れない: 規則の表の行 R-9 の変更・ほかの注の変更・判断の記録の欄の決まりの値域の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の木 | adr.rs の prose_note の 1 か所 |
| region | 生成区間 | schema --write と凍結 anchor の定数 |
| copies | 写し | fixture 17 本の同じ 1 行 |
| teeth | 歯 | tests/schema.rs に r9_population_ の 2 本 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "br"
title = "判断の記録の欄の決まりの床の木 FLOOR の prose_note の母集団を〔憲法・rules・要件書・語彙〕にし、実の生成区間を folio schema --write で書き直して凍結 anchor（117 行・22,263 byte・sha256 a5e1efca…）に合わせ、写し 17 本の同じ 1 行を同じ字にする（天井の 12 周目の整合 F-4）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "design-intent/adr/schema.yaml", "tests/fixtures/schema/adr-region.txt", "crates/folio/tests/schema.rs", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test schema r9_population_", "cargo nextest run -p folio --test schema --test adr --test floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "r9_population_ の歯 2 本（生成区間の注が語彙を含む・anchor の自己検査）が緑、tests/schema.rs（判断の記録の側の byte 数 22263 と要約値の歯を含む）・tests/adr.rs・tests/floor_cases.rs の既存の歯が全部緑、folio schema --check が 4 行とも一致、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
