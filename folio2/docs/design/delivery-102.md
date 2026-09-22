# 設計: 便 102 — 天井の読む文書の一覧に索引の欄の決まりを足し、材料の束の骨格の 6 語を生成区間へ写す（FR19 / FR18）

- 要件: FR19（欄の決まりの file と天井の正本ほかの生成区間へ、床の定数から決定的に導出した決まりの部分〔観点と読む文書の閉じた id の集合を含む〕を書く）/ FR18（所見 file の欄の決まり・観点の一覧の一致・根拠の逐語の実在を数え、観点ごとに 3 値を返す）
- 条: P-2.4（閉じた一覧の追加には判断の記録を要する）/ P-5.1（規則・閾値・型の一覧は型付きデータに置く）/ P-5.6（実装の型付きの定数を規則の正本とするあいだ、その写しを設計文書の置き場へ決定的に導出する）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.4（2 つの面を人が書き一致を検査で強制する設計を採らない）/ P-10.1（独立した凍結 anchor を 1 本以上持つ）/ P-12.2（受け取った承認は逐語と日付を添えて記帳する）/ A-2.1（版を上げる前に承認を得る）/ N-3.1（規則の例外機構を足さない）
- 出所: 一括 12 の仕分け `docs/design/batch12-triage.md` の問 4 の 2 と 3。持ち主の承認は 2026-09-22 20:52 JST・対話面 R-8・逐語「全部承認する」（台帳 f2-648 notes・一括 12 の承認要求の問い 5 つへの回答）。同じ仕分けの「本一括で動かさなかったもの」は、文書の一覧を足すと実装の固定長の定数 2 本と生成区間と凍結 anchor が同時に動くので一括では運べず、**承認だけを問い、便 1 本で運ぶ**と書く。連れて動くものの列挙は `docs/design/delivery-95.md` §1、骨格の 6 語は `docs/design/delivery-97.md` §1 (d)、固定長 2 本の実在は検証役の報告 `~/.local/share/folio2/handoff-2026-09-22/d94-verify.md` §6 (7)。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cz が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（**新しい file は 1 本も無い**・縮む file も無い・新しい dir は作らない）。
- 門: 本便は design-intent の下の正本 `design-intent/ceiling.yaml` を書き換えるので、天井の門の対象である。起草役が本便の write-set 28 本をそのまま `folio ceiling --gate --dir design-intent --write-set …` に渡して実測すると **2（まだ分からない・断りの字は 印が古い）**（2026-09-22・main 278ba59）。判定は本便の中身によらない（印は 23 周目のまま）。持ち主の指示（2026-09-22 00:0x JST・対話面 R-8・台帳 f2-648 notes・逐語「一通り完成を目指して速度を上げたいので、あまりにも無駄に似たような審査を繰り返しまくっているならやめてどんどん進めて」の (2)）により、**設計文書を触る便は印が古くても門を経ずに出す**（便 95・99・101 と同じ扱い）。
- 前の便: 便 99（行 cu）・便 100（行 cw）・便 101（行 cx）と一括 12 は着地済みで、本便の base は **main 278ba59**。天井の正本は第 0.11 版が発効済み（読む欄を 8 対広げた分は本便に含まれない）。
- 並行の便との重なり: **便 98（PR #255・行 cy・束を章まで絞る）と `crates/folio/src/bundle.rs` が重なる。器が順に運ぶ。** 重なりの中身と、後に着地する側が何を合わせるかは §1 (f) に書く。**束の凍結 anchor `tests/fixtures/ceiling/bundle-anchor.txt` と所見 fixture 10 本の要約値は、本便では 1 字も動かない**（§1 (e) の実測）。便 98 の設計ノート §0 は「どちらが先でも後の便が束の anchor を組み直す」と書くが、本便の側は組み直す必要が無い。
- 並行の便との重なり（2 本目）: **便 103（PR #257・行 da・設計ノートの欄の決まりの側）と `tests/fixtures/schema/node-digest-anchor.txt` と `crates/folio/tests/graph.rs` の 2 本が重なる。器が順に運び、後に着地する側が §1 (e) の手順（独立の script `tests/fixtures/schema/node-digest.py` で組み直し、その file の要約値を `F99_ANCHOR_SHA256` へ写す）で anchor を作り直す。** したがって **§1 (e) の表の 2 つの 16 進の値（残差の要約値と anchor file の要約値）は、本便が先に着地したときの値である**。便 103 が先なら、本便の作業者は手順だけを踏み、値は script の出力に従う（§1 (e) の表の 2 値は使わない・行数 192 と byte 数 3,007 は便 103 の側でも変わらない見込みだが、これも script の出力に従う）。
- 改訂 b（2026-09-22 22:0x JST・検証役の report `~/.local/share/folio2/handoff-2026-09-22/d102-verify.md` への応答）: §1 (b) に ③（天井の正本の版と承認欄の逐語）を足し、§0 に便 103 との重なりを足し、§1 (h) の `tests/graph.rs` の行と §1 (i) の門の 1 文の字を直した。**行 cz・write-set・歯 4 本・done・生成区間と 2 つの凍結 anchor の値は 1 つも変えていない**（meta と承認欄の行は生成区間の外なので anchor は動かない。起草役が写しの repo で逐語のとおりに書き替え、生成区間が 3,176 byte のまま・2 つの anchor の要約値が同じまま・`folio check` が違反 0・workspace 全体の歯 764 本が緑であることを測り直した）。

