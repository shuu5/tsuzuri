# 設計: 便 57 — 設計ノートの欄の決まりの注 4 つに「未実装」を正直に書く（床の木の注と生成区間・天井の 4 周目の実態 F-1・FR19 / FR9）

- 要件: FR19（決まりの部分を床の定数から導出する）/ FR9（設計ノートを 1 つの型で生成する）
- 条: P-4.1・P-4.2（無い検査を在ると言わない・判定できないものは「まだ分からない」）/ P-5.6 / P-6.2 / P-10.1
- 出所: 天井の 4 周目（2026-09-20・main 0816727）の止める所見 実態 F-1（反証 支持）= 設計ノートの欄の決まり design-intent/design-note/schema.yaml の生成区間の注が「契約表について folio2 が持つ検査は 3 つだけ」と現在形で言い、その 1 つ 導出物の差分 0（要件書 FR11）は口が版管理のどこにも無い。
- 根拠の判断: rules 行 D-11 の範囲の外（説明の注の文言だけ・値域と欄の集合は変えない）。注も床の木 FLOOR の値なので、直す先は実装の定数（ADR-9・ADR-11 決定 (3)(イ)）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bf が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

設計ノートの欄の決まり design-intent/design-note/schema.yaml の schema の節は生成区間で、正本は crates/folio/src/note.rs の床の木 FLOOR である（便 46）。その注（_note で終わる欄・人が読む説明）のうち 4 つが、まだ実装の無い口を在るものとして書いている = (1) contract_table の folio_check_note「契約表について folio2 が持つ検査は 3 つだけ…導出物の差分 0（FR11…）」(2) derived の derived_note（導出物の形と検査の説明）(3) index の index_note（機械が読む id の索引・FR14）(4) guards の guards_note（post の一覧に derived-diff-zero が在る）。実測（main 0816727）= 導出物を組む口も差分を数える口も索引を出す口も crates/folio/src に無く、folio build --check は配信先の byte 一致だけを見る。要件書 FR11 と FR14 は M1 の要件で、便はまだ起きていない。本便は、この 4 つの注の末尾に「未実装である」ことと、それまでの扱いを書き足す。値の欄（folio_check の一覧・derived の形・guards の一覧・index の形）は決めた形の記録なので変えない。

設計判断の席の実測（2026-09-20・main 0816727）: note.rs は正規化 1,321 行（余地 179）。注は床の木の中で Floor の値として 1 葉 1 行で書かれている。実の設計ノートの欄の決まりの生成区間は今 135 行・14,618 byte で、crates/folio/tests/schema.rs が定数（行数でなく byte 数 14618 と sha256）で固定している。設計ノートの欄の決まりの写しは fixture に 1 本も無い（実の 1 本だけ）。注は床の突き合わせ（floor_diff）の外で、床の結果には影響しない。

(a) note.rs の FLOOR の注 4 つの末尾に、次の文を 1 字も変えずに足す（前の文との間の句点「。」から始まる）。
1. folio_check_note の末尾へ:「。このうち導出物の差分 0（derived-diff-zero）は未実装である = 要件書 FR11 の便が入るまで folio はこの検査を回さず、folio build --check もこれを数えない（その間この検査の結果は「まだ分からない」として扱う・P-4.2）」
2. derived_note の末尾へ:「。導出物を組む口と差分を数える口は未実装である（要件書 FR11 の便で入る・それまで derived の節は決めた形の記録）」
3. index_note の末尾へ:「。索引を出す口は未実装である（要件書 FR14 の便で入る）」
4. guards_note の末尾へ:「。post のうち derived-diff-zero は未実装である（要件書 FR11 の便が入るまで、極性一覧へは「まだ無い検査」として寄せる）」
note.rs の頭の説明に残る「散文の門（FR12・rules 行 R-16）と導出物（FR11）は本便に入れない。」は、散文の門が便 24 で入った今の実態に合わせて「導出物（FR11）と索引（FR14）は未実装」に直す。

