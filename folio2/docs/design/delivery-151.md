# 設計: 便 151 — 周の引き金から判断の記録の状態の欄と発効の絞りを外す（引き金の便・ADR-26 決定 (3)）

- 要件: FR20（天井の門・要件書 第 1.48 版）。FR20 の規範文と受入基準 AC18 は変えず、門と印が同じ関数で測る引き金の要約値（周を回し直す要否を決める規範の欄の写しの要約値）の中身だけを変える。AC18 は器の要件面に無い id なので req に書かない。
- 条: P-5.1（引き金の欄は型付きの定数に置く）/ P-6.2（天井の正本の生成区間は生成器 `folio schema --write` の出力で、手で直さない）/ P-10.1〜10.3（凍結 anchor は生成器から独立に組み直して測り直す）/ P-15.2（門と印は同じ関数 trigger_digest を呼ぶまま）。
- 出所: 判断の記録 ADR-26 決定 (3)（ADR-18 決定 (1) ② の改訂・2026-09-27 00:40 JST 発効・台帳 f2-648 notes）。要旨の正本は `docs/design/ceiling-rules-triage.md` の §引き金の便の契約の要旨。
- 置き場: 審査の材料は行 `ex` が指す §1 だけ。write-set は 30 本（書き換える 28 本 + 本文を変えない verify の scope 2 本）で、新しい file と新しい dir は無い。src の 2 本は行が減るが、余地が足りるので `-`（縮む面の宣言）は付けない。
- 門: 対象（write-set に天井の正本 design-intent/ceiling.yaml が在る）。本流 949af4b（51 周目の印）の一番上で本流の binary に write-set 30 本を渡すと **0（通す・印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る）**。本便の binary では同じ場所で 2 になる（着地の後の話・§5）。
- 前の便: **base = main 949af4b（一括 28 の着地の後）。数はすべて base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。枝の親は 6c0977b だが、一括 28 は crates/ と tests/ に触れず数は同じ。
- 改訂 b（2026-09-27・独立の検証 d151-verify.md〔支持・blocking 0〕の不一致 4 つ・席の依頼）: base と門の答えを 949af4b に、(g) の 3 を merge 済みの字に、縮む file の字を 1 つに揃え、歯 2 に adr/schema.yaml の 1 通りを足した。

## 1. 設計

### (a) いま起きていること（base の実測・参考値）

1. **絞り。** `crates/folio/src/gate.rs` の関数 effective_adrs は、判断の記録の置き場の直下の .yaml のうち、状態が発効の値域（accepted・retired）で承認欄が表のものだけを拾い、`crates/folio/src/ceiling.rs` の定数 TRIGGER_ADR_FIELDS（8 欄・status を含む）を写す。値域の定数 TRIGGER_ADR_STATUS は床の木 FLOOR の trigger.adr.status の葉として、天井の正本の生成区間にも出る。
2. **門の答え。** 見本の置き場（`crates/folio/tests/gate.rs` の Repo）に合格で新しい印（節点の表つき）を置き、1 か所だけ変えて撃った答え。

| 変えたもの | base | 本便の後 |
| --- | --- | --- |
| 提案中の ADR-2 の決定の字 | 0 通す（引き金の外の変更・節点 1 個） | 2 印が古い（引き金の要約値が違う） |
| 提案中の ADR-3 を足す | 0 通す（同上） | 2 同上 |
| ADR-2 を発効へ・承認欄を足す | 2 印が古い（引き金の要約値が違う） | 0 通す（引き金の外の変更・節点 1 個） |
| adr/retired/ に記録を置く | 0 通す（正本の要約値が同じ） | 同じ |
| adr/schema.yaml を置く | 0 通す（引き金の外の変更・節点 0 個） | 同じ |

