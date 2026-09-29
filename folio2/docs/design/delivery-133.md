# 設計: 便 133 — 列の根の表に scribe3 の憲法の行を足す（判断の記録 ADR-16 決定 (7) ⑦・ADR-21 決定 (4)・利用者の持ち主の承認 s2-07l.214）

- 要件: FR23（憲法の列の根を、憲法の名ごとの表で照らす・要件書 第 1.41 版）。FR23 の注は「表に行を足すのは folio2 の便で、契約表の行か設計ノートが利用者の持ち主の承認の裁定 id を名指す（規則の表の行 D-11 の手順）」と書き、規範文は「folio2 の行は 1 字も変えない」「表の写しは置き場の名の行だけを導出する」と書く。本便は FR23 の規範文を変えず、表に 2 行目を足すだけである。契約表の行の req は FR23 の 1 つ（main に在る id）。
- 条: P-5.1（閉じた一覧は型付きデータ＝表は実装の型付きの定数）/ P-10.1（独立した凍結 anchor＝足す行の digest を folio の code から独立に組んで一致させる）/ P-12.2（承認は逐語と日付を添える）/ N-3.1（例外の口を足さない＝検査される側の data は行を選べるだけ）。
- 出所: 判断の記録 **ADR-16**（発効 2026-09-23 17:39 JST・裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）の決定 (7) ⑦「列の根の表に利用者の行を足す便（小・1 便）は、利用者が骨格を書き、持ち主が骨格を承認し、始まりの凍結の命令で要約値を得た後に起こす」と、判断の記録 **ADR-21**（発効 2026-09-24 22:38 JST・裁定 id = 台帳 f2-648 notes 2026-09-24 22:38 JST）の決定 (4)「⑦ は残し、利用者を v3 の置き場と読む。起こすのは、v3 の持ち主が v3 の憲法の第 1.0 版を承認し、始まりの凍結の命令が要約値の全桁を出して断った後である」。**利用者の持ち主の承認（行 D-11 が名指しを求める裁定 id）** は scribe3 の憲法 第 1.0 版の承認欄のとおりで、承認者 = 持ち主・日付 2026-09-25・裁定 id = scribe2 台帳 **s2-07l.214**（裁定 id = user 2026-09-24T22:39Z・notes に逐語）・逐語「これでよい」・対話面 R-8 である（scribe3 の版 e52d24c の design-intent/constitution.yaml の meta.approval・起草役が読んだ）。依頼は scribe3 の設計席から席へ（2026-09-25 07:5x JST 受領）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ef` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 3 本。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えない（表の写しは置き場の名の行だけで、folio2 の置き場の写しは 1 字も変わらない・(b) の 3）ので、天井の門の対象外である。起草役が write-set 3 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は便 121（列の根の表と始まりの凍結の命令・台帳 f2-648.170・着地）。**base = main 9d18031（`crates/`・`tests/`・`design-intent/` は ed966c1 と 1 byte も違わない）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 131（行 `ed`）・便 132（行 `ee`）とは write-set が 1 本も重ならない。受付の順は席が決める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため、共通の検証を同時に撃たない逐次を勧める）。

## 1. 設計

### (a) いま起きていること（実測・base 9d18031）

1. **表は 1 行。** 床の定数の列の根の表（`crates/folio/src/floor_adr.rs` の ROOT_DIGESTS・便 121）は、鍵 folio2-constitution・値 acb52acd…（folio2 の憲法 第 1.0 版の凍結の digest）の 1 行だけの閉じた一覧である。引く口は `crates/folio/src/adr.rs` の root_digest で、床の照らし（`crates/folio/src/anchor.rs`）と凍結の命令（`crates/folio/src/freeze.rs`）が共有する。表の写しは、判断の記録の欄の決まりの生成区間（adr/schema.yaml の anchor の節の欄 root_digests）へ、**置き場の憲法の名の行だけ**が導出される（床の木の変種 Pick）。
2. **scribe3 の置き場は凍結できない。** scribe3 の置き場（scribe3 の repo の design-intent・folio2 の外・読むだけ）の憲法は、名 scribe3-constitution・版 v1.0・status effective・binding true で、承認欄は (§0 の出所) のとおり埋まっている。起草役が scribe3 の版 e52d24c の design-intent を `git archive` で一時 dir に取り出し、git の init と 1 commit をした写しに base の binary を撃った結果は次のとおり。

| 命令 | 終了コード | 出力の要点 |
| --- | --- | --- |
| folio check（旗なし） | 2 | 違反 0・まだ分からない 2（凍結 anchor が 0 本・id の一覧が無い） |
| folio check --freeze-start | 1 | 列の根の凍結だが憲法の名 scribe3-constitution が列の根の表に無い＝凍結しない・組んだ木の digest 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 |

   scribe3 の HEAD は e52d24c の後も進む（起草の時点 8c94b22 では design-intent に差 0・検証の時点 e60d58e では design-note/bakeoff-surface.yaml に +9 行）が、constitution.yaml の blob と根の digest は e52d24c と同じである（`git diff` と --freeze-start の実測）。本便が縛るのは憲法の木の digest であって、design-intent の他の file ではない。scribe3 の設計席の実測（本流 68196cf の binary）と、席の実測（ed966c1 の binary・scribe3 の HEAD の作業ツリーの写し）も同じ digest である。

3. **digest の独立の再現（2 実装）。** 起草役は folio の code を呼ばない python の script（持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d133-scripts/root_digest.py`）で、始まりの凍結の木（種別・digest の方式・版・previous は空・写しの範囲と欄・承認の写しと承認一覧 1 項・憲法の写し）を憲法の正本から組み、正規化（キー順固定・空白なし・非 ASCII はそのまま・日付は文字列の json）の sha256 を撃った。