## 1. 目的と中身

一括 12 の問 4 は天井の正本を 3 つの向きで動かすことを承認した。1 つ目（観点が読むと宣言する欄を 8 対広げる）は一括の PR で着地済みである。**本便は残る 2 つを 1 本で運ぶ。**

1. **索引の欄の決まりの正本 `design-intent/graph.yaml` を、天井が読む文書の閉じた一覧に足す**（9 → 10）。
2. **材料の束の骨格の 6 語（meta・id・title・status・date・schema）を、実装の型付きの定数から天井の正本の生成区間へ導出する**（P-5.6）。

design-intent の下で人が書く節を書き換えるのは `design-intent/ceiling.yaml` の 1 file だけで、**観点（4 つ）・重さの値域・観点ごとの読む欄（reads）・問いの文は 1 字も触らない**（条 N-5 の観点の増設に当たらない）。

### (a) 実測（2026-09-22・base main 278ba59）

**src と歯の余地**（幅 120 で正規化 = 全行 + 各行の ceil(len/120) − 1・上限は規則 R-C4-2 の 1,500）。増分は起草役が写しの repo で実際に書いて測った値である。

| file | 正規化の行 | 余地 | 本便の増分 |
| --- | --- | --- | --- |
| crates/folio/src/ceiling.rs | 532 | 968 | +35 |
| crates/folio/src/bundle.rs | 539 | 961 | +17 |
| crates/folio/tests/ceiling.rs | 529 | 971 | +136 |
| crates/folio/tests/schema_docs.rs | 1,137 | **363** | +3 |
| crates/folio/tests/graph.rs | 608 | 892 | +0（凍結の値 1 つを書き替えるだけ） |

**歯を `crates/folio/tests/schema_docs.rs` に置いてはいけない。** その file には便 89 が置いた別の上限があり、歯 `f89_schema_teeth_are_split_and_under_the_cap` が**器の式で 1,200 行以下**を数える。今が 1,137 行なので余地は **63 行しか無い**。起草役が最初に本便の歯 2 本をそこへ置いたら 1,248 行になり、この歯が落ちた（実測）。だから歯は `crates/folio/tests/ceiling.rs`（天井の正本の歯の file・余地 971）へ置き、`schema_docs.rs` は凍結の値 3 つと頭の注釈だけを直す。

**器の受付の余地の判定**（規則 pipe.size_m_lines = 300）は write-set の各 .rs に当たる。5 本の余地は 968 / 961 / 971 / 363 / 892 で、**いちばん小さい 363 でも M の見積 300 を上回る**。1 file あたりの増分のいちばん大きいのは tests/ceiling.rs の 136 で、S の見積 100 を超えるので **size は M**。

### (b) 何をどこへ足すか

**① 読む文書の一覧（9 → 10）。** 床の定数は 2 本あり、便 95 §1 が「連れて動く」と書いたとおり同時に動く。

| 場所 | 今 | 本便 |
| --- | --- | --- |
| crates/folio/src/ceiling.rs の DOCUMENT_IDS | `[&str; 9]` | `[&str; 10]`・末尾に graph |
| crates/folio/src/bundle.rs の FACE_NAMES | `[(&str, Option<FaceName>); 9]` | `[(&str, Option<FaceName>); 10]`・末尾に (graph, None) |

`FACE_NAMES` の graph は **面が無い（None）**。索引の欄の決まりは人が読む面を持たないので、rules・vocabulary・intake・ceiling と同じ扱いである。

`DOCUMENT_IDS` の順は `crates/folio/src/ceiling.rs` の単体の歯が `crate::check::FILES`（正本 7 file）+ adr + design-note と突き合わせているので、**graph は末尾に足し、その歯の名と当てる列に graph を足す**（`check::FILES` は 7 のまま触らない。graph.yaml は床 `folio check` が読む 7 file の仲間ではなく、`folio schema` が生成区間を書く 9 本目である）。

**② 骨格の 6 語（生成区間へ導出）。** 置き場は `crates/folio/src/ceiling.rs`（`BUNDLE_CONTENTS` の行の次）。名は `BUNDLE_SKELETON`、型は 6 の固定長の文字列の配列で、要素は順に meta・id・title・status・date・schema（Rust の文字列の literal として書く）。説明の注は 1 行で、束の sources の写しで常に残す最上位の節（順も同じ・`bundle.rs` が組む）と書く。

