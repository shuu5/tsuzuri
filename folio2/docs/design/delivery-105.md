# 設計: 便 105 — 面の歯の 2 file（tests/face.rs・tests/face_index.rs）を面ごとに割り、受付の余地を空ける（振る舞い不変・歯の本数不変・FR4 / FR15）

- 要件: FR4（入口・憲法・要件書の面を 1 つの生成器と 1 組の design token で出す・第 1.29 版）/ FR15（図は型付き記述から生成し、通らない図は出さない）
- 条: P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-3.1（機械で決定的に検査できる項目は床に置く）/ P-6.2（生成物を手で直さない）/ P-4.1（検査・生成が実行できなかった結果を異常なしとして扱わない）
- 出所: 総点検の設計ノート `docs/design/audit-2026-09-23.md` §3-c の列の 1 番目。持ち主が 2026-09-23 08:13 JST に承認済み。同ノート §3-a の表は、この 2 本を「今、上限を超えている歯の file」として挙げ、断りの帰結を「この 2 本を write-set に含む便は受付で断られる＝面の生成器の歯を足せない」と書いている。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 dc と行 dd がそれぞれ指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は `+`・行が減る元の file は `-`）。**改訂 c で契約表の行を 2 つに割った**（行 dc = 要件書の面の側・行 dd = 入口の面の側）。§1 は 2 行に共通で、どちらの行がどの分を運ぶかは (b)(d)(e)(f)(g) に明示する。
- 門: 行 dc も行 dd も `design-intent/` の下を 1 file も読み書きしない（触るのは `crates/folio/tests/` の 3 本ずつだけ）。**起草役が write-set 6 本をそのまま渡して実測し、天井の門が 0（通す・断りの字は 設計文書の正本を書き換えない便）を返すことを確かめた**（2026-09-23・base main c7379ee）。したがってどちらの行も規則の表の開発規律行 D-12 の裁定（設計文書を触る便は門の外で受ける）を要しない。**ただし器の門は正本の書き換えのほかに便の diff の大きさも測る**（上限 150,000 byte）。6 本を 1 便で運ぶとこの上限に掛かるので、改訂 c で契約表を 2 行に割った（§0 の改訂 c に実測を書いた）。受付の手順はふだんどおり preflight → dispatch。
- 前の便: 便 26（入口の面の歯を `face.rs` から `face_index.rs` へ移した）・便 87（`face_labels.rs` の切り出し）・便 89（`schema_docs.rs` の切り出し）。どれも着地済みで、本書の 2 行は同じ流儀（凝集した区間を字を変えずに移し、helper は写しを採る）を踏む。base は **main c7379ee**（改訂 c の実測は **main fbdcf85**）。

- 改訂 b（2026-09-23 09:0x JST・検証役の report `~/.local/share/folio2/handoff-2026-09-23/d105-verify.md` の非 blocker 3 点への応答）: §1 (b)(g) の数に D-13 の参考値の断りを足し、(g) の頭の注の行数を「およそ 60（base に無い新しい行は 31）」に正し、(f) に当たらないものを覆う既存の歯（floor_cases・stamp・gate）を名指した。write-set・歯・done・verify・行 dc は変えない。
- 改訂 c（2026-09-23・便の run `f2-648.149-20260922T235352Z`〔作業者の commit e3157d9・base main fbdcf85〕が天井の門で INCONCLUSIVE になったことへの応答）: 契約表の 1 行（dc）を **2 行（dc = 要件書の面の側・dd = 入口の面の側）に割った**。理由は器の門が測る diff の上限（150,000 byte）で、6 本を 1 便で運ぶと diff が **165,779 byte**（run の worktree で `git diff fbdcf85 e3157d9` を測った実測）になり上限を超えるためである。file を割るだけの便でも、diff は消す側と足す側の両方を持つので元の 2 倍に育つ。2 行に割ったときの各行の diff は、同じ run の木で測ると **行 dc の 3 file が 90,433 byte・行 dd の 3 file が 75,346 byte**（どちらも参考値・規則の表の開発規律行 D-13）で、どちらも上限の下に収まる。§1 の中身・割り方・歯の本数と名・size・接頭辞の作法は改訂 b のまま変えない。作業者の commit e3157d9 の割り方はそのまま使える（新しい run で同じ形を再現する）。

## 1. 目的と中身

### (a) いま起きていること（実測・参考値は base main c7379ee での測り）

