# 設計: 便 100 — 要件書の面の章 03〜06 を face_srs_items.rs へ切り出す（受付の余地を空ける・面の生成物は byte 不変・FR4 / NFR2）

- 要件: FR4（正本から入口・憲法・要件書の 3 面を単一の生成器と単一の design token で生成する）/ NFR2（全生成面の design token は単一の定義に由来し、部品目録に無い class は rules 行 R-3 の値にする）
- 条: P-2.1（人が読むページはすべて 1 つの生成器から出力する）/ P-2.3（design token は 1 か所で定義する）/ P-6.2（生成物を手で直さない）/ P-3.1（機械で決定的に検査できる項目は床に置く）
- 出所: 判断の記録 ADR-13 決定 (16) の便の列の ⑧（要件書の面を組む file を割る）。設計ノート docs/design/graph-and-incremental-ceiling.md §8 の行 G7b（`face_srs.rs` → 新 file（+）・触る src の余地 102）。同じ §8 の行 G8（要件ごとの実装の状態を導出して要件書の面へ出す）が「`face_srs.rs` の割った側」を触ると書いており、本便はその受け皿を作る 1 本である。前例は便 89（docs/design/delivery-89.md・歯の file `tests/schema.rs` を割った便）と便 35（`face_srs.rs` から章 07・08 を `face_srs_rtm.rs` へ分けた便）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cw が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 1 本・縮む file は - を付けた 1 本）。
- 門: 本便は design-intent の下を 1 file も書き換えない（触るのは `crates/folio/src/` の 3 本と `crates/folio/tests/` の 1 本だけ）ので、天井の門の対象外である。席は `folio ceiling --gate --write-set`（本便の 4 本）を実測し、0（通す・設計文書の正本を書き換えない便）を確かめた（2026-09-22・main 420d910）。
- 並行する便: 便 99（`crates/folio/src/graph.rs`・`stamp.rs`・`gate.rs`・`design-intent/graph.yaml`・`tests/fixtures/schema/` の 4 本・`crates/folio/tests/graph.rs`・`tests/stamp.rs`・`tests/schema_docs.rs`）と、本便の write-set は **1 本も重ならない**。便 101（行 cx）とも重ならない。

- 改訂 b（2026-09-22 17:5x JST・検証役の report handoff-2026-09-22/d100-verify.md への応答）: verify の 2 行目の --test face の scope crates/folio/tests/face.rs を write-set に足す（本文は変えない・器の受付は verify の --test X の X の file を write-set に求める）。ほかは変えない。
- 改訂 c（2026-09-22 18:0x JST・run f2-648.137-20260922T085029Z の審査 FAIL〔literal-mismatch〕への応答）: done が tests/face_srs.rs と tests/face.rs の 2 file に workspace の本数 743 を帰属させていた。verify の 2 行目で測れるのは 2 file の 60 本（face_srs 8・face 52・席が main で実測）なので、done を 2 file の 60 本と、共通の検証の workspace 743 + 1 = 744 本とに書き分ける。ほかは変えない。
## 1. 目的と中身

`crates/folio/src/face_srs.rs` は要件書の面の生成器で、器が便を受けるときに測る行数の上限（1500）に対する余地が **102 行**しか残っていない。この file に行を足す便は size S の見積（100 行）をかろうじて満たすだけで、size M の見積（300 行）では受け付けられない。設計ノート §8 の行 G8 はこの file の側に要件ごとの実装の状態を出す M の便なので、先に file を割る 1 本が要る。本便は章 03〜06（機能要件・非機能要件・受入基準・制約）の生成を新しい file へ **そのまま移す**。移す行は 1 字も変えず、関数名も部品の名札も出力の字面も 1 つも変えない。面の生成物は byte まで同じである。

### (a) 実測（2026-09-22・main 420d910・便 97 の着地後）