| 撃った相手 | 結果 |
| --- | --- |
| folio2 の凍結 anchor 5 本（v1.0〜v1.4）の digest の欄の撃ち直し | 5 本とも一致 |
| folio2 の憲法 第 1.0 版（anchor v1.0 を足した commit の正本）から組んだ根 | acb52acd…（表の folio2 の行と一致） |
| scribe3 の版 e52d24c の憲法から組んだ根 | 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356（binary の出力と一致） |

4. **台帳の控えとの関係。** 控え f2-648.192（外の置き場の始まりの凍結の手順・行を足す便）は本便の出所の 1 つで、本便の着地で行を足す部分が閉じる（手順を設計ノートに書く部分は (e) の 2 の手順で代える）。控え f2-648.193（承認欄の空いた憲法を --freeze-start が凍結する罠）は、scribe3 の承認欄が埋まっているので本便の手順では起きない（表の digest が meta.approval を覆うので、承認欄が未記入の写しは「列の根の表の行と違う」で断る＝検証役の実測。旗なしの check の違反 0 は ruling が未記入でも 0 のままで、罠の有無を分けない）。罠そのものは閉じない。控え f2-648.194（行 R-17 が無い置き場で散文の言及の歯が黙る）と f2-648.195（値域を狭める変更が黙って通る）は本便の外で、scribe3 の写しの床の結果（(e) の 2 の手順の後で合格 0/0）には現れない。
5. **base の歯（参考値）。** workspace の nextest 861 / 861・床 4 本 rc 0。`git grep -n 'f133_' -- crates` は 0 件。`git grep -n 'scribe3-constitution' -- crates tests` は 0 件。

### (b) 直す先 — 表に 2 行目を足す

