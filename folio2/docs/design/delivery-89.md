# 設計: 便 89 — folio schema の歯の 6 本の正本の側を schema_docs.rs へ切り出す（受付の余地を空ける・振る舞いは不変・FR19）

- 要件: FR19（欄の決まりの file と天井の正本と規則の表の決まりの部分を床の定数から導出する）
- 条: P-10.1（検査は独立した凍結 anchor を 1 本以上持つ）/ P-6.2（生成物を手で直さない）/ P-3.1（機械で決定的に検査できる項目は床に置く）
- 出所: 台帳 f2-648.126 の段 1。便 88（docs/design/delivery-88.md §1 (g)）が「床の定数の人が読める写しの導出（P-5.6）は、生成区間の行数・byte 数・要約値を凍結する歯 crates/folio/tests/schema.rs の改訂を要するが、その file の余地は 97 行で size S の見積 100 にも足りない。先に tests/schema.rs の切り出しの便を 1 本置く」と自認している。本便はその 1 本で、便 87（docs/design/delivery-87.md）と同じ流儀で凝集した区間をそのまま移す。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cl が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き（新規は + を付けた 1 本・縮む file は - を付けた 1 本）。
- 門: 本便は design-intent の下を 1 file も書き換えない（触るのは crates/folio/tests/ の 2 本だけ）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

crates/folio/tests/schema.rs は命令 folio schema の歯で、命令が順に見る 8 本の file それぞれについて、生成区間の行数・byte 数・要約値の凍結の定数と、その file の側の歯（一致・ずれ・印・書き直し・凍結 anchor との byte 一致）を 1 file に持つ。器が便を受けるときに測る行数の上限（1500）に対する余地が 97 行しか残っておらず、この file に行を足す便は size S の見積（100 行）にも届かず断られる。本便は 8 本のうち、欄の決まり（schema）を持つ 2 本（adr/schema.yaml・design-note/schema.yaml）を除いた 6 本の正本の側を、新しい歯の file へ**そのまま移す**。字は 1 字も変えず、歯の関数名も凍結の定数の値も 1 つも変えない。

### (a) 実測（2026-09-22・main 7258db2・便 88 の着地後）

- 器の行数の式は「空行を含む全行を数え、1 行の字数（Unicode の字の数）が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す」。この式で測った tests/schema.rs は 生 1403 行・120 字を超える行 0・値 1403・余地 **97**。ほかの歯の file の余地 = tests/check.rs 638／tests/floor_cases.rs 740／tests/freeze.rs 1191／tests/ids.rs 1500 に近い（本便は触らない）。
- tests/schema.rs の中身は 5 つの層に分かれる。頭の注釈（1〜64 行・便ごとに歯の一覧）／取り込み（66〜71 行）／凍結の定数（73〜147 行）／helper（149〜388 行）／歯（389〜1403 行・#[test] は 40 本）。
- 凍結の定数は file ごとに閉じている。判断の記録の側 REGION_LINES・REGION_BYTES・REGION_SHA256（73〜78 行）／設計ノートの側 NOTE_REGION_*（80〜84 行）／天井の正本の側 CEILING_REGION_*（86〜90 行）／規則の表の側 RULES_REGION_*（92〜96 行）／入口の正本の側 INDEX_REGION_*（98〜102 行）／便 77 の 3 本の表 F77_REGIONS（104〜127 行）と変異の表 F77_DRIFTS（129〜140 行）／共通の TARGETS・BEGIN・END（142〜147 行）。そのほかに 便 85 の F85_RULES_ANCHOR（1259 行）と 便 86 の F86_SRS_*（1314〜1317 行）が歯の区間の中に在る。
- どの定数も、それを使う歯はその file の側の区間の中だけに在る（実測で全参照を数えた）。唯一の例外は共通の TARGETS・BEGIN・END と helper で、これは両方の側が使う。
- 取り込み use yaml_rust2::{Yaml, YamlLoader} を使うのは 便 86 の 2 本の歯（1339・1346・1358・1385 行）だけである（実測）。
- helper は 149〜388 行。repo_root／copy_tree／git／sha256_hex／struct Work と impl Work（new・dir・schema_yaml・note_schema_yaml・read・read_note・ceiling_yaml・read_ceiling・rules_yaml・read_rules・index_yaml・read_index・mutate_index・mutate・mutate_note・mutate_ceiling・mutate_rules・schema・check）／Drop／mutate_file／folio／stdout／stderr／assert_outcome／region。
- 歯の file どうしは互いに use できない（cargo は tests/ の直下の各 .rs を別々の crate として組む）。crates/folio/tests/ の下に dir は 1 つも無く（実測）、新規 dir は便で運べないので、共通の helper の module（tests/common/mod.rs の形）は採れない。この repo では helper の写しを持つのが既定の形で、repo_root は 34 本・copy_tree は 19 本・git は 15 本・sha256_hex は 6 本の歯の file が同じ helper の写しを持つ（tests/schema.rs の 194 行と 512 行の注釈も tests/bundle.rs・tests/ceiling.rs と同じ形であることを自認している）。本便も写しを採る。**写すのは helper だけで、凍結の定数と歯は 1 か所にしか置かない。**
- crates/folio/Cargo.toml に [[test]] の宣言は 1 つも無い（実測）。歯の file は cargo の自動の見つけ方で拾われるので、新しい file のために Cargo.toml を触る必要は無い（便 87 の tests/face_labels.rs・便 88 の tests/ids.rs も宣言なしで回っている）。
- 歯の関数名の接頭辞。grep -rn 'fn f89_' crates/folio/tests は今 0 本。

