# 設計: 便 134 — 列の根の表の 2 行目の鍵を tsuzuri-constitution に改める（利用者の改名・digest は変わらない）

- 要件: FR23（憲法の列の根を、憲法の名ごとの表で照らす・要件書 第 1.41 版）。FR23 の注は「表に行を足すのは folio2 の便で、契約表の行か設計ノートが利用者の持ち主の承認の裁定 id を名指す（規則の表の行 D-11 の手順）」と書く。本便は FR23 の規範文を変えず、便 133 で足した 2 行目の鍵（憲法の名）だけを、利用者の改名の後の名に改める。契約表の行の req は FR23 の 1 つ（main に在る id）。
- 条: P-5.1（閉じた一覧は型付きデータ＝表は実装の型付きの定数）/ P-7.1（id を再利用・改番しない＝古い名の行を残して 2 つの名で同じ列を指させない）/ P-10.1（独立した凍結 anchor＝鍵と digest を歯の中の手で写した字で持つ）/ N-3.1（例外の口を足さない＝検査される側の data は行を選べるだけ）。
- 出所: 便 133（行 `ef`・台帳 f2-648.202・本流 3269ff4 に着地）が、利用者（器の次の世代の設計の置き場・判断の記録 ADR-21 の読み替え）の憲法の行を、鍵 scribe3-constitution・値 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 で足した。その直後に、利用者の持ち主が project の名を scribe3 から tsuzuri に改めた。利用者の判断の記録 ADR-6（利用者の repo の版 1c93e70・発効）の承認欄は、承認者 = 持ち主・日付 2026-09-25・裁定 id = scribe2 台帳 **s2-07l.214**（裁定 id = user 2026-09-25T00:39Z・notes に逐語）・逐語「プロジェクトの名前はtsuzuriにする。ただしCLIコマンドはもっと短くしたい」・対話面 R-8 である。同じ版で利用者の憲法の meta.id が tsuzuri-constitution に改まった（憲法の凍結の前）。**規則の表の行 D-11 が名指しを求める利用者の持ち主の承認の裁定 id は、行の中身（digest）については便 133 と同じ s2-07l.214（user 2026-09-24T22:39Z・逐語「これでよい」・利用者の憲法 第 1.0 版の承認）、鍵の改名については同じ s2-07l.214（user 2026-09-25T00:39Z・ADR-6 の承認）である。** 依頼は席から起草役へ（2026-09-25）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `eg` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 3 本（便 133 と同じ 3 本）。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 3 本を base の binary で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は便 133（列の根の表の 2 行目）の着地。**base = main 94e6d3e（起草は 3269ff4 の上・席が 94e6d3e へ載せ替え・独立の検証は 94e6d3e で実測）。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 起草の時点で受付を待つ便のうち、write-set が重なる便は無い。共通の検証は同時に撃たない逐次を勧める（`crates/folio/tests/sheet.rs` の一時 dir の衝突を避けるため）。

## 1. 設計

### (a) いま起きていること（実測・base 3269ff4 → 94e6d3e で検証役が撃ち直し）

1. **表の 2 行目の鍵は改名の前の名。** 床の定数の列の根の表（`crates/folio/src/floor_adr.rs` の ROOT_DIGESTS）は、1 行目が鍵 folio2-constitution・値 acb52acd…、2 行目が鍵 scribe3-constitution・値 35eb6b36…（便 133）の閉じた一覧である。引く口は `crates/folio/src/adr.rs` の root_digest で、床の照らしと凍結の命令が共有する。表の写しは、判断の記録の欄の決まりの生成区間（adr/schema.yaml の anchor の節の欄 root_digests）へ、置き場の憲法の名の行だけが導出される（床の木の変種 Pick）。
2. **利用者の置き場は凍結できない。** 利用者の repo（今の置き場 ~/projects/local-projects/scribe3・近く ~/projects/local-projects/tsuzuri に改名される・folio2 の外・読むだけ）の憲法は、版 1c93e70 から名 tsuzuri-constitution・版 v1.0・status effective・binding true で、承認欄は便 133 の時点と同じである。版 e52d24c から HEAD までの憲法の差は、頭の注の 1 行（YAML の注で木の外）と meta.id の 1 行だけで、anchors の dir はまだ無い（凍結の前）。起草役が利用者の各版の design-intent を `git archive` で一時 dir に取り出し、git の init と 1 commit をした写しに base の binary を撃った結果は次のとおり。