器が便を受けるときに測る行数の式は「空行を含む全行を数え、1 行の字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す」。上限は 1,500 行である。

| 歯の file | この式の値 | 余地 |
| --- | --- | --- |
| `crates/folio/tests/face.rs` | 1,785 | **−285** |
| `crates/folio/tests/face_index.rs` | 1,528 | **−28** |

どちらも上限を超えている。上限を超えた file は、行が減る便であっても余地の見積が負なので、受付の cap への断りに掛かる。面の生成器（`folio face`）の歯はこの 2 本に集まっているので、面に関わる便は size S（見積 100 行）も運べない。

2 本の中身は面ごとに分かれている（起草役が base で全部の歯と helper の参照を数えた）。

- `tests/face.rs` は 52 本の歯を持ち、**憲法の面**の歯（凍結 fixture との byte 一致・escape・実の正本の census と部品の検査・導出できない入力 11 本・組み立てた版とのずれ・章の札の凡例・数えの字）と、**要件書の面**の歯（同じ 5 つの観点と、用語集の章・受入基準の章の凡例）と、**要件書の面の図の章**の歯（便 34 と便 74）と、**命令そのものの共通**の歯（check の 3 値・旗が 1 つだけ・表に無い面の名）が 1 file に同居している。
- `tests/face_index.rs` は 45 本の歯を持ち、**入口の面の骨**（凍結 fixture との byte 一致・実の正本の census と部品の検査・check の 3 値・導出できない入力・読む順番の行き先）と、**棚の行**（判断の記録の card と設計ノートの card・便 26 と便 29）と、**節「支度表」**（便 20・22・66）が同居している。
- 歯の file どうしは互いに use できない（cargo は `tests/` の直下の各 `.rs` を別々の crate として組む）。`crates/folio/tests/` の下に dir は 1 つも無く（実測）、新しい dir は便で運べないので、共通の helper を 1 つの module に集める形（cargo が歯の crate から読める共有 module の置き場は `crates/folio/tests/` の下の dir なので、新しい dir が要る）は採れない。この repo では helper の写しを持つのが既定の形で、便 87 と便 89 も写しを採った。**本便も写しを採る。写すのは helper だけで、歯は 1 か所にしか置かない。**
- `crates/folio/Cargo.toml` に `[[test]]` の宣言は 1 つも無い（実測）。歯の file は cargo の自動の見つけ方で拾われるので、新しい file のために Cargo.toml を触る必要は無い。

### (b) 割り方（面ごと・責務ごと）

`tests/face.rs` を 3 本に、`tests/face_index.rs` を 3 本に割る。**行の中身は 1 字も変えない。** 歯の関数名・期待の字面・helper の中身・凍結 fixture の path は 1 つも変えない。新しく書くのは、6 本それぞれの file の頭の注（`//!` の 9〜13 行）だけで、それ以外はすべて字を変えない移動と、既に在る helper の写しである。

**改訂 c で、この割り方を契約表の 2 行に分けて運ぶ**（理由は §0 の改訂 c＝門が測る diff の上限）。2 つの元の file は互いに参照しないので、行 dc と行 dd は書き換える file が 1 つも重ならず、どちらの順でも運べる。

**行 dc が運ぶ分**（元の file は `crates/folio/tests/face.rs` の 1 本）。

| file | 持つもの | 歯 |
| --- | --- | --- |
| `crates/folio/tests/face.rs`（残る） | 憲法の面の歯と、命令そのものの共通の歯 | 24 |
| `crates/folio/tests/face_srs_body.rs`（新規） | 要件書の面の本体の歯（便 15・36・64・70） | 18 |
| `crates/folio/tests/face_srs_figure.rs`（新規） | 要件書の面の図の章の歯（便 34・74・FR15） | 10 |

行 dc の 3 本の歯の合計は 52 で、base の `tests/face.rs` の 52 本と同じである。

**行 dd が運ぶ分**（元の file は `crates/folio/tests/face_index.rs` の 1 本）。

| file | 持つもの | 歯 |
| --- | --- | --- |
| `crates/folio/tests/face_index.rs`（残る） | 入口の面の骨の歯（凍結・census・check の 3 値・導出できない入力・読む順番） | 20 |
| `crates/folio/tests/face_index_shelf.rs`（新規） | 入口の面の棚の行の歯（便 26・29） | 11 |
| `crates/folio/tests/face_index_sheet.rs`（新規） | 入口の面の節「支度表」の歯（便 20・22・66） | 14 |