(b) 実の設計ノートの欄の決まりの生成区間を書き直す = folio schema --write を実の置き場に当て、design-intent/design-note/schema.yaml の生成区間だけが変わる（生成区間の外・ほかの 3 本の file は 1 byte も変わらない）。設計判断の席が独立の実装で組んだ凍結 anchor tests/fixtures/schema/note-region.txt（135 行・15,305 byte・sha256 = 836e07fadc3e32b897e975cab8454aacb4ca02507e992d2979b332c4d96a019f）と byte 一致するまで合わせる。anchor を書き換えて合わせてはいけない（変えるなら設計判断の席へ問う）。

(c) 歯。
1. crates/folio/tests/schema.rs の設計ノートの側の定数を新しい値に上げる = 14618 を 15305 に・sha256 を (b) の値に。標準出力の 2 行目の期待「14618 byte」も 15305 に。ほかの 3 本（22182・2915・1764）は不変。
2. 同じ file に 1 本足す（関数名は schema で始める）: 実の設計ノートの欄の決まりの生成区間が tests/fixtures/schema/note-region.txt と byte 一致 ∧ 注 folio_check_note の行に「未実装である」を含む。
3. src/note.rs の unit test に 1 本足す（関数名は note_floor で始める）: schema.rs の derive に FLOOR を渡した結果が note-region.txt と byte 一致。関数 derive は crates/folio/src/schema.rs に既に pub の関数として在り（床の木を受けて生成区間の字面を返す）、同じ形の unit test が既に 2 本それを呼んでいる（crates/folio/src/ceiling.rs の ceiling_floor_derives_the_frozen_anchor_byte_for_byte と crates/folio/src/rules.rs の rules_floor_derives_the_frozen_anchor_byte_for_byte）= schema.rs は変えない。
4. 回帰（期待不変・共通の検証が回す）: tests/note.rs・tests/face_note.rs（設計ノートの面は欄の決まりの注を出さない = 面の凍結の fixture は不変）・tests/check.rs・tests/floor_cases.rs（凍結の場合 134 件）。

(d) 大きさと接続。新規 file は無い。既存 = note.rs（注 4 行の書き足しと頭の説明 = 約 +10 行・歯 約 +15 行）・design-intent/design-note/schema.yaml（生成区間の 4 行）・tests/schema.rs（約 +25 行）。size S。値の欄・床の検査・ほかの設計文書・fixture（anchor は本便の前に設計判断の席が置く）・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: note.rs の注 4 つと頭の説明・実の生成区間の書き直し・歯。
- 入れない: FR11・FR13・FR14 の実装（器の側の口が揃ってから別の便）・値の欄の変更・要件書の側の「未実装」の注（設計判断の席の PR・持ち主の承認）。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| notes | 注 | note.rs の FLOOR の注 4 つに未実装を明記 |
| region | 生成区間 | design-note/schema.yaml を folio schema --write で書き直す（anchor と byte 一致） |
| teeth | 歯 | schema の定数 2 つ・anchor の一致 2 本 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bf"
title = "設計ノートの欄の決まりの注 4 つ（folio_check_note・derived_note・index_note・guards_note）の末尾に、導出物の差分 0 と索引の口が未実装であることを書き足す（note.rs の床の木の注 + 実の生成区間を folio schema --write で書き直す・凍結 anchor 15,305 byte と byte 一致・値の欄は不変・天井の 4 周目の実態 F-1）"
req = ["FR19", "FR9"]
section = "1"
write-set = ["crates/folio/src/note.rs", "design-intent/design-note/schema.yaml", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test schema schema", "cargo nextest run -p folio note_floor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "schema の歯（設計ノートの側 15305 byte と新しい sha256・実の生成区間が anchor と byte 一致で「未実装である」を含む・ほかの 3 本は不変）が緑、note_floor の歯（床の木の導出が anchor と byte 一致・schema.rs の既に pub の関数 derive を呼ぶので schema.rs は変えない）が緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
