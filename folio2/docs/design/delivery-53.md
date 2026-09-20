# 設計: 便 53 — 規則の表の生成区間を命令 folio schema の 4 本目の対象に足す（ADR-11 決定 (4)② の 3 本目の 3 段目・FR19）

- 要件: FR19（第 1.10 版 = 対象に規則の表）/ AC17
- 条: P-5.6（写しを設計文書の置き場へ導出し続ける）/ P-6.2・P-6.3・P-6.4 / P-10.1 / P-14.2 と同じ型の検査
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ) と (4)②。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)②。値は 1 つも変えない。
- 位置（3 段の 3 段目）: 便 51（f2-648.81・着地 87d1c18）= 床の木 FLOOR（crates/folio/src/rules.rs）と、床が定数から読む形 → 設計判断の席の PR = 実の rules.yaml の schema の節を生成区間に替える・要件書 第 1.10 版（持ち主の承認）→ 本便 = 命令の対象に足す。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bb が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

便 51 で crates/folio/src/rules.rs に規則の表の欄の決まりの節の床の木 FLOOR が入り、その導出（crates/folio/src/schema.rs の関数 derive）が凍結 anchor tests/fixtures/schema/rules-region.txt と byte 一致することを unit test が数えている。設計判断の席の PR で、実の規則の表 design-intent/rules.yaml の schema の節は、印で挟んだ生成区間（begin の印の次の行が schema の見出し・end の印まで）に替わっている。本便は命令 folio schema の対象に規則の表を足し、実の file の生成区間と床の木の導出の byte 一致を、命令と歯が数え続けるようにする（P-5.6）。値・床の木・体裁の規則・設計文書・fixture は変えない。

設計判断の席の実測（2026-09-20・main b2faebb に第 1.10 版の PR を重ねた状態）: schema.rs（正規化 466 行）の対象の一覧 TARGETS は 3 本（判断の記録の欄の決まり adr/schema.yaml と adr.rs の FLOOR・設計ノートの欄の決まり design-note/schema.yaml と note.rs の FLOOR・天井の正本 ceiling.yaml と ceiling.rs の FLOOR）。関数 run は TARGETS を順に見て file ごとに標準出力 1 行を足し、最初に合格でない file でそこで返す。印は 1 file に 1 対で、生成区間の位置は問わない（関数 region_of）= 規則の表の生成区間は file の先頭の注釈の次に在るが、この前提のまま扱える。crates/folio/tests/schema.rs（正規化 679 行・余地 821）の歯は全部、実の設計文書の置き場の写し（一時 dir・git の 1 commit）に命令を当てる形で、命令 folio schema を呼ぶ歯の file はこの 1 本だけである。

(a) schema.rs の TARGETS に 1 行足す = 規則の表 rules.yaml と rules.rs の FLOOR。順は 4 本目（判断の記録 → 設計ノート → 天井の正本 → 規則の表）。ほかの関数は変えない。

(b) 凍結 anchor（P-10.1）。実の規則の表の生成区間は、設計判断の席が独立の実装で組んだ byte である = 27 行・1,764 byte・sha256 = dcf207ced150b5ebd3d6ae03bcdb5bc42ef9a06d6f74ff3f830ff96729dafad2（begin の次の行から end の手前の行まで・tests/fixtures/schema/rules-region.txt と同じ byte）。作業者は folio schema --check が実の置き場に対して 0 を返し、標準出力が 4 行（判断の記録の側 22182 byte・設計ノートの側 14618 byte・天井の正本 2915 byte・規則の表 1764 byte）になるまで合わせる。規則の表や anchor の file や rules.rs の床の木を書き換えて合わせてはいけない（変えるなら設計判断の席へ問う）。

(c) 歯。crates/folio/tests/schema.rs に足す（関数名は schema で始める・既存の歯の期待のうち対象の本数を数える所は 3 から 4 に上げ、ほかは不変）。
1. 実の正本の写しに --check → 0 ∧ 標準出力が 4 行 ∧ 4 行目に rules.yaml と 1764 byte ∧ 規則の表の生成区間を sha256 で測り直して (b) の値と同じ ∧ 行数 27。
2. 写しの規則の表の生成区間の 1 byte を書き換えて --check → 1 ∧ rules.yaml ∧「≠ 導出」。
3. 規則の表の begin の印を消す → 2 ∧「rules.yaml: 印が 1 対でない」。
4. 歯 2 の写しに --write → 0 ∧ 規則の表の全体が元と byte 一致（人が書く行 thresholds と discipline も先頭の注釈も不変）。
5. 回帰（期待不変）: tests/schema.rs の残りの歯・src/rules.rs の unit test・tests/check.rs・tests/floor_cases.rs（凍結の場合 134 件）・実の正本を写す歯の全部。共通の検証が回す。

(d) 大きさと接続。新規 file は無い。既存 = schema.rs（+1〜3 行）・tests/schema.rs（約 +90 行）。size S。rules.rs・check.rs・main.rs・設計文書・fixture・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: TARGETS に規則の表・歯（schema +4・既存は対象の本数だけ 4 に）。
- 入れない: 値と床の木と体裁の規則の変更・設計文書と fixture の変更・面の生成器の規則の表の側の表（RULE_KIND・RULE_STATUS = 後続の小さな便）・床が規則の表の schema の節を床の木と突き合わせること（写しの fixture 25 本を直さないため入れない）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| target | 対象 | schema.rs の TARGETS に rules.yaml と rules.rs の FLOOR |
| anchor | 凍結 | 実の規則の表の生成区間（1,764 byte・sha256 dcf207ce…） |
| teeth | 歯 | schema +4・ほかは期待不変 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bb"
title = "規則の表の生成区間を folio schema の 4 本目の対象に足す（TARGETS に 1 行・実の規則の表は凍結 anchor と byte 一致）・歯は一致と要約値と行数・ずれ・印・書き直し（値と床の木と設計文書と fixture は不変・ADR-11 決定 (4)② の 3 本目の締め）"
req = ["FR19"]
section = "1"
write-set = ["crates/folio/src/schema.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の歯（規則の表の一致と要約値と行数・ずれ・印・書き直し・既存は対象の本数だけ 4 に上げて期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