行 dd の 3 本の歯の合計は 45 で、base の `tests/face_index.rs` の 45 本と同じである。

2 行を合わせた歯の合計は 97 で、base の 52 + 45 と同じである（参考値・base main c7379ee の実測。受付後の数は各行の verify の 1 行目で測る）。**歯の名は 1 つも変わらず、増えも減りもしない。**

区切りの注（`// ── …… ──` の行）は、それが導く節と同じ file へ付いて行く。便 70 の区切りの注だけは、憲法の面の側と要件書の面の側の両方に節が在るので、`face.rs` と `face_srs_body.rs` の両方に同じ字で置く。

### (c) helper の写し

6 本それぞれに、その file の歯が呼ぶ helper だけを写す。**呼ばない helper は写さない**（写すと使われない定義として clippy が落ちる）。起草役は写しを機械で組んだうえで、clippy の落ちる定義を 1 つずつ外して 0 警告に落とした（外れたのは 2 つ＝要件書の面の本体の側の憲法の面用の写しの 1 本と、図の章の側の yaml の欄を読む写しの 1 本）。

写す helper は、repo の根を辿る口・凍結 fixture と実の正本の写しを組む口・命令を起動する口・終了コードと標準出力と標準エラーを取り出す口・写しに変異を当てる口・5 字の escape・yaml を読む口・byte 列を比べる口・数えの字を組む口などである。どれも base に在るものを字を変えずに写す。

### (d) 振る舞いが変わらないことの確かめ（起草役の実測・patch は `~/.local/share/folio2/handoff-2026-09-23/d105-measured.patch`）

行 dc も行 dd も `crates/folio/src/` を 1 file も触らず、`design-intent/`・`tests/fixtures/`・`Cargo.toml`・CI の yml も 1 byte も触らない。起草役は base main c7379ee の clone で 6 本を一度に割り、次を実測した。**行 dc と行 dd は、この 6 本のうち互いに重ならない 3 本ずつを運ぶので、下の 1〜6 はそれぞれの行の 3 本についてもそのまま成り立つ**（1・2 は file ごとに閉じた確かめ・3 は各行の verify の 1 行目が数える・4〜6 は workspace と門の全体の確かめで、片方の行だけを当てた木でも同じ結果になる）。

1. **base に無い行が 1 行も無い。** 6 本それぞれから頭の注を除いた残りの空でない行は、すべて base の元の file に同じ字で在る（機械で全数を突き合わせた）。
2. **97 本の歯が 1 本ずつ、6 本のうち 1 本にだけ、字を変えずに在る。** 歯の本文（属性の行から閉じ括弧まで）を base から取り出し、6 本の本文に部分列として何回現れるかを数えて、どれも 1 回であることを確かめた。helper も 1 つも落ちていない。
3. `cargo nextest run -p folio --test face --test face_srs_body --test face_srs_figure --test face_index --test face_index_shelf --test face_index_sheet` が **97 本すべて緑**。base の `--test face --test face_index` も **97 本すべて緑**で、名も本数も 1 つ違わない。行ごとに数えると、行 dc の 3 本が **52 本**（base の `--test face` と同じ）・行 dd の 3 本が **45 本**（base の `--test face_index` と同じ）である。
4. `cargo nextest run --workspace` が **777 本すべて緑**。base も **777 本**。
5. `cargo clippy --workspace --all-targets -- -D warnings` が **0 警告**（使われない取り込みも使われない定義も出ない＝写しの一覧が過不足ない）。
6. 天井の門が **0（通す・断りの字は 設計文書の正本を書き換えない便）** を返す（write-set 6 本をそのまま渡した）。行ごとの 3 本だけを渡しても同じ 0 を返す（門が見るのは `design-intent/` の下を書き換えるかどうかだけで、どちらの行も 1 file も書き換えない）。

一時の置き場の名は、歯の側の口が 場合の名 と 処理の id で組むので、6 本を並べて回しても衝突しない（97 本を並べて回して実測した）。

### (e) 割った後の余地（器の式・参考値は base main c7379ee での実測）

**行 dc の分。**

