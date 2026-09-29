# 設計: 便 46 — 設計ノートの欄の決まりの生成区間も folio schema の対象にし、説明の注 27 本を note.rs の FLOOR へ・床の機械は schema.rs のものを使う（ADR-9・FR19 / AC17）

- 要件: FR19（欄の決まりの file の決まりの部分を床の定数から導出する）/ FR9（設計ノートを 1 つの型で生成する = 設計ノートの欄の決まり）/ FR5（3 値）
- 条: P-6.2・P-6.3・P-6.4 / P-14.2 / P-5.5（生成区間の外 = 先頭の欄・対応表・平易文は書き換えない）/ P-10.1（凍結 anchor は生成側から独立）/ N-3.1
- 判断の記録: ADR-9（発効 2026-09-19）。本便は 2 便のうち 2 本目（設計ノートの側）。1 本目 = 便 45（f2-648.61・着地 9954cfa・命令 folio schema と schema.rs と判断の記録の側）。
- 裁定: 持ち主 2026-09-19「どちらも承認する」（ADR-9 の発効）・「承認する」（要件書 v1.7 = FR19 / AC17）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 au が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い・note.rs は縮む file なので - の接頭辞）。

## 1. 目的と中身

便 45 で命令 folio schema（--write / --check）と、床の機械を共有する file crates/folio/src/schema.rs が着地した。対象の一覧（schema.rs 29 行の TARGETS）はまだ判断の記録の欄の決まり 1 本だけで、設計ノートの側（crates/folio/src/note.rs）は床の木の型 Floor・マクロ keys_floor・関数 strip_notes・関数 floor_diff の自分の写しを持ち続けている。本便は (1) note.rs の自分の写し 4 つを消して schema.rs のものを使い、(2) 設計ノートの欄の決まりの説明の注（名が _note で終わる欄 27 本）を note.rs の FLOOR に file に在る順と字面のまま足し、(3) TARGETS に design-note/schema.yaml を足して folio schema が 2 本を順に見る形にする。folio check の設計ノートの検査（欄単位の突き合わせ）と床の出す文言は 1 字も変えない。正本 design-intent/design-note/schema.yaml は write-set の外（planner が書いた byte が凍結 anchor）。

planner の実測（2026-09-19・main 9954cfa）: schema.rs（正規化 462・余地 1,038）は crate の中へ公開した Floor（5 変種 Val / Num / Strs / Seq / Map・Seq は「便 46 で寄せる設計ノートの側が使う」の注つきで dead_code の許可が付いている）・keys_floor・strip_notes・floor_diff（FLOOR 側の名が _note で終わるキーを和集合から除く・表のどの深さでも同じ関数を通る）・印の定数 BEGIN / END・導出 derive・区間 region_of・run を持つ。run は TARGETS を順に見て、file ごとに標準出力 1 行を足し、最初に 合格 でない file が出たらそこで返す。note.rs（正規化 1,349・余地 151）の写しの所在 = Floor 36 行から・keys_floor 139 行から・FLOOR 149 行から（非公開）・strip_notes 562 行から・floor_diff 576 行から。note.rs の FLOOR は真偽を Val の字面（true / false）で持つ。床の違反の文は「design-note/schema.yaml: 床の定数と違う: schema.<道>」で、道の末尾の飾り 3 種は schema.rs の floor_diff が同じ字面で出す。歯 crates/folio/tests/note.rs は 24 本（関数名は全部 note で始まる）で、うち 1 本は実の design-note/schema.yaml の写しの字面「teeth-table, contract-table]」を変異の針に持つ（今の生成区間の体裁で 1 か所に在る）。歯 crates/folio/tests/schema.rs は 10 本（正規化 382）。

(a) note.rs の直し: 自分の Floor・keys_floor・strip_notes・floor_diff を消し、schema.rs のものを use する（約 −98 行）。FLOOR を crate の中へ公開にし、_note の欄 27 本を足す（約 +62 行・葉 30・最深は表 4 段の下）。差し引きで縮む = write-set は - の接頭辞。足す位置と字面は実の file の生成区間のとおりで、値の欄の順も file の順に揃える（floor_diff は和集合を並べ替えて比べるので、順を変えても床の違反の並びは変わらない）。真偽の値は今のとおり Val の字面で持つ（導出は裸の true / false を出す）。schema.rs の Floor の Seq から dead_code の許可を外す。