床の木 `FLOOR` の `bundle` の表へ、鍵 skeleton の葉（`Floor::Strs` に `BUNDLE_SKELETON` を渡した形）を `digest` の次に足し、`bundle_note` の末尾へ 1 文を足す。**新しい `_note` の欄は足さない**（歯 `ceiling_floor_notes_are_outside_the_diff` が注の数を 8 と数えているため・その歯は 1 字も触らない）。

**言い回しは、便 98 が着地する前でも後でも真になるものを選んだ。** 常に残す は、いま（絞らない＝全部残す）も、便 98 の後（宣言に無い節を落とす）も真である。本便が「いま束を絞っている」とは書かない。

**③ 天井の正本の版と承認欄（`design-intent/ceiling.yaml` の meta と頭の注釈）。** 版を **v0.11 → v0.12** に上げ（`status` は `effective` のまま）、承認欄（`meta.approval`）の末尾へ次の 2 行を足す。**逐語**はこれで、ASCII の二重引用符は要らない（起草役が folio 自身の読み口と独立の実装の 2 つで parse し、`20:52` を含む字が 1 つも欠けずに読めることと、`folio check` が違反 0 で通ることを測った）。

```
    - {role: 作成, who: orchestrator 席（AI・fable 5.1）, when: 2026-09-22, stamp: v0.12 起草（便 102・読む文書の一覧に索引の欄の決まりの正本 graph.yaml・束の骨格 6 語を生成区間へ導出）, version: v0.12}
    - {role: 承認, who: 持ち主（shuu5）, when: 2026-09-22, stamp: 発効（f2-648 notes 2026-09-22 20:52 JST・対話面 R-8・一括 12 の問 4 (2)(3)〔便 1 本で運ぶ〕への回答）, verbatim: 全部承認する, version: v0.12}
```

作成の行に `version` を付けるのは、v0.9・v0.10・v0.11 の 3 つの作成の行が同じ形だからである（形をそろえる）。

頭の注釈も 2 か所を直す。1 行目の版を `v0.12` にし、状態の行（`# 状態:` で始まる行）の末尾へ v0.12 の 1 文を足す。字は次のとおり。

```
 ／ v0.12 は発効（2026-09-22 20:52 JST 持ち主「全部承認する」・一括 12 の問 4 の 2 と 3 = 読む文書の一覧に索引の欄の決まり（graph.yaml）を足し〔生成区間を持つ file が読む文書の一覧に無いと、そこを書き換えても周の引き金が立たない穴が残る〕、材料の束の骨格の 6 語を生成区間へ写した〔P-5.6〕。観点の数と重さと観点の読む欄は不変）
```

**この承認は既に持ち主が与えたものなので、作業者が写してよい。** 根拠は 2 つで、どちらも本便より前に在る。① 台帳 f2-648 の notes の 2026-09-22 20:52 JST の項（対話面 R-8・逐語「全部承認する」・一括 12 の承認要求の問い 5 つ全部への回答・読み方は `bd --readonly show f2-648`）② 一括 12 の仕分け `docs/design/batch12-triage.md` の問 4（「1 は本一括の PR で、2 と 3 は次の便 1 本で運ぶ」）。**本便はその 2 と 3 そのもの**である。作業者が新しく承認を取りに行くことはしない（A-1 の対話面を開かない）。

**meta と承認欄は歯が 1 本も数えない。** 床は天井の正本の meta の `generated` と `approval` を数えず（`crates/folio/src/ceiling.rs` の頭の注釈が明記）、本便の歯 4 本もここを見ない。**だから書き落としても全部緑のまま着地する**＝この 2 行と版の字は、作業者が §1 の逐語から手で写すところである（P-12.2 の記帳がここで落ちると誰も落とさない）。歯の本数と done の項目数は 4 のまま変わらない。

### (c) 生成区間の逐語と、凍結 anchor の自己検査

`folio schema --write --dir design-intent` が書く生成区間は、今の 24 行・2,915 byte から **27 行・3,176 byte** になる。変わるのは 2 か所だけである（起草役が写しの repo で実際に走らせ、下の 3 つの値を sha256sum と wc で測り直した）。

**変わる行（前）**

```
  documents: [constitution, rules, vocabulary, srs, index, intake, ceiling, adr, design-note]
  bundle: {contents: [sources, faces, question, finding, reads], digest: sha256-files-1}
  bundle_note: 材料の束の中身（観点ごとに 1 つの置き場）と要約値の規則。要約値は束の file を path の byte 順に並べ、中身を連結した sha256（rules 行 R-15 の写しの要約値と同じ規則）
```

**変わる行（後・逐語）**

```
  documents: [constitution, rules, vocabulary, srs, index, intake, ceiling, adr, design-note, graph]
  bundle:
    contents: [sources, faces, question, finding, reads]
    digest: sha256-files-1
    skeleton: [meta, id, title, status, date, schema]
  bundle_note: 材料の束の中身（観点ごとに 1 つの置き場）と要約値の規則。要約値は束の file を path の byte 順に並べ、中身を連結した sha256（rules 行 R-15 の写しの要約値と同じ規則）。skeleton は sources の写しで常に残す最上位の節の閉じた一覧（観点が読むと宣言した欄に関わらず残す＝どの版の何を読んでいるかを決める欄）
```