| 利用者の版 | 命令 | 終了コード | 出力の要点 |
| --- | --- | --- | --- |
| 1c93e70（改名）と 5b792cc（起草の時点の HEAD） | folio check（旗なし） | 2 | 違反 0・まだ分からない 2（写しの列の根の欄は空の表で、置き場の名の行は表に無いので Pick も空＝一致） |
| 同上 | folio check --freeze-start | 1 | 列の根の凍結だが憲法の名 tsuzuri-constitution が列の根の表に無い＝凍結しない・組んだ木の digest 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 |
| e52d24c（便 133 の出所） | folio check --freeze-start | 1 | 違反 1（写しの列の根の欄が空の表で、床の定数の scribe3-constitution の行と違う） |

3. **digest は改名で変わらない（2 実装）。** 憲法の名は凍結の木の digest の外に在る（ADR-16 の帰結）。起草役は folio の code を呼ばない python の script（持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d134-scripts/root_digest.py`・便 133 の script の写し）で、利用者の 3 つの版（e52d24c・1c93e70・5b792cc）の憲法から始まりの凍結の木を組み、3 つとも 35eb6b36…（全桁が binary の出力と一致）を得た。同じ script で folio2 の凍結 anchor 5 本（v1.0〜v1.4）の digest の欄を撃ち直し、5 本とも一致した（script の正しさの確かめ）。
4. **base の歯（参考値）。** workspace の nextest 867 / 867（3269ff4・94e6d3e では 868 / 868 = 便 132 の歯 1 本が増えた）・clippy 0 警告・床 4 本 rc 0。`git grep -n 'f134_' -- crates` は 0 件。便 133 の歯は f133_ の 2 本（`crates/folio/src/adr.rs` の単体の歯 f133_the_root_table_holds_folio2_and_scribe3 と `crates/folio/tests/freeze_root.rs` の f133_scribe3_name_picks_its_row）で、どちらも鍵 scribe3-constitution を字で持つ。

### (b) 直す先 — 2 行目の鍵を改める

1. **表（floor_adr.rs）。** ROOT_DIGESTS の 2 行目の鍵を scribe3-constitution から tsuzuri-constitution に改める。値（digest の全桁）は 1 字も変えない。scribe3-constitution の行は残さない（表はちょうど 2 行のまま）。folio2 の行は 1 字も変えない（FR23）。表の頭の注の 2 行目の出所の文を、tsuzuri の憲法の行とし、旧 scribe3 の版 e52d24c の名で足して版 1c93e70 の改名で鍵を改めた（便 134）史実を残す形に直す。
2. **ほかの code は変えない。** 引く口・床の照らし・凍結の命令・写しの導出（Pick）は、表の鍵の字に依らない形のまま（便 121）。
3. **写しの全数（find で列挙）。** repo の中の判断の記録の欄の決まり（`adr/schema.yaml`）は 18 本（実の置き場 1・床の凍結の土台 1・fixture 16）。置き場の憲法の名は folio2-constitution が 2 本・fixture-constitution が 16 本で、scribe3-constitution の置き場も tsuzuri-constitution の置き場も 0 本である。写しは置き場の名の行だけなので、18 本とも 1 byte も変わらない。`folio schema --dir design-intent --check` は一致のまま。
4. **folio2 の床と面は変わらない。** folio2 の置き場の床の結果・`folio build` の出力（30 file）は base と 1 byte も変わらない（起草役の実測）。

### (c) 歯（便 133 の 2 本を改める・関数名を f134_ に）

関数名は f133_ から f134_ に改める（verify の絞り込みの語・base で 0 件）。関数名にも改名の前の名が入っているので、名と中身を同じ便で揃える。

1. **f134_the_root_table_holds_folio2_and_tsuzuri（単体の歯・`crates/folio/src/adr.rs` の既存の tests の区間・f133_the_root_table_holds_folio2_and_scribe3 を置き換える）。** 表 ROOT_DIGESTS がちょうど 2 行で、並び・鍵・値の全桁が (b) の 1 のとおり（folio2 の行が先・2 行目の鍵が tsuzuri-constitution）であることを等しさで数え、口 root_digest が tsuzuri-constitution から 2 行目の値を引き、folio2-constitution から 1 行目の値を引き、scribe3-constitution からは何も引かない（無い）ことを数える。鍵と digest は歯の中の手で写した字で持つ（凍結 anchor・P-10.1）。**base では 2 行目の鍵が違って落ちる＝RED。**
2. **f134_tsuzuri_name_picks_its_row（`crates/folio/tests/freeze_root.rs`・binary 経由・f133_scribe3_name_picks_its_row を置き換える）。** 実の design-intent の写しの憲法の名を tsuzuri-constitution に書き換え、`folio schema --write` → 終了コード 0・欄の決まりの写しの列の根の欄がちょうど 2 行「root_digests:」と「tsuzuri-constitution: <digest の全桁>」で、folio2 の digest が写しに無い。commit の後に `folio check` → 終了コード 1・違反はちょうど 1 件で、種別 anchor・字「列の根の表の tsuzuri-constitution の行」を持ち、字「表に無い」を持たない。歯の file の定数 SCRIBE3 と SCRIBE3_ROOT は TSUZURI と TSUZURI_ROOT に改め（値の digest は同じ）、file の頭の注の 8 の項も tsuzuri の名に直す（便 133 の出所の記述は残す）。**base では表に tsuzuri の名が無く、写しの欄が空の表で落ちる＝RED。**

改名の前の名 scribe3 の字は、code の中では出所の史実（旧 scribe3・版 e52d24c・版 1c93e70）と、歯 1 の無いことを数える鍵にだけ残る。

### (d) 採らなかった形

1. **scribe3-constitution の行を残し、tsuzuri-constitution の行を 3 行目に足す。** 同じ digest の列を 2 つの名が指すことになり、利用者が古い名に戻しても床が通す口が残る（P-7.1 の向き・N-3.1）。利用者はまだ凍結していない（anchors の dir が無い）ので、古い名の行を使う列は無い。
2. **関数名 f133_ を残して中身だけ改める。** 関数名に改名の前の名（scribe3）が残り、verify の絞り込みの語が本便の歯と便 133 の歯を区別しない。f134_ に改めると、base で 0 件の語で本便の歯だけを撃てる。

### (e) 既存の歯のうち落ちるもの・利用者の側の手順

1. **落ちる既存の歯は、置き換える便 133 の 2 本だけ（起草役の実測）。** 本便の差分を base に当てた写しで、workspace の nextest は 867 / 867（base と同じ本数・f133_ の 2 本が f134_ の 2 本に替わる）・clippy 0 警告・床 4 本 rc 0・`crates/folio/tests/freeze_root.rs` 8 / 8。便 121 の歯（freeze_root の 7 本）は緑のまま。
2. **突然変異（起草役の実測・本便を当てた写しの表だけを変える）。**

| 変異 | 単体の歯 f134_ | freeze_root の f134_ |
| --- | --- | --- |
| 2 行目の鍵を scribe3-constitution のまま | 落ちる | 落ちる |
| digest の末尾の 1 桁を違える | 落ちる | 落ちる |
| scribe3-constitution の行を残して 3 行にする | 落ちる | 緑（写しは置き場の名の行だけ） |

3. **利用者の側の手順（着地の後・利用者の持ち主の手番・起草役の模擬）。** 本便を当てた binary を利用者の版 5b792cc（改名の後）の写しに撃つと、次の順で合格まで届く。

| 段 | 命令 | 結果 |
| --- | --- | --- |
| 1 | folio check | 終了コード 1・違反 1（写しの列の根の欄が空の表で、床の定数の tsuzuri の行と違う）・まだ分からない 2 |
| 2 | folio schema --write | 判断の記録の欄の決まりの写しだけを書き直す（列の根の欄に tsuzuri-constitution の行・ほかの file は変わらない） |
| 3 | commit の後に folio check --freeze-start | 合格 0/0 の上で始まりの凍結をした（憲法の anchor v1.0・id の一覧 v0.1・索引・条 38・id 6 本） |
| 4 | commit の後に folio check | 終了コード 0・合格（違反 0・まだ分からない 0）・索引の digest は 35eb6b36… |

   利用者の置き場は利用者の repo の持ち物で、folio2 の便は書かない。段 2〜4 は利用者の側の手番として、席が利用者の設計席へ返す。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 3 本とも印なし（`crates/folio/src/floor_adr.rs`・`crates/folio/src/adr.rs`・`crates/folio/tests/freeze_root.rs`）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/floor_adr.rs` | 518 | 982 | 519（+1） |
