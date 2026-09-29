# 設計: 便 157 — 憲法の値域を狭める変更も「まだ分からない」で止める（群 A・台帳 f2-648.195）

- 要件: FR25（置き場の憲法の値域を床で数える）。規範文と受入基準 AC22 は変えない（§1 (i) の 1）。
- 条: P-4.1・P-4.2（判定できないものを黙って通さない）/ N-3.1（値域を置き場ごとに変える口を持たない）/ P-10.1（歯の期待は手書きと凍結 anchor）。
- 出所: 外の置き場の実測（`.local/share/folio2/handoff-2026-09-24/folio2-v3-entry-notes.md` §4 の末尾）と台帳 **f2-648.195**。持ち主の裁定 2026-09-26 15:22 JST（棚卸しの問 2「推奨で」）= 狭める向きも広げる向きと対称に止める・ADR-16 の注に「狭める向きは許す」の断りは書かない。
- 置き場: 審査の材料は行 `fd` が指す §1 だけ。write-set は 2 本で、新しい file も dir も無い。
- 門: 対象外。作業ツリー planner-d157 の一番上で、base の binary・本便の binary・本流の target/debug/folio に write-set 2 本を渡すと、どれも **0（通す・設計文書の正本を書き換えない便）**。
- 前の便: **base = main eecbff9（便 153〜156 の着地の後）。数は base の写しの実測（参考値・行 D-13）**で、受付の時点の main が違えば数え直す。
- 同乗しない: 台帳 f2-648.185（main.rs の説明の字）は本便の write-set に main.rs が入らないので別の S にする（§1 (i) の 2）。
- 改訂 b（2026-09-27・検証役の記録 `d157-verify.md`）: 歯 1 に順の段と骨格の置き場の段を足し、check.rs の頭の注を直す先に入れ、床の側の代価と古くなる字を §1 (i) の 1 に補った。
- 改訂 c（2026-09-27・便 156 の着地の後）: base を main eecbff9 へ載せ替え、数を撃ち直して §1 (a) の 5・(e)・(f)・(g) の数を改めた（中身は改訂 b と同じ）。

## 1. 設計

### (a) いま起きていること（base eecbff9 の実測・参考値）

1. **外の置き場での答え。** 一時 dir で git の init → `folio init --dir design-intent` → commit の後、骨格の憲法の値域の節を 1 か所ずつ変えて commit し、`folio check` を撃った。右の欄は本便を当てた写しの binary。

| 変えたこと | base | 本便の後 |
| --- | --- | --- |
| 骨格のまま | 2（違反 0・まだ分からない 2 = 凍結の基準の不在だけ） | 同じ |
| 段（tier）に someday を足す | 2（まだ分からない 3・広げた字 1 件） | 同じ |
| strength から should を消す | **2（まだ分からない 2＝何も言わない）** | 2（まだ分からない 3・狭めた字 1 件） |
| strength の should を may に替える | 2（まだ分からない 3・広げた字だけ） | 2（まだ分からない 4・広げた字と狭めた字） |

2. **床は部分集合かだけを見る。** `crates/folio/src/check.rs` の関数 place_range は、置き場の値域の鍵に組み立てた版（build.rs が folio2 の憲法から導出した定数 ENUMS）に無い値があるときだけ「まだ分からない」を出す。外した値は、条が使っていれば違反になるが、使っていなければ黙って通る。
3. **面は既に集合の一致で断る。** 同じ置き場で `folio face --face constitution` は、should を消した憲法を「組み立てた版の 3 値全部でない・組み立て時の憲法の値域と違う（組み立て直す）」の まだ分からない（rc 2）で断る（`crates/folio/src/face_constitution_read.rs` の enum_skew）。床と面の答えが食い違っている。本便が面に揃えるのは狭める向きだけで、同じ値を 2 度書いた値域は本便の後も床が黙り面が「2 度ある」で断る（便 122 の「重複は問わない」のまま・直すなら別の便の判断）。
4. **folio2 自身は変わらない。** 組み立てた版の値域は folio2 の憲法から導出するので、folio2 自身の値域はいつも組み立てた版と等しい。
5. **base の歯。** workspace の nextest 972 / 972・clippy 0 警告・床 4 本 rc 0・`folio build --write` の出力 34 file。`--test constitution_range` は 5 本。`f157_` と行 id `fd` は 0 件。

