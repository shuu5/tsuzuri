# 設計: 便 101 — 判断の記録に、判断どうしの改訂の欄 revises を足し、ADR-13 が ADR-8 を狭めた 1 対を書き写す（FR19 / NFR3）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から決定的に導出する）/ NFR3（文書間の参照 id の未解決は rules 行 R-4 の値にする＝参照は必ずつながる）
- 条: P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.2（文書と台帳からは規則を id で参照する）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-12.2（承認は逐語と日付を添えて記帳する）/ N-2.1（散文にしか無い規則を規則として扱わない）/ N-3.1（規則の例外機構を足さない）
- 出所: 判断の記録 ADR-13 決定 (14)。逐語は 本判断は判断の記録 ADR-8 の決定 (4)（天井が合格でない間は設計文書の便の着地を既定で止める）を狭める。…判断の記録どうしの改訂を記す型付きの欄が今は無い（改訂の範囲の欄は条を指す欄で、判断の記録を指せない）ので、その欄を欄の決まりへ足す便を列に入れる（実装の型付きの定数の改訂・便 1 本）。その便が着地したら、本判断はその欄で ADR-8 を指す行を持つ。それまでは帰結の文章だけが記帳である。 である。同じ判断の決定 (16) の便の列の ⑦、設計ノート docs/design/graph-and-incremental-ceiling.md §8 の行 G7a（判断の記録の欄の決まりに 判断 → 判断 の改訂の欄を足す・実装の定数 → 生成区間・触る src は `adr.rs`・余地 257）。雛形は便 92（docs/design/delivery-92.md・帰結の欄 produced を足した便）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cx が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規の file は無い・縮む file も無い）。
- 門: 本便は design-intent の下の正本（`adr/`）を書き換えるので天井の門の対象である。席が実測した `folio ceiling --gate --write-set`（本便の 6 種）は **2（まだ分からない・印が古い）** を返す（2026-09-22・main 420d910）。持ち主の裁定（2026-09-22 00:0x JST・対話面 R-8・逐語「一通り完成を目指して速度を上げたいので、あまりにも無駄に似たような審査を繰り返しまくっているならやめてどんどん進めて」・台帳 f2-648 notes）を規則の表の開発規律行 D-12 が言う持ち主の裁定として受け、設計文書を触る便は印が古くても門を経ずに出す。
- 並行する便: 便 99（`crates/folio/src/graph.rs`・`stamp.rs`・`gate.rs`・`design-intent/graph.yaml`・`tests/fixtures/schema/` の 4 本・`crates/folio/tests/graph.rs`・`tests/stamp.rs`・`tests/schema_docs.rs`）と、本便の write-set は **1 本も重ならない**。便 100（行 cw）とも重ならない。

## 1. 目的と中身

判断の記録は、**発効した判断が生きたまま、その決定の範囲が別の判断で変わる**ことを記す型付きの欄を持たない。憲法の条の改訂は amends と条の側の amended_by が持ち、判断を丸ごと置き換える形は supersedes と superseded_by が持ち、その判断が生んだものは便 92 が足した produced が持つが、**判断どうしの改訂**はどれでも受けられない。ADR-13 は自分が ADR-8 の決定 (4) を狭めたことを帰結の散文にしか書けず、決定 (14) が「その欄を欄の決まりへ足す便を列に入れる」と自認している。本便はその 1 本で、判断の記録に改訂の欄 revises を足し、ADR-13 の 1 対を書き写す。欄の決まりの正本は実装の型付きの定数（P-5.6）で、設計文書の側の生成区間へは `folio schema --write` が導出する。

### (a) 実測（2026-09-22・main 420d910・便 97 の着地後。席は当てた木で測り、戻した）

