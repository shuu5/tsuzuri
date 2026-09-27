# 設計: 便 178 — 門で、置き場そのものか置き場を下に持つ dir を名指す write-set の項目を、置き場の file を全部書き換える項目として読む（`.`・`design-intent` が 通す になる穴）

- 要件: FR20（天井の門・要件書 第 1.51 版）。持ち主が 2026-09-27 22:24 JST に第 1.51 版を承認した（逐語「承認する」・対話面 R-8・PR #367）。第 1.51 版の FR20 の規範文は「照らせるとき、その一覧の項目が指す file（dir の項目はその下の file を全部指し、**置き場そのものか置き場を下に持つ dir〔作業ツリーの一番上を含む〕の項目は置き場の file を全部指す**）に設計文書の置き場の file（生成物の棚 preview/ と退役の置き場の file を除く）が 1 つも無ければ 0（通す）。在るときは、…」で、確かめ方と受入基準 AC18 は門の 9 場合（足した 1 つ = 書き換える file の一覧が置き場そのものか置き場を下に持つ dir を名指す〔置き場の file を全部書き換える便として扱う〕）。本便はこの字を実装に写す。契約表の行の req は FR20 の 1 つ（本流に在る id・便 142・150・169 と同じ）。受入基準 AC18 は器の要件面に無い id なので req に書かない（便 142 の受付の先撃ちの実測）。
- 条: P-4.1 / P-4.2（照らせない形や読み違いで 通す と言わない＝置き場を丸ごと書き換える便を「正本を書き換えない便」と読まない）/ P-3.3（0 は天井の合格ではない＝理由の行に印の周と「印の後の変更は審査していない」）/ P-10.1（期待の字は歯の側の手書き＝`tests/fixtures/ceiling/` は 1 byte も変えない）/ N-3.1（例外の口を足さない＝旗や data で名指しを狭められない）。
- 出所: tsuzuri（外の置き場）の写しで天井を 1 周した検証の写し（t3v の tz2・印に反証で支持された 止める が在る）に本流 c52baba の binary の門を撃つと、write-set の `.`・`design-intent`・`design-intent/`・`./design-intent` がどれも 0（通す・設計文書の正本を書き換えない便）、`design-intent/srs.yaml` は 1（止める）だった（席の実測 2026-09-27 22:2x・起草役が写しで撃ち直した＝(a) の 2）。要件の字は第 1.51 版で直った。依頼は席から起草役へ（2026-09-27）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fy` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 2 本（src 1・歯の file 1・どちらも増える）。新しい file・縮む file・消す file・新しい dir は無い。
- 門: 対象外。本便は設計文書の正本（`design-intent/` の下）を書き換えない。write-set 2 本を本流の作業ツリーの binary（`target/debug/folio`・c52baba の組み立て＝src は 93e00c0 と同じ）と 93e00c0 の写しの binary で `folio ceiling --gate` に渡すと、どちらも **0（通す・`通す（設計文書の正本を書き換えない便）`）**。
- 前提: **base = main 93e00c0（要件書 第 1.51 版・PR #367 の着地の後）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。起草は main c52baba（便 177 の着地の後）から始め、PR #367 の着地の後に 93e00c0 で全部を測り直した（c52baba と 93e00c0 の差は `design-intent/srs.yaml` だけで、write-set の 2 本・歯・fixture・`folio build` の file 数と、見本の数〔歯の本数・余地・差分の byte・突然変異〕は同じ。`folio build` の要約だけが要件書の字の分だけ違う）。本便は要件書を書き換えない。
- 実装の見本: origin の枝 `impl/d178`（commit 6cba2eb = 見本の commit e6d502d〔親 c52baba〕に本流 93e00c0 を merge したもの）が本便の後の中身で、`git diff 93e00c0 6cba2eb` が便の全体の差分。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 6cba2eb -- <write-set の file>`・e6d502d と同じ中身）。write-set の外は変えない。
- 並行の便との重なり: base の時点で、門（`crates/folio/src/gate.rs`）と `crates/folio/tests/gate.rs` を書き換える未着地の便の契約は無い（便 175・177 は着地済み＝本流 90dcc26・c52baba。枝 impl/d177 の差分は着地した中身。PR #367〔93e00c0〕は `design-intent/srs.yaml` だけ）。受付の時点で precheck が重なりを見る。

