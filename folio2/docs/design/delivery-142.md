# 設計: 便 142 — 天井の門を作業ツリーの一番上以外から撃ったとき、黙って通さず まだ分からない を返す（--dir と write-set の根の突き合わせ・床の穴 f2-648.214）

- 要件: FR20（天井の門・要件書 第 1.43 版）。本便は FR20 の規範文も受入基準 AC18 の 7 場合も変えない。門が write-set の path を設計文書の置き場の file と照らせないときに 0（通す）を返していた穴を、既存の「判定が実行できなかったら まだ分からない」の形（印が読めない・引き金の要約値が測れない・印の節点の表が読めない と同じ族）に収めて塞ぐ。契約表の行の req は FR20 の 1 つ（main に在る id・便 73・126・129 と同じ）。受入基準 AC18 は器の要件面に無い id なので req に書かない（req に足した版の受付の先撃ちが requirement-missing で断った・起草役の実測）。
- 条: P-4.1（検査が実行できなかった結果を異常なしとして扱わない＝照らせない write-set を 通す と言わない）/ P-4.2（判定できないものは まだ分からない として表に出す＝理由の行に撃ち直し方を書く）/ P-15.2 の向き（門の判定の式は 1 つの関数のまま＝設計文書の判定 is_design_source は変えず、その前に根の突き合わせを置く）/ P-10.1（凍結 anchor と期待の字は歯の側の手書きで持つ＝`tests/fixtures/ceiling/` は 1 byte も変えない）/ N-3.1（例外の口を足さない＝旗を足さない）。
- 出所: 台帳 **f2-648.214**（一括 21 の独立の検証 batch21-verify.md の別件の発見・2026-09-25・検証役の notes 18:37 JST = 相対 path でも当たる）。要件書 第 1.43 版の FR20 の注（天井の 35 周目 実態 F-2）が同じ穴を「未解決・直るまでは作業ツリーの一番上の置き場から撃つ」と書いている。依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eo` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 3 本（書き換える 2 本 + 本文が変わらない verify の scope 1 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が作業ツリーの一番上で write-set 3 本を base の binary で `folio ceiling --gate --dir design-intent --write-set …` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。便の全差分を当てた写しの binary でも同じ字で 0 だった。
- 前の便: 前提の着地は無い。**base = main f85d346（一括 21 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 並行の便は無い（一括 21 は main f85d346 に着地済み・契約の枝のうち main に着地していない便の write-set に本便の 3 本は無い）。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。

## 1. 設計

### (a) いま起きていること（実測・base main f85d346・数は参考値）

1. **門の口の前提。** `folio ceiling --gate`（`crates/folio/src/gate.rs` の関数 run）は、write-set の各 path を repo の根（作業ツリーの一番上）からの相対と読み、`--dir` も同じ根からの相対と読む（関数 run の上の注）。今の dir がその根であることは確かめていない。`--dir` の要素は関数 dir_parts が作る。相対 path はそのまま要素に分け、絶対 path は今の dir の下なら今の dir からの相対に直し、下でなければ絶対 path の要素を全部使う。`..` と `.` の要素は黙って落とす。設計文書の判定（関数 is_design_source）は、write-set の path の要素の先頭が `--dir` の要素の列と全部一致するかで決める。write-set に設計文書の path が 1 つも無ければ、印を読まずに 0（通す・設計文書の正本を書き換えない便）を返す。
2. **穴の再現（本物の作業ツリー・base の binary）。** 作業ツリー planner-d142（main f85d346 と同じ置き場）の要件書を write-set に渡した答え。

| 撃つ所（今の dir） | --dir | 終了コード | 標準出力 |
| --- | --- | ---: | --- |
| 本流の一番上 | .worktrees/planner-d142/design-intent（相対） | 0 | 通す（設計文書の正本を書き換えない便） |
| 本流の一番上 | 作業ツリーの置き場の絶対 path | 0 | 通す（設計文書の正本を書き換えない便） |
| 作業ツリーの一番上 | design-intent | 0 | 通す（印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る（節点 3 個と節点の外の字・次の引き金の周が読む）） |

   終了コードは 3 つとも 0 だが、本流の一番上から撃った 2 つは印を読んでいない（理由の字が 設計文書の正本を書き換えない便）。write-set の `design-intent/srs.yaml` が `--dir` の要素の列（.worktrees・planner-d142・design-intent）の先頭と一致しないので、要件書を書き換える便が 設計文書の正本を書き換えない便 と読まれている。

3. **答えが食い違う再現（模した置き場・base の binary）。** 一時 dir に本流の一番上（main/）と作業ツリーの一番上（main/.worktrees/x/）を模し、作業ツリーの置き場だけ要件書 FR1 の規範文を 1 字変えた（正しい答えは まだ分からない・印が古い）。write-set は `design-intent/srs.yaml` と `crates/folio/src/gate.rs`。

| 名 | 撃つ所（今の dir） | --dir | base の答え | 本便の後の答え |
| --- | --- | --- | --- | --- |
| A | 作業ツリーの一番上 | design-intent | 2 印が古い（引き金の要約値が違う） | 同じ |
| B | 作業ツリーの一番上 | 作業ツリーの置き場の絶対 path | 2 印が古い | 同じ |
| C | 本流の一番上 | .worktrees/x/design-intent | **0 通す（設計文書の正本を書き換えない便）** | 2 --dir と write-set の根が違う |
| D | 本流の一番上 | 作業ツリーの置き場の絶対 path | **0 通す（同上）** | 2 --dir と write-set の根が違う |
| E | 無関係の dir | 作業ツリーの置き場の絶対 path | **0 通す（同上）** | 2 --dir が今の dir の下に無い |
| F | 作業ツリーの置き場の adr/ | ../../design-intent | 2 印が古い（`..` を落として偶然に根が合う） | 2 --dir が今の dir の下に無い |
| G | 本流の一番上 | design-intent（対照・本流の置き場） | 0 通す（印が 4 観点とも合格…） | 同じ |
| H | 本流の一番上 | ./design-intent（対照） | 0 通す（同上） | 同じ |

   C・D・E が穴で、正しくは 2 の所で 0 を返す（P-4.1 / P-4.2 に反する）。main の `target/debug/folio`（3e66abd の build・f85d346 まで crates は不変）と base の写しの binary は 8 通りとも同じ字を返した。

4. **今の撃ち方は当たらない。** 席の受付の手順 admit.sh と一括の取り込みの手順は、本流の一番上で `--dir design-intent` と repo の根からの相対の write-set で撃つ（G の形）。歯も同じ形（`crates/folio/tests/gate.rs` は一時 dir を今の dir にして `--dir design-intent`・`crates/folio/tests/stamp.rs` は周の一時 dir を今の dir にして `--dir src`）。穴に当たるのは、作業ツリーの置き場を本流の一番上から名指すときと、今の dir の外の置き場を名指すときである。
5. **要件書との関係。** FR20 の規範文は、write-set に設計文書の置き場の file が 1 つも無ければ 0 と言う。C・D の write-set は作業ツリーの置き場の要件書を持つので、規範文の上でも 0 ではない。受入基準 AC18 の 7 場合（設計文書の置き場の file を触らない・合格で同じ引き金の要約値・合格で引き金の要約値は同じで正本の他の字が違う・不合格・引き金の要約値が違う・まだ分からないの観点・印が無い）は、どれも根が合っている前提の場合である。根が合わないときは 7 場合のどれにも当たらず、判定が実行できない。門は既にこの族の まだ分からない を持つ（関数 run の中の 印が読めない・天井の正本が読めない・引き金の要約値が測れない・正本の要約値が測れない・印の節点の表が読めない・印の後に変わった節点が数えられない の 6 つ）。そのうち引き金の要約値が測れないと印の節点の表が読めないは便 126 の歯 f126_the_gate_is_unknown_when_the_trigger_cannot_be_measured と f126_the_gate_does_not_pass_without_the_node_table が見ており、要件書の改訂なしで運んだ。本便の まだ分からない も同じ族に置くので、7 場合を増やさず、要件書の改訂は要らない。規範文の「1 つも無ければ 0」は、write-set と置き場を同じ根で照らせることを前提にしている。だから今の dir の外の `--dir` では、実装の file だけの write-set でも照らせず 2 になる（(c) の 3 の 4 通り目・(b) の 5 の末尾と対）。次の一括で FR20 の注を直すときも、この前提を 1 句で書く。
6. **FR20 の注。** 要件書 第 1.43 版の FR20 の注は、この穴を「既知の穴・未解決」と書き、「直るまでは、門を作業ツリーの一番上の置き場から撃つ」と書いている。本便の着地の後は、この注の字が実態と食い違う（穴は閉じ、一番上以外から撃つと 0 でなく 2 が返る）。注は設計文書の正本で、便では触れない（門の対象になる）。**着地の後の次の一括で注を直す**（字の向き: 穴は便 142 で閉じた・一番上以外から撃つと まだ分からない と撃ち直し方を返す）。席が台帳 f2-648.214 を閉じるときに、この 1 行を次の一括の材料として控える（§5）。
7. **base の歯（参考値）。** workspace の nextest 906 / 906・clippy 0 警告・床 4 本（check・inject --check・schema --check・derive --check）rc 0・`folio build` の出力 30 file。歯の file の本数は tests/gate.rs 14・tests/stamp.rs 13・gate.rs の単体 1（gate_classifies_design_sources）。`git grep -n f142_ -- crates` は 0 件。行 id `eo` は repo の docs/design と枝に 0 件。

### (b) 直す先 — 根の突き合わせを門の判定の最初に置く（案 A）

1. **関数 dir_parts を、照らせないときに無いと言う形にする（`crates/folio/src/gate.rs`）。** 戻り値を要素の列の Option にする。
   1. 絶対 path は、今の dir の下なら今の dir からの相対に直す。下でなければ無い（None）。今の dir が取れないときも無い。
   2. 要素を字面で解く。`.` は落とす。`..` は 1 つ前の要素を外し、外す要素が無ければ（今の dir の外へ出る）無い。要素が UTF-8 でなければ無い。
   3. symlink は解かない（字面だけ）。今の dir は OS の返す字（symlink を解いた字）なので、symlink を通した絶対 path は今の dir の下と読めず、無いに倒れる（通さない側）。
2. **関数 other_root を足す。** write-set の path（接頭辞 + / - / ~ は剥がす）が作業ツリーの一番上からの相対で読めないときに真を返す。次のどれかで真。
   1. path が `/` で始まる（絶対 path）。
   2. path の要素に `..` が在る。
   3. path の要素の先頭が `--dir` の要素の列の途中から後ろの部分（2 番目の要素から後ろ・3 番目から後ろ…）と一致し、`--dir` の要素の列の全部とは一致しない。例えば `--dir` の要素が .worktrees・x・design-intent のとき、`design-intent/srs.yaml` と `x/design-intent/srs.yaml` は真、`.worktrees/x/design-intent/srs.yaml` と `crates/folio/src/gate.rs` は偽。`--dir` が 1 要素（design-intent）なら 3 はいつも偽で、本流の今の撃ち方（(a) の 4）には効かない。
3. **関数 run の最初に 2 つの確かめを置く。** 設計文書の判定（通す・設計文書の正本を書き換えない便）より前に置く。
   1. dir_parts が無いと言えば、2（まだ分からない）と理由の字 UNKNOWN_DIR_OUTSIDE（--dir が今の dir の下に無い・write-set の根と照らせない・作業ツリーの一番上から撃つ）。
   2. write-set のどれかで other_root が真なら、2 と理由の字（--dir と write-set の根が違う＝<最初の path・接頭辞を剥がした字>・作業ツリーの一番上から撃つ）。字の頭 --dir と write-set の根が違う と末尾 作業ツリーの一番上から撃つ は型付きの定数 UNKNOWN_OTHER_ROOT と FROM_THE_TOP に置く。
   3. 標準出力は今の形のまま 1 行（folio ceiling: まだ分からない（<理由>））。
4. **変えないもの。** 設計文書の判定 is_design_source（`--dir` の下・preview の下でない・retired の要素が無い）・印の読み方・引き金の要約値と正本の要約値の関数（印の側 `crates/folio/src/stamp.rs` と面の名札 `crates/folio/src/face.rs` も同じ関数を呼ぶ）・3 値の優先・通すときの節点の数の字・命令の口（旗を足さない）・凍結 anchor（`tests/fixtures/ceiling/`）・admit.sh と一括の手順・設計文書。
5. **根が合うときの答えは変わらない。** `--dir` が今の dir の下で、write-set の path が `--dir` の途中から始まらなければ、判定は base と同じ関数で同じ順に進む。今の dir の外を名指さず作業ツリーの一番上から撃つ既存の使い方（(a) の 4）は全部これに当たる。本流の一番上から作業ツリーの置き場を名指しても、write-set が実装の file だけなら 0（設計文書の正本を書き換えない便）のまま（write-set のどれも置き場の名で始まらないので照らせる）。

### (c) 歯（関数名 f142_・base で 0 件）

1. **f142_the_dir_and_the_write_set_share_one_root（単体の歯・`crates/folio/src/gate.rs` の既存の tests の区間 gate_tests の末尾）。** dir_parts が design-intent・./design-intent・a/../design-intent を 1 要素 design-intent に、今の dir の下の絶対 path（今の dir に x/design-intent を足した path）を 2 要素 x・design-intent に解き、../design-intent と今の dir の外の絶対 path（/nonexistent-f142/design-intent）を無いと言うこと。other_root が、要素 .worktrees・x・design-intent に対して design-intent/srs.yaml と +design-intent/adr/ADR-9.yaml と x/design-intent/srs.yaml を真、.worktrees/x/design-intent/srs.yaml と crates/folio/src/gate.rs と design-intent（置き場そのもの）を偽とし、要素 design-intent に対して design-intent/srs.yaml と ~./design-intent/srs.yaml と crates/folio/src/gate.rs を偽、/abs/design-intent/srs.yaml と crates/../design-intent/srs.yaml を真とすること。**base では dir_parts の戻り値の型が違い other_root が無く組み立てが落ちる＝RED。**
2. **f142_the_gate_is_unknown_from_above_the_worktree（`crates/folio/tests/gate.rs`・binary 経由）。** 既存の Repo（一時 dir の design-intent/ に束の source の写し）に合格で新しい印（stamp-pass.yaml を fresh）を置き、その置き場を一時 dir の .worktrees/x/design-intent/ へ写す（一時 dir を本流の一番上、.worktrees/x/ を作業ツリーの一番上に見立てる）。write-set は design-intent/srs.yaml と crates/folio/src/gate.rs。
   1. 本流の一番上から相対の --dir（.worktrees/x/design-intent）と絶対の --dir で撃つと、どちらも終了コード 2 で、標準出力が まだ分からない と --dir と write-set の根が違う＝design-intent/srs.yaml と 作業ツリーの一番上から撃つ を持つ。
   2. 作業ツリーの一番上から相対の --dir（design-intent）と絶対の --dir で撃つと、どちらも終了コード 0 で 正本の要約値が同じ を持つ（今までどおり）。
   3. 本流の一番上から相対の --dir で write-set を crates/folio/src/gate.rs だけにすると、終了コード 0 で 設計文書の正本を書き換えない便 を持つ（(b) の 5）。
   **base では 1 が 0（通す）＝RED。**
3. **f142_the_gate_is_unknown_when_the_dir_is_outside_the_cwd（同）。** 2 と同じ置き場で、次の 4 通りが終了コード 2 で、標準出力が まだ分からない と --dir が今の dir の下に無い と 作業ツリーの一番上から撃つ を持つ: 無関係の dir（一時 dir の other/）から絶対の --dir・同じ dir から ../.worktrees/x/design-intent・作業ツリーの一番上の下の dir（crates/）から ../design-intent・無関係の dir から絶対の --dir で write-set が実装の file だけ。加えて、作業ツリーの一番上から --dir design-intent で write-set に置き場の要件書の絶対 path を渡すと、終了コード 2 で --dir と write-set の根が違う を持つ。**base では 1 通り目が 0（通す）＝RED。**

fixture は新しい file を足さない。歯は既存の口（Repo・put_stamp・copy_tree・code・stdout）と、tests/gate.rs に足す小さな口 2 つ（今の dir と --dir を渡して撃つ gate_at・作業ツリーを模す above_the_worktree）を使う。凍結 anchor は読むだけで書き換えない。

### (d) 採らなかった形

1. **案 B: --dir を正規化して write-set と同じ根に揃える（今の dir の外の --dir は許さない）。** 今の dir の下へ正規化しても、C（本流の一番上から相対の .worktrees/x/design-intent）の要素は .worktrees・x・design-intent のままで、write-set の design-intent/srs.yaml と一致しない。案 B だけでは C と D の偽の 通す が残る（起草役の読み・(a) の 3）。C を案 B で直すには、--dir から上へ辿って作業ツリーの一番上（.git の在る dir）を探し、write-set をそこからの相対と読む必要があり、門が git の置き場の形に依る新しい前提になる。今の dir を根と読む今の約束（関数 run の注・admit.sh）を変えずに、照らせないときに止めるほうが小さく、戻しやすい。
2. **--dir の末尾の要素だけで照らす（write-set の path が design-intent で始まれば置き場の file と読む）。** C と D で印を読むようになるが、本流の一番上から撃つと、今の dir と --dir が指す置き場のどちらを write-set が指すのかを門が推し量ることになる。推し量りが外れると、別の置き場の印で 0 を返しうる。
3. **write-set の path を正規化して照らす（`..` や絶対 path を解く）。** write-set は契約表の字のまま器が渡す repo の根からの相対で、`..` も絶対 path も器の決まりの外である。解くより照らせないと言うほうが P-4.2 に合う。
4. **撃つ所の決まりを文書と手順だけで守る（直さない）。** FR20 の注の今の形で、席と器の手順が作業ツリーの一番上から撃つ限りは当たらない。しかし、門が照らせないのに 0 を返す形が残り、P-4.1 に反する。

### (e) 既存の歯のうち落ちるもの・突然変異・面の変化

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest **909 / 909**（base 906 + 単体 1 + gate 2）・clippy 0 警告・床 4 本 rc 0・`folio build` の出力 30 file（base と diff -r で全 file が一致）・tests/gate.rs 16 / 16（既存 14 を含む）・tests/stamp.rs 13 / 13。凍結 anchor（`tests/fixtures/ceiling/` の下）は 1 byte も変わらない（差分は `crates/folio/src/gate.rs` と `crates/folio/tests/gate.rs` の 2 file だけ）。
2. **回帰なし（本流の撃ち方・起草役の実測）。** repo の写しの根で `--dir design-intent` を使い、base と本便の binary に同じ write-set を 7 通り渡した（便 141 の write-set・本便の write-set・要件書と語彙・./ 付き・+ 付きの新しい判断の記録・preview と retired だけ・docs/design だけ）。7 通りとも終了コードと標準出力が byte で同じだった。止める（1）と まだ分からない（2）の答えの形は既存の歯 14 本（AC18 の 7 場合と便 126・129 の歯）が見ており、全部緑。(a) の 3 の G と H（対照）も base と同じ字。
3. **RED（起草役の実測）。** 歯だけを base に当てた写しで、単体の歯は組み立てが落ち（dir_parts の型と other_root が無い）、gate の f142_ の 2 本は 2 本とも落ちる（1 通り目の答えが 0 の 通す）。
4. **突然変異（起草役の実測・本便を当てた写しの `crates/folio/src/gate.rs` を 1 通りずつ変える）。** 8 通りとも f142_ の 2 本以上が落ち、既存の歯は緑のまま。

| 変異 | 単体 f142_ | gate f142_ 一番上より上 | gate f142_ 今の dir の外 |
| --- | --- | --- | --- |
| M1 根の突き合わせを消す（判定を base に戻す） | 緑 | 落ちる | 落ちる |
| M2 根が違うとき 通す を返す | 緑 | 落ちる | 落ちる |
| M3 今の dir の外の絶対 path を要素のまま使う（相対だけ直す） | 落ちる | 緑 | 落ちる |
| M4 `..` を落とす（base の形） | 落ちる | 緑 | 落ちる |
| M5 --dir の途中から始まる path を見ない | 落ちる | 落ちる | 緑 |
| M6 絶対 path と `..` の write-set を見ない | 落ちる | 緑 | 落ちる |
| M7 根の突き合わせを 通す（設計文書の正本を書き換えない便）の判定の後へ移す | 緑 | 落ちる | 落ちる |
| M8 理由の字の末尾を変える（一番上から撃つ） | 緑 | 落ちる | 落ちる |

5. **面の変化は無い。** 門は面を書かず、`folio build` の出力は base と byte で同じ（(e) の 1）。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 3 本とも印なし。書き換える 2 本 = `crates/folio/src/gate.rs`・`crates/folio/tests/gate.rs`。本文を変えない 1 本 = `crates/folio/tests/stamp.rs`（verify の `--test stamp` で名指すので入れる・門を `--dir src` で撃つ歯を持つ）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/gate.rs` | 452 | 1,048 | 510（+58） | 990 |

   余地は S の見積 100 を超える。行数の上限の歯（`grep -rn 1100・1250・700` の 3 本 = face.rs・face_srs.rs・tests/schema.rs）は本便の write-set に当たらない。src の外の参考値は `tests/gate.rs` 765 → 849・`tests/stamp.rs` 773（不変）。rustfmt の差の数は gate.rs 9 → 12・tests/gate.rs 16 → 19（本流も fmt に合わない所を持ち、CI は fmt を見ない・実装者が整えてよい）。
