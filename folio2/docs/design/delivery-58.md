# 設計: 便 58 — 判断の記録の承認者の値域に orchestrator 席 を足し、欄の決まりの注 2 つの席の名を直す（床の木と生成区間と写し・席の呼び名の裁定 2026-09-20・FR19 / FR5）

- 要件: FR19（決まりの部分を床の定数から導出する）/ FR5（構造の床）
- 条: P-12.1（承認は 1 つの対話面を通ったものだけ）/ P-5.6 / P-6.2 / P-10.1 / N-3.1
- 出所: 持ち主の裁定 2026-09-20 15:20 JST（逐語 orchestrator席に直してよい・裁定 id f2-648 notes 2026-09-20 15:20 JST）。設計判断を下す席の名が器の側で planner 席 から orchestrator 席 に替わった。設計文書の側（規則の表 R-8 の値・要件書 第 1.13 版の制約 CON5・語彙）は PR #170（main 615cebb）で着地済み。
- 根拠の判断: rules 行 D-11 の範囲の内（判定に使う値域を変える）= 根拠の裁定 id は上の 1 行。正本は実装の定数（判断の記録 ADR-9・ADR-11 決定 (3)(イ)）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bg が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

判断の記録の欄の決まり design-intent/adr/schema.yaml の schema の節は生成区間で、正本は crates/folio/src/adr.rs の床の木 FLOOR である（便 45）。承認者の値域は同じ file の定数 APPROVER（今は 持ち主 と planner 席 の 2 つ）で、床の木の enums.approver はこの定数を指し、発効した判断の承認欄の who をこの値域で数える。planner 席 という席はもう無く、今の席の名は orchestrator 席 である。本便は値域に orchestrator 席 を足し、注 2 つの席の名を直す。planner 席 は値域に残す = 凍結の場合 134 件（tests/floor_cases.yaml）が planner 席 を「値域の中の、持ち主でない承認者」などとして 8 行で使っており（owner-knob・approver-knob と、承認者や承認の写しを差し替える場合）、過去の記録の名でもあるため。

orchestrator 席の実測（2026-09-20・main 615cebb）: adr.rs は正規化 1,224 行（余地 276）・tests/schema.rs は 861 行。実の判断の記録 11 本の承認者は全部 持ち主 で、planner 席 を承認者に持つ実の記録は無い。判断の記録の欄の決まりの写しは fixture に 17 本在り（tests/fixtures/floor_base/design-intent/adr/schema.yaml と、tests/fixtures の下の場合ごとの置き場 16 本）、どれも approver の行が 1 行ずつ「approver: [持ち主, planner 席]」である。床は写しの schema の節の値を FLOOR と突き合わせる（注は突き合わせの外）ので、値域を変えたら 17 本の同じ行を同じ値にしないと、写しを使う歯が全部落ちる。発効後を模した写しの置き場で、下の (a)(b)(c) を当てて全部の歯を回した結果 = 落ちたのは 3 本だけ（tests/schema.rs の 2 本 = 判断の記録の側の byte 数と要約値の定数・tests/floor_cases.rs の 1 本 = 場合 approver-knob）で、ほかは緑。

(a) adr.rs を 3 か所だけ直す。
1. 定数 APPROVER を 3 つにする = 持ち主・planner 席・orchestrator 席（この順）。
2. 床の木の注 owner_note の後ろの文「planner 席は条文を改訂しない判断だけを承認できる」を「orchestrator 席（2026-09-19 までの名は planner 席）は条文を改訂しない判断だけを承認できる」に。
3. 床の木の注 enums_note の末尾の括弧の中の「R-8 = 持ち主と planner 席の対話面」を「R-8 = 持ち主と orchestrator 席の対話面」に。
ほかの値・注・検査の式は変えない（条文を改訂する判断の承認者が 持ち主 だけという N-4 の検査も不変）。

(b) 実の生成区間を書き直す = folio schema --write を実の置き場に当て、design-intent/adr/schema.yaml の生成区間の 3 行だけが変わる（生成区間の外・ほかの 3 本の file は 1 byte も変わらない）。orchestrator 席が独立の実装で組んだ凍結 anchor tests/fixtures/schema/adr-region.txt（117 行・22,254 byte・sha256 = 29e1188380852b143ca3db715348cc7a74220db206d284d988d66cde8f95d24b・本便の前に main に在る）と byte 一致するまで合わせる。anchor を書き換えて合わせてはいけない（変えるなら orchestrator 席へ問う）。

(c) 写し 17 本の approver の行を「approver: [持ち主, planner 席, orchestrator 席]」にする（各 file その 1 行だけ・ほかの行と注は触らない）。

