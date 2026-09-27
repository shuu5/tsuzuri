# 設計: 便 169 — 天井の門を「書き換える file に反証で支持された 止める が在るか」だけに直し、印に 止める の場所の file を記す（門 C・判断の記録 ADR-30 決定 (6)）

- 要件: FR20（天井の印と門）。門の式と印の場所の欄は判断の記録 ADR-30 決定 (6) と、設計の PR（枝 docs/struct30 b11e70f）の要件書 第 1.49 版の FR20 と受入基準 AC18 の字のとおり（注の「印は周のあとに書く」を含む）。印の中身（観点の 3 値と反証の結果）も FR20 が定めるので req は FR20 の 1 つ。`--check`（FR18）と `--refute` の答えは変えない。
- 条: P-4.1 / P-4.2（印が無い・読めない・観点の結果が欠けている・反証の済んでいない 止める は まだ分からない）/ P-3.3（通す は天井の合格ではない＝理由に「印の後の変更は審査していない」）/ P-15.2 の向き（止める の数えは `--check` と同じ関数の結果を印が運び、門は印の字だけを読む）/ P-10.1（凍結 anchor stamp-expected.yaml は 1 byte も変えない）/ N-3.1（旗を足さない）。
- 出所: 持ち主の承認 2026-09-27 12:50 JST（対話面 R-8・逐語「承認する」・構造の直し C）。診断 loop-diag.md の表 6（周 25 回のうち 8 回は、一括の取り込みで全体の要約値が動き門が 2 を返したので印を取り直した本流の周）。
- 置き場: 審査の材料は行 `fp` の §1 だけ。write-set 8 本（src 4・歯の file 3〔findings は本文を変えない verify の scope〕・fixture 1）。新しい file・dir・消す file は無い。
- 門: 対象外。本便は設計文書の正本（`design-intent/` の下）を書き換えない。作業ツリー planner-d169 の一番上で write-set 8 本（接頭辞は admit.sh と同じく剥がす）を base の binary に渡すと **0（通す・設計文書の正本を書き換えない便）**、本便の binary でも同じ字で 0。印の欄の一覧は生成区間に写っていない（schema の節 9 本を実測）ので生成区間の変更も無い。
- 前提: **設計の PR（docs/struct30・ADR-30 と FR20 の新しい字）の着地の後に受け付ける**。受付の先撃ちの ADR-30 の name-unresolved はその着地の前だけのもの。
- 改訂 a（2026-09-27・席の追記と訂正）: FR20 第 1.49 版の注「印は周のあとに書く」。今の `--stamp` は既にそう書く（base も不合格の周の後に書く＝RED の本文で実測）ので、書く条件は変えず、歯 (c) の 6 と変異 M10 で縛るだけにした。
- 改訂 b（2026-09-27・席の裁定）: まだ分からない の観点は、理由が反証の済んでいない 止める だけのとき（印の観点の行の欄 wait）に限って file ごとの判定に任せ、ほかの理由の まだ分からない は 2 にする（(b) の 1〜3・7）。歯 (c) の 7 と変異 M11〜M13 で縛る。
- 改訂 c（2026-09-27・席の依頼・検証役 d169-verify の blocking B-1 と非 blocking N-1）: ① 本文の性質のうち歯で縛られていなかった 5 つ（検証役の変異 V5・V6・V7・V14・V15 が生き残った）を、検証役の歯の案（tests の 2 file）を取り込んで縛った（(c) の 4・5・7）。② write-set の項目が dir のとき、その下の 止める の file も当たると読む（(b) の 3・歯 (c) の 8・変異 M14・M15）。途中の起草 a8aec7d（未反証の 止める の行の有無で決める形）は採らず、wait の形のまま（改訂 b の FR20〔b11e70f〕と同じ条件）。
- **base = origin/main 62111b1。数は base の写しの実測（参考値・行 D-13）**。受付の時点の main が違えば §1 (h) で数え直す。

## 1. 設計

### (a) いま起きていること（参考値・base 62111b1）