- 器の受付の行数の式（空行を含む全行を数え、1 行の字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測った値と余地（1500 − 値）。src = **`adr.rs` 1287・余地 213**。歯 = `tests/adr.rs` 353・余地 **1147**／`tests/schema.rs` 624・余地 **876**。正本 = `design-intent/adr/ADR-13.yaml` 418・余地 1082／`design-intent/adr/schema.yaml` 198・余地 1302。本便が触る src は `adr.rs` の 1 本だけで、**余地 213 は S の見積（100）を上回るが M の見積（300）には足りない**。本便の実測の増分は **+88**（器の式・生の行では +85）で、着地後の `adr.rs` は 1375・余地 125 になる。設計ノート §8 の行 G7a も大きさを S と見積もっている。
- 判断の記録の行の欄の閉じた一覧の正本は `crates/folio/src/adr.rs` の定数 RECORD で、required 10 語・optional 9 語（amends・grill・approval・consequences・produced・supersedes・superseded_by・note・figures）である。同じ file の FLOOR がその 2 本を木として持ち、`folio schema` はこの木を `adr/schema.yaml` の生成区間へ導出する。
- **受け皿が無いことを実測で確かめた。** amends の各項は target・field・version・previous_text・new_text の 5 欄で、注（amends_note.target）が target を 条 id か、改訂の範囲の節 と定めており、version は 改訂で上がる憲法の版 である＝判断の記録は指せない。supersedes と superseded_by は retired の後継の列で、注（supersede_note）が 後継が proposed の間は supersedes を書かない と定めている＝ADR-8 は accepted のままなので使えない。produced は 生んだもの で、注が 条の改訂は amends と amended_by が持つ と自認している。実の `design-intent/adr/*.yaml` 13 本を席が全部読み、判断の記録を指す型付きの欄は basis（根拠）と produced（帰結）の 2 つだけで、**改訂を表す欄は 1 つも無い**ことを確かめた。本便は不要ではない。
- **ADR-8 は ADR-13 の basis に在る。** 帰結の欄 produced は 自分の id と basis に在る id を受けない（便 92）が、改訂の欄は同じ縛りを置かない＝ある判断の決定を狭めるときにその判断を根拠に挙げるのは普通の形で、禁じると書ける場所が無くなる。本便は basis との重なりを床で見ない。
- **id の形を解く口は既に在る。** 同じ file の `is_adr_id`（判断の記録の id の全体一致・ゼロ詰めを落とす）がそのまま使える。切り出しも新しい関数も要らない。
- **実在の突き合わせも既に在る。** `crates/folio/src/link.rs` の `references` が判断の記録の各記録を全欄歩き（walk）、`scan_adr_ids` が拾った判断の記録の id を、実在しなければ種別 adr の違反として出す。**`link.rs` は 1 字も触らない。** 席は当てた木で実測した（`target` を ADR-99 にすると 種別 adr の違反 1 件・終了コード 1）。
- **欄の集合と値域を見る口も既に在る。** 同じ file の `check_keys`（欄の集合）・`non_empty`（非空）・`in_enum`（値域）・`present`（欄の有無）・`show`（値の見せ方）をそのまま呼ぶ。新しい module も新しい旗も持たない。
- `adr.rs` の中の単体の歯 `floor_notes_are_outside_the_diff` は FLOOR の `_note` で終わる欄の本数を **23** で凍結している。revises_note を足すので **24 に直す**。同じ file の `adr_floor_derives_the_frozen_anchor_byte_for_byte` は FLOOR の導出が凍結 anchor と byte 一致することを見るので、anchor を直せば緑になる。
- 歯の関数名の接頭辞。`grep -rn 'fn f101_' crates/folio` は今 0 本（使われている最大は f97_・便 99 が f99_・便 100 が f100_ を使う）。

### (b) 足す欄と、書き写す 1 対

欄の名は **revises**。判断の記録の記録の**任意**の欄で、値は**表の一覧**である。1 項が 1 つの改訂を表し、欄は 4 つとも必須で、任意の欄は持たない。

| 欄 | 中身 |
|---|---|
| target | 改訂する先の判断の記録の id（ADR-n）。自分の id は書かない |
| decision | 改訂する決定の番号（その判断の decision の中の番号の字・例 (4)） |
| kind | 改訂の向き。narrow = 決定の範囲を狭める／widen = 広げる |
| summary | その決定の何をどう変えたかの 1 文 |