1. **表（floor_adr.rs）。** ROOT_DIGESTS の 2 行目に、鍵 scribe3-constitution・値 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 を足す。folio2 の行は 1 字も変えない（FR23）。表の頭の注に、2 行目の出所（scribe3 の版 e52d24c・ADR-16 決定 (7) ⑦・ADR-21 決定 (4)・裁定 id s2-07l.214）を 1 文足す。
2. **ほかの code は変えない。** 引く口・床の照らし・凍結の命令・写しの導出（Pick）は、表の行の数に依らない形のまま（便 121）。
3. **写しの全数（find で列挙）。** repo の中の判断の記録の欄の決まり（`adr/schema.yaml`）は 18 本（実の置き場 1・床の凍結の土台 1・fixture 16）。置き場の憲法の名は、実の置き場と土台が folio2-constitution、fixture の 16 本が fixture-constitution で、scribe3-constitution の置き場は 0 本である。写しは置き場の名の行だけなので、18 本とも 1 byte も変わらない（凍結 anchor `tests/fixtures/schema/adr-region.txt` と `crates/folio/tests/schema.rs` の要約値の定数も変わらない）。`folio schema --dir design-intent --check` は一致のまま。
4. **folio2 の床と面は変わらない。** folio2 の置き場の床の結果・`folio build` の出力（30 file）は base と 1 byte も変わらない（起草役の実測）。

### (c) 歯（新しく 2 本）

関数名は f133_ で始める（verify の絞り込みの語・base で 0 件）。

1. **f133_the_root_table_holds_folio2_and_scribe3（単体の歯・`crates/folio/src/adr.rs` の既存の tests の区間）。** 表 ROOT_DIGESTS がちょうど 2 行で、並び・鍵・値の全桁が (b) の 1 のとおり（folio2 の行が先）であることを等しさで数え、口 root_digest が scribe3-constitution から 2 行目の値を引くことを数える。本数と digest は歯の中の手で写した字で持つ（凍結 anchor・P-10.1）。行を足す次の便はこの字も同じ要求で直す。**base では表が 1 行で落ちる＝RED。**
2. **f133_scribe3_name_picks_its_row（`crates/folio/tests/freeze_root.rs`・binary 経由）。** 実の design-intent の写し（便 121 の歯 7 と同じ土台）の憲法の名を scribe3-constitution に書き換え、`folio schema --write` → 終了コード 0・判断の記録の欄の決まりの写しの列の根の欄がちょうど 2 行「root_digests:」と「scribe3-constitution: <digest の全桁>」で、folio2 の digest が写しに無い。commit の後に `folio check` → 終了コード 1・違反はちょうど 1 件で、種別 anchor・字「列の根の表の scribe3-constitution の行」を持ち、字「表に無い」を持たない（写しの folio2 の列の根〔acb52acd…〕を scribe3 の行と照らして違う、の違反）。digest の字は歯の file の定数に手で写す（folio の code を呼ばない）。**base では写しの欄が空の表で落ちる＝RED。**

scribe3 の憲法そのものを fixture に写す歯は置かない（写しは gate の cap を超え、fixture は最小の手書きに限る・(d) の 2）。

### (d) 採らなかった形

1. **行を足さず、scribe3 の置き場を folio2 の外の手順（移行）で凍結する。** ADR-16 決定 (7) ⑦ と ADR-21 決定 (4) が、利用者の行を足す便を M3 の律速として置いた。
2. **scribe3 の憲法の写しを fixture に置き、--freeze-start の digest を binary で数える。** 写しは 1 本の file で数万 byte になり、fixture は最小の手書きに限る（便の教訓）。digest の値は (a) の 3 の独立の script と歯 2 の写しの導出で数え、scribe3 の置き場の上の通しの確かめは起草役の模擬（(e) の 2）に置く。
3. **表の頭の注を設計文書の側（adr/schema.yaml の注）に足す。** 生成区間の注は床の実装の定数の写しで、注を変えると folio2 の写しが変わり門の対象になる。今の注（anchor_note・limits_note）は「行を足すのは folio2 の便」と既に書くので足さない。

### (e) 既存の歯のうち落ちるもの・scribe3 の側の手順