1. **門は印の古さで止める。** `crates/folio/src/gate.rs` の run は、根の突き合わせ（便 142）・置き場の確かめ（便 150）・設計文書の file が無ければ 0 の後、印の欄 trigger と今の引き金の要約値が違えば 2、まだ分からない の観点が在れば 2、不合格 が在れば 1、正本の要約値が違えば節点の表（印の rest と nodes・`main.rs` が渡す `graph::stamp_table`）を数えて 0。作業ツリー planner-d169 の一番上で base の binary に `design-intent/srs.yaml`・`design-intent/ceiling.yaml`・`design-intent/adr/ADR-18.yaml` を渡すと 3 通りとも **2（まだ分からない（印が古い（引き金の要約値が違う）））**。本流の印は 51 周目（4 観点合格・refutes は空）。
2. **印は 止める の場所を持たない。** `crates/folio/src/stamp.rs` の derive は refutes の行を `{viewpoint, finding, refute}` で書き、反証の値が読めた 止める だけを載せる（済んでいない 止める は行が無い）。値は `crates/folio/src/findings.rs` の count_findings が数え、Counted の refutes（id と値の対）で渡す。印を書く条件は周の 3 値に依らない（束・所見 file・起動の記録が揃えば、不合格・まだ分からない の周の後も書く・揃わない周だけ まだ分からない で書かない）。印の観点の行は 3 値だけを持ち、まだ分からない の理由を持たない。
3. **印の欄の読み手（grep の実測）。** sources = 門と面の名札（`face.rs`）。trigger・rest・nodes = 門だけ。viewpoints = 門と名札。refutes = 読み手なし。TRIGGER_* は `ceiling.rs` の床の木の葉でもあり、生成区間 `design-intent/ceiling.yaml` の schema.trigger と trigger_note・凍結 anchor `tests/fixtures/schema/ceiling-region.txt`・fixture の ceiling.yaml 20 本・`tests/gate.rs` の独立の実装 trigger_hex が同じ一覧を持つ。
4. **過去の周の束から組み直した新しい印（参考値）。** 43 周目 = 支持 2 行（coherence F-1 → srs.yaml・reality F-1 at ADR-24.decision → adr/ADR-24.yaml）。52 周目 = 支持 1 行（coherence F-1 at ADR-16.decision → adr/ADR-16.yaml）。47 周目 = `refutes: []`。周 1〜52 の所見の場所の頭には file に解けないもの（adr の plain・figures、design-note の sections・meta）も在る。
5. **base の歯。** nextest 1003 / 1003・clippy 0 警告・床 4 本 rc 0・`folio build --write` 34 file。`tests/gate.rs` 20 本・`tests/stamp.rs` 14 本・`tests/findings.rs` 39 本。`git grep -n f169_ -- crates` 0 件・行 id `fp` 0 件。

### (b) 直す先（src 4 本）