(d) 凍結の場合 approver-knob（tests/floor_cases.yaml）の mutate の値を [持ち主, planner 席, 誰でも] から [持ち主, planner 席, orchestrator 席, 誰でも] に直す。理由 = この場合は「値域を data 側で広げられない」を確かめるもので、期待の文言は「schema.enums.approver が床の定数と違う」（数が違うときの字面）。値域が 3 つになると元の値は数が同じで 3 つ目だけ違う形になり、床は「schema.enums.approver[2] が床の定数と違う」と言う = 期待の文言と合わなくなる。広げる形（4 つ）に直せば、場合の意図も期待の文言も期待の終了の値も元のまま。ほかの 133 件と期待の欄は 1 字も変えない。

(e) 歯。
1. crates/folio/tests/schema.rs の判断の記録の側の定数を新しい値に上げる = REGION_BYTES を 22182 から 22254 に・REGION_SHA256 を (b) の値に（行数 117 は不変）。ほかの 3 本（15305・2915・1764）は不変。
2. 同じ file に 1 本足す（関数名は schema で始める）: 実の判断の記録の欄の決まりの生成区間が tests/fixtures/schema/adr-region.txt と byte 一致 ∧ approver の行に orchestrator 席 を含む。
3. src/adr.rs の unit test に 1 本足す（関数名は adr_floor で始める・今この語で始まる歯は無い）: schema.rs の derive に FLOOR を渡した結果が adr-region.txt と byte 一致。関数 derive は crates/folio/src/schema.rs に既に pub の関数として在り、同じ形の unit test が既に 3 本それを呼んでいる（ceiling.rs・rules.rs・note.rs）= schema.rs は変えない。
4. src/adr.rs の unit test に 1 本足す（関数名は adr_floor で始める）: 値域 APPROVER が 持ち主・planner 席・orchestrator 席 の 3 つをこの順で持つ。
5. tests/floor_cases.rs の歯 floor_cases_all_pass_with_folio（本文は不変）が 134 / 134 で緑 = (c) と (d) を確かめる。
6. 回帰（期待不変・共通の検証が回す）: tests/adr.rs・tests/check.rs・tests/link.rs・tests/anchor.rs・tests/face_adr.rs（判断の記録の面は値域を出さない = 面の凍結の fixture は不変）。

(f) 大きさと接続。新規 file は無い（anchor は本便の前に orchestrator 席が main へ置く）。既存 = adr.rs（3 行の直しと歯 約 +25 行）・design-intent/adr/schema.yaml（生成区間の 3 行）・写し 17 本（各 1 行）・tests/floor_cases.yaml（1 行）・tests/schema.rs（約 +25 行）。tests/floor_cases.rs は本文不変（検証の範囲に入るので write-set に置く）。size S。ほかの設計文書・ほかの fixture・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: 値域と注 2 つ・実の生成区間の書き直し・写し 17 本の 1 行・凍結の場合 1 件の値・歯。
- 入れない: 過去の承認欄と判断の記録の本文の planner 席（当時の記録）・判断の記録の欄の決まりの meta の author・規則の表と要件書と語彙（PR #170 で着地済み）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| approver | 値域 | adr.rs の APPROVER に orchestrator 席 を足す・注 2 つの席の名 |
| region | 生成区間 | adr/schema.yaml を folio schema --write で書き直す（anchor と byte 一致） |
| copies | 写し | fixture 17 本の approver の行と凍結の場合 approver-knob の値 |
| teeth | 歯 | schema の定数 2 つ・anchor の一致 2 本・値域の 1 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bg"
title = "判断の記録の承認者の値域 APPROVER に orchestrator 席 を足して 3 つにし（planner 席 は凍結の場合と過去の記録のために残す）、床の木の注 owner_note と enums_note の席の名を直す（adr.rs + 実の生成区間を folio schema --write で書き直す・凍結 anchor 22,254 byte と byte 一致 + 写し 17 本の approver の行 + 凍結の場合 approver-knob の値・席の呼び名の裁定 2026-09-20 15:20 JST）"
req = ["FR19", "FR5"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "design-intent/adr/schema.yaml", "crates/folio/tests/schema.rs", "crates/folio/tests/floor_cases.rs", "tests/floor_cases.yaml", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo nextest run -p folio adr_floor", "cargo nextest run -p folio --test floor_cases floor_cases", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の歯（判断の記録の側 22254 byte と新しい sha256・実の生成区間が anchor と byte 一致で orchestrator 席 を含む・ほかの 3 本は不変）が緑、adr_floor の歯 2 本（床の木の導出が anchor と byte 一致・値域が 3 つ）が緑、floor_cases の歯が 134 / 134 で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
