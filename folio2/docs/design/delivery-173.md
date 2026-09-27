# 設計: 便 173 — 判断の記録の改訂の欄 revises を欄の決まり・床・面から削る（便 170 を割った後半・判断の記録 ADR-30 決定 (2)）

- 要件: FR19（判断の記録の欄の決まりの生成区間を床の定数から導く＝revises を削る）・FR16（判断の記録の面は正本の欄から組む＝消えた欄の札と行を描かない）・FR5（床は revises を持つ記録を未知の欄の違反で返す）。中身は判断の記録 ADR-30 決定 (2)（改訂の欄を削る）。決定 (3)(4)（封・束）は前半の便 172（行 fs）。
- 条: P-5.6（床の定数の写しは生成区間へ導く）/ P-6.2（生成区間は folio の出力）/ P-10.1（凍結 anchor = 歯の fixture）/ P-7.2（退役は status で）/ N-3.1（欄の閉じた一覧に例外を足さない）。
- 出所: 持ち主の承認 2026-09-27 12:50 JST（対話面 R-8・逐語「承認する」・構造の直し A）。便 170 の契約 `docs/design/delivery-170.md`（行 fq・記録として残す）。
- 置き場: 審査の材料は行 `ft` の §1 だけ。write-set 30 本（src 3・歯の file 5・設計文書 1〔生成区間〕・fixture 21）。新しい dir・新しい file・消す file は無い。
- 門: **便 172 の後の本流（見本 57648da）の binary で 0（通す）**（write-set 30 本・`通す（印の周 2026-09-27-round51（判定 合格）に、書き換える file を場所とする反証で支持された 止める は無い・印の後の変更は審査していない）`）。今の本流 f27c859 の binary でも 0。
- 前提: **便 172（行 fs）の着地の後に受け付ける**。base = 便 172 の後の本流（中身は origin の枝 `impl/d172` の 57648da と同じ見込み・数はその写しの実測・参考値・行 D-13）。

## 1. 設計

### (a) いま起きていること（参考値・便 172 の後 = 57648da）

1. **便 170 は審査の上限で止まった。** 便 170（行 fq・台帳 f2-648.254）の run `f2-648.254-20260927T073348Z` は verify 17 行が全部 rc 0 だったが、門の審査（lens・別の AI が差分を読む段）が「diff 196520 byte が cap 150000 を超えた」で Gated INCONCLUSIVE（まだ分からない）。cap は器の規則の行 gate.token_cap（持ち主の裁定）で変えない。**同じ実装を 2 便に割った。** 前半の便 172 = 封と束（108,793 byte）、本便 = 改訂の欄の削除（差分 104,073 byte・30 file・目安 120,000 byte 以下）。
2. **実装の見本。** origin の枝 `impl/d173`（commit 8e4d0c8・親は便 172 の見本 57648da）が本便の後の中身。**作業者は write-set の file をこの commit の中身にしてよい**（`git checkout 8e4d0c8 -- <write-set の file>`）。write-set の外は変えない。見本は便 170 の実装（`impl/f2-648.254-run1` = 6099cf4）と中身が同じで、違いは歯の名 2 本（f170_ → f173_）・tests/seal.rs の頭の注・tests/graph.rs の注の行・face_adr.rs の空白 1 字（`let f = frame`）だけ（`git diff 6099cf4 8e4d0c8` で 4 file・+7 −6）。
3. **改訂の欄は欄の決まりに在る。** `floor_adr.rs` の REVISES（optional の欄）・REVISE_KIND・REVISES_ENTRY・床の木の revise_kind・revises_entry・revises_note。`adr.rs` の check_revises が形・target・同じ決定の二重を見る。生成区間 `design-intent/adr/schema.yaml`（140 行）と凍結 anchor `tests/fixtures/schema/adr-region.txt` が同じ字を持つ。本流の記録 28 本は行 0。
4. **面が改訂の欄を描く。** `face_adr.rs` は表紙の札「判断の記録の改訂 n 件」・章 05 の行・受けた改訂の逆向きの行と札（便 148）・機械の面の `<dt>revises</dt>` を出す。
5. **封は在る（便 172）。** 発効した記録の本文を変えると封の違反が立つ。便 101 の歯 7 本（`tests/adr.rs`）は写しの ADR-13 に行を植え、ADR-13 の封の違反 1 行を確かめて外した残りを見ている。
6. **base の歯。** nextest 1008 / 1008・clippy 0 警告・床 4 本 rc 0・`folio build --write` 35 file。`git grep -n f173_ -- crates` 0 件・行 id `ft` 0 件。数は封の一覧を持つ枝（run の枝・impl/f2-648.254-run1・impl/d172・impl/d173）を参照に持つ写しで取った（便 172 の後は本流が封の一覧を持つので、便 172 の §1 (a) の 5 の違反は出ない・参照が 1 本の写しでも床 4 本 rc 0）。