**`bundle` が 1 行から 4 行に変わるのは体裁の規則の当然の結果である。** `crates/folio/src/schema.rs` は表を 1 行（flow）で書くのは幅 100 字までと決めている。skeleton を足した flow の 1 行は 100 字を超えるので block の形に落ちる。**この落ち方は導出の側が決めるので、`schema.rs` は 1 行も触らない。**

**`documents` の行はちょうど 100 字で、上限と同じである。** 幅の判定は 100 以下なら 1 行なので、この行は 1 行のまま残る。**文書をもう 1 本足すと、この行も block の形（1 行 + 10 行）に落ちる。** 次に文書を足す便はそれを織り込むこと（本便の範囲外・§1 (i) の 🔴 3）。

**凍結 anchor の自己検査の値**（`tests/fixtures/schema/ceiling-region.txt` と `crates/folio/tests/schema_docs.rs` の 3 つの定数）。

| 何 | 今 | 本便 |
| --- | --- | --- |
| 行数 | 24 | **27** |
| byte 数 | 2,915 | **3,176** |
| sha256 | 5ad2f19b…9fa2 | **1cc1401cc474f16b272e02cfa434e4fb5b00aead4b803b1d0adcb2fe9e4f5386** |

**変異の当て先も直す。** 歯 13・15 が使う 1 byte の変異は今 `, digest: sha256-files-1}`（flow の行）を当て先にしているが、block の形になるとその字が消える。当て先を `\n    digest: sha256-files-1\n` → `\n    digest: sha256-files-2\n` に替える（写しの中で 1 か所だけ・起草役が実測）。

### (d) 生成区間の写しの全数（21 本）

天井の正本の生成区間は、実の正本のほかに **fixture の写し 20 本**が同じ byte で持っている（起草役が 21 本すべてを走査して、今どれも凍結 anchor と byte 一致することを確かめた）。**床は写し全部しか受けない**ので、21 本とも同時に動かす。

| 置き場 | 本数 | 人が書く documents の行の形 |
| --- | --- | --- |
| design-intent/ceiling.yaml | 1 | 注（note）付き |
| tests/fixtures/floor_base/design-intent/ceiling.yaml | 1 | 注付き（短い） |
| tests/fixtures/ceiling/bundle/source/ceiling.yaml | 1 | 注付き（短い） |
| tests/fixtures/{adr,anchor,check,face,link,refs,vocab}/… の ceiling.yaml | 18 | 注なし |

**足す行（逐語・3 つの形）。** 実の正本はこれ（`ceiling` の行の次に置く）。

```
  - {id: graph, file: graph.yaml, note: 索引の欄の決まり。節点と辺の種類と型の閉じた一覧を持つ。索引の中身そのものはこの file に置かない（毎回 folio graph --print が正本から組み直す導出物）。末尾の schema の節は生成区間で、正本は実装の定数である。生成区間への所見の直す先は実装の側（便）で、file を手で直さない}
```

**凍結の土台 `tests/fixtures/floor_base/design-intent/ceiling.yaml` と `tests/fixtures/ceiling/bundle/source/ceiling.yaml` はこれ**（この 1 行の byte が (e) の残差の値を決めるので、字を変えない）。

```
  - {id: graph, file: graph.yaml, note: 索引の欄の決まり}
```

**ほかの 18 本はこれ**（注の欄を持たない写しなので注を付けない）。

```
  - {id: graph, file: graph.yaml}
```

**注の字は語彙の検査を通る。** 床は `documents` の行の note を英字の語の母集団に入れるので、実の正本の注に出る folio・graph・print・file・schema は語彙の正本に在る語であることを起草役が実測で確かめた（写しの repo で `folio check` が違反 0）。

**行の `file` が実在するかは床が数えない**（起草役が `crates/folio/src/ceiling.rs` の検査の口を読んで確かめた。数えるのは id の集合・欄の非空・重複・注の語だけ）。だから graph.yaml を持たない fixture の写しにも同じ行を足してよい。

**整合の観点の読み手の想定「9 種の文書」は直さない。** その字は整合の観点が読む文書の数（reads の 9 行）を指しており、本便は reads を 1 行も足さないので 9 のままが正しい。

### (e) 巻き添え（全数を固定した歯を全部当たった）

起草役が写しの repo で本便の変更を全部当て、workspace 全体の歯（760 本）を回して落ちる歯を数えた。**落ちたのは 4 本だけで、どれも凍結の値の写しである。**

| 落ちた歯 | 直す先 |
| --- | --- |
| schema_docs の 3 本（実の正本・1 byte の変異・書き直し） | (c) の 3 定数と変異の当て先 |
| graph の f99_the_independent_script_matches_the_anchor | 下の残差 |

