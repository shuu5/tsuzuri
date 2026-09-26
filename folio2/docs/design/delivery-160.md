# 設計: 便 160 — tests/sheet.rs の一時 dir を process ごとに一意にし、歯の終わりに消す（台帳 f2-648.237・S）

- 要件: FR1（相談窓口）と FR8（途中で足す窓口）の歯の file の直し。規範文・確かめ方・受入基準 AC1 と凍結 anchor（tests/fixtures/intake/ の期待 2 本）は変えない。
- 条: P-3.1（床は機械で決定的に検査できる項目）・P-10.1（凍結 anchor の歯は byte 一致のまま）。
- 出所: 台帳 **f2-648.237**（便 157 の起草役の報告・並行の 2 run で歯が 1 本落ちた実測 d157-scripts/base-nextest-race.log）と、その追記（便 158 の検証役の workspace の nextest で ceiling_viewpoint_id_off_the_list_fails が 1 回だけ落ちた）。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `fg` が指す §1 だけ。write-set は tests の 1 本で、src も新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d160 の一番上で base と本便の写しの binary に write-set を渡すと、どちらも **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main 346f825（便 158 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。便 159（inject.rs・main.rs・tests/inject.rs・tests/init.rs）と便 161（note.rs 系）とは write-set が重ならない。
- 改訂 b: 独立の検証（d160-verify.md）の B-1 と非 blocking 1・2・3・5 を採った。子は path を標準エラーに出し、子の役は印の値が親の process の id と一致するときだけにした（(c)）。歯の限り（(d) の 3・(e) の 3）・居残りの字（(i) の 2）・数え直しの TMPDIR（(i) の末尾）を直した。非 blocking 4（落ちたときに写しを残す）は 21 本の先例に揃えて採らず、6（req に FR2）も採らない。write-set・歯の本数・verify・size・門・base は変わらない。

## 1. 設計

### (a) いま起きていること（参考値・base 346f825）

1. **固定の名の一時 dir。** tests/sheet.rs の口 work（25 行目）は、一時 dir を名 folio-sheet-〈case〉 で作る。作る前に同じ名を消して design-intent を写し、歯の終わりにも消さない。歯 9 本がどれもこの口を通る（case の名は 13）。
   - 同じ host の別の写しで同じ歯が同時に走ると、後の方が先の方の写しを消して写し直す。先の方は支度表か intake.yaml を見失って落ちる。
   - 名は写しの置き場に依らない。器の run・起草役・検証役の nextest が重なると起きる。
2. **再現（repro-160.sh）。** base の sheet の test binary を 2 つの process で同時に撃つ（2 つの写しが同時に撃つのと同じ形）。**30 回のうち 29〜30 回で落ちた**（改訂 a と b で 1 度ずつ撃った）。落ちる歯は回ごとに違い、例は rc 2「intake.yaml: 読めない」。本便を当てた写しでは 30 回とも落ちない。
3. **全数（census-160.py・`git grep -n 'temp_dir()' -- crates` の全ての当たり 55 件）。**
   - process の id を名に持つ: 50 件（tests 46・src の #[cfg(test)] 3・src の本体 1 = figure.rs は id と時刻）。
   - 固定の名: 5 件。**作るのは sheet.rs の口 work の 1 件だけ**。残り 4 件は作らない path で、ぶつからない（sheet.rs の無い置き場・face.rs・face_adr.rs・face_note.rs の旗の誤りで書かれない出力先）。
   - 口に字のまま渡す case の名が同じ file の中で重なる所は 0。
4. **揺れの追記（ceiling_viewpoint_id_off_the_list_fails）。** 原因は突き止められなかった＝本便では直さない（(i) の 1）。
   - 形が違う: tests/ceiling.rs の口は名に process の id を持ち、歯の終わりに消す。
   - 時間に依る道: src の時刻と待ちの全数のうち、folio check の中で時間に依るのは、git の命令 1 本ごとの待ち上限 20 秒（gitcheck.rs）だけ。超えるか子 process を起こせないと まだ分からない（rc 2）になり、この歯は落ちる。
   - 記録: 落ちた回の本文（panicked の行）は検証役の記録に無い。
   - 再現の試み（flake-160.sh）: base の ceiling の test binary を 2 つの process で同時に 100 回撃った（機械の負荷 31〜68・参考値）。**落ちた回は 0**。
5. **base の歯。** workspace の nextest 977 / 977・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file・2146076 byte。`--test sheet` は 9 本。`f160_` と行 id `fg` は 0 件。

### (b) 直す先

1. **tests/sheet.rs の口 work だけ。**
   - 名に process の id を足し、folio-sheet-〈case〉-〈id〉 にする（ほかの 46 か所と同じ形）。作る前に同じ名の前の回の残りを消すのは今のまま。
   - 返す型を、同じ file に足す包みの型 Work にする。PathBuf を 1 つ持ち、Deref で Path として読め、Drop で dir を消す（ほかの 21 本の tests の口 Work と同じ役）。
