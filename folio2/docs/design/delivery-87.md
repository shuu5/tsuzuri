# 設計: 便 87 — 面の共有の口の名札を face_labels.rs へ切り出す（受付の余地を空ける・振る舞いは不変・FR4）

- 要件: FR4（人が読むページを 1 つの生成器から出す）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-6.2（生成物を手で直さない）/ P-10.1（凍結 anchor）
- 出所: 便 83（台帳 f2-648.113）と便 84 が受付の cap で断られた。器は crates/folio/src/face.rs の余地を 93 行と測る（下の (a) の実測）。同じ理由で便 76 も改訂 b で face.rs を write-set から外している（PR #214）。face.rs に行を足す便を通せるようにするため、凝集した塊を 1 つ別の module へそのまま移す。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cj が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 2 本）。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

crates/folio/src/face.rs は面の命令の口・木を辿る型・字面の口・名札・名札の表・部品目録の上限・面の骨格・用語集の語の行・図の枠を 1 file に持つ。器が便を受けるときに測る行数の上限（1500）に対する余地が 93 行しか残っておらず、face.rs に行を足す便は size S の見積（100 行）に届かず断られる。本便は face.rs の中で完全に閉じている 2 つの区間（名札と名札の表）を新しい module へ**そのまま移す**。字は 1 字も変えず、出力の byte も変えず、公開の名も変えない。呼び出し側は 1 file も触らない（face.rs が新しい module を丸ごと再輸出するため）。

orchestrator 席の実測（2026-09-22・main bb80fef・便 79 / 80 / 81 の着地後）:

- 器の行数の式は「空行を含む全行を数え、1 行の字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す」。この式で測った値と余地（1500 − 値）は次のとおり。face.rs = 生 1397 行 + 折り返し 10 = 1407・余地 **93**／face_srs.rs = 1383 + 15 = 1398・余地 102／face_index.rs = 1363 + 15 = 1378・余地 122／face_constitution.rs = 1265 + 24 = 1289・余地 211／main.rs = 538 + 4 = 542・余地 958。
- face.rs は区切りの注釈で 9 つの区間に分かれる。52 命令の口／156 木を辿る口／307 字面の口／**480 名札（β）**／**659 名札の表（β）**／788 部品目録の上限／797 面の骨格／1035 用語集の語の行／1062 図の枠。末尾の 1123 行から file の終わり（1397 行）までが単体の歯 mod face_tests。
- 移す区間 480 行から 787 行（末尾の 787 行は空行）は、生 308 行・120 字を超える行 0・器の式で 308。区間の中に在る公開の名は 27 個。
  - 名札（β・480 から 658 行）: 型 Tier／tier_label／tier_of／strength_label／strength_meaning／strength_prio／pattern_label／binds_label／mechanism_kind_label／mechanism_live_label／stage_label／polarity_label／rationale_kind_label／retreat_kind_label／rule_kind_label／rule_status_class
  - 名札の表（β・659 から 787 行）: DOC_STATUS／METHOD／TONE／型 Shelf／SHELF_DOCS／SHELF_RELATIONS／ANNEXES／SHELF_LEGEND／INDEX_STATUS／stop_anchor／method_label
- 区間が face.rs のほかの所から使う口は 3 つだけ。型の別名 ce（crate::constitution_enums）・型 X・型の別名 R。区間は esc・anchor・val・hint・card・Frame・Value・catalog・図の口を 1 つも使わない（実測）。
- 区間の外が区間の名を使う所。face.rs の本体は 460 行（rationale が rationale_kind_label を呼ぶ）と 1009・1013 行（glossary_chip が ANNEXES を読む）の 2 か所。face.rs の単体の歯 mod face_tests は 1128 行から 1358 行で 18 の名を使う。ほかの module は use crate::face::{…} の形で名指す（face_constitution.rs 11 行／face_srs.rs 13 行／face_index.rs 23 行／face_adr.rs 17 行／face_note.rs 15 行／face_srs_rtm.rs 5 行）。
- 型の別名 ce は区間の外の本体（460 行）でも使うので face.rs に残る。crate::rules は区間の外では単体の歯だけが使う（実測で本体の使用は 0）ので、face.rs の先頭に残すと歯を組まない build で使われない取り込みになり、clippy が落ちる。移し先を (c) で定める。

