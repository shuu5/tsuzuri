# 設計: 便 104 — 天井の印の欄 sources を、束の写しでなく `--dir` の正本から門と同じ関数で測る（FR20 / FR17）

- 要件: FR20（天井の門 — 印に「正本の要約値」を書き、`--gate` はその要約値と今の正本の要約値を突き合わせる・第 1.29 版）/ FR17（束は観点が読むと宣言した節まで絞った正本の写しを持つ・第 1.28 版）
- 条: P-3.1（機械で決定的に検査できる項目は床に置く）/ P-4.1（検査・生成が実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは まだ分からない として表に出す）/ P-6.2（生成物を手で直さない）/ P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-6.4（2 つの面を人が書き、一致を検査で強制する設計を採らない）/ P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-10.2（生成物どうしの突き合わせを唯一の合格判定にしない）/ P-15.2（止める仕掛けを持つときは、その判定の式を事後の検査が同じ関数で確かめる）/ N-3.1（規則の例外機構を足さない）
- 出所: 天井の 26 周目（束 `~/.local/share/folio2/ceiling/2026-09-22-round26/`・4 観点とも 合格・止める 0）で印が書けない。席が実測した 1 行は `folio ceiling: まだ分からない（sources/adr/ADR-1.yaml: 観点で中身が違う（readability））`（終了 2・main 44f22ab）。原因は便 98（`docs/design/delivery-98.md`・main f547ce7）が束の `sources/` の写しを「観点の reads が宣言した最上位の節まで絞る」形にしたこと。同じ文書の写しが観点ごとに違う byte になり（実の 26 周目の `adr/ADR-1.yaml` は coherence 10,046 / fidelity 7,286 / readability 6,288 / reality 6,013 byte）、印の側がまだ「写しは 4 観点とも全文で同一」を前提に和集合を取っている。
- **便 98 の契約 §1 は「床の判定も天井の印も門も 1 つも変わらない」と書いたが、天井の印は変わった。** 凍結の土台（`tests/fixtures/ceiling/bundle/`）では拾えなかった（理由は §1 (a) の 2 つ目）。事実の記録であって、便 98 の判断の当否を問うものではない。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 db が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新しい file は 1 本・`+` を付けて宣言する。縮む file も消す file も無い・新しい dir は作らない）。
- 門: 本便は `design-intent/` の下の正本を 1 file も書き換えない（読むだけ）ので、天井の門の対象外である。**起草役が本便の write-set 5 本をそのまま渡して実測し、`folio ceiling --gate` が 0（通す・断りの字は 設計文書の正本を書き換えない便）を返すことを確かめた**（2026-09-22・base main 44f22ab・印は 23 周目のままで古いが、門はその手前で 通す を返す）。**したがって本便は規則の表の開発規律行 D-12 の裁定（設計文書を触る便は門の外で受ける）を要しない。** 受付の手順はふだんどおり preflight → dispatch。
- 前の便: 便 72（印・`crates/folio/src/stamp.rs`）・便 73（門・`crates/folio/src/gate.rs`）・便 98（束を絞る・main f547ce7）・便 99（印の `rest` と `nodes`）。どれも着地済み。base は **main 44f22ab**。

- 改訂 b（2026-09-23 00:2x JST・検証役の report `~/.local/share/folio2/handoff-2026-09-22/d104-verify.md` §7 への応答）: §1 (e) 2 の「まだ分からない」に逃げてよい条件を外の命令の起動失敗だけに狭め、それ以外の測れない理由では歯を落とすと明記。既に在る歯の測り直しを残すことを明記。数値・write-set・歯の本数・done・verify は変えない。
## 1. 設計

### (a) いま起きていること（実測）

印 `folio ceiling --stamp` は `<dir>/preview/ceiling-stamp.yaml` の欄 `sources` を、**束の観点ごとの `sources/` の写しの和集合**から測っている（`crates/folio/src/stamp.rs` の `union_digest`・228〜250 行付近）。和集合は「同じ相対 path が観点で違う byte なら Err」で、Err は 全部か無しか の規則により「まだ分からない（終了 2）」になり、印を 1 byte も書かない。