### (b) 直す先 — `crates/folio/src/check.rs` の関数 place_range

1. 組み立てた版に在る鍵について、組み立てた版の値のうち置き場の一覧に無い値を組み立てた版の順に集め、1 つ以上あれば「まだ分からない」1 件を出す。字は「constitution.yaml: schema.enums.〈鍵〉: 組み立て時の値域に在る値が無い（「値」・「値」・値域を置き場ごとに狭める口は無い・FR25）」。広げた向きの字の直後に出す（名を替えた鍵は 2 件）。
2. 狭めた鍵も今までどおり値域の表に入れる（条の値は置き場の値域で引き続け、置き場の値域に無い値を使った条は違反のまま）。違反は出さない。
3. 関数の注を「集合で等しいかを数える・広げる口も狭める口も持たない」に、file の頭の注（6〜7 行の「部分集合か…部分集合でない鍵と引けない鍵は まだ分からない」）を「集合で等しいか（便 122・便 157）…広げた鍵と狭めた鍵と引けない鍵は まだ分からない」に直す。
4. **変えないもの。** 広げた向きと引けない鍵（鍵が無い・一覧でない・節が表でない）の字と数え方・`crates/folio/src/link.rs` の撤退条件の種類の突き合わせ（判断の記録の床の定数との部分集合のまま）・面・init の骨格・命令の旗・folio2 自身の正本と出力。

### (c) 歯（関数名 f157_・`crates/folio/tests/constitution_range.rs`・binary 経由）

土台は既存の口 Work（design-intent の写しの 1 commit）。値域の節を変えると改憲の違反（種別 N-4）がちょうど 1 件出るので、既存の歯と同じくそれを除いて数える。外の置き場の段のために、Work に `folio init` の骨格を git の 1 commit にする口 skeleton を足す。

1. **f157_narrowed_range_is_pending_and_never_pass。** ① strength から should を外すと、N-4 のほかの違反は 0、まだ分からない はちょうど [狭めた strength「should」] の 1 件、判定は「不合格（違反 1・まだ分からない 1）」。② should を may に替えると [広げた「may」, 狭めた「should」] の順の 2 件。③ そのまま binds を [tool] に狭めると [狭めた binds「practice」・「both」, 広げた strength「may」, 狭めた strength「should」] の順の 3 件（外した値は組み立てた版の順で字の順と逆・字は鍵の順で、同じ鍵では広げた字の直後）。④ must だけにすると 1 件の中に「must-not」・「should」。⑤ 別の写しで撤退条件の種類から measure を外すと [狭めた retreat_kind「measure」] の 1 件だけ（判断の記録の床は黙る）で「不合格（違反 1・まだ分からない 1）」。⑥ `folio init` の骨格の置き場（外の置き場と同じ形・名は 未記入）で、骨格のままは値域の字が 0 件、strength から should を外して commit すると違反 0 のまま まだ分からない が 1 件増え、値域の字はちょうど [狭めた strength「should」]、終了コード 2。**base では狭めた字が出ない＝RED。**
2. **f157_every_narrowed_key_is_pending_per_key。** 凍結 anchor（手書きの `tests/fixtures/check/enum-range-anchor.yaml`・10 鍵）の各鍵から末尾の値を 1 つずつ外して値域の節と差し替えると、まだ分からない が鍵ごとに 1 件・anchor の鍵の順でちょうど 10 件。**base では 0 件＝RED。**
3. **置き換える既存の歯 2 本（名も替える）。** 狭めた値域を「黙る」と期待していた歯を直す。
   1. f122_subset_range_is_silent_and_article_values_are_looked_up_in_the_place_range → **f122_reordered_range_is_silent_and_article_values_are_looked_up_in_the_place_range**。1 段目は並べ替えただけの値域（M0 を残す）で黙る。2 段目で M0 を外し、条 P-1 の機構の live を M0 にすると、違反は「条 P-1 の mechanism.live の値「M0」が憲法の値域 schema.enums.mechanism_live に無い」の 1 件のまま、まだ分からない は [狭めた mechanism_live「M0」] の 1 件（base では 0 件＝落ちる）。
   2. f122_narrowed_or_reordered_retreat_kind_is_silent → **f122_reordered_retreat_kind_is_silent**。並べ替えだけにし、外した一覧は (c) の 1 の ⑤ が数える。頭の注に便 157 の 1 行を足す。