3. **size は S。** 触る src は 1 本で、src の増分は +58（関数 1 つを直し 1 つを足し、定数 3 つ・run の頭に 7 行）と単体の歯 1 本。
4. **verify は 5 行**で、done の 5 の塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f142_` = (c) の 1。
   2. `cargo nextest run -p folio --test gate f142_` = (c) の 2・3（2 本）。
   3. `cargo nextest run -p folio --test gate` = 門の歯の全部（AC18 の 7 場合と便 126・129 の歯を含む・参考値 16 本）。
   4. `cargo nextest run -p folio --test stamp` = 印の歯の全部（周の一時 dir を今の dir にして `--dir src` で門を撃つ歯を含む・参考値 13 本）。
   5. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
   base f85d346 では 1 と 2 が 0 件で終了コード 4（歯が無い）、3 は 14 本・4 は 13 本で緑、5 は 0 警告である。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は gate・stamp の 2 本で、どちらも write-set に在る。`--bin folio` の絞り込みの語 f142_ を関数名に持つ src は `crates/folio/src/gate.rs` で、write-set に在る。

### (g) 門と受付

1. **門（本便が直す穴そのものなので作業ツリーの一番上で撃った）。** 作業ツリー planner-d142 の一番上で `folio ceiling --gate --dir design-intent --write-set crates/folio/src/gate.rs crates/folio/tests/gate.rs crates/folio/tests/stamp.rs` を main の binary で撃つと 0（通す・設計文書の正本を書き換えない便）。本便を当てた写しの binary でも同じ字で 0。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測・(h) の 6）。並行の便は無い。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d142-draft.md`、模擬の差分と script は同じ dir の d142-scripts（repo には入れない）。

