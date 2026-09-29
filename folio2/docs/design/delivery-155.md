# 設計: 便 155 — 承認欄の空いた憲法を始まりの凍結で凍結しない（群 A・台帳 f2-648.193）

- 要件: FR24（始まりの凍結・書いた後はふだんの床が全部の検査を回す）。規範文・確かめ方・受入基準 AC21・AC23 は変えない。
- 条: P-15.2（編集の時点で止める判定の式を、事後の検査が同じ関数で確かめる）/ P-12.2（承認は逐語と日付を添えて承認欄に記帳する）/ P-4.1（検査が落とす凍結を黙って通さない）/ N-3.1（旗を足さない）。
- 出所: 外の置き場の実測（`.local/share/folio2/handoff-2026-09-24/folio2-v3-entry-notes.md` §4 の 1・§5 の 5）と台帳 **f2-648.193**。持ち主の直命 2026-09-26「folio2 の道具としての完成を目指す」の群 A。
- 置き場: 審査の材料は行 `fb` が指す §1 だけ。write-set は 2 本で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d155 の一番上で、base の binary と本便の binary に write-set 2 本を渡すと、どちらも **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main d6d84fa（便 153 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 並行の便: 便 154 と write-set が重ならない（§1 (g)）。改訂 b = 独立の検証の B-1・N-1 を採り、歯を 3 本にした。

## 1. 設計

### (a) いま起きていること（base d6d84fa の実測・参考値）

1. **骨格のままの凍結の答え。** 一時 dir で git の init → `folio init --dir design-intent` → commit の後に撃った。骨格の憲法は名（meta.id）が 未記入 で、承認欄（meta.approval）の承認者・日付・裁定 id・逐語が 未記入。

| 命令 | 答え |
| --- | --- |
| check | 2（違反 0・まだ分からない 2 = 凍結の基準の不在だけ） |
| check --freeze-start | 1。違反は「憲法の名 未記入 が列の根の表に無い」の 1 件だけで、組んだ木の digest（要約値）27303a5d… の全桁を「表に行を足す値」として出す。承認欄には何も言わない |
| check --freeze-anchor | 1（同じ違反 1 件）と まだ分からない 1（id の一覧が無い） |
| check --freeze-ids | 2（凍結 anchor が 0 本の まだ分からない） |

   名を付けても digest は同じ（名は写しの外）。承認欄を埋めると digest が変わる（4c572170…）。
2. **罠。** 写しの列の根の表に上の 2 つの digest の行を足した**測りだけの binary**（契約に入れない・行を足すのは folio2 の便で持ち主の裁定を要する・行 D-11）で撃つと、承認欄が空のままの --freeze-start が **0 で 3 file を書き**、commit の後の check が **1「[anchor] constitution-v1.0.yaml: approvals[0].ruling に台帳 id が無い」**になる。09-24 の spike と同じで、まだ起きる。今の binary では表に無い名の断りが先に立つが、その断りが出す digest は承認欄の空いた木の値で、その行を足せば罠が弾ける。
3. **凍結の後の床。** `crates/folio/src/anchor.rs` の check_approvals（anchor の承認一覧の各項に、承認者・裁定 id・逐語が空でないことと、裁定 id に台帳 id の形が在ることを見る）が落とす。凍結の前（素の check と --freeze-start）は憲法 meta.approval を見ない。--freeze-start が書く承認一覧は meta.approval の写し 1 項（`crates/folio/src/freeze.rs` の組む関数 build が、欄 adr を空にし、床の定数 approval.required の 5 欄を写す）。
4. **ほかの 2 つの旗（同じ測りだけの binary で測った）。**
   1. --freeze-ids は承認を写さない。承認欄を埋めて始まりの凍結をした後に要件書の版を上げて撃つと 0 で、commit の後の check も 0。罠は無い。
   2. --freeze-anchor の次の版は、写す承認が発効した判断の記録の承認欄で、判断の記録の床（`adr.rs` の check_approval・日付の形・承認者の値域・台帳 id の形）が同じ check の中で先に落とす（ADR-1 を accepted にし承認欄を 未記入 で足すと違反 3・凍結しない）。罠は無い。
   3. --freeze-anchor の最初の版の道（列が無く、承認一覧が meta.approval の写しになる）は凍結に届かない。id の一覧が無ければ まだ分からない、commit 済みなら「列を始め直す凍結は認めない」、未追跡なら「版管理に追跡されていない」の違反で断る。罠は無い。