fixture は足さない（凍結 anchor はそのまま読む）。

### (d) 採らなかった形

1. **狭めた向きを違反にする。** 持ち主の裁定は広げる向きの既存の形に揃える。値域の食い違いは条の誤りでなく組み立てた版との食い違いで、判定できないもの（P-4.2）として出すのが広げる向き・面の断り（「組み立て直す」）と同じ扱いになる。
2. **init した時の値域を基準にし、folio2 が後から足した値は黙る。** 骨格に基準の file を足す形は、受入基準 AC19 の書く file 11 本の閉じた一覧と食い違い、判断の記録が先に要る。

### (e) 既存の歯のうち落ちるもの・突然変異

1. **落ちる既存の歯は (c) の 3 で置き換える 2 本だけ。** 本便を当てた写しで workspace の nextest **974 / 974**・clippy 0 警告・床 4 本 rc 0・`folio build --write` は 34 file で base と diff -r 一致・`--test constitution_range` 7 / 7。
2. **RED。** 歯だけを base に当てると、f157_ の 2 本と置き換えた f122_reordered_range_… が落ち、ほかの 4 本は緑。
3. **突然変異（写しの check.rs を 1 通りずつ変え、`--test constitution_range` を撃つ）。** 11 通りとも 1 本以上が落ちる。表の 歯 1・歯 2 は (c) の 1・2、置換 は (c) の 3 の 1。X1・X2・X8 は (c) の 1 の ③ が、X7 は ⑥ が落とす（改訂 b・検証役の生き残り）。

| 変異 | 落ちる歯 |
| --- | --- |
| M1 狭めた向きを数えない（base と同じ） | 歯 1・歯 2・置換 |
| M2 狭めたかを値の数で見る | 歯 1 |
| M3 狭めた向きを違反にする | 歯 1・歯 2・置換 |
| M4 外した値を 1 つだけ名指す | 歯 1 |
| M5 狭めた鍵を値域の表に入れない | 置換 |
| M6 狭めた字を広げた字より先に出す | 歯 1 |
| M7 撤退条件の種類は判断の記録の床に任せて数えない | 歯 1・歯 2 |
| X1 外した値を字の順に並べる | 歯 1 |
| X2 外した値を逆の順に並べる | 歯 1 |
| X7 folio2 の憲法の名のときだけ数える | 歯 1 |
| X8 狭めた字を値域の節の後ろへまとめて出す | 歯 1 |

### (f) 大きさ・verify と done の対応

1. **write-set。** 2 本とも印なし（書き換えるだけ）: `crates/folio/src/check.rs`・`crates/folio/tests/constitution_range.rs`。
2. **余地（CapHeadroom）。** 各行 ceil(字数 / 120)・空行は 1。python と awk の 2 実装で一致。

| file | base の正規化行数（参考値） | 余地 | 模擬の後 | 便の後の余地 |
| --- | ---: | ---: | ---: | ---: |
| `crates/folio/src/check.rs` | 1033 | 467 | 1044（+11） | 456 |

   src の外は `tests/constitution_range.rs` 446 → 621。rustfmt --check（edition 2024）の差の数は check.rs 4 → 4・tests/constitution_range.rs 10 → 9（足した字は fmt に合う）。
3. **size は S。** src は 1 本で +11（字の 1 件と注・頭の注の 2 行は書き換えで行数は同じ）。
4. **verify は 3 行**で、done の 3 の塊と 1 対 1。
   1. `cargo nextest run -p folio --test constitution_range f157_` = (c) の 1・2。
   2. `cargo nextest run -p folio --test constitution_range` = 値域の歯の全部（AC22 の歯と (c) の 3 を含む・参考値 7 本）。
   3. `cargo clippy --workspace --all-targets -- -D warnings`。

   base では 1 が 0 件で終了コード 4、2 は 5 本で緑、3 は 0 警告。`--test constitution_range` の歯の file は write-set に在り、`--bin` の行は無い。