2. **呼び出しの字。** 変わるのは 1 か所だけ（旗の誤りの歯で Command の arg へ渡す dir を &*dir にする）。ほかの呼び出しは Deref でそのまま読める。
3. **変えないもの。** 歯 9 本の期待の字・凍結 anchor・fixture・src・無い置き場の名（sheet.rs の 252 行目）・ほかの tests の口。folio の出力は変わらない（src に触れない）。

### (c) 歯（関数名 f160_・`crates/folio/tests/sheet.rs`）

1. **f160_another_process_leaves_this_copy_alone。**
   - 親: work で case f160-same の写しを作って印の file を 1 つ置く。同じ process の case f160-other の写しとは path が違うことを見る。
   - 子: 同じ test binary をこの歯 1 本だけで撃ち直す（current_exe と --exact）。環境変数 FOLIO_F160_CHILD の値が親の process の id と一致するときだけ子の役で、同じ case の写しを作り、path を**標準エラーに** 1 行出して終える。同じ host の別の写しで同じ歯が走るのと同じ形である。
   - 標準出力に出さないのは、test のスレッドが 1 つのとき libtest が標準出力に歯の名の行を改行なしで先に書き、path の行の頭が崩れるため。外の環境に印が在っても、値が親の id でなければ親として走る（何も見ずに緑で終えない）。
   - 見ること: 子が rc 0・子の path が親と違う・親の印が残る・子の写しが子の終わりに消えている。
   - 親と子は順に走るので、時刻にも負荷にも依らない。test のスレッド数（RUST_TEST_THREADS=1）と CPU の数（taskset -c 0）にも依らない（det-160.sh の撃ち方 10 通りで緑）。
   - **base では子の path が親と同じ（/tmp/folio-sheet-f160-same）で落ちる＝RED**（red.log）。
2. 歯の file の頭の注に 1 行足す。fixture は足さない。

### (d) 採らなかった形

1. **名に id を足すだけで消さない。** 撃つたびに 13 の dir（22 MB）が /tmp に居残る（今は同じ 13 を上書きする）。
2. **外部 crate の一時 dir（tempfile）。** 依存を足すので A-3.1 の持ち主の確認が要る。id の形は同じ repo に 46 の先例がある。
3. **歯を名の形（末尾が id）で縛る。** 時刻の nanos や OS の一時 dir を使う正しい別解まで落とす。本便の歯は「別の process の同じ歯が自分の写しを消さない」性質を縛る（(e) の M7 が生き残る）。
   - 歯の限り: 歯は順に走る 2 つの process を見る。同じ瞬間の 2 つの process でしかぶつからない名（ミリ秒の時刻・共通の親の process の id）は歯を抜ける（(e) の M8・M9）。実の並行は記録の再現の script で見る。
4. **2 つの写しの nextest の並行 run を歯にする。** 時刻に依り、重い。再現の撃ち方は記録の repro-160.sh に残した。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は 0 本。** 本便を当てた写しで workspace の nextest 978 / 978・clippy 0 警告・床 4 本 rc 0・`--test sheet` 10 / 10。rustfmt --check の差の塊は base と同じ 0。
2. **RED。** 歯だけを base に当てると f160_ の 1 本が落ち、ほかの 9 本は緑。
3. **突然変異。** 写しの口 work と型 Work を 1 通りずつ変えて `--test sheet` を撃った（mut-160.py）。落ちる歯はどれも f160_ の 1 本で、括弧はどの見ることで落ちたか。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 名から id を外す（base の名）/ M2 id の代わりに固定の字 / M6 id の代わりに thread の id | f160_（子の path が親と同じ） |
| M3 Drop で消さない | f160_（子の写しが残る） |
| M4 名から case を外す（id だけ） | f160_（同じ process の別の case と同じ path） |
| M5 作る前に同じ case の残りを id を問わず全部消す | f160_（親の印が消える） |
| M7 id の代わりに時刻の nanos（正しい別解） | 生き残る |
| M8 id の代わりに時刻のミリ秒 / M9 id の代わりに親の process の id | 生き残る（歯の限り） |

   M7 が生き残るのは、歯が名の形でなく性質を縛るため。M8・M9 は (d) の 3 の歯の限りで、順に走る 2 つの process では名が違う。
4. **撃ち方に依らない。** 本便の歯は RUST_TEST_THREADS=1・taskset -c 0 を含む 10 通りで緑（det-b.log）。改訂 a の歯（子が標準出力に出す形）は、このうち 3 通りで「子が path を出さない」で落ちた（det-a.log）。

### (f) 大きさ・verify と done の対応

1. **write-set。** 1 本・印なし（書き換えるだけ）: `crates/folio/tests/sheet.rs`。
2. **余地（CapHeadroom）。** 触る src は無い（write-set に `crates/folio/src/` の下の file が無い）。src の外の参考値は tests/sheet.rs 290 → 364（各行 ceil(字数 / 120)・空行は 1・python と awk の 2 実装で一致）。
3. **size は S。** 歯の file 1 本の口の直しと歯 1 本。
4. **verify は 3 行**で、done の 3 の塊と 1 対 1。
   1. `--test sheet f160_` = (c) の 1。
   2. `--test sheet` = 歯の file の全部（凍結 anchor の歯を含む・参考値 10 本）。
   3. clippy。

   base では 1 が 0 件で終了コード 4、2 は 9 本で緑、3 は 0 警告。`--bin` の行は無い。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0。便 159・161 とは file が重ならない。