便 98 の前は、束の `sources/<file>` は正本 `<dir>/<file>` の**全文**の写しだったので、この和集合は「観点のどれかが読む文書の全 file を `<dir>` からの相対 path の byte 順に連結した byte 列」と**同じもの**だった。便 98 が写しを観点ごとに絞ったので、同じ前提が成り立たなくなった。

**凍結の土台で拾えなかった理由は 2 つある（どちらも起草役が実測した）。**

1. 印の歯の周（`crates/folio/tests/stamp.rs` の `Round::passing`）は、4 観点の `reads` を **同じ 5 行（`SAME_READS`）に揃えてから** 束を組む。所見の凍結 fixture `pass-coherence.yaml` の起動の記録 `record.read` が 5 文書を挙げており、床（`--check`）が `record.read` と `reads.yaml` の doc の集合一致を数えるためである。4 観点が同じ欄を読めば写しも同じ byte になるので、和集合は Err にならない。
2. その周では、**絞った写しが正本の全文と byte で一致する**。凍結の土台の file が小さく、`SAME_READS` が挙げた欄と骨格の 6 語（`meta`・`id`・`title`・`status`・`date`・`schema`）で最上位の節を覆い切るためである。起草役は、印の `sources` を正本から測り直す歯（§1 (e) の 2）を **base の実装のまま** この周に当てて緑になることで確かめた。

つまり凍結の土台は「観点で読む欄が違う」場合を 1 つも持っていない。実の周だけが持っている。

### (b) 直す先 — 印の `sources` は正本から測る

門 `crates/folio/src/gate.rs` の `sources_digest`（165 行付近）は、**`--dir` の正本から** 観点の `reads` が指す文書の file（file 形はその file・dir 形は直下の `.yaml`）を全文で集め、`<dir>` からの相対 path の byte 順に連結した sha256 を測る。門はこの値を印の `sources` と突き合わせて「印が古い」を判定する（FR20 / AC18）。

要件書 FR20 の規範文は印の欄を「**正本の要約値**」と呼ぶ。束の写しの要約値ではない。したがって正しい直しは「印の `sources` も、門と同じ規則で `--dir` の正本から測る」であり、**門と印が同じ 1 つの関数を呼ぶ**（P-6.3 = 一方を正本とし他方を導出する・P-15.2 = 止める仕掛けの判定の式を事後の検査が同じ関数で確かめる）。2 つの面に同じ式を書いて一致を検査で強制する形は採らない（P-6.4）。

**形は 2 つあり、起草役は両方を書いて実測した（どちらも workspace の nextest 777 本が緑・clippy 0 警告）。**

| 形 | 触る src | 行の動き（幅 120 正規化） |
| --- | --- | --- |
| （あ）`gate::sources_digest` を `pub(crate)` にして `stamp.rs` から呼ぶ | 2 本（`stamp.rs`・`gate.rs`） | stamp.rs 338 → 344（+6）・gate.rs 252 → 253（+1）・計 **+7** |
| （い）`sources_digest` と `documents` と `collect` を `bundle.rs` へ移し、`gate.rs` と `stamp.rs` と `graph.rs` が呼ぶ | 4 本（`bundle.rs`・`gate.rs`・`stamp.rs`・`graph.rs`） | bundle.rs 730 → 800（+70）・gate.rs 252 → 181（−71）・stamp.rs 338 → 343（+5）・graph.rs 738（呼ぶ先 2 か所と use 1 行の入れ替え） |

**（あ）を採る。** 理由は 3 つ。