## 1. 設計

### (a) いま起きていること（base 93e00c0 の実測・参考値）

1. **門の判定の順と穴の場所。** `crates/folio/src/gate.rs` の関数 run は、根の突き合わせ（関数 dir_parts・other_root・便 142）と置き場の確かめ（関数 is_place・便 150）の後、関数 is_design_source で write-set に設計文書の正本が在るかを見て、1 つも無ければ印を読まずに 0（`通す（設計文書の正本を書き換えない便）`）を返す。is_design_source は項目の要素の列（頭の `+`・`-`・`~` を剥がし、`.` と空の要素を落とした列）が `--dir` の要素の列より長く、その頭が `--dir` の列と同じときだけ真を返す（base の 224 行 `if parts.len() <= root.len() || … { return false; }`）。そのため置き場そのもの（`design-intent`・`design-intent/`・`./design-intent`）と、置き場を下に持つ dir（作業ツリーの一番上 `.` ほか）の項目は「正本を書き換えない」と読まれ、印に反証で支持された 止める が在っても 0 になる。置き場の下の dir の項目（`design-intent/adr/`）と file の項目は今の式で読める（便 169 の歯 f169_a_dir_item_covers_the_stop_files_under_it）。
2. **穴の再現（t3v の写し tz2 の置き場を起草役の scratch に写し、今の dir をその一番上にして 1 本ずつ撃った・元の写しは触らない）。** tz2 の印は反証で支持された 止める 7 件（srs.yaml 2・design-note/surface-board.yaml・design-note/schema.yaml・constitution.yaml 2・design-note/surface.yaml）と 退けた 1 件を持つ。本流の binary と base の写しの binary は同じ答えだった。

| write-set（1 本ずつ） | base の門 | 本便の後の門 |
| --- | --- | --- |
| `.` | **0（設計文書の正本を書き換えない便）** | 1（srs.yaml・coherence F-1） |
| `design-intent` | **0（同上）** | 1（同上） |
| `design-intent/` | **0（同上）** | 1（同上） |
| `./design-intent` | **0（同上）** | 1（同上） |
| `design-intent/srs.yaml`（対照） | 1（srs.yaml・coherence F-1） | 同じ |
| `design-intent/vocabulary.yaml`（対照） | 0（印の周 t3v-round1・印の後の変更は審査していない） | 同じ |
| `design-intent/preview/`・`crates`（対照） | 0（設計文書の正本を書き換えない便） | 同じ |

3. **folio2 自身の置き場。** 本流の印（周 2026-09-27-round51・4 観点合格）は `refutes: []`。repo の写しの根で `.` と `design-intent` の 4 形を撃つと、base は 0（設計文書の正本を書き換えない便）、本便の後も 0 だが理由が `通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）` に変わる（(a) の 2 の script で実測）。
4. **base の歯（参考値）。** workspace の nextest 998 / 998・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file（全 file の sha256 の要約 dac8d7274867d26f）。`crates/folio/tests/gate.rs` 15 本・binary の単体の歯 151 本。`git grep -n f178_ -- crates` は 0 件・行 id `fy` は 0 件。

### (b) 直す先 — 置き場を名指す項目を、置き場の file を全部指す項目として読む