| `crates/folio/src/adr.rs` | 956 | 544 | 958（+2） |

   2 本とも余地は S の見積 100 を超える。歯の file は src の外なので余地を測らない（参考値 509 行 → 511 行）。
3. **size は S。**
4. **verify は 4 行**で、done の 4 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --bin folio f134_` = (c) の 1。
   2. `cargo nextest run -p folio --test freeze_root f134_` = (c) の 2。
   3. `cargo nextest run -p folio --test freeze_root` = 列の根の表と始まりの凍結の歯の全部（便 121 の 7 本を含む・参考値 8 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は freeze_root の 1 本で write-set に在る。`--bin folio` の絞り込みの語 f134_ を関数名に持つ src は `crates/folio/src/adr.rs` だけで write-set に在る。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付。** 受付の先撃ち（precheck）で契約に起因する断りは 0（起草役の実測）。共通の検証を同時に撃たない逐次で受け付ける。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d134-draft.md`、模擬の差分と script は同じ dir の d134-scripts（repo には入れない）。

1. digest: 利用者の repo で `git archive <版> design-intent` を一時 dir に展開し、git の init と 1 commit の後に base の binary で `folio check --dir design-intent --freeze-start` を撃つ。独立の再現は root_digest.py の root の口（憲法の正本と欄の決まりの file を渡す）。script の正しさは check-anchor の口で folio2 の凍結 anchor 5 本を撃ち直して確かめる。
2. 模擬: 差分 c134.patch を base に当て、run-134.sh（workspace の nextest・clippy・床 4 本・`folio build --write` の出力の file 数と base との差）・mut-134.sh（(e) の 2 の突然変異）・tsuzuri-4steps.sh（(e) の 3 の 4 段）。
3. RED: 歯の 2 か所だけの差分 r134-teeth.patch を base に当てると、f134_ の 2 本とも落ちる。
4. 余地: lines-134.py と lines-134.awk（同じ式の 2 実装）を repo の根で write-set の file に当てる。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 利用者の置き場の写しの書き直しと凍結（利用者の側の手番・(e) の 3）。表の 3 行目以降。folio2 の設計文書の字（判断の記録と要件書の散文の scribe3 の字は史実の記述で、直すなら次の一括で席が書く）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 表が証するのは列の根の中身で、どの repo の列かは証さない（ADR-16 の帰結）。利用者の repo の dir が ~/projects/local-projects/tsuzuri に改名されても、表は名と digest しか持たないので本便の結果は変わらない。利用者の憲法が凍結の前に木の中で変わると digest が変わり、凍結の命令は「行と違う」で断る（その時は行を直す便が要る）。本便の着地の後は、改名の前の名のままの利用者の写し（例えば版 e52d24c）は「表に無い」で凍結できない。
3. **撤退条件。** (1) 受付の時点で、利用者の repo（~/projects/local-projects/scribe3 か、改名の後の ~/projects/local-projects/tsuzuri）の HEAD の constitution.yaml の meta.id が tsuzuri-constitution でないか、HEAD の design-intent の写しに base の binary で撃った --freeze-start の digest が (b) の 1 の値と違えば、止めて席へ返す（鍵か値を取り直してから運ぶ）。design-intent の他の file（判断の記録・設計ノート・語彙等）の差は止めない。(2) 本便の後に folio2 自身の床の結果か `folio build` の出力か 18 本の欄の決まりの写しが 1 byte でも変わったら、止めて席へ返す。(3) 本便の後に、置き換える便 133 の 2 本のほかの既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/floor_adr.rs` の表の 2 行目の鍵と頭の注の出所の文。`crates/folio/src/adr.rs` の単体の歯 1 本（置き換え）。`crates/folio/tests/freeze_root.rs` の定数 2 つ・頭の注の 8 の項・歯 1 本（置き換え）。
- 入れない: 表の値・folio2 の行・引く口・床の照らし・凍結の命令・写しの導出・設計文書・fixture・利用者の repo・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| row | 列の根の表の 2 行目の鍵 | `crates/folio/src/floor_adr.rs` の ROOT_DIGESTS（tsuzuri-constitution・digest は不変） |
| unit | 表の単体の歯 | `crates/folio/src/adr.rs` の f134_（2 行の並び・鍵・値の等しさ・古い名が引けないこと） |
| teeth | 写しと床の歯 | `crates/folio/tests/freeze_root.rs` の f134_（写しの導出と床の照らしを binary で） |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: 便 133（列の根の表の 2 行目・本流 3269ff4）。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。利用者の設計席へ (e) の 3 の段 2〜4 を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "eg"
title = "便 133（行 ef・台帳 f2-648.202）の後の利用者の改名の便: 床の定数の列の根の表 ROOT_DIGESTS（crates/folio/src/floor_adr.rs）の 2 行目の鍵を scribe3-constitution から tsuzuri-constitution に改める（値 35eb6b369f0504167571a27b50c950e1361609d9b19b71e0f1e9de832f8c5356 は不変・名は digest の外・scribe3-constitution の行は残さない）。利用者の持ち主の承認の裁定 id = scribe2 台帳 s2-07l.214（行の中身は user 2026-09-24T22:39Z 逐語 これでよい・改名は利用者の判断の記録 ADR-6 の承認 user 2026-09-25T00:39Z）。folio2 の行・引く口・床の照らし・凍結の命令・写しの導出（置き場の名の行だけ）・設計文書は変えない。歯は便 133 の f133_ の 2 本を f134_ の 2 本（表の単体の歯と、tsuzuri の名に書き換えた実の置き場の写しで folio schema が tsuzuri の行を写し床が表の行と照らす歯）に置き換える"
req = ["FR23"]
section = "1"
write-set = ["crates/folio/src/floor_adr.rs", "crates/folio/src/adr.rs", "crates/folio/tests/freeze_root.rs"]
verify = ["cargo nextest run -p folio --bin folio f134_", "cargo nextest run -p folio --test freeze_root f134_", "cargo nextest run -p folio --test freeze_root", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "表の単体の歯 f134_（表がちょうど 2 行で、並び・鍵・digest の全桁が §1 (b) の 1 のとおり・口が tsuzuri の名から 2 行目の値を引き scribe3 の名からは何も引かない）が緑、freeze_root の f134_ の歯（憲法の名を tsuzuri の名に書き換えた実の置き場の写しで folio schema --write が列の根の欄を tsuzuri の行 1 行にし、commit の後の folio check が列の根の表の tsuzuri の行と違う の違反ちょうど 1 件で終了コード 1）が緑、freeze_root の歯の全部（便 121 の歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致を返す"
<!-- contracts:end -->