1. 再現: repro-142.sh（binary・repo の写しの根・作業 dir）で (a) の 3 の 8 通りを撃つ。(a) の 2 は本流の一番上と作業ツリーの一番上で 3 通りを手で撃つ（real-worktree.log）。
2. 模擬: base の写しの根で apply-142.py（実装）と apply-142-teeth.py（歯）を撃つ。その差分が c142.patch。workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との diff -r・verify の 5 行（verify-142.sh）を撃つ。regress-142.sh（base の binary・本便の binary・repo の写しの根）で (e) の 2 の 7 通り。
3. RED: apply-142-teeth.py の歯だけを base に当てる（r142-teeth.patch）。単体の歯は組み立てで落ちるので、gate の 2 本は `crates/folio/tests/gate.rs` の差分だけを当てて撃つ。
4. 突然変異: mut-142.sh（(e) の 4・撃った後は binary を組み直す）。
5. 余地: lines-142.py と lines-142.awk（同じ式の 2 実装）を repo の根で write-set の file に当てる。
6. 受付の先撃ち: 契約を commit した後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-142.md#eo`。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書 FR20 の注の既知の穴の字（着地の後の次の一括で直す・(a) の 6・§5）。FR20 の規範文と受入基準 AC18 の場合（場合を足すなら要件書の改訂＝一括の材料）。作業ツリーの一番上を探して write-set の根を推し量る形（(d) の 1）。設計文書の判定 is_design_source・印・引き金の要約値の関数。admit.sh と一括の手順（今の撃ち方のまま答えが変わらない）。設計ノート ceiling-gate.md の字。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 門が見るのは path の字面で、symlink を解かない。symlink を通して同じ置き場を名指す絶対 path は照らせないと言う（通さない側に倒れる）。`--dir` が今の dir そのもの（`.`）のときは要素が空で、write-set の全部を設計文書の path と読む（base と同じ・通さない側）。write-set の path が `--dir` の途中から始まらないのに別の根からの相対である形（例えば置き場の名が --dir の要素の列のどこにも無い別の repo の path）は、字面では見分けられない。通さない側の誤りがもう 1 つ在る。`--dir` が 2 要素以上で、repo の一番上に `--dir` の後ろの要素と同じ名の dir が在ると（例えば `--dir docs/design-intent` で write-set が `design-intent/x`）、その dir の正しい path も 根が違う と言われる。答えは 2（まだ分からない）なので黙って通す側には倒れない。
3. **撤退条件。** (1) 本便の後に、置き換えも足しもしない既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に、着地の直前の main の撃ち方（repo の根で `--dir design-intent`）で (e) の 2 の 7 通りの write-set の答えが、着地の直前の main の binary の答えと終了コードか標準出力で違うか、`folio build` の出力か folio2 自身の床 4 本の結果が 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で `crates/folio/src/gate.rs` の関数 run・dir_parts・is_design_source か `crates/folio/tests/gate.rs` の口 Repo が base と違えば、(a)(b)(e) を (h) の手順で数え直してから運ぶ（穴が既に閉じていたら止めて席へ返す）。