3. **読み手の全数（`git grep`）。** TRIGGER_ADR_STATUS を読むのは ceiling.rs（FLOOR と単体の歯 1 本）と gate.rs の effective_adrs だけ。trigger_digest を呼ぶのは門と印（stamp.rs）だけで、面の名札（face.rs）と `folio derive` は引き金を読まない。生成区間の写しは design-intent/ceiling.yaml・tests/fixtures の ceiling.yaml 20 本・凍結 anchor ceiling-region.txt に在る。
4. **base の歯。** workspace の nextest 952 / 952・clippy 0 警告・床 4 本 rc 0・`folio build --write` 33 file。`git grep -n f151_ -- crates` は 0 件、行 id `ex` は docs/design に 0 件。

### (b) 直す先

1. **ceiling.rs。** TRIGGER_ADR_STATUS を消し、TRIGGER_ADR_FIELDS を 7 欄（id・decision・retreat・amends・revises・supersedes・superseded_by）にする。FLOOR の trigger.adr は fields の葉だけにする。trigger_note の字は「adr は状態を問わず判断の記録ごとに fields」と「ADR-18 決定 (1)（ADR-20・ADR-26 が改訂）」に置き換える。
2. **gate.rs。** effective_adrs を adr_records に改名し（呼び手は trigger_digest の 1 か所）、状態と承認欄の絞りを外す。置き場の直下の .yaml を schema.yaml を除いて名の byte 順に全部読み、下の dir（retired/ など）を読まないことと symlink の断りは今のまま。
3. **生成区間。** design-intent/ceiling.yaml の生成区間は `folio schema --dir design-intent --write` の出力で書く（44 行 4619 byte・base は 46 行 4655 byte）。人が書く節は 1 byte も変えない。
4. **変えないもの。** 門の判定の順と 3 値・印の欄・正本の要約値・ほかの文書の引き金の欄・命令の口・判断の記録の床（発効の値域は floor_adr.rs の EFFECTIVE_STATUS に残る）。

### (c) 歯（`crates/folio/tests/gate.rs`・既存の口 Repo・put_stamp_with_nodes・edit・gate だけ）

1. **f151_a_proposed_adr_moves_the_trigger。** 合格で新しい印を置き、先に要件書の write-set で 0 を確かめる（印の引き金が今の関数と合うことの対照）。次に、提案中の ADR-2 の決定の字を変える形と提案中の ADR-3 を足す形の 2 通りで、終了コード 2 と 印が古い・引き金の要約値が違う を見る。**base では 0 通す＝RED。**
2. **f151_the_adr_status_and_approval_do_not_move_the_trigger。** ADR-2 の状態を提案中から発効へ変えて承認欄を足すと 0 と 引き金の外の変更、adr/retired/ に提案中の記録を置くと 0 と 正本の要約値が同じ、欄の決まり adr/schema.yaml を置くと 0 と 引き金の外の変更 を見る。**base では 1 通り目が 2（印が古い）＝RED。** 2・3 通り目は base でも緑の見張り（下の dir と schema.yaml を読まない）。
3. **受入基準 AC18 の場合の見張り。** 門の歯の既存 18 本は緑のまま残す（verify の 2 行目）。ただし (e) の 1 の 2 通りは向きが逆になるので外す。

fixture の file は足さない（記録は歯の中で一時 dir に書く）。

### (d) 採らなかった形

1. **状態の欄だけ外し、発効の絞りは残す。** 提案中の記録の字が要約値に入らず、ADR-26 決定 (3) の「提案中の判断の記録の字も引き金の要約値に入る」を満たさない。
2. **絞りだけ外し、状態の欄は fields に残す。** 発効の記帳で要約値が動き続け、決定 (3) の「発効の記帳だけでは引き金の要約値が動かない」を満たさない（(e) の 5 の M3 の形）。

### (e) 既存の歯・凍結 anchor・突然変異

