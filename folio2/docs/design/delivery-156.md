# 設計: 便 156 — 行 R-17 が無い置き場で散文の言及の歯を黙らせず、違反の名札に置き場に無い行を名指させない（群 A・台帳 f2-648.194）

- 要件: FR5（結果を必ず返し、実行できなかった検査を合格と表示しない）を主に、骨格の置き場の FR22。規範文・確かめ方・受入基準は変えない。
- 条: P-4.1 / P-4.2 / P-10.1（歯の期待は手書き）/ N-3.1（旗を足さない）。前文の順位の「速さより黙らないこと」。
- 出所: 外の置き場の実測（`.local/share/folio2/handoff-2026-09-24/folio2-v3-entry-notes.md` §3 の 6・7）と台帳 **f2-648.194**。ADR-21 文脈 (5) はこの黙りを「床の結果ではなく結果の不在」とし、帰結で v3 の入口の便に数えた。持ち主の直命 2026-09-26 の群 A。
- 置き場: 審査の材料は行 `fc` が指す §1 だけ。write-set は 7 本で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d156 の一番上で base の binary と本便の binary に write-set 7 本を渡すと、どちらも **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main 7dc0cab（便 154・155 の契約の docs だけが d6d84fa〔便 153 の着地〕の上に載った main・コードは d6d84fa と同じ）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 改訂 b: 独立の検証（`.local/share/folio2/handoff-2026-09-27/d156-verify.md`）の blocking 1 と非 blocking 1・2・4・5・6 を入れた＝歯 6 に知らせ 1 回と作法の節の段（変異 V4〜V6）・check.rs の注・§1 (d) の 3・(i) の 1 と 2・(g)。
- 席の推奨との違い: 推奨 (1) の「まだ分からない」は判定の外の 1 行に替え、推奨 (2)「骨格が行 R-17 を書く」は運ばない（§1 (d) の 1・2）。推奨 (3) の名札はそのまま。

## 1. 設計

### (a) いま起きていること（base 7dc0cab の実測・参考値）

1. **外の置き場の撃ち直し。** 一時 dir で git の init → `folio init --dir design-intent` → commit の後、1 通りずつ字を変えて `folio check` を撃った。骨格の規則の表の行は R-2・R-8・R-16 だけ。右の欄は本便を当てた写しの binary。

| 置き場の字 | base | 本便の後 |
| --- | --- | --- |
| 骨格のまま | 2（違反 0・まだ分からない 2） | 同じ判定 + 1 行（(b) の 1） |
| 判断の記録 ADR-1 の平易文に「承認は行 R-8 の対話面で受ける。」 | **2（違反 0）＝言及を数えず、数えていないことも出ない** | 同じ判定 + 1 行 |
| 上に加えて行 R-17（値 0 件・条 P-1 に結ぶ） | 1（[R-17] の 1 件） | 同じ |
| 行 R-17 だけを足す | 2（違反 0・まだ分からない 2） | 同じ（1 行は出ない） |
| 条 P-1 の平易文に foobar | 1・名札 **[R-9]** | 1・名札 [語彙] |
| 条 P-1 の平易文を消す | 1・名札 **[R-10]** | 1・名札 [平易文] |
| 規範文 P-1.1 の強度を must-not | 1・名札 **[R-11]** | 1・名札 [強度と文末] |

2. **黙る所。** `crates/folio/src/mentions.rs` の入口の関数 switch は行 R-17 が無いと何も残さず None を返し、check_mentions はそのまま戻る（値が 0 件 でない行は「まだ分からない」）。同じく行を読む散文の門（R-16・prose.rs）と注入（R-2・inject.rs）は、行が無いと「まだ分からない」か 2 にする。
3. **名札の所。** 語彙の検査（vocab.rs の check_vocab）・条の平易文の有無（check.rs の check_constitution）・強度と文末（check.rs の check_statement_polarity）が名札に R-9・R-10・R-11 を焼いている。
4. **fixture。** `tests/fixtures` の下の rules.yaml 25 本はどれも行 R-17 を持たない（床の凍結の土台 floor_base を含む）。語彙の歯の 2 組（vocab/unknown-word・exemptions）の行は R-1・R-3・R-8・D-1 で、R-9 を持たない。
5. **base の歯。** workspace の nextest 959 / 959・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 34 file。`f156_` と行 id `fc` は 0 件。

### (b) 直す先