## 2. 範囲

- 入れる: `crates/folio/src/gate.rs` の関数 dir_parts の戻り値の型と字面の解き方、関数 other_root、理由の字の定数 3 つ、関数 run の頭の 2 つの確かめ、単体の歯 1 本（既存の tests の区間）。`crates/folio/tests/gate.rs` の口 2 つと歯 2 本。
- 入れない: 設計文書の判定・印の読み方・要約値の関数・3 値の優先・命令の旗・凍結 anchor・admit.sh と一括の手順・要件書（FR20 の注を含む）と設計文書・新しい fixture の file・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| dirparts | --dir の要素 | `crates/folio/src/gate.rs` の dir_parts（今の dir からの相対・字面で解く・外なら無い） |
| otherroot | 根の食い違い | `crates/folio/src/gate.rs` の other_root（write-set の path が絶対・`..`・--dir の途中から始まる） |
| run | 門の頭 | `crates/folio/src/gate.rs` の run の最初の 2 つの確かめと理由の字の定数 |
| teeth | 歯 | gate.rs の単体の f142_ 1 本と tests/gate.rs の f142_ 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 無し（base = main f85d346）。
- 並行の便: 無し。
- 本便の着地の後に席が見ること: 本流の `target/debug/folio` を組み直す（admit.sh と一括の手順はこの binary で門を撃つ・組み直すまでは着地の前の binary のままで穴が残る）。台帳 f2-648.214 を閉じる。要件書 FR20 の注の「門には既知の穴が在る…未解決…直るまでは、門を作業ツリーの一番上の置き場から撃つ」の字を、次の一括で着地の後の字（穴は便 142 で閉じた・一番上以外から撃つと まだ分からない と撃ち直し方を返す）に直す材料として控える（設計文書の正本なので便では直さない・§1 (a) の 6）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eo"
title = "床の穴（台帳 f2-648.214・P-4.1 / P-4.2）: folio ceiling --gate を作業ツリーの一番上以外から撃つと（本流の一番上から作業ツリーの置き場を相対か絶対の --dir で名指す・今の dir の外の --dir）、write-set の path が --dir の要素の列と照らされず、設計文書を書き換える便でも 通す（rc 0・設計文書の正本を書き換えない便）を黙って返す。crates/folio/src/gate.rs の dir_parts を今の dir からの相対に字面で解く形（外へ出るか今の dir の下に無い絶対 path なら無い）にし、write-set の path が絶対・.. を持つ・--dir の途中から始まるかを見る other_root を足し、run の最初（設計文書の判定より前）で どちらかなら まだ分からない（rc 2・理由 --dir が今の dir の下に無い か --dir と write-set の根が違う＝<path>・作業ツリーの一番上から撃つ）を返す。根が合うときの判定・is_design_source・印と要約値の関数・凍結 anchor・命令の旗・要件書は変えない（FR20 の 7 場合は増やさず、判定が実行できないときの まだ分からない の族に置く）。歯は単体の f142_ 1 本と tests/gate.rs の f142_ 2 本。設計文書の正本を書き換えないので門の対象外で、FR20 の注の既知の穴の字は着地の後の一括で直す"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/gate.rs", "crates/folio/tests/gate.rs", "crates/folio/tests/stamp.rs"]
verify = ["cargo nextest run -p folio --bin folio f142_", "cargo nextest run -p folio --test gate f142_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "単体の歯 f142_（dir_parts が相対・./・a/../ と今の dir の下の絶対 path を今の dir からの要素に解き、../ と今の dir の外の絶対 path を無いと言い、other_root が --dir の途中から始まる path・絶対 path・.. を持つ path を真、--dir の全要素で始まる path と実装の path を偽とする）が緑、tests/gate.rs の f142_ の 2 本（作業ツリーを模した置き場を本流の一番上から相対と絶対の --dir で撃つと rc 2 で --dir と write-set の根が違う＝design-intent/srs.yaml と 作業ツリーの一番上から撃つ を出し、作業ツリーの一番上からは相対も絶対も今までどおり rc 0、実装の file だけの write-set は本流の一番上からも rc 0 で 設計文書の正本を書き換えない便 / 今の dir の外の --dir は絶対・..・実装だけの write-set でも rc 2 で --dir が今の dir の下に無い を出し、write-set の絶対 path は rc 2 で根が違う）が緑、tests/gate.rs の歯の全部（受入基準 AC18 の 7 場合と便 126・129 の歯を含む）が緑、tests/stamp.rs の歯の全部（--dir src で門を撃つ歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main の出力と file 数も byte も変わらず、repo の根で --dir design-intent の門の答えは §1 (e) の 2 の 7 通りで着地の直前の main の binary と同じである"
<!-- contracts:end -->