1. **`findings.rs`。** Counted と Sheet の refutes を、止める の所見の全件の行 Refute（id・value〔読めた refute の値・無ければ None〕・doc と at〔所見の place の字〕）の列に替え、count_findings で反証の結果を決めた直後に積む。Counted に waiting（まだ分からない の理由が反証の済んでいない 止める だけ）を足し、count_viewpoint の規則 7（反証が未）で まだ分からない を返すときだけ、所見 file の verdict が まだ分からない でなければ真にする。unrefuted・remaining_stops・refuted_stops・reasons と 3 値の数えは変えない（`--check` と `--refute` の答えは同じ）。
2. **`stamp.rs`。** refutes の行を 止める ごとに `{viewpoint, finding, refute, at, file}` で書く（済んでいなければ欄 refute を書かない）。file は関数 stop_file で解く: doc を天井の正本の documents（`gate::documents`）で file に解き、file 形はその file、dir 形（adr/・design-note/）は at の頭（最初の `.` の前）の `<dir><頭>.yaml` が `--dir` の下に file として在ればそれ、無いか頭が空・`.` 始まり・区切りを含めば dir そのもの（広い側）。doc が文書の一覧に無ければ印を組まず まだ分からない（全部か無しか）。観点の行は、waiting が真の まだ分からない のときだけ末尾に欄 `wait: 反証` を足す。行の順・ほかの欄・止める 0 件の `refutes: []`・書く条件（周の 3 値に依らず書く＝支持の 止める が印に載る・FR20 第 1.49 版の注）は変えない（凍結 anchor は 4 観点合格なので変わらない）。
3. **`gate.rs` の run と read_stamp。** 判定の順 = 根の突き合わせ（2）→ 置き場の確かめ（2）→ 設計文書の file が無い（0・字は今のまま）→ 印が無い（2）/ 読めない（2）→ 天井の正本が読めない（2）→ 観点の結果が欠けている（2・天井の正本の観点の行が印に無いか 3 値でない、または まだ分からない で `wait: 反証` が無い）→ 反証の済んでいない 止める（refute が無いか まだ分からない）の file を書き換える（2）→ 反証で支持された 止める の file を書き換える（1）→ 通す（0）。突き合わせ = write-set の設計文書の path から `--dir` の要素を外した字が行の file と同じか、行の file が dir 形でその下に在るか、write-set の項目が dir（末尾が / か `--dir` の下の dir）で行の file がその下に在るか（改訂 c）。退けた は見ない。2 と 1 が同時なら 2（第 1.49 版の FR20 の字どおり・前の決まりを引き継ぐ）。理由の字は型付きの定数: `印の観点の結果が欠けている: <観点>（無い）` か `<観点>（まだ分からない）`・`反証の済んでいない 止める の場所の file を書き換える: <file>（<観点> <所見>）`・`反証で支持された 止める の場所の file を書き換える: <file>（<観点> <所見>）`・通す `印の周 <round>（判定 <印の verdict>）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない`。印からは round・verdict・viewpoints（id・verdict・wait）・refutes（行ごとに viewpoint・finding・file が要る）だけを読む。**古さでは止めない。** 型 NodeTable と run の 3 つ目の引数を外す。
4. **`main.rs`。** gate::run の呼び出しから `graph::stamp_table` を外す 1 行。
5. **変えないもの。** dir_parts・other_root・is_place・is_design_source・sources_digest・trigger_digest・documents・collect・`graph::stamp_table`・TRIGGER_* と床の木と生成区間・印のほかの欄・面の名札・凍結 anchor・命令の旗・admit.sh。
6. **死ぬものの判断。** 門の NodeTable と印の sources / trigger / rest / nodes の読みは**消す**（読み手が門だけ）。**残す**: 印の trigger と trigger_digest と TRIGGER_*（床の木の葉＝消すと生成区間・凍結 anchor・fixture 20 本・独立の実装まで動き L。trigger だけ消しても TRIGGER_* は生成区間に残る）・rest と nodes と stamp_table（stamp-expected.yaml が nodes を持ち f99_ の 3 本が縛る）・sources（名札が生きた読み手）。撤去は (g) の 3 の後続。
7. **確かめた結果（席の条件・count_viewpoint を読んだ）。** 書かれる印の観点が まだ分からない になる理由は、反証が未（規則 7）のほかに、束が壊れている・起動の記録の違反（要約値が束と合わない ほか）・所見の欄の違反（根拠が正本に無い・反証の結果 file の違反 ほか）・AI が判定できなかった・不合格なのに所見が無い・再判定待ち（止める が全部退けられた不合格）が在る。よってこれらの観点は 2 にし、wait は規則 7 だけで決まり所見の verdict が まだ分からない でない観点に限る。

### (c) 歯（f169_・base で 0 件・どれも binary 経由）