| file | 行 dc の前 | 行 dc の後 | 余地 |
| --- | --- | --- | --- |
| `crates/folio/tests/face.rs` | 1,785 | 821 | −285 → **679** |
| `crates/folio/tests/face_srs_body.rs` | 無し | 681 | **819** |
| `crates/folio/tests/face_srs_figure.rs` | 無し | 543 | **957** |

**行 dd の分。**

| file | 行 dd の前 | 行 dd の後 | 余地 |
| --- | --- | --- | --- |
| `crates/folio/tests/face_index.rs` | 1,528 | 751 | −28 → **749** |
| `crates/folio/tests/face_index_shelf.rs` | 無し | 474 | **1,026** |
| `crates/folio/tests/face_index_sheet.rs` | 無し | 594 | **906** |

6 本とも size M の見積（300 行）を大きく上回る余地を持つ。**2 本でなく 6 本に割ったのは、2 本ずつに割ると要件書の面の側の余地が 375 にしかならず、size M の便を 1 本受けたところでまた上限に着くためである**（起草役が 2 本ずつの形でも測った）。総点検 §3-c の列の 2 番目以降（面の生成器の src の割り直し・要件書の面と入口の面の歯を足す便）が、この余地の上で受けられる。**行 dc と行 dd は互いの余地に影響しない**（どちらも相手の 3 本を 1 file も触らない）。

### (f) 歯（行 dc も行 dd も新しい歯を 1 本も置かない）

どちらの行も **振る舞い不変・歯の本数不変・歯の名と中身は 1 字も変えない** ことを約束するので、新しい歯を置かない。したがって赤い歯は無い。成否は §1 (d) の 3 と 5 を、各行の verify の 2 行で数える形で測る。

- **行 dc** の verify の 1 行目が `face`・`face_srs_body`・`face_srs_figure` の 3 本を名指して回し、**52 本**が緑であることを数える。base の `tests/face.rs` の 52 本と同じ本数で、名も 1 つ違わない。
- **行 dd** の verify の 1 行目が `face_index`・`face_index_shelf`・`face_index_sheet` の 3 本を名指して回し、**45 本**が緑であることを数える。base の `tests/face_index.rs` の 45 本と同じ本数で、名も 1 つ違わない。
- どちらの行も、歯を 1 本でも落とすか名を変えれば、この数か名が合わなくなる。
- verify の 2 行目の clippy が、写しの過不足（使われない定義・使われない取り込み）を落とす。
- 当たらないもの（床の判定・天井の印・門）は、既に在る歯が覆う: `crates/folio/tests/floor_cases.rs`（1 本・134 場合）・`tests/stamp.rs`（11 本）・`tests/gate.rs`（7 本）が base と同じ数で緑であること（workspace の行が数える・本便は `design-intent/` も `tests/fixtures/` も触らない）。
- workspace 全体の回帰は `.vessel.toml` の common-verify（workspace の nextest と clippy）が見る。起草役は当てた木で **777 本**が全部緑であることを実測した（base も 777 本）。

**上限を守り続けるための歯（便 89 の `f89_` や便 100 の `f100_` のような、器の式で行数を数えて上限以下であることを見る歯）は、どちらの行にも入れない。** 歯の本数を変えないという約束と両立しないためである。要るなら 2 行が着いた後に 1 便で置ける（6 本それぞれの上限を数える歯 1 本で足りる）。総点検 §3-c の列の中でどこに置くかは持ち主の裁定に委ねる。

### (g) 大きさ

どちらの行も `crates/folio/src/` を 1 file も触らない。新しく書く行は頭の注だけで、残りはすべて字を変えない移動と helper の写しである。

- **行 dc**: 触るのは歯の file 3 本（`face.rs`・`face_srs_body.rs`・`face_srs_figure.rs`）だけで、割った後の余地は §1 (e) のとおり 679 以上。新しく書く行は 3 本の頭の注のおよそ 31 行（参考値・base main c7379ee）。size **S**（新しい 2 本は `+`・行が減る 1 本は `-`）。
- **行 dd**: 触るのは歯の file 3 本（`face_index.rs`・`face_index_shelf.rs`・`face_index_sheet.rs`）だけで、割った後の余地は §1 (e) のとおり 749 以上。新しく書く行は 3 本の頭の注のおよそ 30 行（参考値・base main c7379ee）。size **S**（新しい 2 本は `+`・行が減る 1 本は `-`）。