**id だけの一覧にしない理由。** 決定 (14) が求めるのは「判断どうしの改訂を**記す**」ことで、どの決定をどちらへ動かしたかが失われると、記帳としても辺としても basis と区別がつかない。逆に、amends が持つ previous_text と new_text の逐語の消し込みは**持たない**。判断の記録は憲法と違って版ごとの凍結 anchor を持たないので、床が字面を突き合わせる相手が存在しない（P-10.2 が禁じるのは生成物どうしの突き合わせだが、ここは突き合わせる相手そのものが無い）。向きが本当かは人が読む（P-12.2）。

**向きの値域を 2 つに閉じる理由。** narrow と widen だけとし、**置き換え（replace）は値に入れない**。判断を丸ごと置き換える形は supersedes と superseded_by が既に持っており、値を 3 つにすると同じ関係を 2 つの欄のどちらに書くかが人の判断に戻る（条 N-2 の向きから遠ざかる）。

**改訂される側に来歴の欄は置かない（片側だけ）。** 憲法の側の amended_by が要るのは、凍結 anchor との差分を欄単位で消し込むためで、その消し込みが無い判断の記録では双方向にしても床が確かめられるものが 1 つも増えない。散文の言及を型付きの欄へ書き写す規則（規則の表の行 R-17）も行ごとに数えるので、**改訂する側に欄が在れば足りる**＝ADR-13 の散文が ADR-8 を指すのに対し、ADR-8 の散文は ADR-13 を指さない（ADR-8 は先に発効している）。この判断は revises_note の reverse に逐語で置く。

**書き写す 1 対**は次のとおり。ADR-13 の 1 本だけで、ほかの 12 本の判断の記録には revises を置かない（席が 13 本の帰結と注を全部読み、判断どうしの改訂を述べているのは ADR-13 の決定 (14) だけであることを確かめた）。

| 判断の記録 | target | decision | kind | 出所の欄 |
|---|---|---|---|---|
| ADR-13 | ADR-8 | (4) | narrow | decision の (14) と consequences の末尾から 2 つ目 |

書く場所は ADR-13 の **basis の次・decision の前**で、1 行の流れ（flow）の表とする。逐語は次の 2 行（前に空行 1 行を置く）。

```
revises:
  - {target: ADR-8, decision: (4), kind: narrow, summary: 天井が合格でない間に設計文書の便の着地を既定で止める範囲を、repo 全体から便が書き換える file の側へ狭める（不合格の側は今のまま repo 全体で止める）}
```

### (c) 床の側の判定（`crates/folio/src/adr.rs` だけを触る）

1. 欄の字の定数 `REVISES`（値は revises）を 1 本置き、RECORD.optional を 9 語から 10 語へ増やす。置き場は **amends の次・grill の前**（条の改訂の隣に判断の改訂を並べる）。
2. 項の欄の集合 `REVISES_ENTRY`（required は target・decision・kind・summary の 4 語、optional は空）を 1 本置く。置き場は AMENDS_ENTRY の次。
3. 向きの値域 `REVISE_KIND`（narrow・widen）を 1 本置く。置き場は VERDICT の次。
4. FLOOR に 3 つ足す。`enums` の表の末尾に revise_kind の 1 行（既存の 5 つの後ろ）・`revises_entry`（欄の集合の木）・注 `revises_note`（5 つの小見出し target・decision・kind・summary・reverse）。revises_entry と revises_note の置き場は **amends_note の次・grill の前**とする。注の字は (d) の anchor の逐語と 1 字も違わないこと。
5. 記録の検査に枝を 1 本足し、置き場は amends の枝の後・承認の枝の前とする。中身は、欄が無いなら何もしない。表の一覧なら各項について、① `check_keys` で欄の集合を見る（合わなければその項はそこで打ち切る）② 4 欄それぞれが非空でなければ 種別 adr の違反を 1 件（字面に その項の道 と 欄名 と が空 を含む）③ target が判断の記録の id の形でないか自分の id と同じなら 種別 adr の違反を 1 件（字面に target が判断の記録の id でない を含む）④ kind が値域に無ければ 種別 adr の違反を 1 件（字面に kind と その値 と 値域でない を含む）⑤ target と decision の対がその記録の中で 2 度目なら 種別 adr の違反を 1 件（字面に decision と その値 と 2 行に在る を含む）。一覧でなければ 種別 adr の違反を 1 件（字面に revises が一覧でない を含む）出す。
6. 単体の歯 `floor_notes_are_outside_the_diff` の注の本数を 23 から 24 に直す。
7. **実在は数えない。** (a) のとおり `link.rs` の `references` が判断の記録の全欄を歩いて数えるので、ここで重ねると 1 つの誤りが 2 件になる。