1. **`gate.rs` の関数 names_the_place（新しい関数）。** write-set の項目の要素の列（頭の `+`・`-`・`~` を剥がし、関数 parts で `.` と空の要素を落とした列＝頭の `./` と末尾の `/` は落ちる）が、`--dir` の要素の列そのものか、その前方の部分列（空の列＝作業ツリーの一番上 `.` を含む）なら真。`--dir design-intent` なら `.`・`./`・`design-intent`・`design-intent/`・`./design-intent`（と頭に `+`・`-`・`~` を付けた形）が真、`design`・`design-intent-x`・`crates`・`design-intent/srs.yaml` は偽。`--dir .worktrees/x/design-intent` なら `.`・`.worktrees`・`.worktrees/x/` も真。
2. **`gate.rs` の関数 run。** (i) 根の突き合わせと置き場の確かめ（便 142・150）は今の順のまま先に撃つ。(ii) 「設計文書の正本を書き換えない便」の 0 の判定を、is_design_source か names_the_place が真の項目が 1 つも無いときに改める。置き場には印の file constitution.yaml（関数 is_place が確かめる・preview/ でも retired でもない正本）が必ず在るので、置き場を名指す項目は必ず置き場の file を指す＝file の一覧を歩いて数えなくても規範文の「置き場の file が 1 つも無ければ 0」と同じ答えになる。(iii) 止める の場所との照らし（関数 covers）では、置き場を名指す項目を「`--dir` からの要素の列が空の dir の項目」として渡す。covers の今の式（項目が dir で場所がその下に在る）で、印のどの 止める の場所にも当たる。印の 止める の場所は印（`stamp.rs` の stop_file）が天井の正本の文書の一覧から解いた file か dir（dir 形の文書は直下の `<頭>.yaml` か meta.id の file か dir そのもの）で、folio2・tsuzuri・凍結の束の天井の正本の文書の一覧（10 本・constitution.yaml ほか file 形 8 本と adr/・design-note/）は preview/ と retired の下を指さないので、「置き場の file（preview/ と retired を除く）を全部指す」と同じ答えになる。(iv) その後の判定（反証の済んでいない 止める → 2 が先・反証で支持された 止める → 1〔その場所の file を理由に出す〕・どれも無ければ 0〔印の周と「印の後の変更は審査していない」〕）と、印が無い・読めない・観点の結果が欠けている の 2 は今の式のまま。
3. **頭の注。** gate.rs の file の頭の注に便 178 の 1 項（2 行）を足す。
4. **変えないもの。** 根の突き合わせ（dir_parts・other_root）と置き場の確かめ（is_place）の式と順と理由の字・is_design_source の式（単体の歯 gate_classifies_design_sources の、項目 design-intent を偽とする行を含めて本文を変えない）・covers の式・印の読み（read_stamp）・置き場の下の dir の項目と file の項目と実装だけの便の答え・理由の字・印（`stamp.rs`）・要件書と判断の記録と設計ノートの字・生成区間・fixture の file。folio2 自身の床の結果と `folio build` の出力（35 file）は base と byte で同じ（起草役の実測・(e) の 1）。

### (c) 歯（f178_・base で 0 件）