1. **mentions.rs。** switch を「行が無い = None・値が 0 件 でない = Some(false)・数える = Some(true)」にし、check_mentions は行 R-17 が無くて数えなかったときだけ false を返す。1 行の字は公開の定数 OFF: `# 行 R-17 が規則の表に無い＝散文の言及の歯は数えていない（床の判定の外・値 0 件 の行 R-17 を足して撃ち直すと数える）`。頭の注の「行が無ければ黙り」を直す。
2. **check.rs。** Materials に mentions_off を足し、check_dir が check_mentions の答えで立てる。check_constitution に規則の表を渡し、R-10 と R-11 の名札を 4 の関数から取る（check_statement_polarity は名札を引数で受け、頭の注の「種別 R-11 の違反」を「名札 R-11（置き場の規則の表に行が無ければ 強度と文末）の違反」に直す）。
3. **main.rs。** 命令 check の出口で mentions_off のときだけ OFF を標準エラーへ出す。置く所は「まだ分からない」の行の後・機構がまだ無い条の行（便 131）の前で、機構の行は要約の直前に残る。判定・終了コード・要約・標準出力は変えず、build ほかの命令は出さない。
4. **rules.rs。** 名札の閉じた一覧 LABELS（R-9 = 語彙・R-10 = 平易文・R-11 = 強度と文末）と関数 label。置き場の規則の表（thresholds か discipline）にその行が在れば行 id、無ければ検査の名を返す。名札は文言の字面なので行 D-11 の写しの対象の外。
5. **vocab.rs。** R-9 の名札を 4 の関数から取る。
6. **変えないもの。** 違反の本文・判定・終了コード・数え方。folio2 自身は R-9〜R-17 を持つので、folio2 の check と `folio build` の出力は 1 byte も変わらない。骨格の命令（行 R-17 は入れないまま）。ほかの名札（(i) の 2）。

### (c) 歯（関数名 f156_・`crates/folio/tests/mechanism_live.rs`・binary 経由）

1. **f156_a_place_without_r17_says_the_mentions_are_not_counted。** 既存の口 Work::skeleton の骨格に (a) の 1 の言及を足して commit すると、check は 2 で標準出力はちょうど要約 1 行（違反 0・まだ分からない 2）、標準エラーの最後の行が OFF でちょうど 1 回。手書きの行 R-17（値 0 件・条 P-1 の relations.rules に足す）を足すと 1 で違反はちょうど [R-17] の 1 行・OFF なし。値を 1 件 にすると 2 で「R-17 の値「1 件」が 0 件 でない」が出て OFF なし。folio2 自身の design-intent にも OFF なし。**base では OFF が出ない＝RED。**
2. **f156_labels_name_only_rows_the_place_has。** 骨格の条 P-1 の見出しに foobar・平易文を消す・強度を must-not にして commit すると、check は 1 で違反はちょうど [平易文]・[強度と文末]・[語彙] の 3 行（本文は base と同じ字）、標準エラーの OFF はちょうど 1 回（違反の在る置き場でも出る）。手書きの行 R-9・R-10・R-11（条 P-1 に結ぶ）を閾値の節に足しても、別の骨格で同じ憲法のまま作法の節（discipline）に置いても、同じ本文で [R-10]・[R-11]・[R-9]。**base では 1 行目が [R-10]＝RED。**
3. **置き換える既存の歯 4 本（期待は手書き）。** 土台 floor_base は行 R-17 を持たないので OFF が出る: f131_floor_base_lists_the_articles_before_the_summary の標準エラーを [OFF, 機構の行]・--emit-amends では [OFF, 機構の行, 要約] に、f131_no_line_when_no_article_waits_for_its_mechanism の「標準エラーが空」を [OFF] に。語彙の 2 組は行 R-9 を持たないので、tests/vocab.rs の共通の確かめ（vocab_unknown_word_fails と vocab_exemptions_count_only_the_unknown_word）の名札を [語彙] に。

fixture は足さず変えない。歯は既存の口（Work・edit・lines・show・folio）と足した口 2 つ（add_rows・violations）だけを使う。

### (d) 採らなかった形

