# 設計: 便 203 — 外の置き場で違反の名札と まだ分からない の行が folio2 の番号を名指さない（名札 A-2・N-4・P-7・P-7.1・P-8・R-3・R-9・R-10・R-11 と、骨格の まだ分からない の行の A-2・N-4・P-10.3・条 P-17.3・FR25）

- 要件: FR5（結果を必ず返し、実行できなかった検査を合格と表示しない）を主に、骨格の置き場の FR22 と、置き場の値域の まだ分からない の FR25。本流の要件書に在る id で、規範文・確かめ方・受入基準は変えない（どれも名札と まだ分からない の行の字の中身を定めない）。
- 条: P-6.3（外の判定と番号の落とし方は便 194・202 と同じ関数＝2 つ目の式を持たない）・P-4.2（まだ分からない の行は外でも同じ数だけ出す）・P-10.1（期待の字は歯の側の手書き）・N-3.1（旗を足さない）。
- 出所: 台帳 f2-648.236 の前半（便 156 の検証の不一致 3・名札の残り [N-4]・[P-8]・[R-3]・[A-2]・[P-7]）と、便 202 の起草で拾った骨格の まだ分からない の行（f2-648.236 の notes 2026-09-29 05:51 JST）。名札の決め方は席の裁定（2026-09-29 08:0x・名札に id を出してよいのは、床が置き場の規則の表の行をその id の字で引いて判定の値か対象を読む検査だけ。それ以外の名札は外の置き場では常に検査の名。folio2 の外の利用者は tsuzuri だけ＝持ち主の裁定 2026-09-27）。外の判じ方と番号の片の落とし方は便 202（`docs/design/delivery-202.md`）と同じ関数。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `gx` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は 16 本（src 9・歯の file 7〔うち tests/polarity.rs は本文不変で verify の scope〕）。縮む file は `crates/folio/src/polarity.rs`（関数を floor.rs へ移す）と `crates/folio/src/vocab.rs`（名札を元の id に戻す）の 2 本。消す file と新しい dir は無い。
- 門: 対象外。write-set に設計文書の正本（`design-intent/` の下）が無い。本流の binary で `folio ceiling --gate --dir design-intent --write-set <write-set の 16 本>` は **0（通す・設計文書の正本を書き換えない便）**。
- 前提: **base = 本流 1c7f927**（便 202 の着地）。この契約の数はすべて 1c7f927 の写しの実測（参考値・規則の表の行 D-13）。
- 実装の見本: origin の枝 `impl/d203`（commit **3782890**・その前 d41ba90・8c2d81b・ec92308・6750219・ece3137・1c7f927）。`git diff 1c7f927 3782890` が便の全体の差分（15 file・+528 −103・62,024 byte）。見本は 6 commit（ece3137 が本体・6750219 と ec92308 は歯を足しただけ・8c2d81b は席の裁定で 6 つの名札を置き場の表に依らない形にした・d41ba90 は同じ基準に R-9〜R-11 を入れ、便 156 の名札の引き方を出力の口の 1 か所に畳んだ・3782890 は検証 guard-verify の指摘で歯の file 2 本〔tests/vocab.rs・tests/link.rs〕に歯を 1 本ずつ足しただけで、src は d41ba90 と byte で同じ）。**作業者は write-set の file をこの commit の中身にしてよい**。write-set の外は変えない。
- 並行の便との重なり: §1 (g) 3。
- 昇格条件との関係: 台帳の前半だけを運ぶ。後半（名札と床が行を id の字で引く意味の予約・判断の記録 ADR-16 決定 (2)(オ) の字）は判断の記録が要り、席が tsuzuri の返事を待つので運ばない。

## 1. 設計

### (a) いま起きていること（base 1c7f927 の実測・参考値）

1. **違反の名札。** `crates/folio/src/main.rs` の出力の口 3 か所（素の床の違反の行・`folio check --proposed` の止める行とつながりの行・`folio parts --check` の違反の行）は、違反の種類の字（名札）を置き場に依らずそのまま [名札] に出す。便 156 は行 R-9・R-10・R-11 の名札だけを `rules::label` で置き場の表に在れば id・無ければ検査の名にした。残りの 6 つの id（条 A-2・N-4・P-7・P-8、規範文 P-7.1、行 R-3）は src の 7 file の 50 か所（adr.rs・anchor.rs・freeze.rs・ids.rs・lineage.rs・link.rs・parts.rs）が名札に積み、外の置き場でもそのまま出る。骨格（`folio init`・憲法の名は 未記入・条は P-1 だけ・行は R-2・R-8・R-16 だけ）の実測:
   - ADR-1 を発効（status accepted）にした写しの素の床: 「[N-4] ADR-1: accepted なのに approval（逐語・日付・裁定 id・対話面）が無い」（終了 1）。同じ中身を編集時の口に渡しても同じ名札で止める。
   - ADR-1 の撤退条件を空（retreat {}）にした写し: [P-8] の 3 行（必須欄が無い・kind が値域外・撤退条件が空（P-8.1））。
   - 組んだ面の index.html に目録に無い class を足した `folio parts --check`: 「[R-3] index.html: 部品目録に無い class「zz-unknown」」。
   - 要件の adrs が無い判断の記録を指す写し: [A-2] の行（便 166 の歯の骨格）。
   - [P-7]・[P-7.1] は凍結 anchor と id の一覧の anchor の検査の名札で、外の置き場で anchor を凍結すると出うる（同じ出力の口を通る）。
2. **まだ分からない の行。** 骨格の素の床の まだ分からない は 6 行で、うち 5 行が folio2 の番号を持つ: 凍結 anchor が 0 本の行の「A-2 / N-4 の差分検査」と末尾の（P-10.3）、骨格の印の 4 行（meta.approval.ruling・行 R-2・R-8・R-16 の ruling）の（骨格の印・裁定の前＝条 P-17.3）。ADR-1 を発効にすると封の一覧が無い行の（folio check --freeze-adrs で封を書き、commit する・P-10.3）が足される。同じ族は、凍結 anchor の索引の entries が空の行の（まだ分からない・P-10.3）と anchor の列が切れた行の（P-10.3）（`crates/folio/src/anchor.rs`）、置き場の値域の まだ分からない の 6 か所の FR25 の片（`crates/folio/src/check.rs` の `place_range`）、撤退条件の種類が床の値域に無い行の（FR25）（`crates/folio/src/link.rs` の `retreat_kind`）。
3. **外の置き場の名。** 外かどうかは便 202 と同じ `floor::abroad`（名が在って folio2 の置き場の名 folio2-constitution でない）で判じ、名は `adr::place_name` で読む。骨格の名 未記入 と、歯の写し 18 本の名 fixture-constitution（`tests/fixtures/` の adr・anchor・check・link・refs・vocab ほか）は外に当たる。名の読めない置き場（憲法の meta.id が無い・字でない）は外に当たらない（名の無い口と同じ）。
4. **tsuzuri の写し（tsuzuri-pin の基準の clone・hook なし）。** tsuzuri の憲法と規則の表は 9 つの id を全部持ち（条 A-2・N-4・P-7・P-8・規範文 P-7.1・行 R-3・R-9・R-10・R-11）、そのうち P-7（検査できなかった結果の扱い）・P-8（値の型の区別）・R-3（同じ種類の失敗の繰り返しの上限）・R-9（変異生存率の記録の間隔）・R-10（lint で deny にする書き方）・R-11（編集時の止めの本数の下限）は folio2 と意味が違う。base では tsuzuri の写しの違反の名札が 9 つとも folio2 の id で出る（(e) 3 の表・R-9〜R-11 は便 156 の形で行が在るので id）。置き場に在る id ならその id を出す形（便 156 の形をそのまま広げる形）では tsuzuri の名札は 1 つも変わらず、別の意味の条と行を名指す。
5. **全数の表（src の本文〔単体の歯の区間と注を除く〕の字で、外の置き場の出力に folio2 の番号が出うる所・起草の記録の scan-203 の実測）。**