1. **f178_a_place_item_stops_on_an_upheld_stop（`crates/folio/tests/gate.rs`・binary 経由）。** 凍結の束の写し（既存の Repo・`--dir design-intent`）に、4 観点合格の印の refutes を 退けた 1 件（constitution.yaml）と反証で支持された 止める 1 件（`adr/ADR-1.yaml`・reality R-1）に替えて置く。4 形（`.`・`design-intent`・`design-intent/`・`./design-intent`）を実装の file と一緒に渡すと、どれも 1 で `止める（反証で支持された 止める の場所の file を書き換える: adr/ADR-1.yaml（reality R-1）`。`~./design-intent/` と `+.` も 1。対照（今どおり）: `design-intent/adr/`（置き場の下の dir）→ 1・`design-intent/adr/ADR-2.yaml` → 0（印の後の変更は審査していない）・`design-intent/preview/` と `design-intent/adr/retired/` → 0（設計文書の正本を書き換えない便）・`design-intent-x`・`design`・`crates`・`docs/` → 0（同）。作業ツリーの一番上の上（便 142 の形・今の dir = 本流の一番上に見立てた一時 dir・`--dir .worktrees/x/design-intent`）から `.`・`.worktrees`・`.worktrees/x/`・`./.worktrees/x/design-intent` → 1。判定の順の対照: 同じ所から `.` と `design-intent/srs.yaml` → 2（`--dir と write-set の根が違う＝design-intent/srs.yaml`）・`--dir design-intnet` に `.` → 2（`--dir が設計文書の置き場でない（design-intnet・`）。**base は 4 形が 0 ＝RED。**
2. **f178_a_place_item_passes_when_no_stop_is_upheld（同）。** 4 観点合格の印で refutes が空のとき・退けた 1 件だけのとき、4 形はどれも 0 で理由が `通す（印の周 gate-case（判定 合格）に、` で始まり「印の後の変更は審査していない」を含み、「設計文書の正本を書き換えない便」を含まない。**base は理由が「設計文書の正本を書き換えない便」＝RED。**
3. **f178_a_place_item_is_unknown_on_an_unrefuted_stop（同）。** 観点 reality が反証待ちの まだ分からない（wait: 反証）の印に、反証で支持された 止める（srs.yaml・coherence C-1）と反証の済んでいない 止める（rules.yaml・reality R-2）を置く。4 形はどれも 2 で `まだ分からない（反証の済んでいない 止める の場所の file を書き換える: rules.yaml（reality R-2）`（2 が 1 より先・C-1 を出さない）。**base は 0 ＝RED。**
4. **f178_a_place_item_is_unknown_without_a_stamp（同）。** 印が無い写しで 4 形 → 2（`まだ分からない（印が無い）`）。4 観点合格の印から観点 reality の行を外した写しで 4 形 → 2（`まだ分からない（印の観点の結果が欠けている: reality（無い）`）。**base は 0 ＝RED。**
5. **f178_an_item_names_the_place_or_a_dir_above_it（`crates/folio/src/gate.rs` の既存の tests の区間・単体の歯）。** names_the_place を直に呼ぶ: `--dir design-intent` で `.`・`./`・空の字・`design-intent`・`design-intent/`・`./design-intent`・`+design-intent`・`~./design-intent/`・`-.` が真、`design-intent/srs.yaml`・`design-intent/adr/`・`design`・`design-intent-x`・`crates`・`docs/` が偽。`--dir .worktrees/x/design-intent` で `.`・`.worktrees`・`.worktrees/x/`・`./.worktrees/x/design-intent` が真、`x`・`design-intent`・`x/design-intent`・`.worktrees/y`・`.worktrees/x/design-intent/srs.yaml` が偽（後方の部分列は名指さない）。`--dir .`（空の列）で `.` が真・`srs.yaml` が偽。covers に空の列の dir の項目を渡すと、場所 `srs.yaml`・`adr/ADR-1.yaml`・`design-note/` のどれにも当たる。**base は names_the_place が無く組み立てが落ちる（E0425）＝RED。**
6. **RED の実測。** 歯だけの差分（r178-teeth.patch = tests/gate.rs と gate.rs の tests の区間・8,135 byte）を base に当てると単体の組み立てが落ちる（E0425 names_the_place）。tests/gate.rs だけを当てた base では f178_ の binary の 4 本とも落ちる（本文は red-151.log・c52baba でも同じ＝red-178.log）。
7. **既存の歯は本文も fixture も変えない。** 便 142・150・169 の歯（tests/gate.rs の 15 本）と単体の gate_classifies_design_sources・f142_・f150_ は変えずに通る（verify の 3）。

### (d) 採らなかった形