1. **落ちる既存の歯は 0 本（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest は 863 / 863（base 861 + 歯 2）・clippy 0 警告・床 4 本 rc 0。便 121 の歯（`crates/folio/tests/freeze_root.rs` の 7 本・実の写しと土台の写しの表が folio2 の 1 行であることを見る歯を含む）は緑のまま。
2. **scribe3 の側の手順（着地の後・scribe3 の持ち主の手番・起草役の模擬）。** 本便を当てた binary を scribe3 の版 e52d24c の写しに撃つと、次の順で合格まで届く。

| 段 | 命令 | 結果 |
| --- | --- | --- |
| 1 | folio check | 終了コード 1・違反 1（写しの列の根の欄が空の表で、床の定数の scribe3 の行と違う）・まだ分からない 2 |
| 2 | folio schema --write | 判断の記録の欄の決まりの写しだけを書き直す（列の根の欄に scribe3 の行・ほかの 8 file は変わらない） |
| 3 | commit の後に folio check --freeze-start | 合格 0/0 の上で始まりの凍結をした（憲法の anchor v1.0・id の一覧 v0.1・索引・条 38・id 4 本） |
| 4 | commit の後に folio check | 終了コード 0・合格（違反 0・まだ分からない 0）・索引の digest は 35eb6b36… |

   段 1 の違反は、表に行が在るのに写しが古い間だけ立つ（便 121 の歯 7 と同じ仕組み）。scribe3 の置き場は scribe3 の repo の持ち物で、folio2 の便は書かない。段 2〜4 は scribe3 の側の手番として、席が scribe3 の設計席へ返す。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 3 本とも印なし（`crates/folio/src/floor_adr.rs`・`crates/folio/src/adr.rs`・`crates/folio/tests/freeze_root.rs`）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/floor_adr.rs` | 510 | 990 | 518（+8） |
| `crates/folio/src/adr.rs` | 939 | 561 | 961（+22） |

   2 本とも余地は S の見積 100 を超える。歯の file は src の外なので余地を測らない（参考値 468 行 → 505 行）。
3. **size は S。**
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f133_` = (c) の 1。
   2. `cargo nextest run -p folio --test freeze_root f133_` = (c) の 2。
   3. `cargo nextest run -p folio --test freeze_root` = 列の根の表と始まりの凍結の歯の全部（便 121 の 7 本を含む・参考値 8 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は freeze_root の 1 本で write-set に在る。`--bin folio` の絞り込みの語 f133_ を関数名に持つ src は `crates/folio/src/adr.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 便 131・132 と write-set が重ならない。共通の検証を同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d133-draft.md`、模擬の差分と script は同じ dir の d133-scripts（repo には入れない）。