1. **置き換える既存の歯。** tests/gate.rs の f126_the_gate_is_stale_on_each_normative_edit から ADR-2 を発効にする 1 通り（5 → 4）、f126_the_gate_passes_edits_outside_the_trigger から ADR-2 の決定の字を変える 1 通り（9 → 8）を外し、(c) の 2 本が向きを逆にして持つ。単体の歯 f126_trigger_lists_name_real_documents_and_sections からは消える定数を読む 1 行を外し、tests/schema_docs.rs の f129 の歯の trigger_note の字を ADR-26 入りにする。
2. **凍結 anchor の測り直し（P-10.1〜10.3）。**
   1. ceiling-region.txt は base の字に 3 か所の置き換えを当てて手で組み（生成器の出力を写さない）、生成器の出力と byte で一致した。tests/schema_docs.rs の CEILING_REGION_* は wc と sha256sum で測り直した。
   2. node-digest-anchor.txt は独立の実装 node-digest.py（土台 floor_base）の出力に替え、残差の 2 行だけが動いた（36 byte 減）。tests/graph.rs の F99_ANCHOR_SHA256 も sha256sum で測り直した。
   3. tests/gate.rs の引き金の要約値の独立の実装 trigger_hex（folio の code を呼ばない）は、anchor から status の葉が消えるので状態の絞りを外す。
   4. tests/fixtures の ceiling.yaml 20 本は生成区間だけを anchor の字に置き換えた（前の区間が base の anchor と一致することを script が確かめた）。
   5. 印の写し stamp-pass / fail / unknown.yaml の trigger は仮の値で、歯が独立の実装で測って差し込むので変えない。束の凍結 anchor も動かない。
3. **(e) の 1 のほかに落ちる既存の歯は 0 本（起草役の実測）。** 本便を当てた写しで workspace の nextest 954 / 954（base 952 + 2）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 33 file（base と diff -r で全 file 一致）。src と anchor だけを当てた段では 38 本が落ち、fixture の写し 20 本で 31 本、(e) の 1・2 で残りの 7 本が緑に戻った。
4. **RED。** 歯だけを base に当てると f151_ の 2 本とも落ちる（(a) の 2 の表の base の列の字）。
5. **突然変異。** 本便を当てた写しの src を 1 通りずつ変え、tests/gate.rs 20 本・単体 ceiling::tests 8 本・tests/stamp.rs 13 本を撃った。

| 変異 | f151_ 提案中 | f151_ 状態 | ほかに落ちる歯 |
| --- | --- | --- | --- |
| M1 発効の絞りを戻す | 落ちる | 落ちる | 門の既存 9 本（独立の実装と要約値が合わない） |
| M2 承認欄を持つ記録だけに絞る | 落ちる | 落ちる | 同上 |
| M3 状態の欄を写しに足す | 落ちる | 落ちる | 同上 |
| M4 下の dir（retired/）も読む | 緑 | 落ちる | なし |
| M5 床の木に status の葉を残す | 緑 | 緑 | 単体の anchor の byte 一致 |
| M6 trigger_note を base の字に戻す | 緑 | 緑 | 同上 |
| M7 schema.yaml を除かない | 緑 | 落ちる | なし |

### (f) 大きさ・verify と done の対応

1. **余地（CapHeadroom）。** 各行を ceil(字数 / 120) で数えて足した（空行は 1）。python と awk の 2 実装で一致した。

| file | base の正規化行数 | 余地 | 本便の後 |
| --- | ---: | ---: | ---: |
| crates/folio/src/ceiling.rs | 742 | 758 | 736 |
| crates/folio/src/gate.rs | 572 | 928 | 565 |

   src は行が減る（`-` は付けない・置き場の行）。src の外は tests/gate.rs 927 → 979・tests/schema_docs.rs 1189 → 1190・tests/graph.rs 613 → 614。rustfmt の差の数は増えない（tests/gate.rs は 21 → 20）。
2. **size は M。** src は 2 本で縮むが、write-set が凍結 anchor 2 本・fixture の写し 20 本・生成区間 1 本に及ぶ。
3. **write-set の印。** 30 本とも印なし。本文を変えない 2 本は crates/folio/tests/stamp.rs と crates/folio/tests/ceiling.rs（verify の `--test` で名指す）。
4. **verify は 8 行**で、done の塊と 1 対 1。順に (c) の 2 本・門の歯の全部 20 本・印の歯 13 本・単体 ceiling::tests 8 本（anchor の byte 一致）・schema_docs 29 本（生成区間の数）・ceiling 23 本（写しの生成区間）・graph 16 本（節点の anchor）・clippy。base では 1 行目が 0 件で rc 4、ほかは緑（門は 18 本）。fixture を読むほかの歯は共通の検証（workspace の nextest）が見る。