1. **f169_a_stale_stamp_without_an_upheld_stop_passes（`tests/gate.rs`）。** 仮の要約値のままの合格の印（古い）で写しの FR1 の規範文を変え、`design-intent/srs.yaml`・`+design-intent/adr/ADR-3.yaml` → 0・`folio ceiling: 通す（印の周 gate-case（判定 合格）に、` で始まり `印の後の変更は審査していない` を持つ。stamp-unknown.yaml の reality の行に `wait: 反証` を足した印（反証待ちだけの まだ分からない）でも 0・`判定 まだ分からない`。**base は 2（印が古い）＝RED。**
2. **f169_the_gate_stops_only_on_the_file_of_an_upheld_stop（同）。** 新しい合格の印の refutes を 4 行（退けた constitution.yaml・支持 srs.yaml・支持 adr/ADR-1.yaml・支持 design-note/）にして 7 通り: srs.yaml + 実装 → 1・`~./design-intent/adr/ADR-1.yaml` → 1・`+design-intent/design-note/new.yaml` → 1（dir 形）・adr/ADR-2.yaml → 0・constitution.yaml → 0（退けた）・rules.yaml + preview の印 → 0・実装だけ → 0（設計文書の正本を書き換えない便）。**base は 1 通り目が 0 ＝RED。**
3. **f169_an_unrefuted_stop_file_is_unknown（同）。** refutes 3 行（支持 srs.yaml C-1・refute なし srs.yaml R-2・まだ分からない rules.yaml F-2）で srs.yaml → 2（`srs.yaml（reality R-2）` を持ち C-1 を持たない＝2 が先）・rules.yaml → 2・constitution.yaml → 0。**base は 0 ＝RED。**
4. **f169_the_gate_is_unknown_without_a_readable_stamp（同）。** 印が無い → 2・reality の行を外した印 → 2（印の観点の結果が欠けている: reality（無い））・wait の無い まだ分からない の印（stamp-unknown.yaml）→ 2（…: reality（まだ分からない））・支持の 止める（srs.yaml）を持ち reality の行を外した印で srs.yaml → 2（…: reality（無い）・2 が 1 より先）・reality の値を 合格？ にした印 → 2（…: reality（無い））・stamp-unknown.yaml の reality の行に `wait: 反証か` を足した印 → 2（…: reality（まだ分からない））・stamp-fail.yaml の行から at と file を外した印（前の形）→ 2（印が読めない: refutes が読めない）。**base は 2 通り目が別の字（まだ分からない観点: reality）で落ちる＝RED。**
5. **f169_the_stamp_writes_the_stop_places_and_the_gate_reads_them（`tests/stamp.rs`）。** 5 文書を読む周で fidelity に stop-upheld.yaml、coherence に stop-unrefuted.yaml（場所を `{doc: adr, at: ADR-1.decision}` に）、reality に stop-refuted.yaml（場所を `{doc: design-note, at: sections.x}` に・verdict を 合格 に）を置いて `--stamp` → refutes が 3 行 `{viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR2.plain, file: srs.yaml}`・`{viewpoint: coherence, finding: F-1, at: ADR-1.decision, file: adr/ADR-1.yaml}`・`{viewpoint: reality, finding: F-1, refute: 退けた, at: sections.x, file: design-note/}`。観点の行で `wait: 反証` を持つのは coherence だけ。同じ印の門は src/srs.yaml → 1・src/adr/ADR-1.yaml → 2・src/design-note/full.yaml と src/rules.yaml → 0。場所を `{doc: nowhere, at: x.y}` にした支持の 止める の周は `--stamp` が 2（文書の一覧に無い）で印を書かない（全部か無しか・改訂 c）。**base は行に at と file が無く coherence の行も無い＝RED。**

6. **f169_a_failed_round_is_stamped_with_its_upheld_stop（`tests/stamp.rs`）。** 合格の周で印を書いた後、同じ周の fidelity に stop-upheld.yaml を置いて `--stamp` を撃つと、rc 0 で 印を書いた・印の verdict が 不合格・refutes が `{viewpoint: fidelity, finding: F-1, refute: 支持, at: requirements.FR2.plain, file: srs.yaml}` の 1 行。同じ印の門は src/srs.yaml → 1。**base は印を書くが行に at と file が無い＝RED**（合格の周でしか書かない形は変異 M10 で落ちる）。
7. **f169_a_viewpoint_unknown_for_another_reason_does_not_wait（`tests/stamp.rs`）。** readability に反証の済んでいない 止める を持ち verdict が まだ分からない の所見 file、reality に止める が全部退けられた不合格（再判定待ち）、coherence に所見の欄の違反（根拠が正本に無い＝規則 1〜6）と反証の済んでいない 止める を併せ持つ所見 file（改訂 c）を置いて `--stamp` → 3 観点とも まだ分からない で `wait:` を書かない。同じ印の門は src/rules.yaml → 2（印の観点の結果が欠けている: readability（まだ分からない）・coherence（まだ分からない）・reality（まだ分からない））。**base は 2 だが別の字＝RED。**
8. **f169_a_dir_item_covers_the_stop_files_under_it（`tests/gate.rs`・改訂 c）。** 新しい合格の印の refutes を 2 行（支持 adr/ADR-1.yaml C-2・refute なし design-note/ R-2）にして 4 通り: `design-intent/adr/` → 1・`~./design-intent/adr`（末尾に / の無い dir）→ 1・`design-intent/design-note/` → 2・adr/ADR-2.yaml → 0。**base は 1 通り目が 0 ＝RED**（改訂 b の実装でも dir の項目は file として照らされて 0＝変異 M14・M15 で縛る）。