- 器の行数の式は「空行を含む全行を数え、1 行の字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す」。この式で測った src の値と余地（1500 − 値）は `face_srs.rs` **1398・余地 102**／`face.rs` 1138・362／`face_constitution.rs` 1294・206／`face_note.rs` 1023・477／`face_adr.rs` 855・645／`face_srs_rtm.rs` 114・1386／`main.rs` 573・927。歯は `crates/folio/tests/face_srs.rs` 352・余地 1148。
- `face_srs.rs` の中身は 6 つの層に分かれる。頭の注釈（1〜8 行）／取り込み（10〜19 行）／部品と章の表と骨格（21〜109 行）／行と段と文脈の型（110〜283 行）と導出の入口 `derive`（255〜283 行）／読みと検査と共有の口（284〜664 行）／章の生成（666〜1295 行）／単体の歯（1296〜1383 行）。
- 章の生成は章ごとに閉じている。章 01 `goals_chapter`（666〜690）／章 02 `scope_chapter`（710〜810）と補助 `band_limit`・`joined`（691〜709）／**章 03〜06（810〜1176）**／章 09 の図と承認欄と脚（1178〜1295）。章 07・08 は便 35 で `face_srs_rtm.rs` へ出ている。
- **810〜1176 行の 367 行が 1 つの塊になる。** 中身は `legend_line`（凡例の 1 行）・`fr_chapter`（章 03・図 2 と図 3 を含む）・`nfr_chapter`（章 04）・`item_row`（要件の行・章 03 と章 04 だけが呼ぶ）・`ac_legend_line`・`ac_chapter`（章 05）・`con_chapter`（章 06）の 7 つの定義で、この 7 つを呼ぶのは `derive` の 4 行（268〜271）とこの塊の中だけである（席が全参照を数えた）。
- 塊が `face_srs.rs` の側に残るものから使うのは、型 `Ctx`（既に pub(crate)）・`Item`（既に pub(crate)）・`Step`（私有）・`Where` の欄 `long`（私有）・口 `band`（既に pub(crate)）・`figure_open`・`figure_close`・`fig_req`・`article_link`・`xref`・`slots`（6 つとも私有）・`Ctx` の欄 `acs`・`cons`・`rail`・`verdicts`（4 つとも私有）・`Ctx` の口 `req`・`article`・`rule`（3 つとも私有）である。6 つの口と `Step` と `Where.long` は **残る側の歯や章も呼ぶ**ので `face_srs.rs` に残し、見え方（visibility）だけを pub(crate) に上げる（便 35 が `Ctx` と `band` に対して行ったのと同じ手当てで、振る舞いは 1 つも変わらない）。
- 塊が `face.rs` から使うのは `METHOD`・`R`・`TONE`・`X`・`anchor`・`card`・`hint`・`hint_q`・`method_label`・`pattern_label`・`strength_label`・`strength_meaning`・`strength_prio` の 13 語と、`constitution_enums` の `Pattern`・`Strength`、部品目録の `Component` である。
- **`crates/folio/src/main.rs` の `mod` の宣言が要る。** 42 本の `mod` が名前の順に並んでおり（3〜44 行）、`mod face_srs;` と `mod face_srs_rtm;` の間に 1 行足す。`Cargo.toml` も CI の yml も触らない。
- 歯の関数名の接頭辞。`grep -rn 'fn f100_' crates/folio` は今 0 本（使われている最大は f97_・便 99 が f99_ を使う）。

### (b) 新しい file

`crates/folio/src/face_srs_items.rs`（新規・+）を作る。持つのは (a) の 367 行の塊（7 つの定義）だけで、**行の中身は 1 字も変えない**。file の頭に 14 行だけを新しく書く。

1. 注釈 3 行。この file が要件書の面の章 03〜06 を持つこと、便 100 で `face_srs.rs` から 1 字も変えずに移したこと、文脈（`Ctx`）と共有の口（`band`・`figure_open`・`figure_close`・`fig_req`・`article_link`・`xref`・`slots`）は `face_srs.rs` のものであることを言う。
2. 空行 1 行。
3. 取り込み 9 行。`crate::constitution_enums as ce`／`crate::face` の 13 語／`crate::face_srs` の 9 語（`Ctx`・`Item`・`article_link`・`band`・`fig_req`・`figure_close`・`figure_open`・`slots`・`xref`）／`crate::parts::catalog::Component`。
4. 空行 1 行。