### (b) 新しい歯の file

crates/folio/tests/schema_docs.rs（新規・+）を作る。持つのは、命令が見る 8 本のうち 欄の決まりを持つ 2 本を除いた **6 本の正本**（ceiling.yaml = 天井の正本／rules.yaml = 規則の表／index.yaml = 入口の正本／srs.yaml = 要件書／vocabulary.yaml = 語彙／intake.yaml = 相談窓口の支度表）の側の歯と、その 6 本の凍結の定数である。tests/schema.rs から次の区間を**行の中身を 1 字も変えずに**移す（行番号は main 7258db2 の tests/schema.rs）。

1. 頭の注釈: 16〜20（便 48）・22〜26（便 53）・38〜42（便 76）・44〜50（便 77）・52〜54（便 78）・56〜59（便 85）・61〜64（便 86）。塊の間には区切りの `//!` の 1 行を置く（もとの file と同じ 4 字）。
2. 取り込み: 66〜69（std の 4 行）と 71（yaml_rust2 の 1 行）。
3. 凍結の定数: 86〜140（天井の正本・規則の表・入口の正本の 3 組と、便 77 の 3 本の表・変異の表。注釈の行を含む）。
4. 共通の定数: 142〜147（TARGETS の注釈と値・BEGIN・END）。
5. helper の写し: 148〜298（repo_root から Work の mutate_index まで）・309〜322（mutate_ceiling・mutate_rules・schema）・327（impl の閉じ）・328〜388（Drop・mutate_file・folio・stdout・stderr・assert_outcome・region）。**写しに載せないもの**: Work の mutate・mutate_note・check（新しい file の歯が 1 本も呼ばないので、載せると使われない定義として clippy が落ちる）。
6. 歯: 683〜930（便 48 の 4 本と便 53 の 4 本）・931（空行）・1009〜1403（便 76 の 4 本／便 77 の impl Work の read_file と 5 本／便 78 の 1 本／便 85 の 2 本／便 86 の 2 本と helper keys・strs）。移す歯は 22 本で、関数名は 1 つも変えない。

file の頭には、この file が何を持つかを言う注釈 4 行だけを新しく書く（6 本の file 名を並べ、便 89 で tests/schema.rs から 1 字も変えずに移したこと、helper は写しであることを言う）。

### (c) tests/schema.rs の側

(b) で移した区間を取り除く。残るのは次のとおりで、**残る行も 1 字も変えない**。

- 頭の注釈 1〜14（便 45 の歯 1〜7 と便 46 の歯 8〜11）・27〜36（便 57・便 58・便 69 の歯 20〜23）。
- 取り込み 65〜70（空行と std の 4 行）。**71 行の yaml_rust2 の取り込みは外す**（使うのは移す側の歯だけなので、残すと使われない取り込みとして clippy が落ちる）。
- 凍結の定数 73〜85（判断の記録の側と設計ノートの側）・142〜147（TARGETS・BEGIN・END）。
- helper 148〜269（repo_root から Work の read_note まで）・299〜308（mutate・mutate_note）・319〜333（schema・check・impl の閉じ・Drop）・334〜388（mutate_file から region まで）。**外すのは 270〜298（ceiling_yaml・read_ceiling・rules_yaml・read_rules・index_yaml・read_index・mutate_index の 7 本）と 309〜318（mutate_ceiling・mutate_rules の 2 本）**。この 9 本は残る歯が 1 本も呼ばないので、残すと使われない定義として clippy が落ちる。
- 歯 389〜566（歯 1〜7）・567〜682（便 46 の歯 8〜11）・932〜1007（便 57・便 58・便 69 の歯 20〜23）。残る歯は 18 本。