fixture は `tests/fixtures/ceiling/findings/stamp-fail.yaml` の refutes の 1 行に `at: requirements.FR1.shall, file: srs.yaml` を足すだけ。ほかの印は歯の中で fixture を書き替えて作る。

### (d) 採らなかった形

1. **欄の名を stops に替え、済んでいない 止める を別の欄に。** 凍結 anchor の `refutes: []` の行と欄の並びの歯 2 本が変わる。行に refute が無い＝済んでいない、で読める。
2. **門が周の置き場の所見 file を読む。** 門に `--out` が要り、印を周の結果の 1 file とする FR20 の形を崩す。
3. **死ぬものを同じ便で全部消す。** (b) の 6 のとおり L。
4. **まだ分からない の観点を全部 2 にする／全部通す。** 全部 2 は、済んでいない 止める が 1 件で file ごとの判定（決定 (6)）が意味を失い、印を取り直す周をまた起こす（決定 (5)）。全部通すは、束の壊れや AI が判定できなかった観点まで通す（P-4.1）。理由が反証待ちだけの観点に限って印に wait を書き、門がそれだけを file ごとの判定に任せる（席の裁定）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけを当てると既存の歯 16 本が落ちる（本文は impl-only-nextest.log）。** 扱いはこの 3 つだけ。gate_is_unknown_on_an_unknown_viewpoint（wait の無い まだ分からない → 2）は緑のまま残る。
   - 外す 11 本（古さと節点の数で答えを縛った歯）: tests/gate.rs の gate_is_unknown_when_the_stamp_is_stale・f126_ の 6 本・f129_ の 1 本・f151_ の 2 本、tests/stamp.rs の f126_the_gate_reads_the_trigger_the_stamp_wrote。道具 put_stamp_with_nodes・REST_PLACEHOLDER・FR1_PLAIN・ADR2_DECISION・proposed_record も外す。置き換えは (c)。
   - 字だけ直す 4 本（終了コードは同じ）: f142_the_gate_is_unknown_from_above_the_worktree・f150_the_gate_reads_the_place_as_before・f104_the_gate_passes_the_stamp_it_just_wrote の通すときの字（正本の要約値が同じ → 印の後の変更は審査していない・道具 folio_gate は write-set を引数に）と stamp_carries_the_refute_results の行の字。
   - fixture で直す 1 本: gate_stops_on_a_failed_viewpoint（本文は不変・1 のまま）。
2. **便を当てた写し。** nextest **1000 / 1000**（1003 − 11 + 8）・clippy 0 警告・床 4 本 rc 0・build 34 file（base と diff -r で byte 一致）・tests/gate.rs 15・tests/stamp.rs 16・tests/findings.rs 39 が緑。作業ツリーの一番上で base と本便の binary に同じ write-set を渡すと、要件書だけ・設計の PR の形・便 170 の形・`~./design-intent/vocabulary.yaml` は 2（印が古い）→ 0（印の周 2026-09-27-round51（判定 合格）…）。本便の write-set・preview と retired だけ・docs/design だけ・`--dir design-intnet` は両方で同じ字。
3. **RED。** 歯だけ（r169-teeth.patch）を base に当てると f169_ の 8 本が落ちる（落ち方は (c)）。
4. **突然変異（src を 1 通りずつ変え tests/gate.rs と tests/stamp.rs の 31 本を撃つ）。** 起草役の M1〜M15 と検証役の V5・V6・V7・V14・V15（字は検証役の vmut-b.py のまま）の 20 通りとも f169_ が落ちる（既存の歯で落ちるのは表に書いたものだけ）。