### (g) 門・受付・並行の便

門は対象外で 0（冒頭）。受付の先撃ち（precheck）で契約に起因する断りは 0（(h) の 5）。便 154（行 fa）と便 155（行 fb・anchor.rs と tests/freeze_root.rs）の write-set とは重ならない。便 156（行 fc）は着地済みで base に含む。便 156 の check.rs の差（Materials・check_dir・check_constitution の名札・正規化行 +6）は place_range と頭の注の 6〜7 行に当たらず、本便の差は base eecbff9 の上にそのまま当たって歯は緑（改訂 c で撃ち直した）。

### (h) 数え直す手順（行 D-13）

記録は `.local/share/folio2/handoff-2026-09-27/d157-draft.md`、script は同じ dir の d157-scripts（repo の外）。

1. 再現: repro-157.sh（(a) の 1）・face-157.sh（(a) の 3）。
2. 模擬: clone-157.sh・build-sim-157.sh（apply-157.py と apply-157-teeth.py・c157.patch）・suite-157.sh・floor-157.sh・verify-157.sh。
3. RED: red-157.sh（r157-teeth.patch）。
4. 突然変異・余地・門: mut-157.py・lines-157.py と lines-157.awk・gate-157.sh。
5. 受付の先撃ち: `~/.cache/folio2-orchestrator/r86/precheck.sh <worktree> docs/design/delivery-157.md#fd`。

### (i) 要件との関係・運ばないもの・撤退条件

1. **要件との関係（正本は書き換えない）。** FR25 の規範文は全部そのまま成り立ち（規範文は まだ分からない の場合を閉じない）、本便は狭めた鍵の まだ分からない を足す。AC22 の red_test（部分集合の写しで床が続いて条 1 つが違反 1・外の値で まだ分からない）も成り立つ。**床の側の代価。** 本便の後は、folio2 が値域に値を足すと、足す前の骨格から起こした置き場の床が まだ分からない になり、戻すには置き場の憲法の改訂が要る（面と build は今でもそう止まる＝(a) の 3）。そのうえ ⓐ 凍結の 3 つの旗（--freeze-anchor・--freeze-ids・--freeze-start）は まだ分からない が 1 件でも在れば凍結しないので、値を足す改訂の前に別の改訂を凍結できない（いまは列の根の表に利用者の行が無く表に出ない）。ⓑ ADR-16 の撤退条件 (3) の 3 文目（組み立てた値域の変更だけを理由に利用者の床が落ちた回を数える）が、folio2 の値の追加でも 1 回進む。どちらも値域の 10 鍵が 2026-09-12 の発効から変わっていないのでまれ。**着地の後に古くなる字**（場所と今の字の要点・直しは設計文書の便）: ① ADR-16 決定 (2)(ウ) の 189 行「…同じ鍵の部分集合か」を数える口・190〜191 行（狭めた鍵の まだ分からない を書いていない）・192 行と帰結 434 行と FR25 の注の「folio2 が値域に値を足しても…利用者の床は変わらない」（偽になる）・帰結 435 行の「値を外すか名を変えると…落ちる」（値を足しても落ちる）。② ADR-16 案 a の「値域は部分集合で比べる」・FR25 の題と平易文と注の「部分集合の判定」・AC22 の題「部分集合なら床が続く」（床は続くが、狭めた鍵には まだ分からない が並ぶ）。③ ADR-16 撤退条件 (3) の 3 文目は字は古くならないが数えが変わる（ⓑ）。これらは設計文書の直しで、この便では書かない（次の節目の材料・行 D-16）。check.rs の頭の注は write-set の中なので本便で直す（(b) の 3）。
2. **運ばないもの。** 台帳 f2-648.185（folio face / check / serve の --help の説明の字・`crates/folio/src/main.rs`）は write-set に main.rs が要らないので同乗させず別の S にする。撤退条件の種類の突き合わせ（link.rs）・面・設計文書の正本・台帳への記帳・外部 crate も運ばない。
3. **撤退条件。** (1) (c) の 3 のほかの既存の歯が 1 本でも落ちたら、歯も fixture も直さずに止めて席へ返す。(2) 着地の後の main で folio2 自身の床 4 本の結果か `folio build` の出力が着地の直前と 1 byte でも違ったら止めて席へ返す。(3) 受付の時点の main で check.rs の place_range か、tests/constitution_range.rs の口 Work・Run・built_outside が base と違えば、(h) で数え直してから運ぶ。