5. **既存の --freeze-start の断りの形は 2 つ。** (ア) anchors/ に列か一覧が在る → 何も数えずに 1 と 1 行。(イ) ほかの検査に違反が在る → 違反の行と「凍結しない — 違反 n 件…」で 1、まだ分からない が在れば 2。承認欄の空きは測れる事実で、凍結の後の床も [anchor] の違反で落とすので、(イ) の違反が合う（まだ分からない は測れないときの形・P-4.2）。
6. **日付。** 日付だけ 未記入 の承認欄は、凍結も凍結の後の床も通る（check_approvals は日付を見ない）。前後で式が揃っているので罠ではない。
7. **base の歯。** workspace の nextest 959 / 959・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file。`--test freeze_root` は 8 本。`f155_` と行 id `fb` は 0 件。

### (b) 直す先 — `crates/folio/src/anchor.rs` の check_anchor の (i) 節

1. flag が --freeze-start のとき、始まりの凍結が書く承認一覧（(a) の 3 と同じ取り方の 1 項）を、凍結の後の床と同じ関数 check_approvals で確かめる。名札は「constitution-<版>.yaml（凍結で書く承認一覧・憲法 meta.approval の写し）」で、違反の字は凍結の後の床と同じ（例「approvals[0].ruling に台帳 id が無い」）。違反は既存の report に積み、freeze.rs の既存の判定（違反 0 で まだ分からない も無いときだけ書く）が凍結を止める。
2. 骨格のままの --freeze-start は、承認欄の違反と表に無い名の違反の 2 件で 1 を返す。表に行を足した後でも、承認欄が空なら凍結しない。
3. **変えないもの。** freeze.rs・check_approvals の式・素の check・--freeze-anchor と --freeze-ids・列の根の表・命令の旗・folio2 自身の床と面。

### (c) 歯（関数名 f155_・`crates/folio/tests/freeze_root.rs`・binary 経由）

1. **f155_skeleton_freeze_start_refuses_the_empty_approval。** 新しい口 Work::init（一時の根の直下の design-intent に init をして git の init と commit・外の置き場と同じ形）の骨格で、check が 2 で違反 0、--freeze-ids と --emit-amends も 2 で違反 0 のまま（前提を変えない・FR24）。--freeze-start が 1 で、違反はちょうど 2 件（承認欄の行が歯の側の手書きの期待と等しいもの 1 件・「表に無い」1 件）、標準エラーに「凍結しない」、anchors/ を作らない。**base では違反が 1 件＝RED。**
2. **f155_freeze_start_checks_the_approval_and_freeze_anchor_is_unchanged。** 床の土台の写し（tests/fixtures/floor_base）の憲法の裁定 id を 未記入 にする。(ア) 列の無い置き場の --freeze-start は 1 で承認欄の行ちょうど 1 件、anchors/ は変わらない。(イ) id の一覧だけが在る置き場の --freeze-anchor は 1 で承認欄の行 0 件、anchors/ は変わらない（前提を変えない・FR24）。**base では (ア) の行が 0 件＝RED。**
3. **f155_freeze_start_approval_is_the_floor_row。** 凍結の前の確かめが凍結の後の床と同じ関数に同じ項を渡すことを、裁定 id と別の 2 形で見る。床の土台の写しの列の無い置き場で、(ア) 承認者を空の字にすると --freeze-start の承認欄の行はちょうど「approvals[0] に who / ruling / verbatim が無い」の 1 件。(イ) meta.approval に実在する判断の記録の欄 adr（ADR-1）を足しても、書く項は欄 adr を空にするので承認欄の行は 0 件（凍結の後の床もその欄を見ない）。どちらも 1 で anchors/ を作らない。**base では (ア) の行が 0 件＝RED。**

fixture は足さない。歯の file には口 Work::init・blank_ruling と定数 APPROVAL_AT（違反の頭の手書き）を足す。

### (d) 採らなかった形