| 変異 | 落ちる f169_ |
| --- | --- |
| M1 場所の file を見ない | gate の 3 本（dir を含む）・stamp |
| M2 済んでいない 止める を支持と同じに | unrefuted・dir・stamp |
| M3 古さでまだ止める（trigger が違えば 2） | stale |
| M4 退けた を支持と同じに | upheld・stamp |
| M5 解けない頭を dir にしない（印） | stamp |
| M6 1 を 2 より先に | unrefuted |
| M7 観点の欠けを見ない | readable・other（既存の gate_is_unknown_on_an_unknown_viewpoint も） |
| M8 済んでいない 止める の行を書かない（前の形） | stamp |
| M9 dir 形を前置きで突き合わせない | upheld |
| M10 合格の周でしか印を書かない | failed・other・stamp（既存の stamp_carries_the_refute_results も） |
| M11 wait を見ず まだ分からない を全部通す（門） | readable・other（既存の gate_is_unknown_on_an_unknown_viewpoint も） |
| M12 理由を問わず wait を書く（印） | other |
| M13 所見の verdict が まだ分からない でも waiting（数え） | other |
| M14 write-set の dir の項目の下の 止める を見ない（門） | dir |
| M15 末尾に / の無い dir の項目を dir と読まない（門） | dir |
| V5 観点の値を 3 値で確かめない（門） | readable |
| V6 文書の一覧に無い doc の 止める を印に載せる（印） | stamp |
| V7 観点の欠けを 止める の判定の後に見る（門） | readable |
| V14 wait の値を確かめない（門） | readable |
| V15 規則 1〜6 の まだ分からない も waiting（数え） | other |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 書き換える 7 本 = src 4 本・`crates/folio/tests/gate.rs`（縮む＝write-set では頭に - を付ける・997 → 991 行）・`crates/folio/tests/stamp.rs`・`tests/fixtures/ceiling/findings/stamp-fail.yaml`。本文を変えない 1 本 = `crates/folio/tests/findings.rs`（`--test findings` の scope）。src の gate.rs は縮まない（+21）ので印を付けない。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）。python と awk の 2 実装が 4 本とも一致した。