2 行を合わせた頭の注はおよそ 60 行（参考値・base main c7379ee・頭の注の塊の総数で、base に無い新しい行はそのうち 31 本）。どちらの行も外部 crate は増やさず、新しい dir は作らず、file を消さない。

### (h) 本便が運ばないもの・撤退条件

- 字の書き換え・歯の関数名の変更・歯の統合や分割・期待の字面の変更・helper の書き換え（写しは字を変えずに写す）。
- 面の生成器の `crates/folio/src/` の側の割り直し（総点検 §3-c の列の 2・3 番目）。本便は src を 1 file も触らない。
- 上限を数える歯（§1 (f) の末尾）。
- ほかの歯の file の切り出し（`tests/findings.rs` 1,281・`tests/check.rs` 1,166・`tests/schema_docs.rs` 1,136 はまだ上限の下なので触らない）。
- 歯の file どうしで helper を 1 か所に集める形（新しい dir が要るので運べない＝§1 (a) の実測）。
- 凍結 fixture（`tests/fixtures/face/`）・`design-intent/` の下の全 file・`Cargo.toml`・CI の yml・台帳への記帳。
- 撤退条件: 割ったことで**同じ歯が 2 つの file に写って二重に回る**か、**helper の写しが片方だけ直されて 2 面が食い違う**事故が 1 度でも起きたとき（数えるのは実の受付の断りと CI の落ち・記帳は台帳 f2-648）は、割り方を面ごとでなく責務ごと（凍結 fixture・census・導出できない入力）へ 1 便で組み替える。**元の 2 本へ戻すことはしない**（戻すと上限を超えた状態に戻り、総点検 §3-c の列が全部止まるため）。

## 2. 範囲

- 行 dc に入れる: 新しい歯の file 2 本（`face_srs_body.rs`・`face_srs_figure.rs`）・`face.rs` からの字を変えない移動・helper の写し・3 本の頭の注。
- 行 dd に入れる: 新しい歯の file 2 本（`face_index_shelf.rs`・`face_index_sheet.rs`）・`face_index.rs` からの字を変えない移動・helper の写し・3 本の頭の注。
- どちらの行にも入れない: 字の書き換え・歯の名や本数の変更・新しい歯・`crates/folio/src/`・`design-intent/`・凍結 fixture・`Cargo.toml`・CI の yml・ほかの歯の file・新しい dir・外部 crate・台帳への記帳。相手の行が触る 3 本の file。

## 3. 部品

| id | 行 | 名 | 役 |
| --- | --- | --- | --- |
| srsbody | dc | 新しい歯の file | `crates/folio/tests/face_srs_body.rs`（要件書の面の本体の歯 18 本） |
| srsfig | dc | 新しい歯の file | `crates/folio/tests/face_srs_figure.rs`（要件書の面の図の章の歯 10 本） |
| shelf | dd | 新しい歯の file | `crates/folio/tests/face_index_shelf.rs`（入口の面の棚の行の歯 11 本） |
| sheet | dd | 新しい歯の file | `crates/folio/tests/face_index_sheet.rs`（入口の面の節「支度表」の歯 14 本） |
| trim | dc / dd | 取り除き | `crates/folio/tests/face.rs`（24 本が残る・行 dc）と `crates/folio/tests/face_index.rs`（20 本が残る・行 dd）から、移した区間と、残る歯が呼ばなくなった helper を外す |
| helper | dc / dd | helper の写し | その行が作る file それぞれに、その file の歯が呼ぶ helper だけを字を変えずに写す（歯の file どうしは互いに use できない） |
| head | dc / dd | 頭の注 | その行が触る file それぞれの `//!` の区間（何を持つか・どこへ移したか・写しであること） |

## 4. 検査（歯）