**節点の要約値の凍結 anchor `tests/fixtures/schema/node-digest-anchor.txt` は、残差の 2 行だけが動く。** この anchor は凍結の土台 `tests/fixtures/floor_base/design-intent` に対して独立の script `tests/fixtures/schema/node-digest.py` が組む。土台の ceiling.yaml が 327 byte 増える（生成区間が 261 + 足す行が 66）ので、節点にも辺の欄にも属さない残りの byte が同じだけ増える。**節点の 189 行は 1 字も動かない。**

| 何 | 今 | 本便 |
| --- | --- | --- |
| 残差 sha256 | 5e7ac0d2…f655 | **377094880637f575e1c42f44bd18d902f72c239256441f0f30f243911ee62531** |
| 残差の byte | 141,004 | **141,331** |
| 合計の byte | 389,176 | **389,503** |
| anchor の行数・byte 数 | 192 行・3,007 byte | **不変**（数字の桁が同じ） |
| anchor file の sha256（`crates/folio/tests/graph.rs` の F99_ANCHOR_SHA256） | c0208821…7b7a | **b9ef20079bab64551044a24c3bf39a943f5a5b6c13d91e5014dbfecd8cfd4178** |

作業者は anchor を手で書かず、`python3 tests/fixtures/schema/node-digest.py tests/fixtures/floor_base/design-intent` の出力をそのまま file にして、その file の sha256 を `F99_ANCHOR_SHA256` へ写す（便 101 と同じ手順）。

**当たらなかったものも数えた（起草役の実測）。**

- **束の凍結 anchor `tests/fixtures/ceiling/bundle-anchor.txt` と `crates/folio/tests/bundle.rs` の凍結の 4 対、`tests/fixtures/ceiling/findings/` の要約値 10 本、印の凍結 `stamp-expected.yaml` は 1 字も動かない。** 理由は、束の sources に入るのは**観点の reads が指す文書だけ**で、凍結の土台 `tests/fixtures/ceiling/bundle/source/ceiling.yaml` の 4 観点が読むのは adr・constitution・design-note・index・rules・srs の 6 つだからである（`ceiling` も `graph` も読まない）。土台の ceiling.yaml を書き換えても束に入らない。便 97 の 3 本を含む束の歯は全部緑のままだった。
- `design-intent/index.yaml` の生成区間にも `documents:` の行が在るが、それは**入口の棚の文書の一覧**（constitution・srs・design-note・adr）で別の閉じた一覧である。問 3 の裁定どおり棚には足さないので 1 字も触らない。
- `crates/folio/tests/check.rs` の `p1_commands_closed_list`（命令も旗も足さない）・`tests/floor_cases.yaml` の 134 件・id の一覧の凍結 anchor・`tests/fixtures/schema/` のほかの anchor 11 本・欄の決まりの写し 17 本・`design-intent/preview/ceiling-stamp.yaml`。
- **`cargo fmt` は共通の検証に入っていない**（`.vessel.toml` の common-verify は nextest と clippy の 2 本）。起草役が base で `cargo fmt --check` を撃つと、本便が触らない file にも差分が出る＝この repo は rustfmt の字面で揃っていない。作業者は fmt で既存の file を直さない。

### (f) 骨格の一覧の置き場（便 98 との調停）

便 98 の設計ノート（PR #255）§1 は、骨格の 6 語の閉じた一覧を `crates/folio/src/bundle.rs` に置くと書き、その写しを生成区間へ導出するかは「便 102 と同じ承認に載る」として本便へ預けている。**本便は置き場を `crates/folio/src/ceiling.rs` にする。**

理由は 3 つ。① `ceiling.rs` の頭の注釈が「天井の床の定数は全部この 1 枚に置く」と書く（便 47・ADR-11 決定 (4)①）。② 束の中身の一覧 `BUNDLE_CONTENTS` も同じ file に在り、`bundle.rs` は `crate::ceiling::` から取っている＝骨格だけ別の file に置くと持ち方が 2 通りになる。③ 生成区間の正本は床の木 `FLOOR` で、`FLOOR` は `ceiling.rs` に在って葉に同じ配列を指す（同じ一覧を 2 回書かない・P-6.3 / P-6.4）。

**どちらが先に着地しても、後の便が合わせる。**

- 本便が先なら、便 98 は `ceiling.rs` の側の `BUNDLE_SKELETON` を use するだけで、`bundle.rs` に一覧を書かない。
- 便 98 が先なら、本便が `bundle.rs` の一覧を `ceiling.rs` へ移して `FLOOR` の葉にする（**値は 1 字も変えない**・移すだけ）。このとき本便の増分は tests 側が変わらず src 側が数行増える。

**重なるのは `crates/folio/src/bundle.rs` の 1 file だけで、本便がそこへ書くのは `FACE_NAMES` の 1 行と単体の歯 1 本である。** 便 98 は `FACE_NAMES` にも `DOCUMENT_IDS` にも触らないと自分の設計ノートに書いている。