頭の注釈の末尾に、移し先を指す 2 行だけを新しく足す。1 行目は区切りの `//!`、2 行目は次の 1 行とする。

- //! 便 89（docs/design/delivery-89.md §1）: 残る 6 本の正本（天井の正本・規則の表・入口の正本・要件書・語彙・相談窓口）の側の歯は tests/schema_docs.rs へ移した（字は 1 字も変えていない）。

### (d) 振る舞いが変わらないことの確かめ

本便は crates/folio/src/ を 1 file も触らず、design-intent/ も tests/fixtures/ も 1 byte も触らない。歯の関数名・期待の字面・凍結の定数の値（行数・byte 数・要約値の 16 進）は 1 つも変えないので、40 本の歯の中身は 2 つの file に分かれるだけで、見るものも落ち方も同じである。席は切り出しを実際に当てて次を実測した（2026-09-22・main 7258db2 の作業ツリーで当てて戻した）。

- cargo clippy -p folio --all-targets -- -D warnings が 0 警告で通る（使われない取り込みも使われない定義も出ない = (b) 5 と (c) の外す一覧が過不足ない）。
- cargo nextest run -p folio --test schema --test schema_docs が 40 本すべて緑（tests/schema 18 本・tests/schema_docs 22 本）。本便の前の main の 40 本と、名前も本数も 1 つ違わない。
- 一時の置き場の名は Work::new が 名前 + 処理の id で組むので、2 つの歯の file を同時に回しても衝突しない（40 本を並べて回して実測）。

### (e) 切り出しの後の余地（器の式・実測）

| file | 本便の前 | 本便の後 | 余地 |
|---|---|---|---|
| crates/folio/tests/schema.rs | 1403 | 624 | 97 → **876** |
| crates/folio/tests/schema_docs.rs | 無し | 1033 | **467** |

移す割合は 1403 行のうち 782 行（約 56%）。どちらの file も size M の見積（300）を大きく上回る余地が残るので、台帳 f2-648.126 の段 2（床の定数の人が読める写しを生成区間へ導出する便・P-5.6）は、要件書の側（srs.yaml = 新しい file）でも判断の記録の側（adr/schema.yaml = tests/schema.rs）でも受けられる。

### (f) 歯（関数名は f89_ で始める。`grep -rn 'fn f89_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f88_）

新しい file crates/folio/tests/schema_docs.rs の末尾に、その file だけの区間として 1 本置く。