1. **置き場は既に決まっている。** `gate.rs` の `documents` と `collect` は既に `pub(crate)` で、`crates/folio/src/graph.rs` の `stamp_table`（666〜670 行）が印の `rest` と `nodes` を組むのに呼んでいる。つまり「`--dir` の正本から観点の読む文書の file を集める」口の置き場は `gate.rs` だと既に決まっており、印の 3 欄のうち `rest` と `nodes` は**もう** `gate.rs` の口を通って正本から測っている。`sources` だけが束の写しから測っていた。（あ）はその 1 欄を残り 2 欄と同じ経路に揃える。
2. **直す便の大きさが小さい。** （い）は 70 行の引っ越しで、本便が直す欠陥とは関わらない `graph.rs` を write-set に入れる。欠陥を直す便に引っ越しを載せない。
3. **（い）の利点（module の依存の向きを一方向にする）は本便で払う必要が無い。** `gate.rs` は既に `stamp::STAMP_FILE` を読み、`stamp.rs` が `gate::sources_digest` を呼ぶと 2 方向の辺になるが、Rust の 1 つの crate の中の module は相互に参照でき、build も clippy も通る（起草役が実測）。置き場を動かしたくなったら（い）は本便の後でも 1 便で運べる（本便が足すのは `pub(crate)` の 1 語と呼ぶ 1 行だけ）。

**書き換えは 3 か所（起草役の実測の patch は `~/.local/share/folio2/handoff-2026-09-22/d104-measured.patch`）。**

- `gate.rs`: `fn sources_digest` → `pub(crate) fn sources_digest`（1 語）。file の頭の注 2 行と関数の注 1 行を、印もこの関数を呼ぶ形に書き替える。
- `stamp.rs`: `use crate::gate;` を足し、`derive` の中で欄 sources を組む 1 行を、`union_digest` に旗 sources を渡す形から `gate::sources_digest(dir, &ceiling)?` に替える。
- `stamp.rs`: file の頭の注に便 104 の 3 行を足し、全部か無しか の 1 行から「正本と」を落とす（正本の写しが観点で食い違うことは、もう まだ分からない の理由ではない）。

### (c) 面の側（欄 `faces`）は和集合のまま

束の `faces/` の写しは便 98 が絞っていない（`copy_faces` は `--faces` の直下の file を全文で写す）。同じ面の file は 4 観点とも同じ byte なので、和集合は今も成り立つ。**起草役は実の 26 周目の束で実測し、`faces` の和集合が Err にならず `sha256 b6c821aa2265e59589e25ec5dbe3074663b286c557eb461a702159c91b36aac8` を返すことを確かめた。** 面は生成物であって正本ではないので、門は `faces` を突き合わせない（FR20 の門の判定は `sources` だけを見る）。したがって `faces` を `--dir` の側から測る先は無く、和集合のままにする。

呼ぶ先が `faces` 1 つになるので、`union_digest(out_dir, &ceiling, sub)` は `faces_digest(out_dir, &ceiling)` に名を替え、旗 `sub` を落とす（中で旗 sub を faces に固定し、Err の字面「`faces/<相対 path>: 観点で中身が違う（<観点>）`」は変えない）。**「観点で中身が違うなら Err」の枝は残す**（P-4.1 = 実行できなかった結果を異常なしとして扱わない。面が周の途中で組み直されたら黙らずに止まる）。

### (d) 歯の土台 — 観点ごとに読む欄が違う周

新しい file 1 本を手で書く。置き場は既に在る dir の下の `tests/fixtures/ceiling/split-reads.yaml`（新しい dir は作らない）。中身は **4 観点とも同じ 5 文書（`constitution`・`rules`・`srs`・`adr`・`design-note`）を読み、欄（`fields`）だけが観点で違う** `reads` の行の塊で、行の形は天井の正本 `ceiling.yaml` の `viewpoints[].reads` と同じ。**28 行・1,376 byte。**

文書を 4 観点で同じに保つのは、所見の凍結 fixture `pass-coherence.yaml` の起動の記録 `record.read`（5 文書）をそのまま使うためである（床が `record.read` と `reads.yaml` の doc の集合一致を数える）。これで**印の `round`・`verdict`・`viewpoints` の行・`refutes`・`reads`・`rest`・`nodes` はどれも今の周と同じ値になり、動くのは `sources` と `faces` と束の要約値だけ**になる。§1 (f) の凍結 anchor がそのまま使える所以である。