### (a) 新しい module

crates/folio/src/face_labels.rs（新規）を作り、face.rs の 480 行から 787 行をそのまま移す。行の中身は 1 字も変えない（区切りの注釈 2 本も含めてそのまま）。file の頭に次の 2 つだけを足す。

1. module の説明の注釈。面の共有の名札（憲法の値域の型への網羅の場合分け）と、値域に依らない名札の表を持つこと。便 87 で face.rs から移したもので、字は 1 字も変えていないこと。公開の名は face.rs から丸ごと再輸出されるので、呼び出し側は crate::face::… のまま名指せること。
2. 取り込み 3 行。crate::constitution_enums を ce の名で／crate::face から R と X を／crate::rules を。

### (b) mod の宣言

crates/folio/src/main.rs の mod の列（3 行から 40 行）に face_labels を 1 行足す。位置は face_index の次・face_note の前（今の列と同じ並べ方）。ほかの行は触らない。

### (c) face.rs の側

1. 480 行から 787 行を取り除く。
2. 取り除いた所に 3 行を置く。区切りの注釈 1 行（名札と名札の表は face_labels.rs へ移した・便 87・振る舞いは不変）と、新しい module を丸ごと再輸出する 1 行（pub use crate::face_labels 以下の全部）と、空行 1 行。この再輸出により、face.rs の本体（460・1009・1013 行）も単体の歯も ほかの 6 つの module の use crate::face::{…} も 1 字も変えずに名が解ける。
3. 先頭の取り込みから crate::rules の 1 行を外し、同じ 1 行を mod face_tests の中（use super::* の次）へ置く。ほかの取り込みは触らない（ce は本体の 460 行が使うので残す）。
4. 単体の歯 mod face_tests の中身は 1 字も変えない。

### (d) 振る舞いが変わらないことの確かめ

公開の名も引数も戻り値も字面も変えないので、5 面の出力は 1 byte も変わらない。凍結の写し tests/fixtures/face/expected*.html（7 本）は 1 byte も触らない。既存の歯（凍結の写しとの byte 一致・部品目録との一致・逐語と件数の census・単体の名札の凍結の針）がそのまま緑であることが、振る舞いが変わっていないことの物差しになる。

### (e) 歯（関数名は f87_ で始める。`grep -rn 'fn f87_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f86_）

新しい file crates/folio/tests/face_labels.rs（新規・+）に 1 本。

1. f87_face_is_split_and_under_the_cap: 歯の側で器の式（空行を含む全行を数え、字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す・字数は Unicode の字の数）を持ち、次の 4 つを見る。
   - crates/folio/src/face.rs をこの式で数えた値が 1250 以下（本便の後の見込みは約 1102）。本便の前の main では 1407 なので赤い歯。
   - crates/folio/src/face_labels.rs が在り、その中に移した名の定義の頭が在る（pub fn tier_label／pub fn rule_kind_label／pub fn method_label／pub fn stop_anchor／pub const SHELF_DOCS／pub const ANNEXES／pub struct Tier／pub struct Shelf の 8 本を名指す）。
   - crates/folio/src/face.rs にその 8 本の定義の頭が 1 つも無い。
   - crates/folio/src/face.rs に face_labels を丸ごと再輸出する行が 1 本だけ在る。
   file の読みは歯の file の位置（CARGO_MANIFEST_DIR）から repo の根を辿る既存の作り方（tests/face.rs の repo_root と同じ形）で行う。
2. 回帰（期待不変）は 2 段で見る。verify の 2 行目 = write-set に在る歯の file 5 本（tests/face.rs・tests/face_constitution.rs・tests/face_srs.rs・tests/site.rs・tests/badge.rs）の既存の歯すべて。凍結の写し 7 本との byte 一致はこの段で見られる。face.rs の単体の歯 mod face_tests の 9 本（名札の凍結の針を含む）と、本便が触らない歯の file 4 本（tests/face_index.rs・tests/face_adr.rs・tests/face_note.rs・tests/parts.rs）は verify にも write-set にも名指さず、共通の検証（.vessel.toml の common-verify = workspace 全体の nextest）で回る。器の受付は verify の旗 --bin の次の語を filter 語と読むので、単体の歯を verify の行で名指さない（席の実測 2026-09-22）。