| # | 族 | 置き場所と数 | 字 | 本便 |
| --- | --- | --- | --- | --- |
| 1 | 違反の名札（9 つの id） | adr.rs・anchor.rs・freeze.rs・ids.rs・lineage.rs・link.rs・parts.rs の 50 か所と、便 156 の vocab.rs 1・check.rs 2 か所 | [A-2]・[N-4]・[P-7]・[P-7.1]・[P-8]・[R-3]・[R-9]・[R-10]・[R-11] | 拾う（出力の口 3 か所で引き直し、外では置き場の表に依らず検査の名・(a) 6） |
| 2 | 骨格の印の まだ分からない | check.rs の `check_rulings` の 1 か所（骨格で 4 行） | （骨格の印・裁定の前＝条 P-17.3） | 拾う（外は（骨格の印）） |
| 3 | 凍結 anchor の まだ分からない | anchor.rs の 3 か所 | A-2 / N-4 の・（まだ分からない・P-10.3）・（P-10.3） | 拾う（外は番号の片を落とす） |
| 4 | 封の一覧が無い まだ分からない | seal.rs の 1 か所 | （…commit する・P-10.3） | 拾う |
| 5 | 置き場の値域の まだ分からない | check.rs の `place_range` の 6 か所・link.rs の `retreat_kind` の 1 か所 | （FR25）・・FR25 | 拾う |
| 6 | 違反の本文の中の番号 | adr.rs 10・anchor.rs 6・check.rs 1・freeze.rs 2・ids.rs 2・lineage.rs 2・link.rs 1・note.rs 1・seal.rs 2・figure.rs 1（道具の失敗の字）の 28 か所 | （P-8.1）・（P-7.2）・（N-1.1）・（P-12.2）・（N-4.1）ほか | 拾わない（便 205・行 gz の案・今は起草しない） |
| 7 | 書き出す file の見出しと面の凡例 | freeze.rs 2・ids.rs 1・seal.rs 1・stamp.rs 1 の見出しと face_constitution.rs の凡例 1 | ADR-2・P-7.1・ADR-30 決定 (3)・P-6.2・P-7 | 拾わない（置き場に残る file の字が変わる＝便 206・行 ha の案・今は起草しない） |
| 8 | 行を id の字で引く所（意味の予約） | 行 R-2（inject.rs）・R-8（adr.rs・link.rs・note.rs の違反の字と床の定数）・R-16（note.rs・prose.rs）・R-17（mentions.rs の知らせ） | 行 R-17 が規則の表に無い ほか | 拾わない（台帳の後半） |
| 9 | 床の定数の字（生成区間） | floor_adr.rs・floor_note.rs・graph.rs ほかの 65 か所 | 判断の記録 ADR-9 ほか | 拾わない（便 194 の `text_for` で外は落ちる・tsuzuri の写しで跡 0） |
| 10 | 骨格が自分で持つ id | init.rs（ADR-1・P-1・R-2・R-8・R-16） | 置き場の自分の id | 拾わない（番号の跡でない） |

6. **検査ごとの表（folio2 の id の形の名札と字を持つ検査と、床が置き場の規則の表の行をその id の字で引いて読むか）。** 席の裁定の基準（2026-09-29）で、外の置き場に folio2 の id を残してよいのは、床が置き場の行をその id の字で引いて判定の値か対象を読むときだけ。判断の記録 ADR-16 決定 (2)(オ)（床が id で名指すのは行 R-8 と R-16 と、生成区間の注が名指す条 N-3・注から条 id を外す）と揃う。行 R-8 だけは値を読まない（行の在否だけを見る）が、ADR-16 (オ) が名指し、置き場の文書の欄 approval.surface が値 R-8 で対話面を行 id で指すので残す（名札には出ない・本便の出力に響かない）。

| id | 検査（file） | 置き場の規則の表の行を id の字で引くか（引く関数・欄） | 外の置き場の字 | 本便 |
| --- | --- | --- | --- | --- |
| A-2 | 改訂の記録の欄（adr.rs の amends・grill）・憲法の版と anchor の食い違い（anchor.rs）・凍結（freeze.rs）・記録の消し込み（lineage.rs）・amended_by と要件の adrs の実在（link.rs） | 引かない（読むのは憲法・判断の記録・anchor） | 名札 改訂と判断の記録 | 拾う |
| N-4 | 承認欄（adr.rs の `check_approval`・対話面の値は床の定数と比べる）・条文と anchor の写しの一致（anchor.rs）・freeze.rs・lineage.rs・link.rs | 引かない | 名札 改訂の承認 | 拾う |
| P-7・P-7.1 | 番号の再利用・条の消失・規範文の改番（anchor.rs・lineage.rs）・id の一覧の anchor の消失と付け替え（ids.rs） | 引かない | 名札 id の再利用と改番 | 拾う |
| P-8 | 撤退条件の欄（adr.rs・種類の値域は置き場の憲法の schema.enums.retreat_kind） | 引かない（憲法の値域は読むが規則の表の行ではない） | 名札 撤退条件 | 拾う |
| R-3 | 部品目録に無い class（parts.rs・目録は床の定数と CSS） | 引かない（行 R-3 の値を読まない） | 名札 部品目録 | 拾う |
| R-9 | 語彙（vocab.rs の `check_vocab`・母集団は規則の表の全行の what） | 引かない（base の `rules::label` は行 R-9 の在否だけで名札を選び、値も対象も読まない） | 名札 語彙 | 拾う（便 156 の形を畳む） |
| R-10・R-11 | 条の平易文の有無・規範文の強度と文末（check.rs の `check_constitution`） | 引かない（同じく在否だけ） | 名札 平易文・強度と文末 | 拾う（同） |
| P-18 | 編集時の止めの本数の下限（polarity.rs） | 欄 key in-loop-min で引く（id では引かない・名札は folio2 の条） | 名札 polarity | 便 202 で済み |
| R-8 | 承認の対話面（link.rs の `surface` は置き場の規則の表の id の集合に R-8 が在るかだけを見る・adr.rs と note.rs は欄 surface を床の定数 SURFACE〔R-8〕と比べるだけで行 R-8 を引かない） | 行の在否だけを見る（値は読まない） | id を残す（理由は値を読むからでなく、ADR-16 (オ) が名指し、置き場の文書の欄 approval.surface が値 R-8 で対話面を行 id で指すため・違反の字の（対話面は rules 行の id で指す・R-8）・名札は adr・改訂の承認・note） | 変えない |
| R-16 | 設計ノートの散文の門（prose.rs が行 R-16 を id で探し値を読む・無ければ まだ分からない〔note.rs〕） | 引く（行 id と値・ADR-16 (オ)） | id を残す（名札は note） | 変えない |
| R-17 | 散文の言及（mentions.rs が行 R-17 を id で探し値 0 件 を読む・無ければ数えない知らせ） | 引く（行 id と値） | 名札 R-17・知らせの 行 R-17 を残す | 変えない（ADR-16 (オ) の一覧に R-17 は無い＝便 156 が足した読み。一覧の字を直すのは台帳の後半で本便は運ばない） |
| N-3 | 条（行ではない）。床は置き場の条 N-3 を引かない。字として出るのは生成区間の注（床の定数）と違反の本文の（N-3）・（…N-3.1）（adr.rs） | 引かない | 名札には出ない。生成区間の注は便 194 の落とし方で外では落ちる（ADR-16 (オ) の「注から条 id を外す」と同じ向き）。違反の本文の片は便 205 の族 | 本便の外 |

7. **base の歯（参考値）。** workspace の nextest 1147 / 1147・clippy 0 警告・床 4 本 rc 0・`folio build --write` 39 file。`git grep -n f203_ -- crates` は 0 件・行 id `gx` は 0 件。

### (b) 直す先