| file | base | base の余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/gate.rs` | 565 | 935 | 586（+21） | 914 |
| `crates/folio/src/stamp.rs` | 354 | 1146 | 398（+44） | 1102 |
| `crates/folio/src/findings.rs` | 1096 | 404 | 1120（+24） | 380 |
| `crates/folio/src/main.rs` | 674 | 826 | 674（0） | 826 |

3. **size は S。** src の増分は 1 本あたり最大 +44・正味 +89。余地の最小は 380。足した字は rustfmt に合う（gate.rs の差の数 12 → 9・stamp.rs と findings.rs 0 → 0）。
4. **verify 6 行 = done の 6 の塊。** 1 `--test gate f169_`（(c) の 1〜4・8）・2 `--test stamp f169_`（(c) の 5〜7）・3 `--test gate`（便 142・150 の照らしを含む全部・参考値 15 本）・4 `--test stamp`（凍結 anchor と f99_ を含む全部・16 本）・5 `--test findings`（`--check` と `--refute` の全部・39 本）・6 clippy 0 警告。base では 1 と 2 が 0 件で終了コード 4、3〜5 は緑（20・14・39 本）。`--test` の 3 本はどれも write-set に在り、src に f169_ の単体の歯は無い。

### (g) 門・受付・並行の便・後続

1. **門。** 冒頭のとおり対象外・0。
2. **並行の便。** 設計の PR は `design-intent/` と docs/design だけを書き、本便と重ならない（先に着地）。便 170（docs/d170）は起草役の script で `main.rs` に mod と旗を足すので **main.rs が重なる**（本便は 1 行・別の塊）。仕様の順（169 → 170）で逐次に受け付け、便 170 は本便の着地の後の main で数え直す。ほかの file は重ならない。
3. **後続の便の案（起票は席）。** 引き金の仕掛けの撤去を 1 便で: TRIGGER_* と床の木の trigger / trigger_note（生成区間・ceiling-region.txt・fixture 20 本）・trigger_digest・印の trigger / rest / nodes と stamp_table・歯の独立の実装（trigger_hex・f99_ 3 本・f126_ 1 本・stamp-expected.yaml の nodes）。見当は M〜L。生成区間を書くが、本便の後の門は ceiling.yaml に支持の 止める が無ければ通す。それまでの間、生成区間の trigger_note の「印と門が同じ関数で測る」は偽の字になる（席が台帳に控える）。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d169-draft.md`、差分と script は同じ dir の d169-scripts。1 base-169.sh（base）。2 c169.patch を当てて run-169.sh（床・build の diff -r・verify・nextest・clippy）と real-169.sh（作業ツリーの答え）。3 r169-teeth.patch（apply-169-teeth.py）を base に当てて RED。4 mut-169.py（突然変異・元へ戻す）。5 lines-169.py / .awk（余地）。6 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-169.md#fp`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** 今の本流の FR20 と受入基準 AC18 は古さの門を言うので、設計の PR の着地の前に受け付けない。設計ノート ceiling-gate.md の門の字も設計の PR の領分。改訂 b（b11e70f）の FR20 と照らした。前の差 2 つ（解けない頭は置き場の全体・印は trigger・rest・nodes を後続まで書く）は字になり、まだ分からない の観点の条件（理由が反証の済んでいない 止める だけでないとき 2）は本便の wait の決まりと同じ。残る差は 1 つ: 印の中身の数え上げが観点の行の `wait: 反証` を名指さない（席が docs/struct30 で合わせる）。write-set の dir の項目（改訂 c）は「書き換える file の一覧」の読み方で、字の変更は要らない。
2. **言えないこと。** 門は印の後に書かれた字を見ない（理由の行が言う）。照らすのは file の単位（write-set の dir の項目はその下の file 全部）。場所の file は所見の place の字に依り、解けない頭は dir 全体に広げる。`--stamp` が 2 の周は前の印が残り、門は前の印で答える（理由の周の id で分かる）。凍結された判断の記録（決定 (1)）への支持の 止める は、その記録を書き換える便を止め続ける。前の形の印（行に file が無い）は 2。本流の印 51 周目は `refutes: []` で当たらず、tsuzuri は印をまだ持たない（実測）。
3. **撤退条件。** (1) (e) の 1 の 16 本のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 着地の直前の main で、設計文書を書かない write-set の門の答え・`folio build` の出力・床 4 本のどれかが着地の直前の main の binary と違えば止める。(3) 受付の時点の main で gate.rs の run / read_stamp・findings.rs の Counted / count_findings・stamp.rs の derive が base と違えば (h) で数え直す。(4) 設計の PR が未着地なら受け付けない。

## 2. 範囲