### (g) 門と受付

1. **門。** 本流 949af4b の一番上で write-set 30 本を渡すと、本流の target/debug/folio と base の写しの binary は 0（通す・印が 4 観点とも合格・引き金の要約値が同じ・印の後に引き金の外の変更が在る）。本便の binary は同じ場所で 2（印が古い）で、着地で本流の印が古くなることに当たる（§5）。生成区間は引き金の欄（重さ・文書の一覧・観点の行）の外なので、本便の ceiling.yaml の変化は引き金を動かさない。
2. **受付の先撃ち。** 契約に起因する断りは 0（(h) の 3）。
3. **並行。** base の時点で gate.rs・ceiling.rs と歯の file を書き換える便の契約は無い。一括 28 は merge 済み（949af4b・design-intent/adr/ADR-1.yaml・adr/schema.yaml・preview/ceiling-stamp.yaml の 3 file）で、write-set と重ならない。

### (h) 数え直す手順

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d151-draft.md`、script は同じ dir の d151-scripts（repo には入れない）。

1. 模擬は base の clone で build-sim-151.sh（実装 → `folio schema --write` → fixture の区間 → anchor の測り直し → 歯）。差分は c151.patch、検査は verify-151.sh と workspace の nextest・床 4 本・build の diff -r。
2. RED は r151-teeth.patch、5 通りの答えは probe-151.py、突然変異は mut-151.py、余地は lines-151.py と lines-151.awk。
3. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-151.md#ex`。

### (i) 言えないこと・撤退条件