1. **置き場を名指す項目が来たら一律に まだ分からない（2）。** 持ち主が第 1.51 版の承認の要求で退けた案（規範文の案と退けた案を提示した上での「承認する」）。印に 止める が無ければ 通す、支持の 止める が在れば 止める、と普通の file の項目と同じ式で答えるのが規範文の字である。
2. **is_design_source そのものを、置き場と置き場の上の dir でも真にする。** 項目の要素の列から `--dir` の列を外す所（`parts(p)[root.len()..]`）が短い列で壊れ、既存の単体の歯 gate_classifies_design_sources の、項目 design-intent を偽とする行の向きを変えることになる。「置き場の下の path か」と「置き場を名指すか」は別の問いなので、別の関数に分ける（突然変異 M5・M6 で字の照らし方を縛る）。
3. **置き場の file を file system で歩いて数え、1 つずつ照らす。** 置き場の印の file が必ず在り、印の 止める の場所は正本の file か dir だけなので、空の列の dir で照らすのと答えが同じ（(b) の 2 の (ii)(iii)）。歩くと読めない file や symlink の扱いという新しい問いが増える。

### (e) 既存の歯のうち落ちるもの・突然変異・外の置き場

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest **1003 / 1003**（998 + f178_ の 5 本）・clippy 0 警告・床 4 本 rc 0（check 合格 違反 0・まだ分からない 0）・`folio build --write` 35 file（base と全 file の sha256 の要約が同じ dac8d7274867d26f）。tests/gate.rs 19 / 19・binary の単体の歯 152 本。
2. **突然変異（本便を当てた写しの names_the_place / run だけを 1 通りずつ変え、f178_ の 5 本を撃つ）。** 10 通りとも f178_ のどれかが落ちる（見本 6cba2eb で撃った mut.log・各変異の本文は mut-M<n>.out。c52baba の上の e6d502d で撃った結果〔mut-c52baba/〕と同じ）。落ちる歯の略: 止める = a_place_item_stops_on_an_upheld_stop・通す = a_place_item_passes_when_no_stop_is_upheld・未反証 = a_place_item_is_unknown_on_an_unrefuted_stop・印なし = a_place_item_is_unknown_without_a_stamp・単体 = an_item_names_the_place_or_a_dir_above_it。

| 変異 | 落ちる f178_ |
| --- | --- |
| M1 置き場を名指す項目を読まない（base と同じ） | 5 本とも |
| M2 `--dir` の列そのものだけ（前方の部分列を読まない） | 5 本とも |
| M3 作業ツリーの一番上（空の列）を読まない | 5 本とも |
| M4 前方の部分列だけ（`--dir` の列そのものを読まない） | 5 本とも |
| M5 要素を照らさず長さだけ見る | 止める・単体 |
| M6 要素でなく字の前方一致で照らす（`design` が `design-intent` に当たる） | 止める・単体 |
| M7 頭の `+`・`-`・`~` を剥がさない | 止める・単体 |
| M8 `--dir` の後方の部分列も名指すと読む | 単体 |
| M9 名指す項目が 止める の場所を覆わない（0 の判定だけ直る） | 止める・未反証 |
| M10 名指す項目を 0 の判定で数えない（場所の照らしだけ直る） | 止める・通す・未反証・印なし |

   空の列の項目を末尾の `/` なしとして covers に渡す変異は、covers が `--dir` そのものを dir と見るので答えが変わらない（同値の変異として数えない）。
3. **外の置き場（tsuzuri）。** 着地の後の binary の門は、tsuzuri の今の印のまま `.` と `design-intent` の 4 形を 1（(a) の 2 の表の右の列・t3v の写しで実測）で返す。印の 止める の直しは tsuzuri の手番で、本便は門の読み方だけを直す。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 2 本とも印なし（`crates/folio/src/gate.rs` と `crates/folio/tests/gate.rs`・どちらも増える）。差分 11,167 byte（`git diff 93e00c0 6cba2eb | wc -c`・2 file・+176 −3）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/gate.rs` の 1 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した（lines151.log）。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/gate.rs` | 466 | 1034 | 508（+42） | 992 |

   歯の file は src の外なので余地を測らない（参考値 685 行 → 816 行・wc -l）。