§1 (f) のとおり。本便は新しい歯を置かず、既に在る 97 本が 6 本の file に分かれて全部緑であることと、clippy が 0 警告であることで測る。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。`Cargo.toml` も触らない。
- **書き換える file が重なる並行の便は無い**（起草役が base main c7379ee で確かめた）。`crates/folio/tests/face.rs` と `crates/folio/tests/face_index.rs` を触る便は、便 85 以降 1 本も無い。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "dc"
title = "面の生成器（folio face）の歯が集まっている crates/folio/tests/face.rs を面ごとに 3 本へ割り、器が受付で測る行数の上限（幅 120 正規化 1,500）に対する余地を空ける。この file は憲法の面と命令そのものの共通の歯 24 本を残し、要件書の面の本体の歯 18 本を新しい file crates/folio/tests/face_srs_body.rs へ、要件書の面の図の章の歯 10 本を新しい file crates/folio/tests/face_srs_figure.rs へ移す。移す行は 1 字も変えず、歯の関数名も期待の字面も helper の中身も 1 つも変えない。この 3 本の歯の総数は base の crates/folio/tests/face.rs の 52 本のままで、増えも減りもせず、名も 1 つ違わない。歯の file どうしは互いに use できないので helper は 3 本それぞれに写しを置き、その file の歯が呼ばない helper は写さない。新しく書くのは 3 本それぞれの頭の注だけで、新しい歯は 1 本も置かず、crates/folio/src/ と design-intent/ と凍結 fixture と Cargo.toml と CI の yml は 1 file も触らず、新しい dir も作らず file も消さない。入口の面の側（crates/folio/tests/face_index.rs）は行 dd が運ぶので本行は 1 file も触らない。割った後の 3 本の余地は参考値で 679 以上となり、総点検の設計ノート docs/design/audit-2026-09-23.md §3-c の列の 2 番目以降（面の生成器の src の割り直しと、要件書の面の歯を足す便）が受けられるようになる"
req = ["FR4", "FR15"]
section = "1"
write-set = ["-crates/folio/tests/face.rs", "+crates/folio/tests/face_srs_body.rs", "+crates/folio/tests/face_srs_figure.rs"]
verify = ["cargo nextest run -p folio --test face --test face_srs_body --test face_srs_figure", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "3 本の歯の file を合わせた歯が 52 本で全部緑（base の crates/folio/tests/face.rs の 52 本と同じ本数で、歯の名も 1 つ違わず、face が 24 本・face_srs_body が 18 本・face_srs_figure が 10 本）、clippy が 0 警告（写しに使われない定義も使われない取り込みも無い）で、workspace の nextest が base と同じ 777 本すべて緑で CI が通る"

[[contract]]
id = "dd"
title = "面の生成器（folio face）の入口の面の歯が集まっている crates/folio/tests/face_index.rs を節ごとに 3 本へ割り、器が受付で測る行数の上限（幅 120 正規化 1,500）に対する余地を空ける。この file は入口の面の骨の歯 20 本（凍結 fixture との byte 一致・実の正本の census と部品の検査・check の 3 値・導出できない入力・読む順番の行き先）を残し、棚の行の歯 11 本を新しい file crates/folio/tests/face_index_shelf.rs へ、節 支度表 の歯 14 本を新しい file crates/folio/tests/face_index_sheet.rs へ移す。移す行は 1 字も変えず、歯の関数名も期待の字面も helper の中身も 1 つも変えない。この 3 本の歯の総数は base の crates/folio/tests/face_index.rs の 45 本のままで、増えも減りもせず、名も 1 つ違わない。歯の file どうしは互いに use できないので helper は 3 本それぞれに写しを置き、その file の歯が呼ばない helper は写さない。新しく書くのは 3 本それぞれの頭の注だけで、新しい歯は 1 本も置かず、crates/folio/src/ と design-intent/ と凍結 fixture と Cargo.toml と CI の yml は 1 file も触らず、新しい dir も作らず file も消さない。要件書の面の側（crates/folio/tests/face.rs）は行 dc が運ぶので本行は 1 file も触らない。割った後の 3 本の余地は参考値で 749 以上となり、総点検の設計ノート docs/design/audit-2026-09-23.md §3-c の列の 2 番目以降（面の生成器の src の割り直しと、入口の面の歯を足す便）が受けられるようになる"
req = ["FR4"]
section = "1"
write-set = ["-crates/folio/tests/face_index.rs", "+crates/folio/tests/face_index_shelf.rs", "+crates/folio/tests/face_index_sheet.rs"]
verify = ["cargo nextest run -p folio --test face_index --test face_index_shelf --test face_index_sheet", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "3 本の歯の file を合わせた歯が 45 本で全部緑（base の crates/folio/tests/face_index.rs の 45 本と同じ本数で、歯の名も 1 つ違わず、face_index が 20 本・face_index_shelf が 11 本・face_index_sheet が 14 本）、clippy が 0 警告（写しに使われない定義も使われない取り込みも無い）で、workspace の nextest が base と同じ 777 本すべて緑で CI が通る"
<!-- contracts:end -->