- 入れる: findings.rs の Refute と refutes と waiting・stamp.rs の観点の行の wait・stamp.rs の場所と stop_file・gate.rs の run（write-set の dir の項目を含む）と read_stamp と理由の字・main.rs の 1 行・tests/gate.rs の f169_ 5 本と §1 (e) の 1 の扱い・tests/stamp.rs の f169_ 3 本と字の直し・stamp-fail.yaml の 1 行。
- 入れない: 設計文書と生成区間・TRIGGER_* と床の木・凍結 anchor・印のほかの欄・`--check` と `--refute` の答え・面の名札・旗・admit.sh・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| refute | 止める の行 | findings.rs の Refute と refutes |
| place | 場所の file | stamp.rs の stop_file と行の at・file |
| gate | 門の式 | gate.rs の run・read_stamp・理由の字 |
| teeth | 歯 | f169_ の 8 本・stamp-fail.yaml |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。
- 前提: 設計の PR（docs/struct30）の着地。並行: 便 170 と main.rs が重なる。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す（admit.sh はこの binary で門を撃つ）・便 170 の門を撃ち直す・§1 (g) の 3 を起票する。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fp"
title = "構造の直し C（持ち主の承認 2026-09-27 12:50 JST・判断の記録 ADR-30 決定 (6)）: folio ceiling --gate は印の古さで まだ分からない を返し、一括の後に印を取り直す周を起こしていた。findings.rs の Counted の refutes を 止める の所見の全件の行（id・反証の結果・場所の doc と at）に替え、stamp.rs は印の refutes の行に at と天井の正本の documents で解いた file（判断の記録と設計ノートは at の頭の 1 file・解けなければ dir）を記し、反証の済んでいない 止める の行も書く。gate.rs の run は根の照らしと置き場の確かめと設計文書を書かない便の 通す をそのままに、印が無い・読めない・観点の結果が欠けている・反証の済んでいない 止める の file を書き換えるなら まだ分からない（2）、反証で支持された 止める の file を書き換えるなら 止める（1）、ほかは 通す（0・理由に印の周と 印の後の変更は審査していない）を返し、印の sources・trigger・rest・nodes を読まない（main.rs は stamp_table を渡さない）。TRIGGER_* と印の trigger・rest・nodes と生成区間は残す（撤去は後続）。印は周の 3 値に依らず書く（不合格の周の後も支持の 止める が載る）。まだ分からない の観点は、理由が反証の済んでいない 止める だけのとき（count_viewpoint の waiting・印の観点の行の wait: 反証）に限って file ごとの判定に任せ、ほかの理由なら 2。write-set の dir の項目はその下の file を全部含むと読む。古さで答えを縛った既存の歯 11 本を外し f169_ の 8 本に置き換える。門の対象外。設計の PR（docs/struct30）の着地の後に受け付ける。base = main 62111b1・受付の時点の main で数え直す"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/gate.rs", "crates/folio/src/stamp.rs", "crates/folio/src/findings.rs", "crates/folio/src/main.rs", "-crates/folio/tests/gate.rs", "crates/folio/tests/stamp.rs", "crates/folio/tests/findings.rs", "tests/fixtures/ceiling/findings/stamp-fail.yaml"]
verify = ["cargo nextest run -p folio --test gate f169_", "cargo nextest run -p folio --test stamp f169_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --test findings", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/gate.rs の f169_ の 5 本（支持の 止める が無い古い印と wait: 反証 の まだ分からない観点の印は rc 0 で 印の周 gate-case（判定 …）に、と 印の後の変更は審査していない を出す / 支持の 止める の file〔srs.yaml・~./ 付きの adr/ADR-1.yaml・dir 形 design-note/ の下〕は rc 1、ほかの設計文書〔adr/ADR-2.yaml・退けた の constitution.yaml・rules.yaml〕は rc 0、実装だけは rc 0 / 反証の済んでいない 止める の file は rc 2 で同じ file の支持より先 / 印が無い・観点の行が欠けた〔支持の 止める の file を書き換えても〕・観点の値が 3 値でない・wait の無いか 反証 でない まだ分からない観点・行に file が無い印は rc 2 / write-set の dir の項目は、その下の支持の 止める の file で rc 1〔末尾に / が無くても〕・その dir 形の未反証の 止める で rc 2）が緑、tests/stamp.rs の f169_ の 3 本（印の refutes が 支持・refute なし・退けた の 3 行を at と解いた file で書き、同じ印の門が rc 1・2・0・0 / 合格の周の後に不合格になった周でも印を書き、verdict 不合格と支持の 止める の行が載り、門が rc 1 / 反証待ちのほかの理由〔AI が判定できない・再判定待ち・所見の欄の違反〕の まだ分からない観点には wait を書かず、門が rc 2 / 文書の一覧に無い doc の 止める の周は --stamp が rc 2 で印を書かない）が緑、tests/gate.rs・tests/stamp.rs・tests/findings.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0、folio build の出力は着地の直前の main と file 数も byte も同じで、設計文書を書かない write-set の門の答えは着地の直前の main の binary と同じである"
<!-- contracts:end -->