3. **size は S。** 余地 1034 は S の見積 100 を超える。
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1 に揃える。便の後の写しで 4 行とも rc 0（run151.log）。base では 1 と 2 が 0 件で rc 4、3 と 4 は rc 0。
   1. `cargo nextest run -p folio --test gate f178_` = (c) の 1〜4（4 本）。
   2. `cargo nextest run -p folio --bin folio f178_` = (c) の 5（1 本）。
   3. `cargo nextest run -p folio --test gate` = 門の歯の全部（AC18 の場合と便 142・150・169 の歯を含む・参考値 19 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は gate の 1 本で write-set に在る。`--bin folio` の絞り込みの語 f178_ を関数名に持つ src は `crates/folio/src/gate.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 冒頭のとおり対象外・0。
2. **受付。** 受付の先撃ち（precheck）は、本契約を commit した作業ツリーで契約に起因する断り 0（起草の記録）。共通の検証は同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d178-draft.md`、差分と script は同じ dir の d178-scripts（repo には入れない）。

1. 模擬: 差分 c178.patch を base に当て、run-178.sh（組み立て・workspace の nextest・clippy・床 4 本・`folio build --write` の file 数と要約）・verify-178.sh（verify の 4 行）。
2. RED: r178-teeth.patch を base に当てて `cargo nextest run -p folio f178_`（単体は組み立てが落ちる）、tests/gate.rs の分だけを当てて `cargo nextest run -p folio --test gate f178_`（4 本とも落ちる）。
3. 突然変異: mut-178.py（(e) の 2 の 10 通り・最後に戻した gate.rs で組み直す）。
4. 外の置き場と folio2 自身の置き場: tsuzuri-178.sh（t3v の写しを scratch に写し、本流・base・本便の binary で門を 1 本ずつ）。
5. 余地と差分の byte: lines-178.sh（lines-178.py と lines-178.awk）。門: gate-178.sh。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 要件書・判断の記録・設計ノートの字（第 1.51 版の字は PR #367 が運ぶ）・印（`stamp.rs`）・根の突き合わせと置き場の確かめ・is_design_source・fixture の file・外の置き場の印の直し（その置き場の手番）・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** (1) 項目は今の dir（作業ツリーの一番上）からの相対で読む。`--dir .worktrees/x/design-intent` のとき項目 `design-intent`（`--dir` の後方の部分列）は今の dir の design-intent（別の置き場）を指し、置き場も置き場を下に持つ dir も名指さないので今どおり（ほかに正本が無ければ 0）。規範文の照らせない形（`--dir` の 2 番目か後ろの要素から末尾までの並びの**下に在る** path・便 142）はこの項目そのものを含まない。席の受付の手順（admit.sh）は本流の作業ツリーの一番上から `--dir design-intent` で門を撃つので、この形は起きない。(2) 空の字の項目（と `+`・`-`・`~` だけの項目）は、要素の列が `.` と同じ空の列になるので作業ツリーの一番上を名指すと読む（広い側）。(3) 置き場を名指す項目は、書き換える file が実際に在るかを見ない（置き場の file を全部書き換える便として扱う・規範文の字どおり）。(4) 天井の正本の文書の一覧が preview/ か retired の下の file を名指す置き場（今は無い）では、そこを場所とする 止める にも置き場を名指す項目が当たる（広い側）。
3. **撤退条件。** (1) 本便が要件書 FR20（第 1.51 版）の字を変えないと書けないと分かったら、止めて席へ返す。(2) 受付の時点で本流の gate.rs の run・is_design_source・covers が base（93e00c0）と違えば、止めて席へ返す（数え直してから運ぶ）。(3) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(4) 本便の後に folio2 自身の床 4 本の結果か `folio build` の出力（35 file）が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/gate.rs` の関数 names_the_place・run の 2 か所（0 の判定と場所の照らしの項目）・頭の注・単体の歯 1 本。`crates/folio/tests/gate.rs` の定数 2 つと歯 4 本。
- 入れない: 印・根の突き合わせ・置き場の確かめ・is_design_source・covers・設計文書・生成区間・fixture の file・外の置き場・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| place | 置き場の名指し | `gate.rs` の names_the_place（`--dir` の列そのものか前方の部分列） |
| gate | 門の式 | `gate.rs` の run（名指す項目は置き場の file を全部指す・空の列の dir で場所を照らす） |
| teeth | 歯 | tests/gate.rs の f178_ 4 本（AC18 の 9 場合目の 4 形 × 4 通りと対照を binary で）・gate.rs の単体の f178_ 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 便 142（根の突き合わせ）・便 150（置き場の確かめ）・便 169（門の式）。どれも本流に在る。要件の字は第 1.51 版（PR #367）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。本流の `target/debug/folio` を組み直す。tsuzuri へ (e) の 3 を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fy"
title = "門の穴の直し（要件書 第 1.51 版の FR20・AC18 の 9 場合目）: folio ceiling --gate は、置き場そのものか置き場を下に持つ dir を名指す write-set の項目（. と design-intent と design-intent/ と ./design-intent・--dir .worktrees/x/design-intent なら .worktrees と .worktrees/x/ も）を、置き場の file を全部書き換える項目として読む。crates/folio/src/gate.rs に関数 names_the_place（頭の + - ~ を剥がし . と空の要素を落とした項目の要素の列が --dir の列そのものかその前方の部分列）を足し、run は is_design_source か names_the_place が真の項目が無いときだけ 設計文書の正本を書き換えない便 の 0 を返し、名指す項目は --dir からの列が空の dir の項目として 止める の場所と照らす（印の 止める のどれにも当たる）。その後の判定（反証の済んでいない 止める と印の欠けの 2 が先・反証で支持された 止める の 1・印の周を添えた 0）と、根の突き合わせと置き場の確かめ（便 142・150）の順と字・is_design_source・covers・置き場の下の dir と file の項目と実装だけの便の答え・印・要件書と設計文書は変えない。歯は f178_ の 5 本（tests/gate.rs の 4 本 = 4 形 × 支持の 止める 1・止める 無し 0・未反証 2・印が無いか欠け 2 と今どおりの対照を binary で・gate.rs の単体 1 本 = names_the_place の真偽と空の列の covers）。実装の見本は origin の枝 impl/d178 の commit 6cba2eb（見本 e6d502d に本流 93e00c0 を merge したもの）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。設計文書の正本を書き換えないので門の対象外。base = main 93e00c0"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/gate.rs", "crates/folio/tests/gate.rs"]
verify = ["cargo nextest run -p folio --test gate f178_", "cargo nextest run -p folio --bin folio f178_", "cargo nextest run -p folio --test gate", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/gate.rs の f178_ の 4 本（--dir design-intent で . と design-intent と design-intent/ と ./design-intent の 4 形が、反証で支持された 止める adr/ADR-1.yaml の印では 1 でその場所を理由に出し〔~./design-intent/ と +. も 1・作業ツリーの一番上の上からは .worktrees と .worktrees/x/ も 1〕、止める が無いか 退けた だけの印では 0 で印の周と 印の後の変更は審査していない を出して 設計文書の正本を書き換えない便 を出さず、反証の済んでいない 止める rules.yaml の印では 2 でその場所を出し、印が無いか観点の行が欠けた印では 2。置き場の下の dir・置き場の下の file・preview/ と retired・design-intent-x と design と crates と docs/ の項目と、根の違う path と置き場でない --dir の 2 は今どおり）が緑、binary の単体の f178_ の 1 本（names_the_place が --dir の列そのものと前方の部分列〔空の列と + - ~ の付いた形を含む〕だけを真とし、置き場の下の path・字の前方一致・後方の部分列を偽とし、空の列の dir の項目が covers でどの場所にも当たる）が緑、tests/gate.rs の歯の全部（便 142・150・169 の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