1. f89_schema_teeth_are_split_and_under_the_cap: 歯の側で器の式（空行を含む全行を数え、字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す・字数は Unicode の字の数）を持ち、次の 4 つを見る。
   - crates/folio/tests/schema.rs をこの式で数えた値が 700 以下（本便の後の実測は 624）。本便の前の main では 1403 なので赤い歯。
   - crates/folio/tests/schema_docs.rs を同じ式で数えた値が 1200 以下（本便の後の実測は 1033）。本便の前の main にはこの file が無いので、読めずに落ちる = ここでも赤い歯。
   - crates/folio/tests/schema_docs.rs に、移した定義の頭 8 本が行の先頭に在る。fn schema_check_matches_the_real_ceiling_file_and_its_frozen_digest(／fn schema_check_matches_the_real_rules_file_and_its_frozen_digest(／fn f76_schema_check_matches_the_real_index_file_and_its_frozen_digest(／fn f77_regions_match_the_frozen_anchors(／fn f86_srs_region_matches_the_new_anchor(／const CEILING_REGION_BYTES:／const RULES_REGION_SHA256:／const F77_REGIONS:
   - crates/folio/tests/schema.rs に、その 8 本の頭が 1 つも無い。
   file の読みは歯の file の位置（CARGO_MANIFEST_DIR）から repo の根を辿る、この file がすでに持つ helper repo_root をそのまま使う（便 87 の f87_ と同じ形）。
2. 回帰（期待不変）は verify の 2 行目で見る。tests/schema.rs と tests/schema_docs.rs の歯すべて（41 本 = 18 + 22 + f89_ の 1 本）。ほかの歯の file は 1 本も触らないので、共通の検証（.vessel.toml の common-verify = workspace 全体の nextest）で回る。器の受付は verify の旗 --bin の次の語を filter 語と読むので、verify の行に --bin は書かない。

### (g) 大きさ

src は 1 file も触らない。歯は 2 本で、crates/folio/tests/schema.rs（器の式で 1403・余地 97・782 行ぶんが取り除かれ、指し先の 2 行（式では 3）が増えるので 624・余地 876）と、新規 crates/folio/tests/schema_docs.rs（1033・余地 467）。本便が新しく書く行は、新しい file の頭の注釈 4 行・区切りの `//!` 6 行・f89_ の区間 49 行と、tests/schema.rs の指し先 2 行の合わせて 61 行で、残りはすべて字を変えない移動である。size **S**（増える file の余地は 467・減る file は - を付けて宣言する）。外部 crate は増やさない。Cargo.toml・CI の yml・crates/folio/src/・design-intent/・tests/fixtures/・tests/floor_cases.yaml・ほかの歯の file は 1 file も触らない。

### (h) 本便が運ばないもの

字の書き換え・歯の関数名の変更・凍結の定数の値の変更・歯の統合や分割・期待の字面の変更・helper の書き換え（写しは (b) 5 の一覧のとおり字を変えずに写す）。床の定数の人が読める写しの導出（台帳 f2-648.126 の段 2・本便の次の便）。ほかの歯の file の切り出し（tests/check.rs 余地 638・tests/floor_cases.rs 余地 740 はまだ足りているので触らない）。tests/schema.rs に残る側のさらなる分割（判断の記録の側と設計ノートの側は余地 876 なので分けない）。歯の file どうしで helper を 1 か所に集める形（新規 dir が要るので運べない＝(a) の実測）。

受付の cap への断り: tests/schema.rs は本便で行が減る file なので、write-set では縮む file の印（-）を付けて宣言する。増える file は新規の tests/schema_docs.rs 1 本で、その余地は 467 なので size S の見積（100）を満たす。

## 2. 範囲

- 入れる: 新しい歯の file 1 本・782 行ぶんの取り除きとそのままの移動（helper は写し）・使われなくなる取り込み 1 行と定義 9 本の取り除き・頭の注釈の指し先 2 行・f89_ の歯 1 本。
- 入れない: 字の書き換え・歯の関数名や凍結の定数の値の変更・src・design-intent・fixture・Cargo.toml・CI の yml・ほかの歯の file・床の定数の写しの導出。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| file | 新しい歯の file | crates/folio/tests/schema_docs.rs（6 本の正本の側の歯 22 本と凍結の定数） |
| helper | helper の写し | repo_root から region まで（歯の file どうしは互いに use できない） |
| trim | 取り除き | tests/schema.rs から yaml_rust2 の取り込み 1 行と Work の 9 本 |
| pointer | 指し先 | tests/schema.rs の頭の注釈の 2 行 |
| teeth | 歯 | crates/folio/tests/schema_docs.rs の f89_ 1 本 |

## 4. 検査（歯）

§1 (f) のとおり。共通の検証は .vessel.toml の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cl"
title = "folio schema の歯 tests/schema.rs のうち、命令が見る 8 本から欄の決まりの 2 本を除いた 6 本の正本（天井の正本・規則の表・入口の正本・要件書・語彙・相談窓口）の側の歯 22 本と凍結の定数を、新しい歯の file tests/schema_docs.rs へ 1 字も変えずに移す（歯の関数名も定数の値も不変・歯の総数は 40 から 41 へ・器の式で tests/schema.rs の余地を 97 行から 876 行へ空け、台帳 f2-648.126 の段 2 が受けられるようにする）"
req = ["FR19"]
section = "1"
write-set = ["-crates/folio/tests/schema.rs", "+crates/folio/tests/schema_docs.rs"]
verify = ["cargo nextest run -p folio --test schema_docs f89_", "cargo nextest run -p folio --test schema --test schema_docs", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f89_ の歯 1 本（器の式で tests/schema.rs が 700 行以下・tests/schema_docs.rs が 1200 行以下・移した 8 本の頭が新しい file に在り tests/schema.rs に無い）が緑、tests/schema.rs と tests/schema_docs.rs の既存の歯が 2 つの file を合わせて全部緑（本便の前の 40 本と名前も本数も違わない 40 本 + f89_ の 1 本 = 41 本）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