1. **行が無ければ「まだ分からない」（推奨 (1) の字どおり）。** 写しで、読めない（unknown）で立てると workspace の歯 36 本（floor_cases の 133 組を含む）、測れない（pending）でも 12 本（同 32 組）が落ちた。(a) の 4 の fixture が行 R-17 を持たないので土台が合格せず、途中の凍結も断られる。直すには fixture の規則の表・憲法の relations・凍結 anchor の要約値まで写し直すことになり M を超える。外の置き場でも行 R-17 を足すまで床が合格せず凍結できず、ADR-16 決定 (2)(オ) の「利用者が要れば、利用者の持ち主の裁定で足す」と食い違う。本便は行を持たない置き場を「その歯を採っていない」と読み、判定の外の 1 行で黙りだけを解く（便 131 の機構がまだ無い条の 1 行と同じ扱い）。
2. **骨格の命令が行 R-17 を書く（推奨 (2)）。** 推奨 (2) は (1) で骨格の判定を増やさないための対で、(1) を採らないと理由が残らない。そのうえ要件 FR22 の注と ADR-16 決定 (2)(オ) が「行 R-17 は骨格に入れない」と書き、既存の歯 f125_sources_match_folio2 がそれを確かめる＝設計文書の字を先に変える仕事になる。v3 の置き場（scribe3）は骨格を 2026-09-24 に書き済みで init の変更は届かない（1 行は届く）。
3. **名札を常に検査の名にする。** 違反を持つ状態の folio2 の名札が変わり（今の folio2 は違反 0 なので実の出力は変わらない）、写しに変異を当てて名札 [R-11] を確かめる check の r11_ の 2 本ほかを書き換えることになる。推奨 (3) の形なら folio2 の名札は変わらない。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は (c) の 3 の 4 本だけ。** 本便を当てた写しで workspace の nextest **961 / 961**・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 34 file で base と diff -r 一致・folio2 自身の check の出力も一致。
2. **RED。** 歯だけを base に当てると f156_ の 2 本と置き換えた 4 本が落ち、`--test mechanism_live` と `--test vocab` のほかの 2 本は緑。
3. **突然変異（写しの src を 1 通りずつ変え、`--test mechanism_live`・`vocab`・`check` を撃つ）。** 15 通りとも 1 本以上が落ちる（V4〜V6 は検証役が足した変異で、改訂 b の前は workspace の全部をすり抜けた）。知らせ・名札 は (c) の 1・2、土台 は f131_ の 2 本、語彙 は tests/vocab.rs の 2 本、r11 は check の r11_ の 2 本。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 入口が 1 行を出さない | 知らせ・名札・土台 |
| M2 行が無くても数えたと返す | 知らせ・名札・土台 |
| M3 1 行を機構の行の後に出す | 土台（1 本） |
| M4 名札を常に検査の名に | 名札・r11 |
| M5 名札を常に行 id に（base の形） | 名札・語彙 |
| M6 R-11 の名札に R-10 を引く | 名札・r11 |
| M7 行 R-17 が在っても 1 行を出す | 知らせ |
| M8 行が無ければ まだ分からない（(d) の 1） | 知らせ・名札・土台・語彙・check の 4 本 |
| M9 名札が discipline の節だけを見る | 名札・r11 |
| M10 1 行から撃ち直し方の字を落とす | 知らせ・名札・土台 |
| M11 値が 0 件 でないときも 1 行を出す | 知らせ |
| M12 語彙の名札を R-9 に焼く | 名札・語彙 |
| V4 名札が thresholds の節だけを見る | 名札（作法の節の段） |
| V5 違反が在る置き場では 1 行を出さない | 名札（知らせ 1 回の段） |
| V6 不合格の置き場では 1 行を出さない | 名札（知らせ 1 回の段） |

### (f) 大きさ・verify と done の対応

1. **write-set。** 7 本とも印なし（書き換えるだけ）: `crates/folio/src/mentions.rs`・`crates/folio/src/check.rs`・`crates/folio/src/main.rs`・`crates/folio/src/rules.rs`・`crates/folio/src/vocab.rs`・`crates/folio/tests/mechanism_live.rs`・`crates/folio/tests/vocab.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/check.rs` | 1027 | 473 | 1032（+5） | 468 |
| `crates/folio/src/main.rs` | 664 | 836 | 668（+4） | 832 |
| `crates/folio/src/mentions.rs` | 341 | 1159 | 348（+7） | 1152 |
| `crates/folio/src/rules.rs` | 295 | 1205 | 314（+19） | 1186 |
| `crates/folio/src/vocab.rs` | 297 | 1203 | 299（+2） | 1201 |

   src の外は `tests/mechanism_live.rs` 232 → 430・`tests/vocab.rs` 60 → 61。rustfmt --check（edition 2024）の差の数は check.rs 4 → 4・mentions.rs 4 → 4・rules.rs 1 → 1・vocab.rs と main.rs 0 → 0・tests/mechanism_live.rs 5 → 4・tests/vocab.rs 0 → 0（足した字は fmt に合う）。