1. **言えないこと。** 本便の後は、提案中の判断の記録を置き場に置くだけで本流の印が古くなる（ADR-26 決定 (3) の帰結・行 D-16 によりそれだけでは周を起こさない）。置き場の直下の .yaml は判断の記録でなくても引き金に入る（読めない .yaml が まだ分からない になるのは base と同じ）。
2. **撤退条件。** (1) (e) の 1 の置き換えのほかに既存の歯が 1 本でも落ちたら、歯も fixture も直さず止めて席へ返す。(2) 生成区間の外の byte・`folio build` の出力・床 4 本の結果のどれかが変わったら止めて席へ返す。(3) 受付の時点の main で effective_adrs か生成区間の写しの本数が base と違えば、(h) の手順で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1〜3、(e) の 1・2 の置き換えと測り直し、(c) の歯 2 本。
- 入れない: 生成区間の外の設計文書（判断の記録・要件書・規則の表・語彙・憲法を含む）・admit.sh と席の手順・新しい fixture の file と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| fields | 引き金の欄 | ceiling.rs の TRIGGER_ADR_FIELDS と FLOOR の trigger.adr・trigger_note |
| records | 拾い方 | gate.rs の adr_records |
| anchors | 凍結 anchor | ceiling-region.txt・node-digest-anchor.txt・定数・fixture の区間 |
| teeth | 歯 | tests/gate.rs の f151_ 2 本と独立の実装 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。
- 前提の着地は無い（base = main 949af4b・本流の印は 51 周目で門は 通す）。
- 並行の便は無い。
- 本便の着地の後に席が見ること: 本流の target/debug/folio を組み直す（admit.sh はこの binary で門を撃つ）。引き金の要約値の関数が変わるので本流の印は古くなり、以後の設計文書の便は次の周（止める の直しか節目）まで門で まだ分からない になる（行 D-16・それだけでは周を起こさない）。ADR-26 の注の【引き金の便】の段は消える定数 TRIGGER_ADR_STATUS を名指し、要件書 FR20 と規則の表の行 D-16 の注には「引き金の便の着地まで」の字が残る。どれも次の節目の材料として台帳に積む。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ex"
title = "引き金の便（判断の記録 ADR-26 決定 (3)・ADR-18 決定 (1) の改訂）: 周の引き金の要約値は crates/folio/src/gate.rs の effective_adrs が発効して承認欄を持つ判断の記録だけを拾い、crates/folio/src/ceiling.rs の TRIGGER_ADR_FIELDS（status を含む 8 欄）を写すので、発効の記帳だけで動き、提案中の記録の字は入らない。TRIGGER_ADR_STATUS と床の木の trigger.adr.status を消して TRIGGER_ADR_FIELDS を 7 欄に、effective_adrs を adr_records に改名して置き場の直下の .yaml（schema.yaml を除く）を状態を問わず全部写し、trigger_note を置き換え、天井の正本の生成区間を folio schema --write で書き直す。凍結 anchor 2 本と測った定数・fixture の ceiling.yaml 20 本の生成区間を測り直し、向きが逆になる既存の歯 2 通りを f151_ の 2 本へ移す。門は対象で本流 949af4b（51 周目の印）では 通す。base = main 949af4b・受付の時点の main で数え直す"
req = ["FR20"]
section = "1"
write-set = ["crates/folio/src/ceiling.rs", "crates/folio/src/gate.rs", "crates/folio/tests/ceiling.rs", "crates/folio/tests/gate.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/schema_docs.rs", "crates/folio/tests/stamp.rs", "design-intent/ceiling.yaml", "tests/fixtures/adr/effective-no-approval/ceiling.yaml", "tests/fixtures/adr/schema-drift/ceiling.yaml", "tests/fixtures/adr/two-adopted/ceiling.yaml", "tests/fixtures/anchor/no-anchor/ceiling.yaml", "tests/fixtures/anchor/root-digest-drift/ceiling.yaml", "tests/fixtures/ceiling/bundle/source/ceiling.yaml", "tests/fixtures/check/dup-key/ceiling.yaml", "tests/fixtures/check/empty-field/ceiling.yaml", "tests/fixtures/check/missing-file/ceiling.yaml", "tests/fixtures/check/unknown-section/ceiling.yaml", "tests/fixtures/face/ceiling.yaml", "tests/fixtures/floor_base/design-intent/ceiling.yaml", "tests/fixtures/link/adr-id-missing/ceiling.yaml", "tests/fixtures/link/amended-by-orphan/ceiling.yaml", "tests/fixtures/link/retreat-kind-drift/ceiling.yaml", "tests/fixtures/refs/bad-counts/ceiling.yaml", "tests/fixtures/refs/dangling-id/ceiling.yaml", "tests/fixtures/refs/orphan-rule/ceiling.yaml", "tests/fixtures/schema/ceiling-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "tests/fixtures/vocab/exemptions/ceiling.yaml", "tests/fixtures/vocab/unknown-word/ceiling.yaml"]
verify = ["cargo nextest run -p folio --test gate f151_", "cargo nextest run -p folio --test gate", "cargo nextest run -p folio --test stamp", "cargo nextest run -p folio --bin folio ceiling::tests", "cargo nextest run -p folio --test schema_docs", "cargo nextest run -p folio --test ceiling", "cargo nextest run -p folio --test graph", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "tests/gate.rs の f151_ の 2 本（提案中の ADR-2 の決定の字と提案中の ADR-3 の追加が rc 2 で 印が古い・引き金の要約値が違う / ADR-2 の発効と承認欄が rc 0 で 引き金の外の変更、adr/retired/ の記録が rc 0 で 正本の要約値が同じ、adr/schema.yaml が rc 0 で 引き金の外の変更）が緑、tests/gate.rs の全部（AC18 の場合を含む）・tests/stamp.rs の全部・単体の ceiling::tests（床の木と凍結 anchor の byte 一致）・tests/schema_docs.rs（生成区間 44 行 4619 byte）・tests/ceiling.rs（ceiling.yaml の写しが全部 anchor と同じ生成区間）・tests/graph.rs（節点の anchor）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