`derive` から呼ばれる 4 つ（`fr_chapter`・`nfr_chapter`・`ac_chapter`・`con_chapter`）だけを pub(crate) にする。`legend_line`・`item_row`・`ac_legend_line` はこの file の中だけで呼ぶので私有のままにする（pub(crate) にすると使われない公開として clippy が落ちる形ではないが、見え方を要るところまでに閉じる）。

### (c) face_srs.rs の側

(b) で移した 810〜1176 行と、その後ろの空行 1 行（1177 行）を取り除く。残る行は次の 5 か所を除いて **1 字も変えない**。

1. 頭の注釈の末尾に指し先 2 行を足す（章 03〜06 の生成は `face_srs_items.rs` へ移した・字は 1 字も変えていない、と言う 2 行）。
2. 取り込みを削る。`use crate::constitution_enums as ce;` の 1 行を外し、`crate::face` の取り込みから `METHOD`・`TONE`・`hint_q`・`method_label`・`pattern_label`・`strength_label`・`strength_meaning`・`strength_prio` の 8 語を外す（残るのは `self`・`DOC_STATUS`・`Frame`・`MAX_PER_BAND`・`MAX_RAIL_NODES`・`MAX_STATE_NODES`・`R`・`X`・`anchor`・`card`・`count_word`・`hint` の 12 語で、3 行が 2 行になる）。**外さないと使われない取り込みとして clippy が落ちる。**
3. 見え方を pub(crate) に上げる。型 `Step` とその 5 欄（`x`・`n`・`who`・`owner`・`what`）／`Where` の欄 `long`／`Ctx` の 4 欄（`acs`・`cons`・`rail`・`verdicts`）／`Ctx` の 3 つの口（`req`・`article`・`rule`）／6 つの関数（`slots`・`figure_open`・`figure_close`・`fig_req`・`article_link`・`xref`）。**22 か所とも、行に pub(crate) の 10 字を足すだけで、ほかは 1 字も変えない。**
4. `derive` の 4 行を、新しい file の口を呼ぶ形に書き換える（章 07・08 が既に `crate::face_srs_rtm::` を呼んでいるのと同じ字面）。
5. 単体の歯の module の頭に `use crate::face::{TONE, method_label};` の 1 行を足す。2 で外した 2 語を歯だけが使うためで、足さないと歯が組めない。**歯の中身は 1 字も変えない。**

### (d) 振る舞いが変わらないことの確かめ（席が当てて実測した・2026-09-22・main 420d910）

本便は `design-intent/` も `tests/fixtures/` も `vendor/` も 1 byte も触らず、面の生成の道筋（`derive` が呼ぶ順）も出力の字面も変えない。席は切り出しを実際に当てて次を実測した。

- **要件書の面は byte まで同じ。** 実の正本から `folio face --face srs --write` で組んだ面は、本便の前も後も **238824 byte**・要約値（sha256）**7c123601c53fd8c3029e425323d775079b1f9bc1a8212e62930e7a4246ee0c90** で、`cmp` が 1 byte の差も出さない。席は同じ作業ツリーで、切り出しを当てた binary と、`face_srs.rs` と `main.rs` だけを元に戻した binary の 2 本で組んで突き合わせた。
- `cargo clippy --workspace --all-targets -- -D warnings` が 0 警告で通る（(b) 3 と (c) 2 の取り込みの一覧、(c) 3 の見え方の一覧が過不足ない）。
- `cargo nextest run --workspace` が **743 本すべて緑**。本便の前の main と、本数も名前も 1 つ違わない。本便が足す f100_ の 1 本を入れて、着地後は 744 本になる。
- CI が回すのは nextest と clippy の 2 本だけで、書式（rustfmt）の検査は無い（`.github/workflows/ci.yml` を実測）。

### (e) 切り出しの後の余地（器の式・実測）