1. **`crates/folio/src/rules.rs`（名札の読み・表 1 つ・式 1 つ）。** 便 156 の表 LABELS と関数 `rules::label`（行の在否で名札を選ぶ）を畳み、9 つの id の検査の名の表 ABROAD_LABELS と置き場の名札の読み（型 Labels・置き場の dir から 1 度だけ作り、関数 shown で名札を引く）にする: A-2 = 改訂と判断の記録・N-4 = 改訂の承認・P-7 と P-7.1 = id の再利用と改番・P-8 = 撤退条件・R-3 = 部品目録・R-9 = 語彙・R-10 = 平易文・R-11 = 強度と文末。
   - 外かどうかは `floor::abroad` に `adr::place_name` の名を渡して判じる（便 202 と同じ）。読みは置き場の名だけで、憲法と規則の表は読まない。
   - folio2 の置き場（名が folio2-constitution）と名の読めない置き場と名の無い口は、名札の字のまま（id）。本物の folio2（規則の表に行 R-9〜R-11 を持つ）は便 156 の字と同じ id で、字も判定も変わらない。
   - 行 R-9〜R-11 を持たない置き場のうち、名が folio2-constitution か読めない置き場では、名札 R-9〜R-11 が検査の名（語彙・平易文・強度と文末）から id（[R-9]・[R-10]・[R-11]）に変わる。便 156 の行の在否の形を畳むためで、意図どおり（行の在否を読む 2 つ目の式は持たない・条 P-6.3）。判定と件数と本文の字は変わらない（歯 (c) 7）。外の置き場は表の 9 つの id を、置き場の表に同じ id が在っても検査の名にする（(a) 6 の表のとおりどれも置き場の行を引かない＝置き場の同じ id は別の意味でありうる）。表に無い名札（adr・schema・note・polarity・行を引く R-17 など）は変えない。
2. **`crates/folio/src/check.rs` と `crates/folio/src/vocab.rs`（名札を積む所は元の id）。** 語彙・平易文・強度と文末の違反は便 156 の前の種別 R-9・R-10・R-11 で積む（`rules::label` の 3 か所を戻す・`check_constitution` は規則の表を受けなくなる）。名札の引き方は次の出力の口の 1 か所だけになる。
3. **`crates/folio/src/main.rs`（出力の口 3 か所）。** 素の床・`folio check --proposed`・`folio parts --check` が、置き場の dir で名札の読みを 1 度作り、[名札] を出すときだけ引く。`--proposed` の書く前と後の突き合わせは名札の元の字で数える（止めるか・つながりか・件数は変わらない）。つながりに数える網（参照 id・語彙・判断の記録との突き合わせ・凍結 anchor の列・散文の言及）の違反のうち、名札が表に在るもの（vocab.rs の R-9・link.rs と anchor.rs〔anchor.rs が呼ぶ lineage.rs を含む〕の A-2・N-4・P-7）は、外の置き場ではつながりの行でも検査の名に変わる（例「# つながり（編集は止めない・事後の床が数える）: [改訂と判断の記録] srs.yaml: requirements[0].plain: 判断の記録 ADR-99 が実在しない」）。つながりの行も素の床の行と同じ字を保つために同じ引き方を通す（条 P-15.2・歯 (c) 8）。
4. **`crates/folio/src/floor.rs` と `crates/folio/src/polarity.rs`（番号の片の関数）。** 便 202 が polarity.rs に置いた関数 `said`（`val_for` の注の外・folio2 の置き場と名の無い口は定数のまま・外は folio2 の番号の印を持つ括弧の項と文を落とす）を floor.rs へ移し、crate の中から呼べるようにする（式は同じ）。polarity.rs はそれを呼ぶ（字と判定は変わらない）。
5. **まだ分からない の行の番号の片を同じ関数に通す。** 片だけを通し、置き場の自分の id（行 R-2 など）を持つ片は通さない。
   - `crates/folio/src/check.rs`: 骨格の印の行の（骨格の印・裁定の前＝条 P-17.3）と、値域の 6 か所の FR25 の片（（FR25）・（組み立てた版に無い鍵・値域を置き場ごとに広げる口は無い・FR25）・・FR25）。`check_dir` は置き場の名の読みを値域の数えの前へ上げ、`place_range` と `check_rulings` へ渡す。
   - `crates/folio/src/anchor.rs`: `check_anchor` が置き場の名を読み、3 行の片（A-2 / N-4 の・（まだ分からない・P-10.3）・（P-10.3））を通す。
   - `crates/folio/src/link.rs`: `retreat_kind` が置き場の名を引数に取り、（FR25）を通す。
   - `crates/folio/src/seal.rs`: 封の一覧が無い行の（folio check --freeze-adrs で封を書き、commit する・P-10.3）を通す。
6. **外の置き場の字（見本の binary の実測・骨格）。** 名札 =「[改訂の承認] ADR-1: accepted なのに approval（逐語・日付・裁定 id・対話面）が無い」・「[撤退条件] ADR-1: 撤退条件が空（P-8.1）」ほか・「[部品目録] index.html: 部品目録に無い class「zz-unknown」」（置き場に行 R-3 を足しても同じ）・「[改訂と判断の記録] srs.yaml: …」・「[語彙] P-1 title: 語彙に無い英字の語「foobar」」・「[平易文] P-1: plain が無い」・「[強度と文末] …」（規則の表に行 R-9〜R-11 を足しても同じ）。まだ分からない =「凍結 anchor が 0 本（<dir>）＝差分検査は「まだ分からない」。発効版で --freeze-anchor を実行する」・「<場所> が 未記入（骨格の印）」・「…の本文の凍結を測れない（folio check --freeze-adrs で封を書き、commit する）」・「…比較元が立たない（まだ分からない）。…」・「…anchor file が無いか読めない＝差分検査は「まだ分からない」。anchors/ は消さない」・値域の行は末尾の FR25 の片が無い。
7. **変えないもの。** 本物の folio2 の置き場（規則の表に行 R-9〜R-11 を持つ）と名の無い口の字と判定（終了コード・違反と まだ分からない の件数と字・--proposed の答え・--polarity の出力）・`floor.rs` の `abroad`・`val_for`・`text_for` の式・床の定数の字・生成区間・本物の folio2 の置き場の名札 R-9〜R-11（id・名札の読みは行の在否を見ない）・行 R-8・R-16・R-17 を id で名指す所と名札 R-17・違反の本文の中の番号（表の 6）・書き出す file の見出し（表の 7）・行を id で引く所（表の 8）。外の置き場でも判定と件数は変わらず、変わるのは名札と まだ分からない の行の番号の片だけ。例外は (b) 1 の 2 つ目の点で、行 R-9〜R-11 を持たず名が folio2-constitution か読めない置き場では、名札 R-9〜R-11 だけが検査の名から id に変わる（判定と件数と本文の字は同じ）。

### (c) 歯（f203_・base で 0 件・6 本と、便 156 の歯 1 本の改め・既存の歯 6 か所の字の期待）