歯の側は、この土台を読んで一時 dir の写しの `ceiling.yaml` の各観点の `reads` に差し込む（今の `same_reads` を、観点の id から行の塊を引く `put_reads` に一般化し、`Round::passing` は `SAME_READS` を返す閉包を渡す形にする＝既に在る周の振る舞いは 1 字も変えない）。差し込めたことは、4 観点それぞれの塊が本文に在ることと `      - {doc: ` の行の総数が塊の行数の和と等しいことで確かめる。土台の 4 つの塊が互いに違うことも歯が数える（**同じなら土台が役目を果たしていない**）。

### (e) 歯（関数名は `f104_` で始める・置き場は `crates/folio/tests/stamp.rs`）

置き場は印の歯の file で、余地は十分に在る（§1 (g)）。新しい歯の file も新しい dir も作らない。

1. **`f104_the_stamp_survives_viewpoints_reading_different_fields`** — (d) の周で、同じ文書の写しが観点で違う byte であること（`sources/srs.yaml` を fidelity と readability で読み比べる＝便 98 の後の実態がこの周に在ることの確かめ）、`--stamp` が 0 で「印を書いた」を出すこと、そして書けた印が要約値を落とした形で凍結 anchor `stamp-expected.yaml` と byte 一致すること。**base では `--stamp` が 2 を返して印を書かないので 赤い歯。**
2. **`f104_the_stamp_sources_is_the_canonical_digest`** — (d) の周の印の `sources` が、**folio を呼ばずに測った正本の要約値と一致する**こと。測り方は歯の側で天井の正本 `ceiling.yaml` の `documents` と `viewpoints[].reads` を読み、文書の file（dir 形は直下の `.yaml`）を `<dir>` からの相対 path の byte 順に連結して**外の命令 `sha256sum`** に流す（P-10.2 = 生成物どうしの突き合わせを唯一の合格判定にしない。`sha256sum` を起動できない環境〔外の命令の起動そのものが Err〕**だけ**は「まだ分からない」の 1 行を標準エラーへ出して歯を落とさない＝`tests/schema.rs` と同じ形。**それ以外の理由で測れないとき（天井の正本や文書の一覧が読めない・文書の file が集められない・要約値が空）は歯を落とす**＝測れなかった結果を一致として扱わない〔P-4.1〕。起草役の実測 patch では、印の行から id を取り出す助けの関数が常に空で「文書の一覧に無い」の Err に落ち、照合が 1 度も走らないまま緑になった〔検証役の指摘〕。作業者は `--no-capture` で標準エラーに「まだ分からない」の行が出ないことを確かめる）。また、既に在る歯 `stamp_sources_digest_is_recomputable` は印の `sources` を**実際に測り直す**形を残す（測り直す先を正本からの独立の測りに替えるのはよいが、測り直しそのものを無くさない＝P-10.2）。あわせて、**読む欄を揃えた周（`Round::passing`）の `sources` と (d) の周の `sources` が同じ値であること**を数える（＝印の `sources` は正本だけの関数で、どの観点がどの欄を読むかに依らない）。**base では (d) の周で印が書けないので 赤い歯。**
3. **`f104_the_gate_passes_the_stamp_it_just_wrote`** — (d) の周で `--stamp` を撃った直後に、同じ `--dir` に `folio ceiling --gate --write-set <dir>/srs.yaml` を撃つと 0（通す）で、断りの字が「正本の要約値が同じ」であること。**base では印が書けないので 赤い歯。**

**3 件とも起草役が実測した**＝本便の全部を当てた木で 3 件とも緑、実装（`src/`）だけを main の字に戻した木で 3 件とも赤（2026-09-22・赤のときの 1 行は `まだ分からない（sources/adr/ADR-2.yaml: 観点で中身が違う（readability））`）。