新しい module・新しい旗・例外の口（無効化の旗・今回だけの口）は持たない（N-3.1）。

### (d) 生成区間と凍結 anchor

`folio schema --write` が `design-intent/adr/schema.yaml` の生成区間を **127 行 22701 byte から 136 行 24202 byte** に書き直す。増えるのは 9 行だけで、ほかの 8 file の生成区間は 1 byte も動かない（席が `folio schema --check` を当て、9 file とも 一致 で終了コード 0 になることを確かめた）。`schema.rs` の TARGETS は既に `adr/schema.yaml`（`adr::FLOOR`）を持つので **`schema.rs` も `main.rs` も 1 行も触らない**。要件 FR19 の対象も増えない。

増える 9 行と置き場は次のとおり。

- `    - amends` の次に 1 行。

```
    - revises
```

- `    surface: [R-8]` の次に 1 行。

```
    revise_kind: [narrow, widen]
```

- `  grill:` の行の直前に 7 行。

```
  revises_entry: {required: [target, decision, kind, summary], optional: []}
  revises_note:
    target: 改訂する先の判断の記録の id（ADR-n）。自分の id は書かない。実在は判断の記録の全欄の走査が数える。条の改訂は amends と amended_by が持ち、判断を丸ごと置き換える形は supersedes / superseded_by が持つ＝この欄は「発効した判断が生きたまま、その決定の範囲が別の判断で変わる」ときだけに使う
    decision: 改訂する決定の番号（その判断の decision の中の番号の字・例 (4)）。1 本の記録の中で target と decision の対は一意＝同じ決定を 2 行で書かない
    kind: 改訂の向き。narrow = 決定の範囲を狭める／widen = 広げる。床は値域だけを見て、向きが本当かは人が読む（P-12.2）
    summary: その決定の何をどう変えたかの 1 文。逐語の突き合わせ（amends の previous_text / new_text）は持たない＝判断の記録は版ごとの凍結 anchor を持たないので、床が字面を突き合わせる相手が無い（P-10.2）
    reverse: 改訂される側に来歴の欄は置かない（片側だけ）。改訂の有無は改訂する側のこの欄から数える＝条の改訂の来歴（amended_by）と違い、凍結 anchor との消し込みが無いので双方向にしても床が確かめられるものが増えない
```

凍結 anchor `tests/fixtures/schema/adr-region.txt` は、生成器にも検査側にも依らずに組む（P-10.2）。組み方は、いまの anchor の 3 か所へ上の 9 行を差し込むことで、OS の道具だけで足りる。**席は 2 通りで独立に組んで突き合わせた**＝① 床の定数を直した binary の `folio schema --write` が書いた file から生成区間を切り出したもの、② いまの anchor（127 行）に上の 9 行を差し込んだもの。2 つは **1 byte も違わず**、行数・byte 数・要約値も同じだった（2026-09-22）。

- 新しい anchor の行数 = **136**・byte 数 = **24202**・要約値（sha256）= **0c95ab5011ff3a09ec3b6cdb4dd0bc3a6150ca5ca11ee92c7e233e96520f9ef3**
- **本便の前の main ではこの要約値はどこにも無いので、これを見る歯は赤い歯である。**

