# 設計: 便 48 — 天井の正本の生成区間を folio schema の対象に足し、旧 4 節の受け口を消して床を締める（ADR-11 決定 (4)① の 3 本目・FR19 / FR18）

- 要件: FR19（第 1.9 版 = 対象に天井の正本）/ AC17 / FR18（所見の形と天井の 3 値を床で数える）
- 条: P-5.6（写しを設計文書の置き場へ導出し続ける）/ P-6.2・P-6.3・P-6.4 / P-10.1（凍結 anchor は生成側から独立）/ P-14.2 と同じ型の検査 / N-3.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)(ウ) と (4)①。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)①。値は 1 つも変えない。
- 位置（3 段の 3 本目）: 便 47（f2-648.69・着地 0654d9e）= 床の木 FLOOR と受け皿 → 設計判断の席の PR = 天井の正本 第 3 版（生成区間を足し旧 4 節を外す・持ち主の承認）・要件書 第 1.9 版・最小の写しの fixture 19 本と凍結した土台を第 3 版の形に・tests/ceiling.rs の針 7 本を版に依らない形に → 本便 = 締め。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 aw が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

便 47 で crates/folio/src/ceiling.rs に床の木 FLOOR（凍結 anchor tests/fixtures/schema/ceiling-region.txt と byte 一致）と、最上位の節 schema の受け皿が入った。設計判断の席の PR で、実の天井の正本 design-intent/ceiling.yaml は第 3 版の形（人が書くのは meta・weights・documents・viewpoints・末尾に印で挟んだ生成区間 schema・旧 4 節 verdicts / finding / record / bundle は無い）になり、tests/fixtures 以下の天井の正本の写し 19 本と凍結した土台 tests/fixtures/floor_base/design-intent/ceiling.yaml も同じ形になっている。main には旧 4 節を持つ天井の正本はもう 1 本も無い。本便は (1) 命令 folio schema の対象に天井の正本を足し、(2) 移行のあいだだけ残していた旧 4 節の受け口を消して、生成区間 schema を必須にする。値・床の木・体裁の規則・設計文書・fixture は 1 字も変えない。

設計判断の席の実測（2026-09-20・main 0654d9e に第 3 版の PR を重ねた状態）: crates/folio/src/schema.rs（正規化 464・余地 1,036）の対象の一覧 TARGETS は 2 本（判断の記録の欄の決まり adr/schema.yaml と adr.rs の FLOOR・設計ノートの欄の決まり design-note/schema.yaml と note.rs の FLOOR）。関数 run は TARGETS を順に見て file ごとに標準出力 1 行を足し、最初に合格でない file でそこで返す。印は 1 file に 1 対・生成区間の 1 行目は schema の見出しで、天井の正本の生成区間もこの前提のまま扱える（末尾に 1 対）。crates/folio/src/ceiling.rs（正規化 633・余地 867）は定数 LEGACY_SECTIONS（旧 4 節の名）と、旧 4 節を「schema が在れば無くてもよく、在れば今までどおり数える」関数 legacy と、その中で使う突き合わせの関数（same_list・contains_all と、旧 4 節だけが使う分の same_set の呼び出し）を持つ。観点の id の列と読む文書の id の集合の検査（documents と viewpoints の行に対する same_list / same_set）は旧 4 節ではなく、残す。

(a) schema.rs の TARGETS に 1 行足す = 天井の正本 ceiling.yaml と ceiling.rs の FLOOR。順は 3 本目（判断の記録 → 設計ノート → 天井の正本）。ほかの関数は変えない。

(b) ceiling.rs の締め。定数 LEGACY_SECTIONS と関数 legacy と、旧 4 節の中身の検査（verdicts の values・finding の required と place と refute・record の required・bundle の contents と digest）を消す。消した結果どこからも呼ばれなくなる関数（contains_all など）も消す。最上位の節の許す一覧は FLOOR の top_level の 5 語だけにする = 旧 4 節の名を持つ天井の正本は「未知の節」の違反で落ちる（今の未知の節の文言のまま）。最上位の節 schema は必須にする = 無ければ種別 ceiling の違反 1 件（文言は「ceiling.yaml: schema の節が無い」で始める）。schema が在るときの突き合わせ（floor_diff・文言「ceiling.yaml: 床の定数と違う: schema.」に道）は便 47 のまま。weights・documents・viewpoints・meta の検査は変えない。ceiling.rs は縮む見込みだが、余地が十分なので - の接頭辞は付けない。