### (b) 直す先

1. **`floor_adr.rs`。** REVISES（欄の一覧から）・REVISE_KIND・REVISES_ENTRY・床の木の revise_kind・revises_entry・revises_note を削る。revises を持つ記録は欄の閉じた一覧の外＝`[adr] ADR-n: 未知の欄（N-3）: revises` で落ちる。封の定数と節（便 172）は変えない。
2. **`adr.rs`。** check_revises と呼び出しを削る。単体の歯 floor_notes_are_outside_the_diff の注の数を 25 → 24（revises_note が消える）。
3. **`face_adr.rs`。** 判断の記録の改訂の札・h3 と行・受けた改訂の逆向きの行と札（RevisedBy と revised_by）・機械の面の revises の数・表 REVISE と名札の定数・単体の歯 2 本（f137 / f148）を削る。条文の改訂（amends）の札と行は残し、章 05 の空の断りは「条文の改訂なし」の 1 段落。
4. **生成物（手で書かない）。** `design-intent/adr/schema.yaml` の生成区間 = `folio schema --dir design-intent --write` の出力（131 行・25271 byte・sha256 8885ca95…＝凍結 anchor adr-region.txt と byte 一致・便 170 の実装の出力とも byte 一致）。封の一覧は変わらない（本文を変えない＝`--freeze-adrs` は `足す封の行は無い`）。
5. **変えないもの。** 判断の記録の本文・封（seal.rs と封の一覧）・束・amends / supersedes の床・天井の TRIGGER_ADR_FIELDS と生成区間 ceiling.yaml の trigger（revises の字を含む・読む欄が無いだけ・撤去は便 169 の (g) 3 の後続）・`folio init` の骨格。

### (c) 歯（f173_・base で 0 件）

名は便 170 の歯（§1 (c) の 1 と 9）を f173_ に改めたもの（便 172 が本流に置く f170_ と分ける）。

1. **f173_a_record_with_revises_is_an_unknown_field（`tests/seal.rs`）。** 土台 `tests/fixtures/floor_base/design-intent/` の写しの ADR-2 に revises の 1 行 → rc 1・違反 2 行 `[adr] ADR-2: 未知の欄（N-3）: revises` と封と違う。**base は封と違う の 1 行だけ（revises は欄の決まりの中）＝RED。**
2. **f173_the_adr_face_draws_no_revision_of_records（`tests/face_adr.rs`）。** 凍結 fixture の写しと、正本に revises の 1 行を足した写しの面の両方で、札は「条文の改訂 0 件」の 1 つ・断りは「条文の改訂なし」の 1 段落・「ほかの判断の記録による改訂」「`<dt>revises</dt>`」「を狭める」が無い・機械の面は `<dt>amends</dt><dd>0</dd>`・revises の行は面の字を変えない。**base は札「判断の記録の改訂」を出す＝RED。**

### (d) 採らなかった形