### (g) 歯（4 本・置き場は 3 か所）

**1. f102_the_ceiling_reads_every_file_with_a_generated_region**（`crates/folio/tests/ceiling.rs`）— design-intent の写しに `folio schema --check` を撃ち、**出力の各行が名指す file（生成区間を持つ file の全部）が、天井の正本の読む文書の行に 1 本残らず覆われている**こと（覆う = 行の file がその名と同じか、その file を含む dir の形）。覆われない file が在ると、そこを書き換えても周の引き金が立たない穴が残る。**赤い歯**（起草役が graph の行だけを外して実測し、落ちることと断りの字に graph.yaml が出ることを確かめた）。**数は 1 つも固定しない**（命令が見る file が増えても、行が覆っていれば緑）。

**2. f102_every_ceiling_copy_carries_the_frozen_region_and_the_same_documents**（`crates/folio/tests/ceiling.rs`）— `design-intent/` と `tests/fixtures/` の下の `ceiling.yaml` を名で全部見つけ、**どれも生成区間が凍結 anchor と byte 一致**し、**人が書く documents の節の id の集合が生成区間の閉じた一覧と同じ**であること。写しの本数は固定しない（見つけた全部に当て、実の正本が見つかることだけを確かめる）。**緑の歯**（base でも緑）。置く理由は、本便が 21 本を同時に動かす便だからである＝1 本でも取りこぼすと、その fixture で床が違反を 1 件多く数え、その組の歯が落ちる。この歯は取りこぼしをその場で名指す。

**3. f102_the_face_name_table_covers_every_document_id**（`crates/folio/src/bundle.rs` の単体の歯）— `DOCUMENT_IDS` と `FACE_NAMES` の id が**同じ集合**で、どちらにも重複が無いこと。**緑の歯**（base でも緑）。置く理由は、便 95 §1 が名指した「連れて動く」固定長 2 本の対応を、人の目でなく歯に持たせるためである。今は片方だけ増やしても、その文書を読む観点が現れるまで誰も落ちない（`copy_faces` は reads の doc にしか当たらない）。

**4. f102_every_skeleton_word_is_a_top_level_section_of_a_real_source**（`crates/folio/src/ceiling.rs` の単体の歯）— 骨格の 6 語が、**実の正本の最上位の節（列 0 の `<名>:` の行）として 1 語につき 1 本以上の file に実在する**こと。**赤い歯**（本便の前に `BUNDLE_SKELETON` が無い）。置く理由は、常に残す節の一覧が正本の形から外れていないことを、束を絞る口が入る前から数えるためである（起草役の実測: meta と schema は 10 file・id と title と status と date は 14 file に在る）。**数は固定しない**（1 本以上）。