凍結の定数は `crates/folio/tests/schema.rs` の 3 つを直す。REGION_LINES を 127 から 136・REGION_BYTES を 22701 から 24202・REGION_SHA256 を上の要約値へ。**`tests/schema_docs.rs` は 1 字も触らない**（便 89 が移したのは判断の記録以外の 6 本の正本の側で、判断の記録の側は `tests/schema.rs` に残っている）。

### (e) 欄の決まりの写し 17 本

`folio check` は `adr/schema.yaml` の schema の節を床の定数と突き合わせる（種類 adr の違反「床の定数と違う」）ので、閉じた一覧と欄の集合を増やすと写しの側も直さないと歯が落ちる。席は当てた木で実測し、写しを直す前は workspace の nextest が 5 本落ち、字面でなく値を見る突き合わせなので **写し 1 本につき 3 か所**を直すと全部緑になった。

1. optional の 1 行の置き換え（写しは流れの 1 行で持つ＝生成区間の block の形には開かない）。

```
  optional: [amends, revises, grill, approval, consequences, produced, supersedes, superseded_by, note, figures]
```

2. `    surface: [R-8]` の次に 1 行を足す。

```
    revise_kind: [narrow, widen]
```

3. `  grill:` の行の直前に 1 行を足す（注 revises_note は突き合わせの外なので写しには置かない）。

```
  revises_entry: {required: [target, decision, kind, summary], optional: []}
```

直す 17 本: `tests/fixtures/adr/effective-no-approval/adr/schema.yaml`・`tests/fixtures/adr/schema-drift/adr/schema.yaml`・`tests/fixtures/adr/two-adopted/adr/schema.yaml`・`tests/fixtures/anchor/no-anchor/adr/schema.yaml`・`tests/fixtures/anchor/root-digest-drift/adr/schema.yaml`・`tests/fixtures/check/dup-key/adr/schema.yaml`・`tests/fixtures/check/empty-field/adr/schema.yaml`・`tests/fixtures/check/unknown-section/adr/schema.yaml`・`tests/fixtures/floor_base/design-intent/adr/schema.yaml`・`tests/fixtures/link/adr-id-missing/adr/schema.yaml`・`tests/fixtures/link/amended-by-orphan/adr/schema.yaml`・`tests/fixtures/link/retreat-kind-drift/adr/schema.yaml`・`tests/fixtures/refs/bad-counts/adr/schema.yaml`・`tests/fixtures/refs/dangling-id/adr/schema.yaml`・`tests/fixtures/refs/orphan-rule/adr/schema.yaml`・`tests/fixtures/vocab/exemptions/adr/schema.yaml`・`tests/fixtures/vocab/unknown-word/adr/schema.yaml`。席は `find` で全数を列挙し（実の正本を除いて 17 本）、**17 本とも直す前の 3 か所の字が同じで、置き場も同じ**であることを実測した。

### (f) 面と索引の側 — 本便では出さない

- **判断の記録の面には出さない。** 面の章は 5 つ（問題・決定・案・根拠と撤退条件・改訂と帰結）で閉じており、章 05 には既に散文の帰結と改訂の一覧が出ている。id の改訂を面へ出すかは見た目の裁定を伴うので、便 91・便 92 と同じく面の便へ回す。`face_adr.rs` は自分が読む欄だけを描くので、知らない欄が 1 つ増えても落ちない（席が当てた木で面の歯を含む nextest 743 件が全部緑だった）。
- **索引（`folio graph`）の辺の欄の閉じた一覧にはまだ足さない。** その一覧は `crates/folio/src/graph.rs` に在り、**その file は並行する便 99 の write-set に在る**ので、本便で触ると 2 便が同じ file を取り合う。席は当てた木で `folio graph --print` を実測し、本便の前後で **節点 198・辺 777・型 15・端が節点でない参照 27** が 1 つも動かないことを確かめた＝revises の 1 対はまだ索引の辺にならないが、**床は落ちない**。規則の表の行 R-17（散文の中の id の言及は型付きの欄にも在ること）も落ちない＝ADR-13 の散文の ADR-8 の言及は既に basis が受けているからで、席は workspace の nextest 743 本が全部緑であることで確かめた。索引へ足すのは便 99 の着地後の別の便である（(i) に置く）。
- **要件書・規則の表・憲法・語彙は 1 字も触らない。** 欄を足す根拠は FR19 と NFR3 で既に在り、新しい要件は要らない。生成区間の対象 file も増えない。行の追加も変更も起きないので裁定 id は要らない（P-17.1）。憲法の条文は 1 字も動かない（ADR-13 決定 (15)）。

