# 設計: 便 54 — 規則の表の種別と状態の値域を rules.rs の型 1 つずつに寄せ、面の生成器の表 2 枚の鍵の列を消す（ADR-11 決定 (4)② の締め・FR4 / NFR2 / FR19）

- 要件: FR4（面は正本から生成する）/ NFR2（型の閉じた一覧）/ FR19（規則の表の生成区間は byte 不変）
- 条: P-5.1・P-5.6 / P-6.3・P-6.4 / P-10.1
- 判断の記録: ADR-11（発効 2026-09-20）決定 (3)(イ)「実装の型付きの定数が正本」と (4)②・(4)⑧「実装の中で同じ一覧を 2 枚以上持つ箇所は 1 枚に寄せる」。rules 行 D-11 の作法 = 本便の根拠は ADR-11 決定 (4)②。値は 1 つも変えない。
- 位置: 規則の表の 3 段（便 51 = 床の木・第 1.10 版の PR = 実の file の生成区間・便 53 = 命令の対象）の後の締め。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 bc が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（Declared 形・新規は無い）。

## 1. 目的と中身

規則の表の値域のうち種別（deny・build-check・detect・human-review）と状態（仮・凍結・未定）は、ADR-11 決定 (3)(イ) により実装の型付きの定数が正本で、crates/folio/src/rules.rs の定数 RULE_KIND と RULE_STATUS が床の木 FLOOR を通して実の rules.yaml の生成区間へ出ている（便 51・53）。ところが同じ値の字面が、実装の中にあと 2 か所ずつ手書きで在る = (1) 面の生成器の表 crates/folio/src/face.rs の RULE_KIND（種別と名札の対 4 行）と RULE_STATUS（状態と見た目の class の対 3 行・行の順は 凍結・仮・未定 で正本の順と違う）、(2) rules.rs の床の木の中の表の鍵（kind_meaning の鍵 4 つと kind_map_to_constitution の鍵 3 つ）。表引きは片方向で、値域に値が足されても面の表は黙って古いままになる。本便は値域を型 1 つずつに寄せ、ほかの所はその型から出す。面の出力の byte・Err の文言・実の rules.yaml・生成区間の byte は変えない。

設計判断の席の実測（2026-09-20・main c9d9054）: rules.rs は正規化 198 行・face.rs は 1,305 行（余地 195）・face_constitution.rs は 1,164 行。face.rs の 2 枚の表を使う所は face_constitution.rs の 3 か所だけ（種別 2 か所・状態 1 か所・X の関数 lookup・Err の文言は 道・名・「の表に無い値」・値）。表を順に回す所は無い = 行の順が正本と違っても面の出力には出ていない。便 50 が X に足した口 parse（型の from_name を受ける・Err の文言は lookup と同じ）が在る。rules.rs の unit test rules_floor_derives_the_frozen_anchor_byte_for_byte は、床の木の導出が凍結 anchor tests/fixtures/schema/rules-region.txt と byte 一致することを見ている。

(a) rules.rs に型 2 つを置く = 種別の型と状態の型（名は RuleKind と RuleStatus・Debug・Clone・Copy・PartialEq・Eq）。型ごとに 定数 ALL・定数 NAMES（値の字面の列・順は今の RULE_KIND と RULE_STATUS の順 = 生成区間の順）・関数 name（const fn）・関数 from_name を持つ（憲法の値域から導出した型と同じ形）。ここが値域の字面を書く唯一の所になる。定数 RULE_KIND と RULE_STATUS は NAMES を指す形にするか、無くして NAMES を直に使う。床の木の中の表の鍵（kind_meaning の 4 つ・kind_map_to_constitution の 3 つ）は型の各値の name で書く。説明の文と順は変えない = 生成区間は byte 不変。

(b) face.rs の表 RULE_KIND と RULE_STATUS を消し、型の値を受けて名札（種別）と見た目の class（状態）を返す関数 2 つに替える。関数の中は型の値の全部を並べた場合分けで、その他を受ける枝を置かない = 値域に値が足されれば組み立てが通らない。face_constitution.rs の 3 か所は X の口 parse で型へ引いてから名札を引く。Err の文言は変えない。

(c) 歯。新しい歯の関数名は face_rule_labels で始める（face.rs の unit test）。
1. face.rs の unit test（凍結の針・P-10.1）: 型の ALL を回して name と名札の対の列を作り、字面を歯に直に書いた期待と順まで同じ = 種別は deny と 測って落とす・build-check と 生成時の検査・detect と 記録のみ・human-review と 人が守る作法、状態は 仮 と state warn・凍結 と state ok・未定 と state。
2. 生成区間の番（既存・本文は変えない・verify に名指す）: rules.rs の unit test rules_floor_derives_the_frozen_anchor_byte_for_byte が、(a) の鍵と名の列の書き換えで字面か順が 1 字でもずれれば落ちる。
3. 面の番（既存・本文は変えない・verify に名指す）: crates/folio/tests/face.rs の歯は、憲法の面の出力が凍結の fixture と byte 一致することと、規則の表の状態が値域の外なら「まだ分からない」になること（face_unknown_when_a_rules_status_is_outside_the_table）を見ている。
4. 回帰（期待不変・共通の検証が回す）: tests/schema.rs（4 行のまま）・tests/check.rs・tests/floor_cases.rs（凍結の場合 134 件）・束の歯。

(d) 大きさと接続。新規 file は無い。既存 = rules.rs（約 +70 行）・face.rs（表 2 枚が関数 2 つに = 約 +10 行・歯 約 +30 行）・face_constitution.rs（3 か所の書き換え = 増減ほぼ 0）。size S。実の rules.yaml・設計文書・fixture・src/check.rs・src/schema.rs・tests/floor_cases.yaml・CI の yml は触らない。外部 crate は増やさない。

## 2. 範囲

- 入れる: rules.rs の型 2 つと床の木の鍵・face.rs の表 2 枚の置き換え・face_constitution.rs の表引き 3 か所・歯。
- 入れない: 値と名札の字面の変更・規則の表の行の required の全欄を床が数えること・退役待ちの render.rs の表。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| types | 型 | rules.rs の RuleKind と RuleStatus（値域の字面を書く唯一の所） |
| labels | 名札 | face.rs の網羅の場合分け 2 つ |
| teeth | 歯 | 名札の凍結の針・生成区間の番・面の番 |

## 4. 検査（歯）

§1 (c) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "bc"
title = "規則の表の種別と状態の値域を rules.rs の型 1 つずつに寄せ（床の木の表の鍵もその name で書く・生成区間は byte 不変）、面の生成器の表 2 枚の鍵の列を消して名札を網羅の場合分けで持つ（面の出力と Err の文言は不変・ADR-11 決定 (4)② の締め）"
req = ["FR4", "NFR2", "FR19"]
section = "1"
write-set = ["crates/folio/src/rules.rs", "crates/folio/src/face.rs", "crates/folio/src/face_constitution.rs", "crates/folio/tests/face.rs"]
verify = ["cargo nextest run -p folio face_rule_labels", "cargo nextest run -p folio rules_floor", "cargo nextest run -p folio --test face face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_rule_labels の歯（種別 4 対と状態 3 対の凍結の針）が緑、rules_floor の歯（床の木の導出が凍結 anchor と byte 一致 = 鍵と名の列の番）が期待不変で緑、face の歯（憲法の面が凍結の fixture と byte 一致・値域の外の状態は「まだ分からない」）が期待不変で緑、face.rs に種別と状態の字面を鍵にした表が残らず、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