1. **単体 f203_said_drops_only_the_folio2_number_pieces_abroad（`floor.rs` の tests の区間）。** (b) 5 の片 6 つ（骨格の印・（まだ分からない・P-10.3）・封の片・A-2 / N-4 の・（FR25）・・FR25）について、名が無い・folio2-constitution は片のまま、tsuzuri-constitution・未記入・folio2 は外の字（歯の側の手書き: （骨格の印）・（まだ分からない）・（folio check --freeze-adrs で封を書き、commit する）・空の字・空の字・空の字）。
2. **単体 f203_labels_name_the_check_abroad_even_where_the_place_has_the_id（`rules.rs` の tests の区間）。** folio2 の置き場（外でない）は 9 つの id のまま、外の置き場は 9 つとも検査の名（改訂と判断の記録・改訂の承認・id の再利用と改番 2 つ・撤退条件・部品目録・語彙・平易文・強度と文末・歯の側の手書き）。表に無い名札（adr・schema・R-8・R-16・R-17・P-18・polarity）は両方で変わらない。読みは置き場の表を見ないので、同じ id を持つ外の置き場でも検査の名（(c) 3 の (b)(c) と (c) 5 が写しで見る）。
3. **`tests/mechanism_live.rs` f203_abroad_labels_and_unknowns_name_no_folio2_number。** 外の置き場の最小の写し = 骨格（git init の後に `folio init`・名は 未記入）。
   - (a) ADR-1 を発効にした写しの素の床: 終了 1・違反はちょうど [改訂の承認] の 1 行・凍結 anchor が 0 本の行は「）＝差分検査は「まだ分からない」。発効版で --freeze-anchor を実行する」で終わる・封の一覧が無い行は外の字・骨格の印の 4 行は（骨格の印）・標準出力と標準エラーに条と規範文の id の形（P- / N- / A- と数）が 0。編集時の口（新しい骨格に発効にした ADR-1 の中身を渡す）も同じ名札の 1 行と外の字の封の行・id の形 0。
   - (b) 撤退条件を空にし、同じ id の条 P-8（別の意味の条・規範文 P-8.1・meta.counts を合わせる）を憲法に足した写し: 違反はちょうど [撤退条件] の 3 行（置き場が P-8 を持っても検査の名・字の中の（P-8.1）は表の 6 で本便の外）。
   - (c) 面を 1 度組み、index.html に class zz-unknown を足した `folio parts --check`: 終了 1・「[部品目録] index.html: 部品目録に無い class「zz-unknown」」。規則の表に行 R-3 を足しても同じ「[部品目録] …」（床は行 R-3 を引かない）。
   - (d) anchors/index.yaml の entries が空の写しと、版 v1.0 だけを持つ索引で anchor file が無い写し: まだ分からない の行がそれぞれ外の字（（まだ分からない）・P-10.3 の片なし）。
   - (e) 同じ中身で憲法の名だけ folio2-constitution にした写し: 名札 [N-4]・「）＝A-2 / N-4 の差分検査は「まだ分からない」（P-10.3）。発効版で --freeze-anchor を実行する」・骨格の印の 4 行は（骨格の印・裁定の前＝条 P-17.3）（今の字）。
4. **`tests/constitution_range.rs` f203_abroad_range_pendings_drop_the_requirement_id。** 骨格の値域の節に、組み立てた版に無い鍵 colour・一覧でない鍵 tier・広げた値 strength の zz・無い鍵 binds を当てた写しの素の床: 値域の まだ分からない の行がちょうど 4 行で、どれも外の字（（組み立てた版に無い鍵・値域を置き場ごとに広げる口は無い）・末尾の片なし・（「zz」・値域を置き場ごとに広げる口は無い）・末尾の片なし）・出力に FR25 が 0・終了 2。値域の節を表でなくした写し: 撤退条件の種類が読めない行と「schema.enums（置き場の憲法の値域の節）が表でない＝条の値を置き場の値域で引けない」の 2 行・FR25 が 0。folio2 の置き場の同じ 5 通りの字（FR25 の片あり）は便 122 の f122_ の歯が見る。
5. **便 156 の歯の改め（`tests/mechanism_live.rs` の f156_labels_name_only_rows_the_place_has → f156_abroad_labels_name_the_checks_even_with_the_rows）。** 骨格の条 P-1 を 3 通り崩した写し: 行が無い写し・閾値の節に行 R-9〜R-11 を足した写し・作法の節に足した写しのどれも、違反はちょうど「[平易文] P-1: plain が無い」・「[強度と文末] P-1: P-1.1: strength must-not と文末が合わない（must-not ⇔ 〜ない。）」・「[語彙] P-1 title: 語彙に無い英字の語「foobar」」の 3 行（base は行を足すと [R-10]・[R-11]・[R-9]）。同じ中身に行を足し、名だけ folio2-constitution にした写しは、崩した 3 つの字の行が [R-10]・[R-11]・[R-9]（便 156 と同じ字・骨格の欄の決まりの file が folio2 の床の定数と違う [adr]・[note] の行は数えない）。知らせ 1 行の数え（off_count）はそのまま。`tests/vocab.rs` の頭の注の理由を直す（写しの名が外なので [語彙]・既存の 2 本の判定と期待は同じ）。
6. **既存の歯の字の期待 4 file 6 か所（写しの名が外に当たる・判定と件数は同じ）。**
   - `tests/adr.rs` の adr_effective_without_approval_fails: 写し effective-no-approval（名 fixture-constitution・条 N-4 なし）の名札の期待 N-4 → 改訂の承認。
   - `tests/link.rs` の link_amended_by_orphan_fails の 2 か所: [N-4] → [改訂の承認]。link_retreat_kind_drift_is_unknown_and_not_pass: まだ分からない の行の末尾の（FR25）を落とす。頭の注に便 203 の 2 行。
   - `tests/face_srs_adrs.rs` の f166_adrs_without_a_record_are_not_yet_known: 骨格の [A-2] → [改訂と判断の記録]。
   - `tests/constitution_range.rs` の f157_narrowed_range_is_pending_and_never_pass: 骨格の値域が狭い行の期待から ・FR25 を落とす。
7. **`tests/vocab.rs` f203_places_not_abroad_show_the_ids_even_without_the_rows（検証 guard-verify の Bb203-1・変異 Z3）。** 骨格（`folio init`・規則の表は行 R-2・R-8・R-16 だけで行 R-9〜R-11 を持たない・版管理なし）の条 P-1 を (c) 5 と同じ 3 通りに崩し、憲法の meta.id を消した写し（名が読めない）と、名だけ folio2-constitution にした写しの素の床: 終了 1・崩した 3 つの字の違反の行がちょうど「[R-10] P-1: plain が無い」・「[R-11] P-1: P-1.1: strength must-not と文末が合わない（must-not ⇔ 〜ない。）」・「[R-9] P-1 title: 語彙に無い英字の語「foobar」」（歯の側の手書き・骨格の欄の決まりの file が床の定数と違う [adr]・[note] の行は数えない）。base は行が無いので [平易文]・[強度と文末]・[語彙]。名の読めない置き場を外と扱う変異はこの歯が落とす。
8. **`tests/link.rs` f203_abroad_proposed_link_lines_name_the_check（検証 guard-verify の N-203-1・変異 Z8）。** 外の置き場の写し `tests/fixtures/link/adr-id-missing`（名 fixture-constitution・置き場は書かない）に、要件 FR1 の平易文へ実在しない判断の記録の id を足した srs.yaml を渡す編集時の口（`folio check --dir <写し> --proposed srs.yaml`）: 終了 0・止める行 0・要約は「folio check --proposed: 通す（新しい違反 0・つながり 1・まだ分からない 0・」で始まり、つながりの行はちょうど「# つながり（編集は止めない・事後の床が数える）: [改訂と判断の記録] srs.yaml: requirements[0].plain: 判断の記録 ADR-99 が実在しない」の 1 行（歯の側の手書き）。base は [A-2]。つながりの行が名札を引かない変異はこの歯が落とす。
9. **RED の実測。** 歯の file 6 本だけ（見本 3782890 の tests/mechanism_live.rs・adr.rs・link.rs・face_srs_adrs.rs・constitution_range.rs・vocab.rs）を base に当てると 10 本が落ちる（nextest の rc 100・起草の記録の red-203-*.log・改訂 3 の撃ち直し）: tests/mechanism_live.rs の 2 本（7 本のうち・f203_ は違反の名札が [N-4] で期待の [改訂の承認] と違う・f156_ は行を足すと [R-10] で期待の [平易文] と違う）・tests/adr.rs の 1 本（13 本のうち）・tests/link.rs の 3 本（4 本のうち・f203_ はつながりの行の名札が [A-2] で期待の [改訂と判断の記録] と違う）・tests/face_srs_adrs.rs の 1 本（3 本のうち）・tests/constitution_range.rs の 2 本（8 本のうち・f203_ と f157_ の値域の行の末尾に FR25 の片が残る）。・tests/vocab.rs の 1 本（3 本のうち・f203_ は行が無いので [平易文] で期待の [R-10] と違う・既存の 2 本は base でも緑）。base の `--bin folio f203_` は 0 本（単体の歯は src の中・nextest の rc 4）。