3. **size は S。** src は 5 本で計 +37（知らせの 1 行と名札の読み手 1 つ）。
4. **verify は 4 行**で、done の 4 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test mechanism_live f156_` = (c) の 1・2。
   2. `cargo nextest run -p folio --test mechanism_live` = (c) の 3 の f131_ 2 本を含む全部（参考値 6 本）。
   3. `cargo nextest run -p folio --test vocab` = (c) の 3 の 2 本（参考値 2 本）。
   4. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で終了コード 4、2 は 4 本・3 は 2 本で緑、4 は 0 警告。歯の file 2 本は write-set に在り、`--bin` の行は無い。

### (g) 門と受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0（(h) の 5）。便 157（行 fd）の write-set と `crates/folio/src/check.rs` が重なる。157 が直すのは関数 place_range、本便は check_dir・Materials・check_constitution・check_statement_polarity で、字の上では交わらない。後に受け付ける側が受付の時点の main で数え直す（両方を足しても check.rs の余地は 457・参考値）。便 154（fa）・155（fb）とは重ならない。便 153 は着地済み。本便は便 154（PR #348 の実装）と便 155 の着地の後に載る見込みで、その後に席が受付の時点の main で数え直す。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d156-draft.md`、script は同じ dir の d156-scripts（repo の外）。

1. 再現: repro-156.sh（(a) の 1・base と本便の binary）。
2. 模擬: build-sim-156.sh が main の clone に apply-156.py（実装）と apply-156-teeth.py（歯）を当てて組む（c156.patch）。suite-156.sh（組み立て・nextest・clippy）・floor-156.sh（床 4 本・build の diff -r・folio2 自身の check の出力）・verify-156.sh。
3. RED: red-156.sh（歯だけを base に当てて戻す・r156-teeth.patch）。(d) の 1 の測り: alt-156.sh。
4. 突然変異と余地: mut-156.py（撃った後は元に戻して組み直す）・lines-156.py と lines-156.awk。
5. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-156.md#fc`。
6. 改訂 b: edit-contract-156b.py（契約の字）・検証役の d156-verify-scripts（extra-teeth.patch・mut-156v.py の V4〜V6）。数は d156-draft.md の改訂 b の節。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR5 に沿って、行を持たない置き場で数えなかったことを毎回出す。FR5 が名指す folio build は床の要約だけを出し、知らせは check で見る。判定の 3 値と FR22 の確かめ方（骨格の床は違反 0・まだ分からない 2）は変わらない。着地の後に古くなる字は下の表（設計文書の 3 つは次の節目の材料・行 D-16・この便では書かない）。ADR-21 文脈 (5) と帰結の「黙る」は発効の時点の実測と便の列なので直さない。