1. **割り方を逆にする（改訂の欄の削除を前半に）。** 便 172 の §1 (d) の 1 のとおり（前半だけの本流は、封の一覧を持つ枝が参照に在る写しで素の床が不合格のまま）。
2. **便 101 の歯を 1 本残す（植えた行が未知の欄で落ちる歯を tests/adr.rs に）。** 同じ床を (c) の 1 が土台の写しで縛る。本流の記録が行を持たないことは素の床（未知の欄）が毎回見る。残すと写しの ADR-13 を変える道具と封の直しが残り、差分と歯が増える。
3. **生成区間を書き直さずに床の定数だけ削る。** 床が生成区間の字と定数の食い違いを落とす（FR19・P-5.6）。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **実装だけ（src と design-intent の 4 file）を 57648da に当てると既存 62 本が落ちる**（1006 本中・本文は d172-scripts の sim0-s0-173-nextest.log）。扱いは次の 3 つだけ。
   - fixture の直しで緑になる: 欄の決まりの写し 17 本（`tests/fixtures/*/adr/schema.yaml`・floor_base を含む・revises の 3 か所を消す）・adr-region.txt（revises の 9 行を消す）・node-digest-anchor.txt（独立の Python node-digest.py の出力）・面の凍結 fixture 2 本（expected-adr.html・expected-site-adr-2.html）。
   - 消す 18 本（欄と面が無くなる）: `tests/adr.rs` の便 101 の区間の f101_ 7 本と道具（REVISES_ROW・REVISES_HEAD・assert_adr13_violation・beyond_the_adr13_seal・Work::plant_revises・Work::mutate_file）、`tests/face_adr.rs` の f137_ 4 本と便 148 の区間の 5 本、`face_adr.rs` の単体の f137 / f148 の 2 本。
   - 歯の本文を直す: `tests/face_adr.rs` の f137_amends と census と札の道具を条文の改訂だけに。定数 = `tests/schema.rs` の生成区間の行数・byte・sha256（注は便 170 の実装の字）・`tests/graph.rs` の F99_ANCHOR_SHA256・`adr.rs` の単体の注の数。`tests/adr.rs` の頭の注。
2. **便を当てた写し。** nextest **992 / 992**（1008 − 18 + 2）・clippy 0 警告・床 4 本 rc 0・build 35 file（判断の記録の面 28 枚だけが違い、ほかは byte で同じ・面の revises の札と `<dt>revises</dt>` は 0）。封の一覧を持つ枝が参照に在る写しでも、参照が本流だけの写しでも同じ。
3. **RED。** 歯だけ（r173-teeth.patch = tests/seal.rs・tests/face_adr.rs）を 57648da に当てると `--test seal f173_` 1 本（違反が封と違う の 1 行だけ・2 行でない）と `--test face_adr f173_` 1 本（凍結 fixture に札「判断の記録の改訂」が残る）が落ちる（本文は red-red173-*.log）。
4. **突然変異（便の後の写しの src を 1 通りずつ変え、verify の 1〜2 を撃つ）。** 3 通りとも f173_ が落ちる（mut.log）。

| 変異 | 落ちる f173_ |
| --- | --- |
| M4 欄の閉じた一覧に revises を残す | a_record_with_revises |
| M8 面の機械の欄に revises の数を戻す | draws_no_revision |
| M12 章 05 の空の断りに判断の記録の改訂の段落を戻す | draws_no_revision |

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 縮む（`-`・行数）26 本（src 3・`tests/adr.rs`・`tests/face_adr.rs`・生成区間・欄の決まりの写し 17 本・面の fixture 2 本・adr-region.txt）。ほかの 4 本（tests/graph.rs・tests/schema.rs・tests/seal.rs・node-digest-anchor.txt）は増えるか同じ。差分 104,073 byte（`git diff 57648da 8e4d0c8 | wc -c`）。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120) の和（空行は 1）を 1500 から引く。python と awk の 2 実装が 3 本とも一致した（lines.log）。