### (d) 採らなかった形

1. **置き場に在る id ならその id、無ければ検査の名（便 156 の形をそのまま 6 つへ広げる・最初の起草 6750219）。** tsuzuri は 6 つの id を全部持つので名札が 1 つも変わらず、しかも P-7・P-8・R-3 は tsuzuri では別の意味の条と行を名指す（(a) 4）。外の利用者は tsuzuri だけなので効かない便になる。席の裁定で、名札に id を出すのは置き場の行を id で引いて読む検査だけにした。
2. **名札を積む所（50 か所）で引く。** 積む関数の全部に置き場の名を渡す口が要り、`--proposed` の書く前と後の突き合わせの字も置き場で動く。出力の口 3 か所で 1 度引く形なら、判定と突き合わせは元の字のままで、出すときの字だけが変わる。
3. **便 156 の R-9〜R-11 を積む所の `rules::label`（行の在否）のまま残し、名札の引き方を 2 か所に置く（改訂 1 の形）。** 語彙・平易文・強度と文末の検査は行 R-9〜R-11 の値も対象も読まないので、席の基準では id を出さない側に入る。tsuzuri の R-9〜R-11 は別の意味（変異生存率の記録の間隔・lint で deny にする書き方・編集時の止めの本数の下限）で、行が在るので base では語彙の違反が [R-9] で出る。式の違う 2 か所を持つ理由も無くなる。
4. **まだ分からない の片を、置き場にその id が在るかで出し分ける。** 片の番号（A-2 / N-4・P-10.3・条 P-17.3・FR25）は folio2 の条と要件が理由を名指す字で、どれも置き場の行を引かない（名札と同じ席の基準）。tsuzuri の P-10 は失敗の繰り返しで意味が合わない。便 194・202 と同じく外では落とす。
5. **まだ分からない の行を丸ごと生成区間の落とし方に通す。** 字の中の行 R-2・R-8・R-16 と置き場の path が印に数えられ、文ごと落ちる（便 202 の (d) 3 と同じ）。片だけを通す。
6. **違反の本文の中の番号（表の 6・28 か所）も同じ便で拾う。** size が M の上限に近づき（check.rs の余地は本便の後 319）、見る歯も違反の字の 28 通りに広がる。便 205（行 gz）に割る（今は起草しない）。
7. **検査の名の表を規則の表か設計文書に置く。** 名札は検査の名の表示で、閾値でも規則でもない。便 156 の表（rules.rs の LABELS）と同じく実装の型付きの定数に置き、その表を畳んで 1 つにする。

### (e) 既存の歯・突然変異・外の置き場

1. **既存の歯。** 見本の写し（3782890）で workspace の nextest 1153 / 1153（base 1147 + f203_ の 6 本・便 156 の歯は改めた 1 本で本数は同じ・d41ba90 の 1151 から (c) 7・(c) 8 の 2 本が増えた）・clippy 0 警告・床 4 本 rc 0・`folio build --write` 39 file が base と全 file で byte で同じ（起草の記録の impl5-*.log）。folio2 の素の床の標準出力と標準エラーは base の binary と byte で同じ（知らせは今の字）。字の期待を直した既存の歯は (c) 5 の 1 本と (c) 6 の 6 か所だけ。
2. **突然変異（見本の写しの src だけを 1 通りずつ変え、単体〔f203_・floor・rules・polarity の tests〕と tests/ の 8 本〔mechanism_live・adr・link・face_srs_adrs・constitution_range・polarity・proposed・vocab〕を撃つ）。** 27 通りとも落ちる（生き残り 0）。名札の 11 通り（M1〜M9・M24・M25）は見本 d41ba90 で撃った（起草の記録の mut-203-M1-…-M25.log）。改訂 3 の 2 通り（M26 = 検証の Z3・M27 = 検証の Z8）は見本 3782890 で撃った（mut-203-M26-M27.log）。d41ba90 ではこの 2 通りが生き残った（検証 guard-verify の変異）ので、歯 (c) 7・(c) 8 を足した。番号の片の 14 通り（M10〜M23）は 8c2d81b で撃ち（mut-203-M10-…-M23.log）、片の src（floor.rs・anchor.rs・link.rs・seal.rs と check.rs の片）は d41ba90 でも同じなので撃ち直していない。6750219 では M20 が生き残ったので、歯 (c) 4 を足して M20 と同じ族の M21〜M23 も撃った。数えない等価な変異: 番号の片の関数の注の旗を真にする（空の字になるのは同じ）。前の版がもう 1 つ等価と書いた「つながりの行の名札を元の字に戻す」は誤りで（つながりに数える網の中の A-2・N-4・P-7・R-9 は表に在る）、M27 として数える。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 名札を常に元の字 | 単体の rules の f203_・mechanism_live の f203_ と f156_・adr・link・face_srs_adrs の字の期待 3 本・vocab の 2 本 |
| M2 外かどうかの分岐を逆にする | 単体の rules の f203_・mechanism_live の f203_ と f156_・adr・link・face_srs_adrs 3 本・vocab 2 本・constitution_range の f122_ 4 本と f157_ 2 本・proposed の f198_ 1 本 |
| M3 外かどうかを見ない（folio2 の置き場でも検査の名） | 単体の rules の f203_・mechanism_live の f203_ と f156_・constitution_range の f122_ 4 本と f157_ 2 本・proposed の f198_ 1 本 |
| M4 置き場の名を読まず常に folio2 の置き場と扱う | mechanism_live の f203_ と f156_・adr・link・face_srs_adrs の字の期待 3 本・vocab 2 本 |
| M5 置き場の名を読まず常に外と扱う | mechanism_live の f203_ と f156_・constitution_range の f122_ 4 本と f157_ 2 本・proposed の f198_ 1 本 |
| M6 表の R-3 の行を別の id にする（外で [R-3] が出る） | 単体の rules の f203_・mechanism_live の f203_ |
| M7 素の床の違反の行を元の名札で出す | mechanism_live の f203_ と f156_・adr・link・face_srs_adrs の字の期待 3 本・vocab 2 本 |
| M8 --proposed の止める行を元の名札で出す | mechanism_live の f203_ |
| M9 parts --check の違反の行を元の名札で出す | mechanism_live の f203_ |
| M24 表の R-9 の行を別の id にする（外で [R-9] が出る） | 単体の rules の f203_・mechanism_live の f156_・vocab 2 本 |
| M25 表の R-11 の行を別の id にする（外で [R-11] が出る） | 単体の rules の f203_・mechanism_live の f156_ |
| M26 名の読めない置き場を外と扱う（検証の Z3） | vocab の f203_ |
| M27 --proposed のつながりの行を元の名札で出す（検証の Z8） | link の f203_ |
| M10 番号の片の関数が置き場の名を見ない | 単体の floor の f203_ と polarity の f202_・mechanism_live の f203_ と f156_・polarity の f202_・link・constitution_range の f157_ と f203_ |
| M11 番号の片の関数が常に外の字 | 単体の floor の f203_ と polarity の f202_・mechanism_live の f203_ と f131_ 2 本・polarity の f200_ 2 本と f202_・constitution_range の f122_ 4 本と f157_ 2 本 |
| M12 骨格の印の行を定数のまま出す | mechanism_live の f203_ |
| M13 値域が狭い行の FR25 を落とさない | constitution_range の f157_ |
| M14 撤退条件の種類の行の FR25 を落とさない | link の字の期待 |
| M15 封の一覧が無い行の P-10.3 を落とさない | mechanism_live の f203_ |
| M16 anchor 0 本の行の A-2 / N-4 を落とさない | mechanism_live の f203_ |
| M17 索引の entries が空の行の P-10.3 を落とさない | mechanism_live の f203_ |
| M18 anchor の列が切れた行の P-10.3 を落とさない | mechanism_live の f203_ |
| M19 anchor 0 本の行の P-10.3 を落とさない | mechanism_live の f203_ |
| M20 組み立てた版に無い鍵の行の FR25 を落とさない | constitution_range の f203_（6750219 では生き残り） |
| M21 値域の節が表でない行の FR25 を落とさない | constitution_range の f203_ |
| M22 値域の鍵が一覧でない行の FR25 を落とさない | constitution_range の f203_ |
| M23 値域に鍵が無い行の FR25 を落とさない | constitution_range の f203_ |
3. **外の置き場。** 骨格の写し（起草の記録の sk-203.log・(c) 3 と同じ編集）で base と見本の答えの違いは、名札 [N-4] → [改訂の承認]（ADR-1 を発効にした写し）と、まだ分からない の 5〜6 行の番号の片だけ（終了コード 2 と 1・件数は同じ）。見本の出力に残る番号の形は骨格の自分の行 R-2・R-8・R-16 と、行 R-17 の知らせ（表の 8）だけ。tsuzuri の写し（tz-203.log・tz-203b.log）では、素の床・`--polarity`・`folio build`（72 file）は base と見本で byte で同じで、変異を当てた写しの違反の名札が次のとおり変わる（名札を除いた字・終了コード・件数・標準エラーは同じ）。