### (h) 今の置き場の床が変わらないこと

src に触れないので folio の出力は変わらない。念のため base と本便の写しの binary で folio2 自身の床 4 本と `folio build --write` を撃ち、rc は 4 本とも 0 で同じ、build の出力は diff -r で一致した（floor-160.sh）。

### (i) 運ばないもの・撤退条件

1. **揺れ（ceiling_viewpoint_id_off_the_list_fails）は運ばない。** 原因は まだ分からない（(a) の 4）。席へ返す。
   - 次に落ちたら、落ちた歯の本文（panicked の行と標準エラー）を残す。検証役の変異の script は、落ちた歯の名だけでなく本文も書く。
   - 本文が rc 2 の まだ分からない（git の待ち上限か子 process を起こせない）なら、待ち上限の扱いを別の台帳の項で問う。
2. **落ちた歯の写しの居残りは運ばない。** Drop を持たない口（sheet.rs を除いて 22 本の file）は、緑の歯では終わりに消すが、落ちた歯の写しは残す。/tmp の folio- の居残り（参考値 465〜571 個・約 600 MB）は、変異と検証の run で落ちた歯の分である。名に id を持つのでぶつかりはしない。台帳に起こすかは席が決める。
3. **撤退条件。**
   - (1) 既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。
   - (2) 着地の後の main で、folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら止めて席へ返す。
   - (3) 受付の時点の main で tests/sheet.rs の口 work か歯の本数が base と違えば、下の手順で数え直してから運ぶ。

数え直しの手順（行 D-13）: 記録は `.local/share/folio2/handoff-2026-09-27/d160-draft.md`、script は同じ dir の d160-scripts（repo の外）。写し = clone-160.sh・build-sim-160.sh（apply-160.py・apply-160-teeth.py → c160b.patch・r160b-teeth.patch・改訂 a は c160.patch）、全数 = census-160.py、再現 = repro-160.sh・flake-160.sh、撃ち方 = det-160.sh・det-a-160.sh、検査 = suite・verify・red・mut・lines・fmt、置き場 = floor、門 = gate（各 -160）。歯を撃つ script は、一時 dir を自分の TMPDIR（写しの置き場の tmp/）に向けて撃つ。base の口は名が固定なので、/tmp で撃つと同じ時刻の他の agent の base の run とぶつかる。

## 2. 範囲

- 入れる: §1 (b) の口 work と型 Work、(c) の歯 1 本と頭の注 1 行。
- 入れない: src・fixture・凍結 anchor・ほかの tests の口（(i) の 2）・ceiling の揺れ（(i) の 1）・外部 crate・設計文書の正本・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| work | 写しの口 | tests/sheet.rs の口 work と型 Work（名に process の id・Drop で消す） |
| teeth | 歯 | tests/sheet.rs の f160_ 1 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い。
- 着地の後に席が見ること: 台帳 .237 を閉じ、(i) の 1・2 を席の判断に渡す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fg"
title = "台帳 f2-648.237: tests/sheet.rs の口 work が一時 dir を固定の名 folio-sheet-〈case〉 で作り、作る前に同じ名を消して写すので、同じ host の別の写しで同じ歯が同時に走るとぶつかって落ちる（base の test binary を 2 つの process で同時に撃つと 30 回のうち 29〜30 回落ちる）。口 work の名に process の id を足し、同じ file に足す包みの型 Work（Deref で Path・Drop で消す）を返す。呼び出しの字は Command の arg の 1 か所だけ変わる。crates/ の一時 dir の名の全数 55 件のうち、固定の名で作るのはこの 1 件だけ。歯は同じ test binary をこの歯 1 本だけで子として撃ち直し（子は path を標準エラーに出す・子の役は印の値が親の process の id のときだけ）、子の写しが別の path で親の写しを消さず子の終わりに消えることを縛る。src・fixture・凍結 anchor・folio2 の床と面は変えない。ceiling の揺れは原因が まだ分からないので運ばず席へ返す。門の対象外。base = main 346f825・受付の時点の main で数え直す"
req = ["FR1", "FR8"]
section = "1"
write-set = ["crates/folio/tests/sheet.rs"]
verify = ["cargo nextest run -p folio --test sheet f160_", "cargo nextest run -p folio --test sheet", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/sheet.rs の f160_ の 1 本（親が口 work で case f160-same の写しに印の file を置き、同じ process の case f160-other とは path が違い、同じ test binary をこの歯 1 本だけで撃ち直した子〔環境変数 FOLIO_F160_CHILD の値が親の process の id〕が同じ case の写しを作って path を標準エラーに 1 行出すと、子は rc 0・子の path は親と違い・親の印は残り・子の写しは子の終わりに消えている）が緑、tests/sheet.rs の歯の全部（凍結 anchor と byte 一致を見る歯を含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