| file | 本便の前 | 本便の後 | 余地 |
|---|---|---|---|
| crates/folio/src/face_srs.rs | 1398 | 1023 | 102 → **477** |
| crates/folio/src/face_srs_items.rs | 無し | 389 | **1111** |
| crates/folio/src/main.rs | 573 | 574 | 927 → 926 |
| crates/folio/tests/face_srs.rs | 352 | 352 + f100_ の区間 | 1148 |

移す割合は 1398 のうち 367 行（約 26%）。切り出しの後はどちらの file も size M の見積（300）を大きく上回る余地を持つので、設計ノート §8 の行 G8（実装の状態を要件書の面へ出す M の便）は、要件の行を描く側（`face_srs_items.rs`・余地 1111）でも表紙と目次の側（`face_srs.rs`・余地 477）でも受けられる。

### (f) 歯（関数名は f100_ で始める。`grep -rn 'fn f100_' crates/folio` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f97_）

置き場は `crates/folio/tests/face_srs.rs`（要件書の面の歯の file・余地 1148）の末尾。1 本置く。

1. `f100_face_srs_is_split_and_under_the_cap`: 歯の側で器の式（空行を含む全行を数え、字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す・字数は Unicode の字の数）を持ち、次の 4 つを見る。
   - `crates/folio/src/face_srs.rs` をこの式で数えた値が **1100 以下**（本便の後の実測は 1023）。本便の前の main では 1398 なので赤い歯。
   - `crates/folio/src/face_srs_items.rs` を同じ式で数えた値が **600 以下**（本便の後の実測は 389）。本便の前の main にはこの file が無いので、読めずに落ちる = ここでも赤い歯。
   - 移した 7 つの定義の頭（`fn legend_line(`・`fn fr_chapter(`・`fn nfr_chapter(`・`fn item_row(`・`fn ac_legend_line(`・`fn ac_chapter(`・`fn con_chapter(`）が `face_srs_items.rs` に **1 つずつ在る**。
   - 同じ 7 つの頭が `face_srs.rs` に **1 つも無い**。`derive` が書き換わった 4 行は `crate::face_srs_items::fr_chapter(` の形で `fn` を持たないので、この数えに掛からない（席が本便の前後の両方で数えて確かめた＝本便の前は 7 つとも 1 回ずつ在り、後は 0 回）。
   file の読みは、この file が既に持つ `repo_root`（歯の file の位置から repo の根を辿る・16 行）をそのまま使う。
2. 回帰（期待不変）は verify の 2 行目で見る。`tests/face_srs.rs` と `tests/face.rs`（要件書の面の既存の歯）を丸ごと回す。ほかの歯の file と src は 1 本も触らないので、共通の検証（`.vessel.toml` の common-verify = workspace 全体の nextest と clippy）で回る。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない。

### (g) 大きさ

src は 3 本。`crates/folio/src/face_srs.rs`（1398・余地 102・**縮む file** なので write-set で - を付ける）・新規 `crates/folio/src/face_srs_items.rs`（389・余地 1111）・`crates/folio/src/main.rs`（573 → 574・余地 926・`mod` の 1 行だけ）。歯は `crates/folio/tests/face_srs.rs`（352・余地 1148・f100_ の区間が増える）。本便が新しく書く行は、新しい file の頭 14 行・`face_srs.rs` の指し先 2 行と歯の取り込み 1 行・f100_ の区間（見込み 55 行程度）の合わせて 72 行程度で、残る 367 行はすべて字を変えない移動、22 か所は行に pub(crate) を足すだけの見え方の変更である。size **S**。

**受付の cap への断り。** 増える file の余地は、新規の `face_srs_items.rs` が 1111・`main.rs` が 926・`tests/face_srs.rs` が 1148 で、3 本とも S の見積（100）を満たす。`face_srs.rs` は行が減る file なので - を付けて宣言する。設計ノート §8 の行 G7b は大きさを M と見積もっていたが、**本便は S で運ぶ**＝この便の前の `face_srs.rs` の余地は 102 しかなく、M の見積（300）では受付が断る。本便が M を要らないのは、367 行が字を変えない移動で、新しく書く行が 72 行程度に収まるからである。外部 crate は増やさない。

### (h) 本便が運ばないもの