### (g) 歯（関数名は f101_ で始める・置き場は `crates/folio/tests/adr.rs`）

`tests/adr.rs` の Work は実の design-intent を一時 dir へ写し、器の導出 file を写しの根に置き、git init と 1 commit を行う。写しの字面を変異させる口 `mutate` は当て先が ADR-1.yaml に固定なので、**同じ形で file を指せる口を 1 つ足す**（当て先が 1 か所でなければ自分で落ちる・既存の `mutate` は 1 字も変えない）。新しい歯の file も新しい dir も作らない。変異の当て先はどれも (b) の 2 行の中に 1 か所だけ在ることを席が実測した。

1. `f101_the_real_record_carries_the_revises_row` — 実の判断の記録の写しで素の床が 終了コード 0・違反 0。写しの `adr/` の下で revises の欄を持つ file がちょうど **1 本**（ADR-13.yaml）で、その項が 1 つ、target が ADR-8、decision が (4)、kind が narrow であること。本便の前の main には revises の欄が 1 つも無いので **赤い歯**。
2. `f101_an_article_id_target_is_a_violation` — target を P-6 に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-13 と revises と P-6 と target が判断の記録の id でない が出る。変異の当て先が無いので **赤い歯**。
3. `f101_the_record_itself_as_target_is_a_violation` — target を ADR-13 に替えて素の床 → 終了コード 1・違反 1 件で、行に同じ字面が出る（自分の id も書かない）。同じ理由で **赤い歯**。
4. `f101_an_adr_that_does_not_exist_is_a_violation` — target を ADR-99 に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-13.revises[0].target と ADR-99 と 実在しない が出る（(a) のとおり `link.rs` の網が受け持つことを歯で押さえる）。同じ理由で **赤い歯**。
5. `f101_a_kind_outside_the_enum_is_a_violation` — kind を replace に替えて素の床 → 終了コード 1・違反 1 件で、行に kind と replace と 値域でない が出る。同じ理由で **赤い歯**。
6. `f101_revises_that_is_not_a_list_is_a_violation` — 2 行を表 1 つ（一覧でない値）に替えて素の床 → 終了コード 1・違反 1 件で、行に ADR-13 と revises が一覧でない が出る。同じ理由で **赤い歯**。
7. `f101_the_same_decision_twice_is_a_violation` — 同じ項を 2 行に増やして素の床 → 終了コード 1・違反 1 件で、行に decision と (4) と 2 行に在る が出る。同じ理由で **赤い歯**。

**7 件とも席が当てた木で実測し、7 件とも違反はちょうど 1 件だった**（2026-09-22・実の正本の写しで測った）。

回帰は verify の 2 行目で見る。`tests/adr.rs`（本便の前の本数 ＋ f101_ の 7 本）と `tests/schema.rs`（本数は変わらず、凍結の 3 つの値だけが変わる）。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない（`adr.rs` の中の単体の歯 `floor_notes_are_outside_the_diff` と `adr_floor_derives_the_frozen_anchor_byte_for_byte` は `.vessel.toml` の common-verify の workspace の nextest が見る）。席は当てた木で **workspace の nextest 743 本が全部緑**であることを実測した（本便が足す 7 本を入れて、着地後は 750 本になる）。

### (h) 大きさ