### (f) 大きさ

src は 3 本。crates/folio/src/face.rs（器の式で 1407・余地 93・308 行が減り 3 行が増えるので約 1102・余地 約 398）／新規 crates/folio/src/face_labels.rs（移した 308 行 + 頭の約 10 行 = 約 318・余地 約 1182）／crates/folio/src/main.rs（542・余地 958・+ 1 行）。歯は新規 crates/folio/tests/face_labels.rs（+ 約 70 行）。size **S**（触る src の各 file の余地は 100 以上。face.rs は減る側で、器の式でも減る）。外部 crate は増やさない。ほかの 5 面の生成器・部品目録・様式・design-intent の下の正本・tests/fixtures の下の写し・CI の yml は触らない。

### (g) 本便が運ばないもの

字の書き換え・関数の統合や分割・公開の範囲の変更（pub のままにする）・引数や戻り値の変更・呼び出し側の use の付け替え（再輸出で吸収する）。face.rs の単体の歯 mod face_tests の移動（名札の歯は face.rs に残る。再輸出で名が解けるので字は変わらない。歯まで移すかは、face.rs にさらに余地が要るようになったときの別の便で判断する）。face.rs のほかの区間（命令の口・木を辿る口・字面の口・部品目録の上限・面の骨格・用語集の語の行・図の枠）。face_srs.rs（余地 102）と face_index.rs（余地 122）の切り出し（本便では触らない。行を足す便が来たときに同じ形で別に起こす）。

受付の cap への断り: face.rs は本便で行が減る file なので、write-set では縮む file の印（-）を付けて宣言する（器の受付は縮む file に上限の余地を求めない）。ほかの file の余地は size S の見積を満たす。

## 2. 範囲

- 入れる: 新しい module 1 本・308 行のそのままの移動・mod の宣言 1 行・再輸出 1 行・取り込みの置き場の直し 1 行・歯 1 本と新しい歯の file。
- 入れない: 字の書き換え・出力の変化・公開の名の変更・呼び出し側の use・単体の歯の移動・ほかの区間・ほかの file の切り出し・凍結の写し。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| module | 新しい module | crates/folio/src/face_labels.rs（名札 16 + 名札の表 11） |
| decl | mod の宣言 | main.rs の mod の列に 1 行 |
| reexport | 再輸出 | face.rs の 1 行（呼び出し側を 1 file も触らない） |
| import | 取り込み | crate::rules を face.rs の先頭から単体の歯の中へ |
| teeth | 歯 | 新規 crates/folio/tests/face_labels.rs の f87_ 1 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cj"
title = "face.rs の名札と名札の表の 2 区間（308 行・公開の名 27 個）を新しい module face_labels.rs へ 1 字も変えずに移し、face.rs は丸ごと再輸出する 1 行だけを持つ（呼び出し側は 1 file も触らない・5 面の出力と凍結の写しは 1 byte も変わらない・器の式で face.rs の余地を 93 行から約 398 行へ空ける）"
req = ["FR4"]
section = "1"
write-set = ["-crates/folio/src/face.rs", "+crates/folio/src/face_labels.rs", "crates/folio/src/main.rs", "+crates/folio/tests/face_labels.rs", "crates/folio/tests/face.rs", "crates/folio/tests/face_constitution.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/site.rs", "crates/folio/tests/badge.rs"]
verify = ["cargo nextest run -p folio --test face_labels f87_", "cargo nextest run -p folio --test face --test face_constitution --test face_srs --test site --test badge", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f87_ の歯 1 本（器の式で face.rs が 1250 行以下・移した 8 本の定義が新しい module に在り face.rs に無い・再輸出の行が 1 本）が緑、tests/face.rs と tests/face_constitution.rs と tests/face_srs.rs と tests/site.rs と tests/badge.rs の既存の歯が全部緑（凍結の写し 7 本との byte 一致を含む）、共通の検証（workspace 全体の nextest）で face.rs の単体の歯 mod face_tests の 9 本（名札の凍結の針）と残りの歯の file 4 本も緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