| file | base | base の余地 | 便の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/adr.rs` | 1049 | 451 | 993（−56） | 507 |
| `crates/folio/src/face_adr.rs` | 1100 | 400 | 955（−145） | 545 |
| `crates/folio/src/floor_adr.rs` | 542 | 958 | 494（−48） | 1006 |

3. **size は M。** src −249 行・歯 18 本の削除と直し・fixture 21 本。余地の最小は base の face_adr.rs の 400（≥ 300）。
4. **verify 9 行 = done の 9 の塊。** 1 `--test seal f173_`（(c) の 1）・2 `--test face_adr f173_`（(c) の 2）・3 `--bin folio`（148 本・注の数の単体を含む）・4 `--test adr`（13 本）・5 `--test face_adr`（40 本）・6 `--test seal`（7 本）・7 `--test graph`（16 本）・8 `--test schema`（21 本）・9 clippy 0 警告。便の後の写しで 9 行とも rc 0（verify.log）。base（57648da）では 1 と 2 が 0 件で rc 4（歯だけを当てた写しでは各 1 本が落ちる＝(e) の 3）。`--test` の 5 本はどれも write-set に在る。

### (g) 門・受付・並行の便・外の置き場

1. **門。** 57648da の写しとその binary で write-set 30 本は 0（通す）。受付の時点で撃ち直す。
2. **受付。** 便 172 の着地の後に受け付ける。live な run（便 170・f2-648.254）と write-set が重なる＝`write-set-overlap` の断りは run を止めるまで出る（席が持ち主に停止を頼む）。受付の本流が 57648da と中身で違えば (i) の 3。受付の先撃ち（precheck）は、今の本流の上では `write-set-item-unresolved`（crates/folio/tests/seal.rs が base に無い）1 件が出る＝便 172 の着地の前だけの断り。57648da に本契約を置いた写しでは 0（preflight ok）。
3. **外の置き場（tsuzuri）の手順。** 便 172 の §1 (g) の 4 のとおり。便 172 の binary で revises の塊を消してから封を足した置き場は、本便の binary で `folio schema --dir design-intent --write` → commit だけで合格（tsuzuri 56352b2 の clone で実測・tsuzuri.log の P1）。revises を残して封を足した置き場は、本便の binary で revises が未知の欄（3 本）になり、消すと封と違う 3 本が残る（P2）。
4. **init の骨格。** 骨格の欄の決まりは床の定数から出るので revises の 3 か所が消えるだけ（骨格の ADR-1 は proposed）。

### (h) 数え直す手順（行 D-13）

記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-27/d172-draft.md`、差分と script は同じ dir の d172-scripts。1 mk-s173.py（便 172 の後の写しに便 170 の実装から改訂の欄の削除を取り出す）→ fin-regen.sh 173（生成区間と定数と node-digest-anchor.txt）→ commit → suite-172.sh。2 red-172-173.sh（r173-teeth.patch で RED）。3 mut-172-173.py。4 lines-172.sh。5 sim0-172-173.sh。6 verify-172-173.sh・gate-172-173.sh・build-count.sh・tsuzuri-172-173.sh・ws.sh。7 commit の後に `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-173.md#ft`。

### (i) 要件との関係・言えないこと・撤退条件

1. **要件。** FR16 の「改訂の一覧」は条文の改訂（amends）として残る。FR19 の生成区間は床の定数から導いたまま（revises の 9 行が消える）。
2. **言えないこと。** 天井の TRIGGER_ADR_FIELDS と生成区間 ceiling.yaml の trigger には revises の字が残る（読む欄が無いだけ）。本文の散文の「判断の記録の改訂」の字は面に残る（正本の本文の字）。
3. **撤退条件。** (1) (e) の 1 の 62 本のほかに既存の歯が落ちたら、直さずに止めて席へ返す。(2) 便 172 が未着地なら受け付けない。(3) 受付の本流の判断の記録が revises の行を持てば止める（先に行を消す設計の PR が要る）。(4) 差分が 120,000 byte を超えたら止める。(5) 着地の後の本流で `folio build` の file 数が着地の直前と違うか、判断の記録の面のほかの面が byte で変われば止める。

## 2. 範囲

- 入れる: §1 (b) の 1〜4・歯（f173_ の 2 本と (e) の削除と直し）・fixture 21 本。
- 入れない: 封と束（便 172）・判断の記録の本文・天井の TRIGGER_* と ceiling.yaml・門と印・init の骨格・外の置き場への書き込み・外部 crate・新しい dir・台帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| floor | 欄の決まり | floor_adr.rs と adr.rs の revises の削除・生成区間 |
| face | 面 | face_adr.rs の revises の札と行の削除 |
| teeth | 歯 | f173_ の 2 本・欄の決まりの写しと凍結 anchor と面の fixture・消す歯 18 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace の nextest と clippy）。