| tsuzuri の写しに当てた変異 | base の名札 | 見本の名札 |
| --- | --- | --- |
| ADR-1 の撤退条件を空（素の床と編集時の口） | [P-8] 3 行（例「[P-8] ADR-1: 撤退条件が空（P-8.1）」） | [撤退条件] 3 行（「[撤退条件] ADR-1: 撤退条件が空（P-8.1）」） |
| ADR-2 の承認の逐語を 未記入 | 「[N-4] ADR-2.approval.verbatim が 未記入（init の雛形の印・空と同じ）」 | [改訂の承認] で同じ字 |
| 要件 FR1 の adrs を無い ADR-99 に | 「[A-2] srs.yaml: requirements[0].adrs[0]: 判断の記録 ADR-99 が実在しない」 | [改訂と判断の記録] で同じ字 |
| ADR-6 の id を ADR-96 に（本文同じ）・ADR-5 の id と title を変える | 「[P-7.1] ADR-6 の本文が ADR-96 へ付け替えられた（…）」・「[P-7] ADR-5 が消えた（…）」・[A-2] 2 行 | [id の再利用と改番] 2 行・[改訂と判断の記録] 2 行 |
| 規範文 P-1.1 を P-1.9 に改番し憲法の版を v1.2 に | 「[A-2] 憲法の版 v1.2 と最新 anchor の版 v1.1 が違う…」・「[P-7] 規範文の改番: P-1.1 を消して同じ本文を P-1.9 として足している（…）」 | [改訂と判断の記録]・[id の再利用と改番] |
| 組んだ面の index.html に目録に無い class・`folio parts --check` | 「[R-3] index.html: 部品目録に無い class「zz-unknown」」 | [部品目録] で同じ字 |
| 条 P-1 の見出しに foobar・平易文を消す・P-1.1 の強度を must-not（tsuzuri は行 R-9〜R-11 を別の意味で持つ） | 「[R-9] P-1 title: 語彙に無い英字の語「foobar」」・「[R-10] P-1: plain が無い」・「[R-11] P-1: P-1.1: strength must-not と文末が合わない（must-not ⇔ 〜ない。）」・「[N-4] 現行の条文…が凍結 anchor constitution-v1.1.yaml と一致しない…（N-4.1）」 | [語彙]・[平易文]・[強度と文末]・[改訂の承認] で同じ字 |

   まだ分からない の行は、索引の entries を空にした写しの（まだ分からない・P-10.3）→（まだ分からない）と、封の一覧を外した写しの（…commit する・P-10.3）→（…commit する）の 2 行が変わる。
   folio2 の置き場（本流 1c7f927 の design-intent の写し・起草の記録の hm-203.log）では、そのままの写し・条 P-1 を同じ 3 通りに崩した写し（[R-9]・[R-10]・[R-11]・[N-4]）・ADR-1 の撤退条件を空にし要件の adrs を無い判断の記録にした写し（[P-8]・[A-2]）・編集時の口の答えが、base と見本で標準出力と標準エラーとも byte で同じ。

### (f) 大きさ・余地・verify と done の対応

1. **write-set の印。** 新しい file・消す file は無い。縮む file は `crates/folio/src/polarity.rs`（関数を floor.rs へ移す・正規化行数 179 → 174）と `crates/folio/src/vocab.rs`（名札を元の id に戻す・299 → 298）。`crates/folio/tests/polarity.rs` は本文不変で verify の `--test polarity` の scope、`crates/folio/tests/vocab.rs` は頭の注と歯 (c) 7 で verify の `--test vocab` の scope。`crates/folio/src/floor.rs`・`crates/folio/src/rules.rs`・`crates/folio/src/polarity.rs` は単体の歯を持つので verify の `--bin folio` の scope。
2. **余地（CapHeadroom）。** write-set の src の 9 本（python と awk の 2 実装で一致・起草の記録の cap-203-d41ba90.log）。改訂 3 の見本 3782890 は歯の file 2 本だけを変え、src の 9 本は d41ba90 と byte で同じ（`git diff d41ba90 3782890 -- crates/folio/src` は空）なので、表は d41ba90 の実測のまま。