4 本とも期待値を起草役が独立に出した（1 と 4 は実測、2 と 3 は不変条件）。回帰は verify の 2 行目（`crates/folio/tests/ceiling.rs` と `crates/folio/tests/schema_docs.rs` の全部）・3 行目（節点の要約値の anchor）と `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）が見る。**起草役は写しの repo で本便の変更と歯 4 本を全部書き、`cargo nextest run --workspace` が 764 本すべて緑・`cargo clippy --workspace --all-targets -- -D warnings` が 0 警告になるまで確かめた。**

### (h) 大きさ

.rs は**幅 120 で正規化した増分**（器の受付が当てる式）で、括弧の中は版管理が数える生の行。data の file は生の行。

| 何 | 行 |
| --- | --- |
| crates/folio/tests/ceiling.rs（歯 2 本と読み口 3 つ・印の字の定数 2 つ・余地 971） | 136 |
| crates/folio/src/ceiling.rs（DOCUMENT_IDS・BUNDLE_SKELETON・FLOOR の葉と注・単体の歯 1 本・余地 968） | 35（生 39） |
| crates/folio/src/bundle.rs（FACE_NAMES の 1 行・単体の歯 1 本・余地 961） | 17（生 19） |
| crates/folio/tests/schema_docs.rs（凍結の値 3 つ・変異の当て先・頭の注釈・余地 363） | 3（生 8） |
| crates/folio/tests/graph.rs（凍結の要約値 1 つ・余地 892） | 0（生 1） |
| design-intent/ceiling.yaml（読む文書の行 1 本・生成区間・meta の版と承認欄 2 行・頭の注釈） | 12 |
| tests/fixtures/ の ceiling.yaml 20 本（1 本あたり 7 行） | 140 |
| tests/fixtures/schema/ の anchor 2 本 | 8 |
| 合計 | **351**（版管理が数える生の行では足す 363・消える 84） |

**1 file あたりの増分のいちばん大きいのは 136 行**で、M の見積 300 の内側である。写しの 20 本は 1 本 7 行の同じ機械的な変更で、これを割ると床が写しの取りこぼしで落ちる（生成区間を持つ写しは同時にしか動かせない）。新しい file は 1 本も無い。新しい dir は作らない。縮む file は無い。外部 crate は増やさない。

### (i) 運ばないもの・撤退条件・🔴

- 観点の reads に graph を足すこと（🔴 1）・要件書の字・入口の棚・判断の記録・設計ノート・語彙・憲法・規則の表。
- 束を絞ること（便 98）・周の引き金の判定の口（便 99 の 🔴 1）・注意の先・門の単位。
- `crates/folio/src/` の ceiling.rs と bundle.rs 以外（schema.rs・check.rs・graph.rs・stamp.rs・gate.rs・findings.rs・面の生成器）。**床の判定も天井の印も変わらず、門の 3 値も本便の前後で変わらない**（印が既に古いので、前も後も まだ分からない である）。
- 撤退条件: 天井の周で索引の欄の決まりが読む文書に在ることが害になったとき（例えば所見が graph を場所に指してそこが正本でないと分かったとき）は、**行 1 本と定数 2 本と生成区間を戻す便 1 本**で元に戻せる（束も印も門も動いていないので、戻しても他の歯は動かない）。骨格の 6 語の写しは、便 98 が撤退して束を絞る口が消えたときに、**別の便で生成区間から外す**（外さないと、常に残す節の一覧が何も縛らないまま残る）。

**🔴 1（本便が閉じきらない穴・次の一括へ）**: 起草役が `crates/folio/src/gate.rs` と `crates/folio/src/stamp.rs` を読んで測ったところ、**印の正本の要約値（sources）も、門が測り直す要約値も、集めるのは「観点の reads が指す文書」だけで、読む文書の一覧（documents）ではない。** したがって本便の後も、`design-intent/graph.yaml` を書き換えただけでは印が古くならない＝問 4 の 2 が言う穴は**半分しか閉じない**。本便で閉じるのは ① 所見の場所（place.doc）が graph を指せること ② 観点の reads が graph を挙げられるようになること（`bundle::load` は一覧に無い doc を断る）③ 生成区間の閉じた一覧が実在の正本を覆うこと の 3 つである。**穴を閉じきるには、どれかの観点の reads に graph の欄を足す必要があり、それは天井の正本の改訂（どの観点がどの欄を読むか）なので持ち主の裁定が要る。** 推奨は整合の観点（`{doc: graph, fields: [node_kinds, edge_types, edge_fields]}`）で、理由は索引の欄の決まりが判断の記録 ADR-13 の決定と突き合わせる先だからである。足すと束が 1 file 増えるので、束の凍結 anchor と所見 fixture の要約値が動く＝**便 98 の後に回す**のが安い。

**🔴 2（graph.yaml は起草のまま）**: `design-intent/graph.yaml` は `status: draft`・第 0.1 版で、発効していない。天井が読む文書の一覧に起草の文書が入ることを床は止めないが、**実態の観点が「発効していない文書を天井が読むと宣言している」と言う見込みがある**。発効（持ち主の承認欄）を次の一括に載せるか、起草のままでよいかを決める。

**🔴 3（幅の崖）**: (c) のとおり `documents` の行はちょうど 100 字で上限と同じである。**次に読む文書を 1 本足す便は、この行が 1 行から 11 行に落ちることを織り込むこと**（凍結 anchor の行数・byte 数・要約値が大きく動く）。

## 2. 範囲

- 入れる: `crates/folio/src/ceiling.rs` の `DOCUMENT_IDS` と新しい `BUNDLE_SKELETON` と `FLOOR` の `bundle.skeleton` と `bundle_note` の 1 文と単体の歯 1 本・`crates/folio/src/bundle.rs` の `FACE_NAMES` の 1 行と単体の歯 1 本・`design-intent/ceiling.yaml` の読む文書の行 1 本と生成区間と meta の版と承認欄・生成区間の写し 20 本と読む文書の行・凍結 anchor 2 本（生成区間・節点の要約値の残差）・`crates/folio/tests/ceiling.rs` の f102_ 2 本・`crates/folio/tests/schema_docs.rs` の凍結の値 3 つと変異の当て先・`crates/folio/tests/graph.rs` の凍結の要約値 1 つ。
- 入れない: 観点の reads・観点の数・重さ・問いの文・束の中身の閉じた一覧・束を絞ること・入口の棚・要件書・判断の記録・語彙・憲法・規則の表・新しい命令と旗・新しい file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| docids | 読む文書の閉じた一覧 | `crates/folio/src/ceiling.rs` の `DOCUMENT_IDS`（9 → 10・末尾に graph） |
| faces | 面の名の形の表 | `crates/folio/src/bundle.rs` の `FACE_NAMES`（9 → 10・graph は面が無い） |
| skeleton | 骨格の閉じた一覧 | `crates/folio/src/ceiling.rs` の `BUNDLE_SKELETON`（6 語・`FLOOR` の葉） |
| region | 生成区間 | `design-intent/ceiling.yaml` の schema の節と写し 20 本（27 行・3,176 byte） |
| anchor | 凍結 anchor | `tests/fixtures/schema/ceiling-region.txt` と `tests/fixtures/schema/node-digest-anchor.txt` |
| teeth | 歯 | `crates/folio/tests/ceiling.rs` の f102_ 2 本と、src の単体の歯 2 本 |

## 4. 検査（歯）

§1 (g) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい file も無い。
- 先に要るものは無い。**本便は main 278ba59 でそのまま運べる**（一括 12 が着地済み・天井の正本 第 0.11 版が発効済み）。
- 便 98（行 cy・PR #255）とは `crates/folio/src/bundle.rs` が重なるだけで、**どちらが先でもよい**（§1 (f)）。器が順に運ぶ。
- `python3` は host に在る。節点の要約値の anchor を組み直す script はそれを使う（便 99 と同じ・CI では歯が python3 の不在で落ちない形になっている）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cz"
title = "天井が読む文書の閉じた一覧に索引の欄の決まりの正本 graph.yaml を足して 9 本から 10 本にし、材料の束の骨格の 6 語（meta・id・title・status・date・schema）を実装の型付きの定数から天井の正本の生成区間へ導出する。床の定数は crates/folio/src/ceiling.rs の DOCUMENT_IDS を 10 に・crates/folio/src/bundle.rs の FACE_NAMES を 10 に（graph は面が無い＝None）・骨格の一覧 BUNDLE_SKELETON を ceiling.rs へ新設して床の木 FLOOR の bundle の表の skeleton の葉にする。生成区間は folio schema --write が 27 行 3176 byte へ書き替え（documents の行に graph・bundle の表が幅 100 字を超えて block の形に落ちる）、凍結 anchor tests/fixtures/schema/ceiling-region.txt と写し 20 本を同じ byte にそろえ、人が書く読む文書の節にも graph の行を足す。凍結の土台の ceiling.yaml が 327 byte 増えるので節点の要約値の anchor tests/fixtures/schema/node-digest-anchor.txt は残差の 2 行だけを独立の script で組み直す。観点・重さ・観点の読む欄（reads）・問いの文・束の中身の閉じた一覧・入口の棚は 1 字も触らず、束の凍結 anchor と所見 fixture の要約値も動かない（凍結の土台のどの観点も ceiling を読まないため）"
req = ["FR19", "FR18"]
section = "1"
write-set = ["crates/folio/src/ceiling.rs", "crates/folio/src/bundle.rs", "design-intent/ceiling.yaml", "tests/fixtures/schema/ceiling-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "crates/folio/tests/ceiling.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/graph.rs", "tests/fixtures/adr/effective-no-approval/ceiling.yaml", "tests/fixtures/adr/schema-drift/ceiling.yaml", "tests/fixtures/adr/two-adopted/ceiling.yaml", "tests/fixtures/anchor/no-anchor/ceiling.yaml", "tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "tests/fixtures/ceiling/bundle/source/ceiling.yaml", "tests/fixtures/check/dup-key/ceiling.yaml", "tests/fixtures/check/empty-field/ceiling.yaml", "tests/fixtures/check/missing-file/ceiling.yaml", "tests/fixtures/check/unknown-section/ceiling.yaml", "tests/fixtures/face/ceiling.yaml", "tests/fixtures/floor_base/design-intent/ceiling.yaml", "tests/fixtures/link/adr-id-missing/ceiling.yaml", "tests/fixtures/link/amended-by-orphan/ceiling.yaml", "tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "tests/fixtures/refs/bad-counts/ceiling.yaml", "tests/fixtures/refs/dangling-id/ceiling.yaml", "tests/fixtures/refs/orphan-rule/ceiling.yaml", "tests/fixtures/vocab/exemptions/ceiling.yaml", "tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio f102_", "cargo nextest run -p folio --test ceiling --test schema_docs", "cargo nextest run -p folio --test graph f99_the_independent_script_matches_the_anchor", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "f102_ の歯 4 本（folio schema --check が名指す生成区間を持つ file が 1 本残らず天井の読む文書の行に覆われる／design-intent と tests/fixtures の下の ceiling.yaml を全部歩いてどれも生成区間が凍結 anchor と byte 一致し人が書く documents の節の id の集合が生成区間の閉じた一覧と同じ／DOCUMENT_IDS と FACE_NAMES の id が同じ集合で重複が無い／骨格の 6 語が実の正本の最上位の節として 1 語につき 1 本以上の file に実在する）が全部緑、crates/folio/tests/ceiling.rs と crates/folio/tests/schema_docs.rs の既存の歯が全部緑（天井の正本の生成区間が 27 行 3176 byte で凍結 anchor と byte 一致し、凍結の要約値が sha256sum の測り直しと一致する）、folio schema --check が 9 file とも一致、crates/folio/tests/graph.rs の f99_the_independent_script_matches_the_anchor が独立の script で組み直した anchor（残差の 2 行だけが変わる）で緑、workspace 全体の nextest が全部緑、clippy が 0 警告で CI が通る"

<!-- contracts:end -->