(c) 凍結 anchor（P-10.1）。実の天井の正本の生成区間は、設計判断の席が独立の実装で組んだ byte である = 24 行・2,915 byte・sha256 = 5ad2f19b7d8c4a197865c5280f4c82653f97ef6dc0b9b45186a2678539429fa2（begin の次の行から end の手前の行まで・tests/fixtures/schema/ceiling-region.txt と同じ byte）。作業者は folio schema --check が実の置き場に対して 0 を返し、標準出力が 3 行（判断の記録の側 22182 byte・設計ノートの側 14618 byte・天井の正本 2915 byte）になるまで合わせる。天井の正本や anchor の file を書き換えて合わせてはいけない（変えるなら設計判断の席へ問う）。

(d) 歯。
1. crates/folio/tests/schema.rs に足す（関数名は schema で始める・既存の歯の期待のうち対象の本数を数える定数は 2 から 3 に上げ、ほかは不変）: 実の正本の写しに --check → 0 ∧ 標準出力が 3 行 ∧ 3 行目に ceiling.yaml と 2915 byte ∧ 天井の正本の生成区間を sha256sum で測り直して (c) の値と同じ ∧ 行数 24。
2. 同じ file: 写しの天井の正本の生成区間の 1 byte を書き換えて --check → 1 ∧ ceiling.yaml ∧「≠ 導出」。
3. 同じ file: 天井の正本の begin の印を消す → 2 ∧「ceiling.yaml: 印が 1 対でない」。
4. 同じ file: 歯 2 の写しに --write → 0 ∧ 天井の正本の全体が元と byte 一致（人が書く節も不変）。
5. crates/folio/tests/ceiling.rs: 便 47 の歯 ceiling_without_legacy_sections_and_schema_fails は、schema の節を外すと種別 ceiling の違反がちょうど 1 件で文言に「schema の節が無い」を含む、に締める（関数名は変えない）。旧 4 節の名の節を 1 つ足すと「未知の節」の違反 1 件になる歯を 1 本足す（関数名は ceiling で始める）。第 3 版の形を作る補助の関数（旧 4 節を外す・anchor を足す）は、実の正本が既に第 3 版なので何もしない形のまま残してよく、消してもよい。
6. 回帰（期待不変）: tests/ceiling.rs の残りの歯・tests/bundle.rs・tests/findings.rs・tests/floor_cases.rs（凍結の場合 134 件）・実の正本を写す歯の全部。共通の検証が回す。

(e) 大きさと接続。新規 file は無い。既存 = schema.rs（+1 行）・ceiling.rs（縮む）・tests/schema.rs（約 +100 行・余地 970）・tests/ceiling.rs（約 +20 行・余地 960 前後）。size S = 既存 file 1 本あたりの増分は 100 行に収まる見積。bundle.rs・findings.rs・main.rs・設計文書・fixture・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: TARGETS に天井の正本・旧 4 節の受け口の削除と schema の必須化・歯（schema +4・ceiling の締め 1 と追加 1）。
- 入れない: 値と床の木と体裁の規則の変更・設計文書と fixture の変更・読む文書の閉じた集合の変更（直前の版の写しは別の便）・入口や要件書ほかの生成区間（ADR-11 決定 (4) の ④ 以降）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| target | 対象 | schema.rs の TARGETS に ceiling.yaml と ceiling.rs の FLOOR |
| close | 締め | LEGACY_SECTIONS・legacy・旧 4 節の検査を消し、schema を必須に |
| anchor | 凍結 | 実の天井の正本の生成区間（2,915 byte・sha256 5ad2f19b…） |
| teeth | 歯 | schema +4・ceiling 締め 1 + 追加 1・ほかは期待不変 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "aw"
title = "天井の正本の生成区間を folio schema の対象に足し（TARGETS に 1 行・実の正本は凍結 anchor と byte 一致）、ceiling.rs の旧 4 節の受け口を消して生成区間 schema を必須にする（値と床の木と設計文書と fixture は不変・ADR-11 決定 (4)① の締め）"
req = ["FR19", "FR18"]
section = "1"
write-set = ["crates/folio/src/schema.rs", "crates/folio/src/ceiling.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/ceiling.rs"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo nextest run -p folio --test ceiling ceiling", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の歯（天井の正本の一致と要約値と行数・ずれ・印・書き直し・既存は対象の本数だけ 3 に上げて期待不変）が緑、ceiling の歯（schema の節が無ければ違反 1 件・旧 4 節の名は未知の節で落ちる・残りは期待不変）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