(b) 導出の体裁の規則は便 45 の設計文書 §1 (c) と同じ 5 つで、schema.rs に実装済み（W = 100・文字列と数の一覧は flow が W 以内なら flow さもなくば block・表は子が全部 値かその一覧で flow が W 以内なら flow さもなくば block・表の一覧は各項 flow で W を超える項だけ block・値は構造の条件に当たるときだけ単引用符）。設計ノートの側で初めて使う形 = 表の一覧（landing の verdict_cases・4 項）と 4 段の入れ子。規則そのものは変えない。導出が実の file と食い違ったら、まず FLOOR の字面と順を疑い、規則の実装に手を入れるのは表の一覧と深い入れ子の扱いが規則の文と違っていた場合だけにする（判断の記録の側の生成区間 22,182 byte が変わらないことを歯が見る）。

(c) 凍結 anchor（P-10.1）: main の design-intent/design-note/schema.yaml の生成区間は planner が独立の Python で組み、admin も別の実装で byte 一致を確かめた。生成区間 = 135 行・14,618 byte・sha256 = cae43ed2765884f8593aff4925ffae3cc0e69a0e18d376160d615853f437ce8e（begin の次の行から end の手前の行まで・sha256sum で実測）。作業者は folio schema --check が実の置き場に対して 0 を返し、標準出力が 2 行（判断の記録の側 22182 byte・設計ノートの側 14618 byte）になるまで合わせる。design-note/schema.yaml の側を書き換えて合わせてはいけない（変えるなら planner へ問う）。

(d) 歯（既存の file crates/folio/tests/schema.rs に足す・関数名は schema で始める・既存 10 本の期待は不変）:
1. 実の正本: design-intent の写しに --check → 0 ∧ 標準出力が 2 行 ∧ 2 行目に「design-note/schema.yaml」と「14618 byte」∧ 設計ノートの側の生成区間を sha256sum で測り直して (c) の値と同じ。
2. 設計ノートの側のずれ: 写しの design-note/schema.yaml の生成区間の 1 byte を書き換えて --check → 1 ∧「design-note/schema.yaml」∧「≠ 導出」。
3. 設計ノートの側の印: begin を消す → 2 ∧「design-note/schema.yaml: 印が 1 対でない」。
4. 書き直し: 歯 2 の写しに --write → 0 ∧ design-note/schema.yaml の全体が元と byte 一致（対応表と平易文も不変）。
既存の歯は期待不変で回帰に回す: tests/note.rs の 24 本（針の歯を含む・本文は変えない・verify の置き場として write-set に載せる）。

(e) 便 45 までの形との接続: 新規 file は無い。既存 = note.rs（縮む）・schema.rs（TARGETS に 1 行 + Seq の許可を外す・数行）・tests/schema.rs（+4 本・約 +100 行・余地 1,118）・tests/note.rs（本文不変）。adr.rs・main.rs・正本 2 本・fixture・floor_cases.yaml・CI の yml は触らない。size S = 既存 file 1 本あたりの増分の見積は tests/schema.rs の約 +100 行が最大（note.rs は縮む宣言なので余地を測らない）。外部 crate は増やさない。

## 2. 範囲

- 入れる: note.rs の床の機械の写しを消して schema.rs を使う・FLOOR に説明の注 27 本・TARGETS に設計ノートの側・歯 4 本。
- 入れない: 体裁の規則の変更・folio check の検査と文言の変更・正本と fixture の変更・CI の yml の変更。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| floor | 床の機械 | note.rs の写し 4 つを消し schema.rs のものを使う |
| notes | 説明の注 | note.rs の FLOOR に 27 本（file の順と字面のまま） |
| target | 対象 | schema.rs の TARGETS に design-note/schema.yaml |
| anchor | 凍結 | main の design-note/schema.yaml の生成区間（14,618 byte・sha256 cae43ed2…） |
| teeth | 歯 | tests/schema.rs +4 本・note 24 本と schema 10 本は期待不変 |

## 4. 検査（歯）

§1 (d) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "au"
title = "設計ノートの欄の決まりの生成区間も folio schema の対象にし（TARGETS に 1 行）、説明の注 27 本を note.rs の FLOOR へ・note.rs の床の機械の写しを消して schema.rs のものを使う（folio check の検査と文言は不変・note.rs は縮む）"
req = ["FR19", "FR9", "FR5"]
section = "1"
write-set = ["-crates/folio/src/note.rs", "crates/folio/src/schema.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/note.rs"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo nextest run -p folio --test note note", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の歯（設計ノートの側の一致と要約値・ずれ・印・書き直し・既存 10 本は期待不変）が緑、便 23 以降の歯 note 24 本が期待不変で緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