src は `crates/folio/src/adr.rs` の 1 本だけ（1287・余地 **213**・**+88 の実測**・着地後は 1375・余地 125）。歯は `crates/folio/tests/adr.rs`（353・余地 1147・+110 行の見込み）と `crates/folio/tests/schema.rs`（624・余地 876・**行数は不変**で凍結の 3 つの値だけが変わる）。正本は `design-intent/adr/schema.yaml` の生成区間が 127 行から 136 行になり、`design-intent/adr/ADR-13.yaml` が 3 行（空行 1・欄の見出し 1・項 1）増える（418 → 422・余地 1078）。凍結 anchor `tests/fixtures/schema/adr-region.txt` は 127 行から 136 行になり、欄の決まりの写し 17 本はそれぞれ 3 か所が変わる。size **S**（触る src は `adr.rs` 1 本で、余地 213 は S の見積 100 を上回る。**M の見積 300 には足りないので本便は S に収める**）。新しい file も新しい dir も無く、縮む file も無い。外部 crate は増やさない。

### (i) 本便が運ばないもの・撤退条件

- 判断の記録の面への表示（(f)）。面の章の見た目の裁定を伴うので、面の便に回す。
- 索引の辺の欄の閉じた一覧に revises を足すこと（(f)）。`crates/folio/src/graph.rs` は並行する便 99 の write-set に在るので、便 99 の着地後の別の便で運ぶ。
- 改訂される側の来歴の欄（(b) の 片側だけ の判断）。双方向にしても床が確かめられるものが増えないという理由が偽になったとき（= 改訂の対が 1 対でなくなり、改訂された判断を読む人がそれを知る道が要ると分かったとき）に、別の便で問う。
- 要件の項がほかの要件・制約を指す欄（12 対）。ADR-13 決定 (3-b) が便 90 の後の数えを見てから決めると保留している。
- 憲法の条文・規則の表・語彙・要件書・判断の記録の本文（決定・案・撤退条件・平易文・帰結）・ほかの 8 file の生成区間・`face_adr.rs`・`link.rs`・`refs.rs`・`check.rs`・`rules.rs`・`schema.rs`・`main.rs`・`graph.rs`・`mentions.rs`・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・`tests/schema_docs.rs`・CI の yml。
- 撤退条件: revises の欄が機械で数えられる形に閉じていられなくなったとき（向きが narrow でも widen でもない対が出て、supersedes との使い分けが人の判断に戻るとき）は、(c) 5 の枝を `adr.rs` から外し、欄を RECORD.optional から落として ADR-13 の 2 行を消す。便 1 本で戻せる。欄を残したまま向きの値域を 3 つ目へ広げる形は取らない（広げた瞬間に supersedes と revises のどちらに書くのかが人の判断に戻るため）。

## 2. 範囲