## 2. 範囲

- 入れる: §1 (b) の 1〜3、(c) の歯 2 本と置き換え 2 本。
- 入れない: 撤退条件の種類の突き合わせ・面・init の骨格・命令の旗と説明の字（.185）・設計文書の正本・新しい fixture と dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| narrow | 狭めた向き | check.rs の place_range に狭めた鍵の まだ分からない 1 件 |
| teeth | 歯 | tests/constitution_range.rs の f157_ 2 本と置き換え 2 本 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate と新しい dir は無い。前提の着地は無い（base = main eecbff9）。並行の便は §1 (g)。
- 着地の後に席が見ること: 本流の target/debug/folio を組み直す。.195 を閉じる。.185 を別の S として起こす。§1 (i) の 1 の字を次の節目の材料に控える。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "fd"
title = "群 A（台帳 f2-648.195）: 置き場の憲法の値域の節で、組み立てた版の値を外した（狭めた）鍵を床が黙って通す（広げた鍵は まだ分からない）。持ち主の裁定（2026-09-26 棚卸しの問 2・推奨で）どおり、crates/folio/src/check.rs の place_range に、組み立てた版の値のうち置き場に無い値を組み立てた版の順に名指す まだ分からない 1 件（組み立て時の値域に在る値が無い・値域を置き場ごとに狭める口は無い・FR25）を広げた向きの字の直後に足し、関数と file の頭の注を集合の一致に直す。狭めた鍵も値域の表に入れ、違反は出さない。広げた向き・引けない鍵・撤退条件の種類の突き合わせ（link.rs）・面・init の骨格・命令の旗・folio2 自身の出力は変えない。歯は tests/constitution_range.rs の f157_ 2 本（字の順の段と folio init の骨格の置き場の段を含む）と、狭めた値域を黙ると期待していた f122_ 2 本の置き換え（名も替える）。便 154〜156 の着地の後の main に載る。台帳 .185（main.rs の説明の字）は同乗させない。門の対象外。base = main eecbff9・受付の時点の main で数え直す"
req = ["FR25"]
section = "1"
write-set = ["crates/folio/src/check.rs", "crates/folio/tests/constitution_range.rs"]
verify = ["cargo nextest run -p folio --test constitution_range f157_", "cargo nextest run -p folio --test constitution_range", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "tests/constitution_range.rs の f157_ の 2 本（strength から should を外すと改憲の違反 1 件のほかの違反が 0 で まだ分からない がちょうど 狭めた strength の「should」の 1 件・判定 不合格（違反 1・まだ分からない 1）、should を may に替えると 広げた「may」と狭めた「should」の順の 2 件、そのまま binds を tool だけにすると 狭めた binds の「practice」・「both」・広げた「may」・狭めた「should」の順の 3 件、must だけにすると 1 件に「must-not」・「should」、撤退条件の種類から measure を外すと 狭めた retreat_kind の「measure」の 1 件だけ、folio init の骨格の置き場で strength から should を外すと違反 0 のまま まだ分からない が 1 件増えて値域の字はちょうど 狭めた strength の「should」で終了コード 2 / 凍結 anchor の 10 鍵から末尾の値を 1 つずつ外すと まだ分からない が鍵ごとに 1 件で anchor の順にちょうど 10 件）が緑、tests/constitution_range.rs の歯の全部（AC22 の歯と、並べ替えただけの値域は黙り M0 を外した値域で条 P-1 の live M0 が違反 1 件のまま 狭めた mechanism_live の「M0」の まだ分からない 1 件が並ぶ・並べ替えただけの撤退条件の種類は黙る、という置き換えを含む）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）・folio schema --dir design-intent --check が一致・folio inject --check と folio derive --dir design-intent --out ../contracts --check が 0 を返し、folio build の出力は着地の直前の main と file 数も byte も変わらない"
<!-- contracts:end -->