**既に在る歯 `stamp_sources_digest_is_recomputable` も直す。** この歯は印の `sources` を束の `sources/` の和集合と突き合わせていたので、測り直す先を (e) の 2 と同じ「正本から測る」に替える（`faces` の側は和集合のまま）。**この歯は base でも着地後でも緑**（理由は §1 (a) の 2）。

回帰は verify の 2 行目（`crates/folio/tests/stamp.rs` と `crates/folio/tests/gate.rs` の全部）と `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）が見る。**起草役は当てた木で workspace の nextest 777 本が全部緑であることを実測した**（本便の前は 774 本）。

### (f) 凍結 anchor `tests/fixtures/ceiling/findings/stamp-expected.yaml` は 1 byte も動かない

この anchor は印の**要約値を落とした形**（`without_digests` が `at:`・`sources:`・`faces:`・`rest:` の行と各行の `, bundle: ` と `, at: ` を落とす）と突き合わせる。本便が変えるのは `sources` の値 1 つだけなので、anchor の側は動かない。**34 行・1,342 byte のまま**で、`f99_the_stamp_carries_the_node_table` が数えているその 2 つの値も動かない（起草役が実測）。したがって anchor の組み直しは要らず、組み直し方の手順も要らない。

**(d) の周にも同じ anchor を当てる**（§1 (e) の 1）。読む欄が違っても印の形と要約値以外の中身は 1 字も変わらないことが、これで凍結の側から言える（P-10.1）。

anchor を組み直す必要が出る場合は 1 つだけある: 印の**欄の並びか値の字面**が変わるときで、本便はどちらも変えない（欄の並びは `round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes` のまま・`f99_the_stamp_carries_the_node_table` が数える）。

### (g) 大きさ

| 何 | 幅 120 正規化の行 |
| --- | --- |
| `crates/folio/src/stamp.rs`（呼ぶ先の 1 行・`use` 1 行・注 5 行・関数の名と旗・**base 338 → 344**・**余地 1,156**） | +6 |
| `crates/folio/src/gate.rs`（`pub(crate)` の 1 語・注 3 行・**base 252 → 253**・**余地 1,247**） | +1 |
| `crates/folio/tests/stamp.rs`（`put_reads` と `split_reads` と `Round::split` と `Round::build` と `folio_gate` と `canonical_hex`・`f104_` の歯 3 本・既に在る歯 1 本の測り直す先・**base 515 → 674**） | +159 |
| `tests/fixtures/ceiling/split-reads.yaml`（**新しい file**・手書き・28 行 1,376 byte） | +28 |
| 合計 | **+194** |

size **S**（触る src は `stamp.rs` と `gate.rs` の 2 本で、余地はそれぞれ 1,156 と 1,247。**S の見積 100 も M の見積 300 も上回る**。src の増分は 7 行しか無いので S に収める）。新しい dir は作らない。縮む file も消す file も無い。外部 crate は増やさない。`design-intent/` の下は 1 file も書き換えない。

**着地後に席が打つ印は本便に入れない。** 起草役は clone の上で実の 26 周目の束に当てて `folio ceiling --dir design-intent --faces <周>/site --out <周>/2026-09-22-round26 --stamp` が 0 で `design-intent/preview/ceiling-stamp.yaml`（7,757 byte）を書くこと、その `sources` が `sha256 daf9c226b13478b0e0edc0838476f7b5a9755c6de9f190e13d8cf8d978506d18`・`verdict` が 合格 であること、直後の `folio ceiling --dir design-intent --gate --write-set design-intent/ceiling.yaml` が 0（通す・印が 4 観点とも合格・正本の要約値が同じ）を返すことを確かめ、**clone の印を元に戻した**。印は生成物であり、周を回した席が着地後に打つ（P-6.2）。

### (h) 本便が運ばないもの・撤退条件

- 便 98 の絞りの規則そのもの（`crates/folio/src/bundle.rs`）。束の写しは絞ったままでよい。FR17 の規範文がそう言う。
- `sources_digest` と `documents` と `collect` の置き場を `bundle.rs` へ移すこと（§1 (b) の形（い））。要るなら本便の後に 1 便で運べる。
- 束の要約値 `digest.txt` の規則・束の中身の閉じた一覧・凍結 anchor `tests/fixtures/ceiling/bundle-anchor.txt` と独立の script `bundle-anchor.py`・所見の凍結 fixture 11 本・`tests/fixtures/ceiling/findings/stamp-pass.yaml` と `stamp-fail.yaml` と `stamp-unknown.yaml`（門の側の印の写し・本便は門の判定を 1 字も変えない）。
- 印の欄の並び・欄の値の字面・`rest` と `nodes` の組み方（`graph::stamp_table`）・名札（`face_labels.rs`）が印から読む 4 欄・門の 3 値の規則。
- 要件書・憲法・規則の表・語彙・判断の記録・入口の棚・`design-intent/` の下の全 file（`preview/ceiling-stamp.yaml` を含む）・台帳への記帳。
- `crates/folio/src/` の `stamp.rs` と `gate.rs` 以外（`bundle.rs`・`findings.rs`・`graph.rs`・`ceiling.rs`・`face_*.rs` ほか）。
- 撤退条件: 印の `sources` を正本から測る形が、**門の「印が古い」を出せなくしたとき**＝周を回して印を打った直後なのに `--gate` が「印が古い」を返す事故が 2 度起きたとき（数えるのは §1 (e) の 3 の歯の落ちた回数と実の受付の断り・記帳は台帳 f2-648）は、印に `sources` と並べて束の側の要約値を別の欄で持つ形（門は `sources` だけを見る）へ 1 便で移す。**束の写しの和集合へ戻すことはしない**（FR20 の規範文が印の欄を「正本の要約値」と言うため）。

## 2. 範囲

- 入れる: `gate::sources_digest` の見え方 1 語と注 3 行・`stamp.rs` の呼ぶ先 1 行と `use` 1 行と注 5 行・`union_digest` を `faces_digest` に替えること・手書きの土台 `tests/fixtures/ceiling/split-reads.yaml` 1 本・`crates/folio/tests/stamp.rs` の周の組み立ての一般化（`put_reads`・`split_reads`・`Round::split`・`Round::build`）と `folio_gate` と `canonical_hex` と `f104_` の歯 3 本と既に在る歯 1 本の測り直す先。
- 入れない: 束の絞りの規則・束の要約値・凍結 anchor の中身（`stamp-expected.yaml`・`bundle-anchor.txt`・所見 fixture）・印の欄の並びと字面・門の 3 値の規則・`rest` と `nodes` の組み方・置き場の引っ越し・`design-intent/` の下の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| one | 1 つの関数 | `crates/folio/src/gate.rs` の `sources_digest`（`pub(crate)`・`--dir` の正本から観点の読む文書の file を集めて sha256） |
| call | 呼ぶ先 | `crates/folio/src/stamp.rs` の `derive` が欄 `sources` にその関数を使う |
| faces | 面の要約値 | `crates/folio/src/stamp.rs` の `faces_digest`（束の `faces/` の和集合・観点で違えば Err の枝は残す） |
| split | 土台 | `tests/fixtures/ceiling/split-reads.yaml`（手書き・4 観点が同じ 5 文書の違う欄を読む `reads` の行） |
| round | 周の組み立て | `crates/folio/tests/stamp.rs` の `put_reads`・`split_reads`・`Round::split`・`Round::build` |
| meas | 独立の測り | `crates/folio/tests/stamp.rs` の `canonical_hex`（正本から集めて外の命令 `sha256sum` で測る） |
| teeth | 歯 | `crates/folio/tests/stamp.rs` の `f104_` 3 本と、測り直す先を替える既に在る歯 1 本 |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。`sha256sum` と `python3` は host に在るが、歯はどちらも起動できなくても落ちない形にする（`sha256sum` は §1 (e) の 2・`python3` は既に在る `f99_the_stamp_rest_is_recomputable`）。
- 便 98（行 cy・main f547ce7）と便 99 と便 102 は着地済みで、本便の前提である。
- **書き換える file が重なる並行の便は無い**（起草役が base main 44f22ab で確かめた）。`crates/folio/src/gate.rs` を触る便は便 73 以降 1 本も無い。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "db"
title = "天井の印（folio ceiling --stamp）の欄 sources を、材料の束の観点ごとの写しの和集合からでなく、--dir の設計文書の正本から測る形へ改める。門（--gate）が印の sources を突き合わせるのに使う関数 gate::sources_digest（観点の reads が指す文書の file・file 形はその file・dir 形は直下の .yaml を --dir からの相対 path の byte 順に連結した sha256）を pub(crate) にし、stamp.rs の derive がその同じ関数を呼ぶ（2 面に実装しない・P-6.3・P-15.2・要件書 FR20 の規範文が印の欄を 正本の要約値 と言う）。便 98 が束の sources/ の写しを観点の reads が宣言した最上位の節まで絞ってから、同じ文書の写しが観点ごとに違う byte になり、和集合を取る口が Err を返して印が書けなくなっていた。面の側の欄 faces は束の faces/ の和集合のまま残し（面は絞られていない・門は faces を突き合わせない）、呼ぶ先が 1 つになった口を faces_digest に名を替えて旗 sub を落とす。観点で中身が違えば Err とする枝は faces に残す。歯の土台として、4 観点が同じ 5 文書の違う欄を読む reads の行を手で書いた file tests/fixtures/ceiling/split-reads.yaml を既に在る dir の下に 1 本足し、印の歯の周の組み立てを観点ごとに reads を差し替えられる形へ一般化する（既に在る周の振る舞いは変えない）。凍結 anchor tests/fixtures/ceiling/findings/stamp-expected.yaml は 34 行 1,342 byte のまま 1 byte も動かさない（要約値の行を落とした形と突き合わせる anchor なので、動くのは sources の値だけ）。印の欄の並びと字面・門の 3 値の規則・束の絞りの規則・束の要約値・rest と nodes の組み方は 1 つも変えず、design-intent の下は 1 file も書き換えない"
req = ["FR20", "FR17"]
section = "1"
write-set = ["crates/folio/src/stamp.rs", "crates/folio/src/gate.rs", "crates/folio/tests/stamp.rs", "crates/folio/tests/gate.rs", "+tests/fixtures/ceiling/split-reads.yaml"]
verify = ["cargo nextest run -p folio --test stamp f104_", "cargo nextest run -p folio --test stamp --test gate", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f104_ の歯 3 本（観点ごとに読む欄が違う周で同じ文書の写しが観点で違う byte であり、--stamp が 0 で 印を書いた を出し、書けた印が要約値を落とした形で凍結 anchor stamp-expected.yaml と byte 一致する／その印の sources が folio を呼ばずに測った正本の要約値（天井の正本の documents と reads から集めた file を --dir からの相対 path の byte 順に連結し外の命令 sha256sum で測る・sha256sum を起動できない環境は まだ分からない の 1 行で落ちない）と一致し、読む欄を揃えた周の sources と同じ値である／その印を書いた直後の folio ceiling --gate --write-set <dir>/srs.yaml が 0 を返し断りの字が 正本の要約値が同じ である）が全部緑、crates/folio/tests/stamp.rs と crates/folio/tests/gate.rs の既に在る歯が全部緑（印の欄の並びが round・at・verdict・sources・faces・viewpoints・refutes・reads・rest・nodes のままで、凍結 anchor が 34 行 1,342 byte のまま、faces の要約値が束の faces/ の和集合の測り直しと一致し、門の 6 場合の終了コードが今と同じ）、workspace の nextest が全部緑で clippy が 0 警告で CI が通る"
<!-- contracts:end -->