- 入れる: 欄の字の定数 1 本・閉じた一覧 1 本に 1 語・項の欄の集合 1 本・向きの値域 1 本・FLOOR に 3 つ（enums の 1 行・revises_entry・revises_note）・床の枝 1 本・注の本数の凍結 23 → 24・ADR-13 の revises の 2 行・生成区間 127 行 → 136 行・凍結 anchor 1 本・欄の決まりの写し 17 本の 3 か所ずつ・`tests/schema.rs` の凍結の定数 3 つ・`tests/adr.rs` の変異の口 1 つと f101_ の歯 7 本。
- 入れない: 面の側の表示・索引の辺の欄の一覧・改訂される側の来歴の欄・要件どうしの欄・憲法と規則の表と語彙と要件書・判断の記録の本文・`link.rs` と `refs.rs` と `check.rs` と `rules.rs` と `face_adr.rs` と `schema.rs` と `main.rs` と `graph.rs` と `mentions.rs`・`tests/schema_docs.rs`・`tests/floor_cases.yaml`・id の一覧の凍結 anchor・新しい file と新しい dir。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| field | 新しい欄 | 判断の記録の revises（target・decision・kind・summary の 4 欄を持つ表の任意の一覧） |
| floor | 床の枝 | `crates/folio/src/adr.rs` の定数 3 本と閉じた一覧 1 語と FLOOR の 3 つ・記録の枝 1 本 |
| row | 書き写し | `design-intent/adr/ADR-13.yaml` の revises の 1 対（ADR-8 の決定 (4) を狭める） |
| region | 生成区間 | `adr/schema.yaml` の生成区間 136 行（`folio schema --write` が書く） |
| anchor | 凍結 anchor | `tests/fixtures/schema/adr-region.txt`（136 行・24202 byte） |
| copies | 欄の決まりの写し | `tests/fixtures/` の下の 17 本の `adr/schema.yaml` の 3 か所ずつ |
| pins | 凍結の定数 | `tests/schema.rs` の 3 つの値と `adr.rs` の注の本数 |
| teeth | 歯 | `crates/folio/tests/adr.rs` の変異の口 1 つと f101_ 7 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。便 99（行 cu）とも便 100（行 cw）とも、書き換える file が 1 本も重ならない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cx"
title = "判断の記録に、発効した判断が生きたままその決定の範囲が別の判断で変わることを記す任意の改訂の欄 revises を足す。各項は target（改訂する先の判断の記録の id）・decision（改訂する決定の番号の字）・kind（向き・narrow か widen の 2 値）・summary（1 文）の 4 欄を持つ表で、逐語の突き合わせは持たない（判断の記録は版ごとの凍結 anchor を持たないので床が字面を突き合わせる相手が無い）。改訂される側に来歴の欄は置かず片側だけとする。欄の決まりの正本は実装の型付きの定数（adr.rs の閉じた一覧と欄の集合と値域と注）で、adr/schema.yaml の生成区間へ folio schema --write で導出する。床は欄の集合と 4 欄の非空と、target が判断の記録の id で自分の id でないことと、kind が値域に在ることと、target と decision の対が記録の中で一意であることと、一覧であることを数え、実在は既に在る link.rs の網に任せる。書き写すのは判断の記録 ADR-13 が ADR-8 の決定 (4) を狭めた 1 対（ADR-13 決定 (14) が便を列に入れると自認している対）で、面と索引の辺の欄の一覧には当面出さない"
req = ["FR19", "NFR3"]
section = "1"
write-set = ["crates/folio/src/adr.rs", "design-intent/adr/schema.yaml", "design-intent/adr/ADR-13.yaml", "tests/fixtures/schema/adr-region.txt", "tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "tests/fixtures/adr/schema-drift/adr/schema.yaml", "tests/fixtures/adr/two-adopted/adr/schema.yaml", "tests/fixtures/anchor/no-anchor/adr/schema.yaml", "tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "tests/fixtures/check/dup-key/adr/schema.yaml", "tests/fixtures/check/empty-field/adr/schema.yaml", "tests/fixtures/check/unknown-section/adr/schema.yaml", "tests/fixtures/floor_base/design-intent/adr/schema.yaml", "tests/fixtures/link/adr-id-missing/adr/schema.yaml", "tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "tests/fixtures/refs/bad-counts/adr/schema.yaml", "tests/fixtures/refs/dangling-id/adr/schema.yaml", "tests/fixtures/refs/orphan-rule/adr/schema.yaml", "tests/fixtures/vocab/exemptions/adr/schema.yaml", "tests/fixtures/vocab/unknown-word/adr/schema.yaml", "crates/folio/tests/adr.rs", "crates/folio/tests/schema.rs"]
verify = ["cargo nextest run -p folio --test adr f101_", "cargo nextest run -p folio --test adr --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f101_ の歯 7 本（実の判断の記録の写しで床が終了コード 0 で違反 0 かつ revises を持つ file が ADR-13 の 1 本だけで項は target ADR-8・decision (4)・kind narrow の 1 つ／条の id を target にすると違反 1 件／自分の id を target にすると違反 1 件／実在しない判断の記録を target にすると違反 1 件／値域外の向きで違反 1 件／一覧でない値で違反 1 件／同じ target と decision の対を 2 行に書くと違反 1 件）が全部緑、tests/adr.rs と tests/schema.rs の既存の歯が全部緑（生成区間 136 行 24202 byte と凍結 anchor が byte 一致し、凍結の要約値が sha256sum の測り直しと一致する）、folio schema --check が 9 file とも一致、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