| file | base の正規化行数（参考値） | base の余地 | 本便の後 | 本便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/floor.rs` | 794 | 706 | 822（+28・単体の歯を含む） | 678 |
| `crates/folio/src/polarity.rs` | 179 | 1321 | 174（−5） | 1326 |
| `crates/folio/src/rules.rs` | 498 | 1002 | 549（+51・単体の歯を含む） | 951 |
| `crates/folio/src/main.rs` | 778 | 722 | 782（+4） | 718 |
| `crates/folio/src/check.rs` | 1172 | 328 | 1180（+8） | 320 |
| `crates/folio/src/anchor.rs` | 1072 | 428 | 1079（+7） | 421 |
| `crates/folio/src/link.rs` | 647 | 853 | 649（+2） | 851 |
| `crates/folio/src/seal.rs` | 316 | 1184 | 321（+5） | 1179 |
| `crates/folio/src/vocab.rs` | 299 | 1201 | 298（−1） | 1202 |

3. **size は M。** src の増分は +99 で S の見積 100 のきわ（改訂 1 では +113）。write-set が 16 本にわたり、作業者の組み直しで数行ずれても余地の判定が割れないように M とする。余地の最小（check.rs の base 328）は M の 300 を超える。
4. **verify は 12 行**で、done の 12 の塊と 1 対 1 に揃える。見本の 3782890 で 12 行とも rc 0（2・1・7・8・13・4・3・8・3・7・1 本と clippy 0 警告・verify-impl5.log）。
   1. `cargo nextest run -p folio --bin folio f203_` = (c) 1・2（2 本）。
   2. `cargo nextest run -p folio --test mechanism_live f203_` = (c) 3（1 本）。
   3. `cargo nextest run -p folio --test mechanism_live` = (c) 3・(c) 5（便 156 の歯の改め）と骨格と床の土台の知らせの歯（f156_・f131_）。
   4. `cargo nextest run -p folio --test polarity` = 便 200・202 の歯（関数を移しても folio2 の置き場の字と外の字が変わらない）。
   5. `cargo nextest run -p folio --test adr` = (c) 6 の 1 か所と判断の記録の床の歯。
   6. `cargo nextest run -p folio --test link` = (c) 8（1 本）と (c) 6 の 3 か所と突き合わせの歯。
   7. `cargo nextest run -p folio --test face_srs_adrs` = (c) 6 の 1 か所。
   8. `cargo nextest run -p folio --test constitution_range` = (c) 4（1 本）と (c) 6 の 1 か所と置き場の値域の歯（folio2 の置き場の字）。
   9. `cargo nextest run -p folio --test vocab` = (c) 7（1 本）と語彙の歯（外の写しの名札 [語彙]・名札を元の id に戻しても変わらない）。
   10. `cargo nextest run -p folio --bin folio floor::tests` = 便 174・194 の外の字の落とし方の単体の歯と (c) 1。
   11. `cargo nextest run -p folio --bin folio polarity::tests` = 便 202 の単体の歯（移した関数を呼ぶ）。
   12. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。

### (g) 門・受付・ほかのレーンとの重なり

1. **門。** 冒頭のとおり対象外・0（通す）。本流の binary（`target/debug/folio`・06:30 の組み立て・1c7f927）に write-set の 16 本を渡した答え「folio ceiling: 通す（設計文書の正本を書き換えない便）」（gate-203c.log・改訂 3 で撃ち直して同じ＝gate-203d.log）。
2. **受付。** 受付の先撃ち（precheck）は、枝 docs/d203 の本契約で 契約に起因する断り 0（起草の記録の precheck-203c.log・改訂 3 の precheck-203d.log）。見本の写しで verify の 12 行とも rc 0。
3. **ほかのレーンとの重なり。** 起草の時点（2026-09-29 07:2x）の origin の impl/* の枝で write-set の file を書き換え、本流に着地していないのは impl/d201（`crates/folio/src/main.rs`・取り下げ済み）だけで、見本との `git merge-tree` は衝突 0。impl/d184〜d202 は本流に squash で着地済み。契約の枝だけで見本の無い docs/d162（write-set に rules.rs・adr.rs・link.rs）と docs/d164（anchor.rs・adr.rs）は base が古く、着地の順が来たら数え直しが要る（本便が先に着地したら、あちらが数え直す）。改訂 3（2026-09-29 10:2x）の実測: 便 204 の見本 impl/d204 1bc273b と本便の見本 3782890 の `git merge-tree` は衝突 0 で、重なる file は `crates/folio/src/check.rs` と `crates/folio/src/rules.rs` だけ（便 204 の歯の file は tests/ruling.rs と tests/schema_docs.rs で、改訂 3 の歯の file tests/vocab.rs・tests/link.rs と重ならない）。重ねた木で check.rs 1206（余地 294）・rules.rs 644（余地 856）、tests/vocab.rs・tests/link.rs・tests/ruling.rs・tests/schema_docs.rs の 46 本が緑（起草の記録の c35-nextest.log・cap-203-c35.log）。着地の順が 203 → 204 なら 204 の受付時の check.rs の余地は 320（S の 100 の内）、逆なら 203 の受付時の余地は 302（M の 300 を 2 だけ超える＝数え直しの時の注意）。
4. **数え直し。** 改訂 3 の時点（2026-09-29 10:3x）の本流は 001c865（要件書 第 1.56 版・変わった file は design-intent/srs.yaml と design-intent/anchors/ids-v1.56.yaml だけ）で、write-set の file と下の関数は 1c7f927 と同じ（req の FR5・FR22・FR25 は第 1.56 版にも在る）。受付の時点の本流で write-set の file か、`floor.rs` の `abroad`・`val_for`・`text_for`、`adr::place_name`、`rules::label` が base と違えば、見本を本流に取り込み、verify と骨格と tsuzuri の写しの答え（(e) 3）を数え直してから運ぶ。
5. **着地の後（外の置き場）。** 生成区間も床の判定も変わらないので、外の置き場の file の書き直しは要らない。tsuzuri では 9 つの名札（A-2・N-4・P-7・P-7.1・P-8・R-3・R-9・R-10・R-11）が検査の名（改訂と判断の記録・改訂の承認・id の再利用と改番・撤退条件・部品目録・語彙・平易文・強度と文末）で出て、まだ分からない の行が出たときは（P-10.3）と FR25 の片が落ちる。行 R-17 の名札と、行 R-8・R-16 を名指す字は変わらない。席は tsuzuri へこの 1 行を返す。folio2 の側では、行 R-9〜R-11 を持たず名が folio2-constitution か読めない置き場（本流の置き場には無い・歯の写しと骨格だけ）の名札 R-9〜R-11 が id になる（(b) 1）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は `.local/share/folio2/handoff-2026-09-28/p184-draft.md`、script と log は同じ dir の `p184-scripts/d203/`。
1. `run-203.sh <木> <印>`: 組み立て・workspace の nextest（--test-threads 2）・clippy・床 4 本・`folio build --write` の file 数と sha（base と見本で撃って比べる）。
2. `red-203.sh <base の木> <見本の rev>`: 歯の file 6 本だけを base に当てて RED（落ちた歯の本文を log に残す）。
3. `mut-203.py <見本の clone> [M..]`: 変異 27 通り（1 通りずつ src を変え、単体と tests/ の 8 本を撃つ）。
4. `cap-203.sh <clone> <base> <見本>`: 余地（lines.py と lines.awk の 2 実装）。
5. `verify-203.sh <木> <印>`: 契約の verify の 12 行。
6. `sk-203.sh <base の folio> <見本の folio>`: 骨格の写しで base と見本の答えを比べる。
7. `tz-203.sh` と `tz-203b.sh <base の folio> <見本の folio>`: tsuzuri の写し（tsuzuri-pin の基準の clone・hook なし）で base と見本の答えと名札の前後を比べる。`hm-203.sh` は folio2 の置き場の写しで byte で同じかを見る。
8. `scan-203.py <木> <json>`: 全数の表（(a) 5）の元の走査。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** 本物の folio2 の置き場（行 R-9〜R-11 を持つ）の字と判定・`floor.rs` の外の判定と番号の落とし方の式・床の定数の字・生成区間・要件書・規則の表・便 156 の名札・違反の本文の中の番号（(a) 5 の表の 6）・書き出す file の見出しと面の凡例（表の 7）・行を id の字で引く所と判断の記録 ADR-16 決定 (2)(オ) の字（表の 8・台帳の後半）・外の置き場の file・台帳への記帳（席）・外部 crate・新しい dir。
2. **言えないこと。** 外の置き場の違反の行は、名札に番号が出なくなっても、本文の末尾の（P-8.1）・（P-7.2）・（N-3）などの folio2 の番号を持つ（表の 6・便 205 の案・骨格の撤退条件の写しで 1 行）。行 R-17 は床が id で引いて読むので名札に id が残るが、判断の記録 ADR-16 決定 (2)(オ) の一覧（行 R-8・R-16・条 N-3）に R-17 は無い。一覧の字を直すのは台帳の後半（判断の記録が要る）で、本便は運ばない。
3. **撤退条件。** (1) 要件書・判断の記録・憲法・床の定数の字を変えないと書けないと分かったら、止めて席へ返す。(2) 本便の後に §1 (c) 5 の 1 本と (c) 6 の 6 か所のほかに既存の歯が落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(3) 本便の後に folio2 自身の床 4 本の結果か、folio2 の素の床の標準出力と標準エラーか、`folio build` の出力が 1 byte でも変われば、止めて席へ返す。

## 2. 範囲

- 入れる: `crates/folio/src/rules.rs` の名札の読みと検査の名の表（便 156 の表と関数を畳む）と単体の歯・`crates/folio/src/check.rs` と `crates/folio/src/vocab.rs` の名札を元の id に戻す所・`crates/folio/src/main.rs` の出力の口 3 か所・`crates/folio/src/floor.rs` へ移す番号の片の関数と単体の歯・`crates/folio/src/polarity.rs` がそれを呼ぶ形・`crates/folio/src/check.rs`・`crates/folio/src/anchor.rs`・`crates/folio/src/link.rs`・`crates/folio/src/seal.rs` の まだ分からない の行の番号の片・歯 `tests/mechanism_live.rs`・`tests/constitution_range.rs`・`tests/vocab.rs`・`tests/link.rs` の f203_ 各 1 本・便 156 の歯 1 本の改め・既存の歯 4 file 6 か所の字の期待・`tests/vocab.rs` の頭の注。
- 入れない: 本物の folio2 の置き場（行 R-9〜R-11 を持つ）の字と判定・`floor.rs` の外の判定と番号の落とし方の式・床の定数の字・生成区間・要件書・規則の表・違反の本文の中の番号・書き出す file の見出し・意味の予約・外の置き場の file・台帳・外部 crate・新しい dir。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| labels | 名札の読み | `rules.rs` の 1 つの表と 1 つの式: 置き場の名だけで外かどうかを判じ、外なら 9 つの id の名札を検査の名にする（置き場の表に同じ id が在っても） |
| print | 出力の口 | `main.rs` の素の床・--proposed・parts --check が出すときだけ名札を引く |
| pieces | 番号の片 | `floor.rs` の関数が まだ分からない の行の folio2 の番号の片を外で落とす（`check.rs`・`anchor.rs`・`link.rs`・`seal.rs`・`polarity.rs` が呼ぶ） |
| teeth | 歯 | 単体の f203_ 2 本・`tests/mechanism_live.rs`・`tests/constitution_range.rs`・`tests/vocab.rs`・`tests/link.rs` の f203_ 各 1 本・便 156 の歯 1 本の改め・既存の歯 4 file 6 か所の字の期待 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate も外部ライブラリも増やさない。新しい dir は無い。
- 前提の着地: 便 202（本流 1c7f927）。
- 本便の着地の後に席が見ること: 台帳 f2-648.236 の前半を記帳する（後半は残す）。本流の `target/debug/folio` を組み直す。tsuzuri へ §1 (g) 5 の 1 行を返す。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "gx"
title = "外の置き場で違反の名札と まだ分からない の行が folio2 の番号を名指さない（台帳 f2-648.236 の前半・席の裁定 2026-09-29・便 202 と同じ外の判じ方）: crates/folio/src/rules.rs に置き場の名札の読み（名は adr::place_name・外の判定は floor::abroad・置き場の憲法と規則の表は読まない）と 9 つの id の検査の名の表（A-2 改訂と判断の記録・N-4 改訂の承認・P-7 と P-7.1 id の再利用と改番・P-8 撤退条件・R-3 部品目録・R-9 語彙・R-10 平易文・R-11 強度と文末）を置き、便 156 の表 LABELS と関数 rules::label を畳む（crates/folio/src/check.rs と crates/folio/src/vocab.rs の名札は元の id に戻す）。crates/folio/src/main.rs の出力の口 3 か所（素の床・--proposed・parts --check）が出すときだけ名札を引く 1 か所にする（外の置き場では置き場の憲法と規則の表に同じ id が在っても検査の名・外に id を残すのは床が置き場の行を id で引いて値を読む行 R-16・R-17 と、行の在否だけを見て判断の記録 ADR-16 決定 (2)(オ) が名指す行 R-8 だけ。folio2 の置き場と名の読めない置き場は名札の id のまま）。便 202 の番号の片の関数を crates/folio/src/polarity.rs から crates/folio/src/floor.rs へ移し、crates/folio/src/check.rs（骨格の印の条 P-17.3・値域の FR25 の 6 か所）・crates/folio/src/anchor.rs（A-2 / N-4 の・P-10.3 の 3 行）・crates/folio/src/link.rs（撤退条件の種類の FR25）・crates/folio/src/seal.rs（封の一覧が無い行の P-10.3）の まだ分からない の行の番号の片を通す。本物の folio2 の置き場（規則の表に行 R-9〜R-11 を持つ）と名の無い口の字と判定は変えない。行 R-9〜R-11 を持たず名が folio2-constitution か読めない置き場では、名札 R-9〜R-11 だけが検査の名から id に変わる（便 156 の行の在否の形を畳むため・判定と件数と本文の字は同じ）。歯は floor.rs と rules.rs の単体の f203_ 2 本と tests/mechanism_live.rs の f203_ 1 本（骨格 folio init を外の置き場の最小の写しにする）と tests/constitution_range.rs の f203_ 1 本（骨格の値域の まだ分からない の 5 通り）と tests/vocab.rs の f203_ 1 本（行 R-9〜R-11 の無い骨格の名を消した写しと名だけ folio2 の写しで名札 R-10・R-11・R-9）と tests/link.rs の f203_ 1 本（外の置き場の編集時の口のつながりの行の名札が 改訂と判断の記録）と、tests/mechanism_live.rs の便 156 の歯 1 本の改め（外は行 R-9〜R-11 を足しても検査の名・名だけ folio2 の写しは行 id）と、写しの名が外に当たる既存の歯 4 file 6 か所の字の期待（tests/adr.rs・tests/link.rs・tests/face_srs_adrs.rs・tests/constitution_range.rs）と tests/vocab.rs の頭の注。実装の見本は origin の枝 impl/d203 の commit 3782890（base 1c7f927）で、作業者は write-set の file をその中身にしてよく、write-set の外は変えない。base = 本流 1c7f927"
req = ["FR5", "FR22", "FR25"]
section = "1"
write-set = ["crates/folio/src/floor.rs", "crates/folio/src/polarity.rs", "crates/folio/src/rules.rs", "crates/folio/src/main.rs", "crates/folio/src/check.rs", "crates/folio/src/anchor.rs", "crates/folio/src/link.rs", "crates/folio/src/seal.rs", "crates/folio/src/vocab.rs", "crates/folio/tests/mechanism_live.rs", "crates/folio/tests/adr.rs", "crates/folio/tests/link.rs", "crates/folio/tests/face_srs_adrs.rs", "crates/folio/tests/constitution_range.rs", "crates/folio/tests/polarity.rs", "crates/folio/tests/vocab.rs"]
verify = ["cargo nextest run -p folio --bin folio f203_", "cargo nextest run -p folio --test mechanism_live f203_", "cargo nextest run -p folio --test mechanism_live", "cargo nextest run -p folio --test polarity", "cargo nextest run -p folio --test adr", "cargo nextest run -p folio --test link", "cargo nextest run -p folio --test face_srs_adrs", "cargo nextest run -p folio --test constitution_range", "cargo nextest run -p folio --test vocab", "cargo nextest run -p folio --bin folio floor::tests", "cargo nextest run -p folio --bin folio polarity::tests", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "M"
done = "binary の単体の f203_ の 2 本（番号の片は名の無い口と folio2 の置き場で片のまま・名の等しくない置き場で外の字、名札は folio2 の置き場で 9 つの id のまま・外の置き場で 9 つとも検査の名・表に無い名札は変わらない）が緑、tests/mechanism_live.rs の f203_ の 1 本（骨格の素の床と編集時の口と parts --check の名札と まだ分からない の行に folio2 の条と規範文の番号が無く、同じ id の条 P-8 や行 R-3 を足した骨格でも名札は検査の名で、同じ中身の folio2 の名の写しは今の字）が緑、tests/mechanism_live.rs の歯の全部（便 156 の歯の改め＝外の置き場は行 R-9〜R-11 を足しても 語彙・平易文・強度と文末、名だけ folio2 の写しは行 id）が緑、tests/polarity.rs の歯の全部（便 200・202 の字と判定）が緑、tests/adr.rs の歯の全部が緑、tests/link.rs の歯の全部（f203_ の 1 本＝外の置き場の編集時の口のつながりの行の名札が 改訂と判断の記録）が緑、tests/face_srs_adrs.rs の歯の全部が緑、tests/constitution_range.rs の歯の全部（f203_ の 1 本の骨格の値域の まだ分からない の 5 通りに FR25 が無いことと、folio2 の置き場の値域の字）が緑、tests/vocab.rs の歯の全部（外の写しの名札 語彙と、f203_ の 1 本＝行 R-9〜R-11 の無い骨格の名を消した写しと名だけ folio2 の写しの名札 R-10・R-11・R-9）が緑、binary の単体の floor::tests の全部（外の字の落とし方の式）が緑、binary の単体の polarity::tests の全部が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 で、folio build の file 数は着地の直前の main と同じである"
<!-- contracts:end -->