| 字 | 場所 | 扱い |
| --- | --- | --- |
| 歯は行そのものを入り口にし、行が無ければ黙り | design-intent/rules.yaml（行 R-17 の注） | 次の節目で直す |
| 行が無ければ黙る歯で、…M3 の判定点に効かない | design-intent/adr/ADR-16.yaml（決定 (2)(オ)） | 次の節目で直す（骨格に入れない結論と判定点に効かない結論は変わらない） |
| 床が id で名指すのは、規則の表の行 R-8 と R-16 の行 id と…条 N-3 である | 同（決定 (2)(オ)） | base から食い違う（R-9〜R-11・N-4・P-8・R-3 ほかを名指す）。本便は 3 つを行が無ければ名指さない形に減らすが、行が在れば id で名指す。次の節目の材料 |
| 歯そのものは黙る | crates/folio/tests/check.rs（f93_deleting_the_rule_row_is_not_a_silent_escape の注・write-set の外） | 本便の後は標準エラーに 1 行が出る（違反に数えないのは同じ・歯は緑）。次にこの file を書く便で直す |
| 合わない 1 本につき種別 R-11 の違反 1 件 | crates/folio/src/check.rs（check_statement_polarity の注・write-set の中） | 本便で直す（(b) の 2） |
2. **運ばないもの。** 骨格の命令が行 R-17 を書くこと（(d) の 2）。同じ種類の名札の残り＝部品の検査の [R-3]（parts.rs は規則の表を読まない）と、条 id の名札 [A-2]・[N-4]・[P-7]・[P-7.1]・[P-8]（外の置き場の憲法では同じ番号が別の条になりうる）。本便の名札は id の字だけを見るので、利用者が別の意味で行 R-9 を持てば [R-9] と名指す＝R-9〜R-11 の id を利用者の置き場でも folio の意味に予約するのと同じになる。この名札の残りと id の予約は、席が別の控えにする。設計文書の正本・台帳への記帳・外部 crate。
3. **撤退条件。** (1) (c) の 3 のほかの既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か check の出力か `folio build` の出力が着地の直前と 1 byte でも違ったら止めて席へ返す。(3) 受付の時点の main で mentions.rs の switch・check.rs の check_dir と check_constitution・main.rs の check の出口が base と違えば、(h) で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1〜5、(c) の歯 2 本と置き換え 4 本。
- 入れない: 骨格の命令・部品の検査と条 id の名札・判定と終了コード・命令の旗・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| off | 数えなかった知らせ | mentions.rs の OFF と switch・check.rs の mentions_off・main.rs の 1 行 |
| label | 名札 | rules.rs の LABELS と label・vocab.rs と check.rs の 3 か所 |
| teeth | 歯 | tests/mechanism_live.rs の f156_ 2 本と置き換え 2 本・tests/vocab.rs の置き換え |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（base = main 7dc0cab）。並行は §1 (g)。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。.194 を閉じる（(i) の 2 の名札の残りを控えに起こすかは席が決める）。(i) の 1 の古くなる字を次の節目の材料に積む。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fc"
title = "群 A（台帳 f2-648.194）: 規則の表に行 R-17 が無い置き場（folio init の骨格を含む）では散文の言及の歯が何も出さずに止まり、語彙・条の平易文・強度と文末の違反の名札が置き場に無い行 R-9・R-10・R-11 を名指す。crates/folio/src/mentions.rs・check.rs・main.rs で、行 R-17 が無いときは判定を変えずに folio check の標準エラーへ床の判定の外の 1 行（撃ち直し方の字つき）を出し、crates/folio/src/rules.rs に名札の閉じた一覧と読み手を足して、vocab.rs と check.rs の 3 か所が置き場の規則の表に在る行だけを行 id で、無ければ検査の名（語彙・平易文・強度と文末）で名指す。folio2 自身の check と build の出力・判定と終了コード・違反の本文・骨格の命令は変えない。歯は tests/mechanism_live.rs の f156_ 2 本と既存 4 本の置き換え（f131_ 2 本・tests/vocab.rs の名札）。門の対象外。base = main 7dc0cab・受付の時点の main で数え直す"
req = ["FR5", "FR22"]
section = "1"
write-set = ["crates/folio/src/mentions.rs", "crates/folio/src/check.rs", "crates/folio/src/main.rs", "crates/folio/src/rules.rs", "crates/folio/src/vocab.rs", "crates/folio/tests/mechanism_live.rs", "crates/folio/tests/vocab.rs"]
verify = ["cargo nextest run -p folio --test mechanism_live f156_", "cargo nextest run -p folio --test mechanism_live", "cargo nextest run -p folio --test vocab", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/mechanism_live.rs の f156_ の 2 本（根の直下の design-intent に init して commit した骨格で、判断の記録 ADR-1 の平易文が行 R-8 を指すとき check が rc 2 で標準出力はちょうど 違反 0・まだ分からない 2 の要約 1 行、標準エラーの最後の行が 行 R-17 が規則の表に無い の 1 行でちょうど 1 回、値 0 件 の行 R-17 を足すと rc 1 で違反はちょうど [R-17] の 1 行で 1 行は出ず、値を 1 件 にすると rc 2 で 1 行は出ず、folio2 自身にも出ない / 骨格の条 P-1 の見出しに foobar・平易文を消す・強度を must-not にすると違反はちょうど [平易文]・[強度と文末]・[語彙] の 3 行で標準エラーに 1 行がちょうど 1 回、行 R-9・R-10・R-11 を閾値の節に足しても作法の節に置いても同じ本文で [R-10]・[R-11]・[R-9]）が緑、tests/mechanism_live.rs の歯の全部（土台の標準エラーで 1 行が機構の行の前に出る置き換えを含む）が緑、tests/vocab.rs の歯の全部（行 R-9 を持たない組の名札が [語彙] という置き換え）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）で 1 行を出さず・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