1. **freeze.rs の組む関数 build の中で確かめる。** 便 154 の write-set（freeze.rs）と重なる。--freeze-anchor と共有になるが、その最初の版の道は凍結に届かず（(a) の 4 の 3）、FR24 は今ある 2 つの凍結の命令の前提を変えないと言う。
2. **判断の記録の床の check_approval（adr.rs）で meta.approval を見る。** 凍結の後の床の式と違う（P-15.2）。folio2 自身の承認欄は承認者「持ち主（shuu5）」と散文の対話面で、その床の値域（承認者 3 つ・対話面 R-8）の外になり、folio2 の始まりの凍結まで断る（定数から読んだ・写しでは撃っていない）。
3. **素の check でも承認欄を見る。** 骨格のままの check が違反 1 になり、FR22 の骨格の歯と群 A の通しの歯 f152_ が落ちる（(e) の 3 の M8）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 0 本。** 本便を当てた写しで workspace の nextest 962 / 962・clippy 0 警告・床 4 本 rc 0・`folio build --write` は base と diff -r 一致・`--test freeze_root` 11 / 11。
2. **RED。** 歯だけを base に当てると f155_ の 3 本が落ち、ほかの 8 本は緑。
3. **突然変異（写しの anchor.rs を 1 通りずつ変え、`--test freeze_root` を撃つ）。**

| 変異 | 落ちる歯 |
| --- | --- |
| M1 始まりの凍結でも確かめない | 骨格・土台・床の項 |
| M2 --freeze-anchor で確かめ、--freeze-start では確かめない | 骨格・土台・床の項 |
| M3 --freeze-anchor でも確かめる | 土台 |
| M4 写しの項から裁定 id を落とす | 骨格・土台・床の項・f121 の始まりの凍結の歯 2 本 |
| M5 違反の頭の括弧（出所の名指し）を落とす | 骨格・土台・床の項 |
| M6 写しの項でなく meta.approval をそのまま渡す | 床の項 |
| M7 承認一覧を渡さない | 骨格・土台・床の項・f121 の始まりの凍結の歯 2 本 |
| M8 旗の外へ出す（素の check でも見る・`--test init` も撃つ） | 骨格・土台・f125_init_writes_the_skeleton_and_the_floor_passes・f152_ の通しの歯 |
| M11 check_approvals でなく自前の判定（裁定 id の形だけ・同じ字） | 床の項 |
| M13 --freeze-ids と --emit-amends にも掛ける（`--test init` も撃つ） | 骨格 |

   表の 骨格・土台・床の項 は (c) の 1・2・3。生き残る変異は無い。M6 は meta.approval に欄 adr を持つ置き場で凍結の前だけ判断の記録との突き合わせの違反を足し、M11 は承認欄が無い形と承認者が空の形で罠を開け直す（どちらも P-15.2 に反する食い違い）。

### (f) 大きさ・verify と done の対応

1. **write-set。** 2 本とも印なし（書き換えるだけ）: `crates/folio/src/anchor.rs`・`crates/folio/tests/freeze_root.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/anchor.rs` | 1033 | 467 | 1046（+13） | 454 |

   src の外は `tests/freeze_root.rs` 511 → 638。足した字は rustfmt に合う（rustfmt --check の差の数は base と同じ）。
3. **size は S。** src は 1 本で +13（確かめる 1 か所）。
4. **verify は 3 行**で、done の 3 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test freeze_root f155_` = (c) の 1〜3。
   2. `cargo nextest run -p folio --test freeze_root` = 列の根と始まりの凍結の歯の全部（AC20・AC21・AC23 の歯を含む・参考値 11 本）。
   3. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で終了コード 4、2 は 8 本で緑、3 は 0 警告。`--bin` の行は無い。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0。便 153 は着地済み（base に入る）。並行の便 154 の write-set は adr.rs・face 系 7 本・freeze.rs・ids.rs・sheet.rs・stamp.rs・tests の 3 本と fixture で、本便の 2 本と重ならない。本便は freeze.rs を書かないので、便 154 の freeze.rs の頭の注の直しとも当たらない。便 156・157 は起草中で、重なれば席が順を決める。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d155-draft.md`、script は同じ dir の d155-scripts と改訂 b の d155b-scripts（repo の外）。

1. 再現: repro-155.sh（(a) の 1）。build-probe-155.sh（測りだけの binary）と trap-155.sh（(a) の 2・4・6）。
2. 模擬: build-sim-155b.sh が main の clone に apply-155-teeth.py と apply-155b-teeth.py（歯）と apply-155.py（実装）を当てて組む（c155.patch・r155-teeth.patch）。suite-155b.sh（nextest・clippy）・floor-155b.sh（床 4 本と build の diff -r）・verify-155b.sh。
3. RED: red-155b.sh（歯だけを base に当てて戻す）。
4. 突然変異と余地: mut-155b.py（撃った後は元に戻して組み直す）・lines-155.py と lines-155.awk。
5. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-155.md#fb`。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR24 の規範文（2 つの基準の不在を除くすべての検査が違反 0 のときだけ書く・書いた後はふだんの床が全部の検査を回す）と確かめ方は変えない。本便の後は、承認欄の空いた憲法でも「書いた後の床が違反 0」が崩れない。今ある 2 つの凍結の命令の前提は変えない（(c) の 1 と 2 の (イ)）。
2. **運ばないもの。** 列の根の表の行（.233）・freeze.rs と名の焼き込み（.191・便 154）・設計文書の正本・台帳への記帳・外部 crate。次の 2 つは別の便に控える。
   1. 承認欄の形を強める（未記入 を空と見る・日付の形）のは別の S 便（床の check_approvals を変え、凍結の前も後も同時に変わる・(a) の 6）。
   2. 承認一覧の組む口が freeze.rs の build と anchor.rs の確かめの 2 か所に在るのを 1 つに寄せるのは便 154 の着地の後（P-6.3・当面は f121 の 2 本が組む口の変化を落とす＝M4・M7）。