1. digest: scribe3 の repo で `git archive e52d24c design-intent` を一時 dir に展開し、git の init と 1 commit の後に base の binary で `folio check --dir design-intent --freeze-start` を撃つ。独立の再現は root_digest.py の root の口（憲法の正本と欄の決まりの file を渡す）。script の正しさは check-anchor の口で folio2 の凍結 anchor 5 本を撃ち直して確かめる。
2. 模擬: 差分 c133.patch を base に当て、workspace の nextest（863 / 863）・clippy・床 4 本・`folio build --write` の出力の差 0・(e) の 2 の 4 段。
3. RED: 歯の 2 か所（adr.rs の tests の区間と freeze_root.rs）だけを base に当てると、f133_ の 2 本とも落ちる。
4. 余地: 持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d119-draft-lines.py` を repo の根で当てる（便 119 の script・同じ式）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** scribe3 の置き場の写しの書き直しと凍結（scribe3 の側の手番・(e) の 2）。控え f2-648.193・.194・.195 の直し。表の 3 行目以降。folio2 の設計文書の字（要件 FR23 の注の「利用者の行を足す便は…起こす」の着地の後の字は、次の一括で席が書く）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 表が証するのは列の根の中身で、どの repo の列かは証さない（ADR-16 の帰結）。scribe3 の憲法が e52d24c の後に凍結の前に変わると digest が変わり、凍結の命令は「行と違う」で断る（その時は行を直す便が要る）。scribe3 の設計席は凍結まで design-intent の承認欄と裁定 id を変えないと約束している（席の受領）。
3. **撤退条件。** (1) 受付の時点で、scribe3 の repo の HEAD の constitution.yaml が e52d24c のものと違うか、HEAD の design-intent の写しに base の binary で撃った --freeze-start の digest が (b) の 1 の値と違えば、止めて席へ返す（行の値を取り直してから運ぶ）。design-intent の他の file（design-note 等）の差は止めない。(2) 本便の後に folio2 自身の床の結果か `folio build` の出力か 18 本の欄の決まりの写しが 1 byte でも変わったら、止めて席へ返す。(3) 本便の後に既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/floor_adr.rs` の表の 2 行目と頭の注の 1 文。`crates/folio/src/adr.rs` の単体の歯 1 本。`crates/folio/tests/freeze_root.rs` の定数 2 つと歯 1 本。
- 入れない: 引く口・床の照らし・凍結の命令・写しの導出・設計文書・fixture・scribe3 の repo・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| row | 列の根の表の 2 行目 | `crates/folio/src/floor_adr.rs` の ROOT_DIGESTS（scribe3-constitution・digest の全桁） |
| unit | 表の単体の歯 | `crates/folio/src/adr.rs` の f133_（2 行の並び・鍵・値の等しさ） |
| teeth | 写しと床の歯 | `crates/folio/tests/freeze_root.rs` の f133_（写しの導出と床の照らしを binary で） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 便 121（列の根の表と始まりの凍結の命令）。
- 並行の便: 便 131（行 `ed`）・便 132（行 `ee`）と write-set が重ならない。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じ、控え f2-648.192 の行を足す部分を閉じる。scribe3 の設計席へ (e) の 2 の段 2〜4 を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ef"
title = "判断の記録 ADR-16 決定 (7) ⑦（裁定 id = 台帳 f2-648 notes 2026-09-23 17:39 JST）と ADR-21 決定 (4)（裁定 id = 台帳 f2-648 notes 2026-09-24 22:38 JST）の便: 床の定数の列の根の表 ROOT_DIGESTS（crates/folio/src/floor_adr.rs）に、scribe3 の憲法の行（鍵 scribe3-constitution・値 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356・scribe3 の版 e52d24c の始まりの凍結の木の digest・利用者の持ち主の承認の裁定 id = scribe2 台帳 s2-07l.214〔user 2026-09-24T22:39Z・逐語 これでよい〕）を 2 行目に足す。folio2 の行・引く口・床の照らし・凍結の命令・写しの導出（置き場の名の行だけ）・設計文書は変えない。歯は f133_ の 2 本（表の単体の歯と、scribe3 の名に書き換えた実の置き場の写しで folio schema が scribe3 の行を写し床が表の行と照らす歯）"
req = ["FR23"]
section = "1"
write-set = ["crates/folio/src/floor_adr.rs", "crates/folio/src/adr.rs", "crates/folio/tests/freeze_root.rs"]
verify = ["cargo nextest run -p folio --bin folio f133_", "cargo nextest run -p folio --test freeze_root f133_", "cargo nextest run -p folio --test freeze_root", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "表の単体の歯 f133_（表がちょうど 2 行で、並び・鍵・digest の全桁が §1 (b) の 1 のとおり・口が scribe3 の名から 2 行目の値を引く）が緑、freeze_root の f133_ の歯（憲法の名を scribe3 の名に書き換えた実の置き場の写しで folio schema --write が列の根の欄を scribe3 の行 1 行にし、commit の後の folio check が列の根の表の scribe3 の行と違う の違反ちょうど 1 件で終了コード 1）が緑、freeze_root の歯の全部（便 121 の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致を返す"
<!-- contracts:end -->