字の書き換え・関数名の変更・出力の字面の変更・部品の名札（class）の増減・章の数と順の変更・`derive` が呼ぶ順の変更・歯の中身の書き換え（(c) 5 の取り込み 1 行を除く）。章 01・02 と章 09 と承認欄と脚の切り出し（`face_srs.rs` に残る側は余地 477 なので分けない）。ほかの面の生成器（`face.rs` 余地 362・`face_constitution.rs` 余地 206）の切り出し（本便は触らない＝別の便の話で、必要になった時点で同じ形で運ぶ）。要件ごとの実装の状態の導出と面への出力（設計ノート §8 の行 G8・本便の受け皿の上に乗る次の便）。`design-intent/`・`tests/fixtures/`・`vendor/`・`Cargo.toml`・CI の yml・ほかの歯の file。

## 2. 範囲

- 入れる: 新しい src の file 1 本・367 行の字を変えない移動・取り込み 9 行と注釈 3 行・`face_srs.rs` の取り込みの削り 1 行と 8 語・見え方の 22 か所・`derive` の 4 行・指し先 2 行・歯の取り込み 1 行・`main.rs` の `mod` 1 行・f100_ の歯 1 本。
- 入れない: 字の書き換えと関数名と出力の字面の変更・部品の名札と章の増減・ほかの面の生成器・実装の状態の導出・`design-intent`・fixture・`Cargo.toml`・CI の yml・ほかの歯の file。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| file | 新しい src の file | `crates/folio/src/face_srs_items.rs`（章 03〜06 の 7 つの定義・367 行） |
| vis | 見え方 | `face_srs.rs` の 22 か所を pub(crate) へ（`Step` と 5 欄・`Where.long`・`Ctx` の 4 欄と 3 つの口・6 つの関数） |
| trim | 取り込みの削り | `face_srs.rs` から `constitution_enums` の 1 行と `face` の 8 語 |
| calls | 呼び先 | `derive` の 4 行を `crate::face_srs_items::` の形へ |
| modline | `mod` の宣言 | `crates/folio/src/main.rs` に 1 行（名前の順） |
| teeth | 歯 | `crates/folio/tests/face_srs.rs` の f100_ 1 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 99（行 cu）とも便 101（行 cx）とも、書き換える file が 1 本も重ならない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cw"
title = "要件書の面の生成器 crates/folio/src/face_srs.rs のうち、章 03〜06（機能要件・非機能要件・受入基準・制約）を組む 7 つの定義 367 行を、新しい src の file crates/folio/src/face_srs_items.rs へ 1 字も変えずに移す（関数名も出力の字面も部品の名札も不変・面の生成物は byte まで同じ）。残る側に要る手当ては、使われなくなる取り込み 1 行と 8 語の削り・移した側が呼ぶ 22 か所の見え方を pub(crate) へ上げること・derive の 4 行を新しい file の口へ向けること・main.rs の mod の 1 行・単体の歯の取り込み 1 行だけとする。器の式で face_srs.rs の余地を 102 行から 477 行へ空け、設計ノート §8 の行 G8（要件ごとの実装の状態を要件書の面へ出す M の便）が受けられるようにする"
req = ["FR4", "NFR2"]
section = "1"
write-set = ["-crates/folio/src/face_srs.rs", "+crates/folio/src/face_srs_items.rs", "crates/folio/src/main.rs", "crates/folio/tests/face_srs.rs", "crates/folio/tests/face.rs"]
verify = ["cargo nextest run -p folio --test face_srs f100_", "cargo nextest run -p folio --test face_srs --test face", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f100_ の歯 1 本（器の式で face_srs.rs が 1100 行以下・face_srs_items.rs が 600 行以下・移した 7 つの定義の頭が新しい file に 1 つずつ在り face_srs.rs に 1 つも無い）が緑、crates/folio/tests/face_srs.rs の既存の 8 本と crates/folio/tests/face.rs の 52 本（合わせて 60 本・本便の前の main と名前も本数も違わない）が全部緑、共通の検証の workspace 全体の nextest が本便の前の 743 本 + f100_ の 1 本 = 744 本で全部緑、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