3. **撤退条件。** (1) 既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら止めて席へ返す。(3) 受付の時点の main で anchor.rs の (i) 節か check_approvals、tests/freeze_root.rs の口 Work が base と違えば、(h) で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1・2、(c) の歯 3 本と口 2 つ・定数 1 つ。
- 入れない: freeze.rs・check_approvals の式・素の check・ほかの 2 つの凍結の旗・列の根の表・命令の旗・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| guard | 始まりの凍結の承認欄 | anchor.rs の check_anchor の (i) 節で、書く承認一覧を check_approvals に渡す |
| teeth | 歯 | tests/freeze_root.rs の f155_ 3 本・Work::init・blank_ruling・APPROVAL_AT |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（base = main d6d84fa）。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。台帳 .193 を閉じる。日付の形（§1 (i) の 2）を控えるかを決める。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fb"
title = "群 A（台帳 f2-648.193）: folio check --freeze-start は憲法の承認欄（meta.approval）が 未記入 のままでも凍結でき、commit の後の床が approvals[0].ruling に台帳 id が無い の違反 1 で落ちる（列の根の表に行を足すと弾ける罠・今の断りが出す digest は承認欄の空いた木の値）。crates/folio/src/anchor.rs の check_anchor の (i) 節で、flag が --freeze-start のとき、始まりの凍結が書く承認一覧（欄 adr を空・approval.required の 5 欄を meta.approval から写した 1 項）を凍結の後の床と同じ関数 check_approvals で確かめ、違反で凍結を止める。freeze.rs・check_approvals の式・素の check・--freeze-anchor と --freeze-ids・列の根の表・命令の旗・設計文書の正本は変えない。歯は tests/freeze_root.rs の f155_ 3 本（骨格のままの --freeze-start が承認欄と表に無い名の違反 2 件で断り --freeze-ids と --emit-amends は変わらない / 床の土台の写しで裁定 id を空にすると --freeze-start だけが承認欄の違反 1 件を足し --freeze-anchor は足さない / 承認者が空の字なら who・ruling・verbatim の違反 1 件・meta.approval に欄 adr が在っても承認欄の違反 0 件）。門の対象外。base = main d6d84fa・受付の時点の main で数え直す"
req = ["FR24"]
section = "1"
write-set = ["crates/folio/src/anchor.rs", "crates/folio/tests/freeze_root.rs"]
verify = ["cargo nextest run -p folio --test freeze_root f155_", "cargo nextest run -p folio --test freeze_root", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/freeze_root.rs の f155_ の 3 本（一時の根の直下の design-intent に init して commit した骨格で、check と --freeze-ids と --emit-amends が rc 2 で違反 0、--freeze-start が rc 1 で違反ちょうど 2 件〔constitution-v1.0.yaml（凍結で書く承認一覧・憲法 meta.approval の写し）: approvals[0].ruling に台帳 id が無い の 1 件と 表に無い の 1 件〕と 凍結しない を出して anchors/ を作らない / 床の土台の写しで憲法の裁定 id を 未記入 にすると、列の無い置き場の --freeze-start は rc 1 で承認欄の行ちょうど 1 件、id の一覧だけが在る置き場の --freeze-anchor は rc 1 で承認欄の行 0 件、どちらも anchors/ が変わらない / 床の土台の写しの列の無い置き場で、承認者を空の字にすると --freeze-start の承認欄の行がちょうど approvals[0] に who / ruling / verbatim が無い の 1 件、meta.approval に欄 adr: ADR-1 を足すと承認欄の行 0 件、どちらも rc 1 で anchors/ を作らない）が緑、tests/freeze_root.rs の歯の全部（列の根の表と始まりの凍結の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