## 5. 依存

- 外部 crate・新しい dir・host の命令は無い。前提は便 172（行 fs）の着地。
- 着地の後に席が見ること: 本流の `target/debug/folio` を組み直す・本流の素の床が合格・tsuzuri へ便 172 の (g) の 4 を渡す・便 170（f2-648.254）の run の後始末（枝 `impl/f2-648.254-run1` と手元の run の枝は、本便の着地で中身が本流と同じになる）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ft"
title = "便 170（行 fq）を審査の上限（diff 196520 byte が cap 150000 を超えた）で 2 便に割った後半・構造の直し A（持ち主の承認 2026-09-27 12:50 JST・判断の記録 ADR-30 決定 (2)）: floor_adr.rs から改訂の欄 revises の定数と床の木の revise_kind・revises_entry・revises_note を削り、adr.rs の check_revises と face_adr.rs の revises の札と行と逆向きの行を削る（revises を持つ記録は未知の欄で落ちる）。生成区間は folio schema --write の出力。封と束は便 172（行 fs）のまま変えない。実装の見本は origin の枝 impl/d173 の commit 8e4d0c8（親 57648da）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。便 172 の着地の後に受け付ける。base = 便 172 の後の本流"
req = ["FR19", "FR16", "FR5"]
section = "1"
write-set = ["-crates/folio/src/adr.rs", "-crates/folio/src/face_adr.rs", "-crates/folio/src/floor_adr.rs", "-crates/folio/tests/adr.rs", "-crates/folio/tests/face_adr.rs", "crates/folio/tests/graph.rs", "crates/folio/tests/schema.rs", "crates/folio/tests/seal.rs", "-design-intent/adr/schema.yaml", "-tests/fixtures/adr/effective-no-approval/adr/schema.yaml", "-tests/fixtures/adr/schema-drift/adr/schema.yaml", "-tests/fixtures/adr/two-adopted/adr/schema.yaml", "-tests/fixtures/anchor/no-anchor/adr/schema.yaml", "-tests/fixtures/anchor/root-digest-drift/adr/schema.yaml", "-tests/fixtures/check/dup-key/adr/schema.yaml", "-tests/fixtures/check/empty-field/adr/schema.yaml", "-tests/fixtures/check/unknown-section/adr/schema.yaml", "-tests/fixtures/face/expected-adr.html", "-tests/fixtures/face/expected-site-adr-2.html", "-tests/fixtures/floor_base/design-intent/adr/schema.yaml", "-tests/fixtures/link/adr-id-missing/adr/schema.yaml", "-tests/fixtures/link/amended-by-orphan/adr/schema.yaml", "-tests/fixtures/link/retreat-kind-drift/adr/schema.yaml", "-tests/fixtures/refs/bad-counts/adr/schema.yaml", "-tests/fixtures/refs/dangling-id/adr/schema.yaml", "-tests/fixtures/refs/orphan-rule/adr/schema.yaml", "-tests/fixtures/schema/adr-region.txt", "tests/fixtures/schema/node-digest-anchor.txt", "-tests/fixtures/vocab/exemptions/adr/schema.yaml", "-tests/fixtures/vocab/unknown-word/adr/schema.yaml"]
verify = ["cargo nextest run -p folio --test seal f173_", "cargo nextest run -p folio --test face_adr f173_", "cargo nextest run -p folio --bin folio", "cargo nextest run -p folio --test adr", "cargo nextest run -p folio --test face_adr", "cargo nextest run -p folio --test seal", "cargo nextest run -p folio --test graph", "cargo nextest run -p folio --test schema", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "tests/seal.rs の f173_（土台の写しの ADR-2 に revises の 1 行を足すと違反は未知の欄と封と違うの 2 行）が緑、tests/face_adr.rs の f173_（面に判断の記録の改訂の札・行・逆向きの行・機械の面の revises が無く、正本に revises の行が在っても面の字は変わらない）が緑、binary の単体の歯の全部が緑、tests/adr.rs・face_adr.rs・seal.rs・graph.rs・schema.rs の歯の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（封は変わらない）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じで判断の記録の面のほかの面は byte で同じである"
<!-- contracts:end -->
