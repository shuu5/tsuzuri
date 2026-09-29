# 設計: 便 132 — 憲法の面の機構の欄の名札を、機構がまだ無い条の実態に合わせる（判断の記録 ADR-23 決定 (4) の面の名札の便・承認要求の承認の後の欄 17）

- 要件: FR4（読める面を生成する・要件書 第 1.41 版）。面の憲法の条の機構の小窓の字を、正本の憲法の schema 節の機構の段の規則（schema.mechanism_live_rule・第 1.4 版）に合わせる便で、面の型・部品・部品の名札（class）・図は変えない。契約表の行の req は FR4 の 1 つ（main に在る id）。規範文はどれも変えない。
- 条: P-3.3（床の合格を、完成や天井の合格として扱わない＝機構がまだ無い条を「動く」と見せない）/ P-4.2（判定できないものは表に出す）/ P-5.1（名札は値域の型の上の閉じた表で持つ）/ P-6.2（生成物を手で直さない＝面の凍結 anchor は生成器の出力と byte 一致させる）/ P-10.1（独立した凍結 anchor）。
- 出所: 判断の記録 **ADR-23**（憲法 第 1.4 版・発効 2026-09-25・持ち主の承認・対話面 R-8・逐語「すべて承認する」・裁定 id = 台帳 f2-648 notes 2026-09-25 07:03 JST）の決定 (4)「決定 (3) の便と、面の名札の便（実在の段が「いま」（now）でない条の憲法の面の名札「M1 で動く」を「機構がまだ無い（床は判定しない）」の向きの字へ改める・「まだ分からない」の字は使わない・承認要求の承認の後の欄 17）の 2 本が着地するまで、本流の上で天井の周の束を組まない」と、帰結の「面の名札「M1 で動く」は…6 本とも偽の字である」と、承認要求 `docs/design/adr-23.md` の §承認の後に席が書く欄 の 17。憲法の schema 節の機構の段の規則は「…面の機構の欄に条ごとに表に出す」と書く。本便はその「面の機構の欄」の口である（folio check の出力は便 131）。規則の表の開発規律行 D-11 は、この行と契約表の行の title が ADR-23 と裁定 id を名指すことで満たす。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 `ee` が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書きで 7 本。新しい file も縮む file も消す file も無く（`+` も `-` も当たらない）、新しい dir も作らない。面の凍結 anchor `tests/fixtures/face/expected.html` は便の write-set で運ぶ。凍結 anchor を読む歯の file 3 本（`crates/folio/tests/` の下の face.rs・badge.rs・site.rs）は本文を変えないが、verify が `--test` で名指すので write-set に入れる。
- 門: 本便は設計文書の正本（`design-intent/` の下）を 1 本も書き換えないので、天井の門の対象外である。起草役が write-set 7 本を base の binary（main ed966c1）で `folio ceiling --gate` に渡すと **0（通す・断りの字 = 設計文書の正本を書き換えない便）**。
- 前の便: 前提は ADR-23 の発効の取り込み（憲法 第 1.4 版）で、本流 ed966c1 に着地済み。**base = main ed966c1。この契約の数はすべて base の実測（参考値）である**（規則の表の行 D-13）。受付の時点の main が base と違えば、その main で数え直す。
- 並行の便との重なり: 便 131（行 `ed`・folio check の 1 行）とは write-set が 1 本も重ならない。起草役は両便を base に同時に当てた写しで workspace の nextest が全部緑（参考値 866 本）・床 4 本 rc 0 を実測した。席は便 131 → 本便の順で逐次に受け付ける（`crates/folio/tests/sheet.rs` の一時 dir は process id を持たず、同じ host で 2 本の共通の検証を同時に撃つと偶発の Failed が出うる・便 129 / 130 の検証役の実測）。ADR-23 決定 (4) により、便 131 と本便の 2 本が着地するまで本流の上で天井の周の束を組まない。
- 改訂 b（2026-09-25・独立の検証 d131-verify.md〔条件付き支持・blocking 1 は便 131 の側・文面 4〕と席の裁定）: done の塊を verify の 7 行と 1 対 1 に揃えた（凍結 anchor との byte 一致を face・badge・site の 3 つの塊に分けた・文面 1）。(i) の 2 に、種別が human-review か none で now でない条ができた日の問い直しを席が台帳の控えに起こすことを足した（文面 4）。write-set（7 本）・歯・verify（7 行）・size は変えていない。

## 1. 設計

### (a) いま起きていること（実測・base ed966c1）

1. **名札は live の値だけで決まる。** 面の憲法の条の機構の小窓（部品 hint の本文）は、`crates/folio/src/face_constitution.rs` が「<種別の名札>・<live の名札>（<live の意味>）・<段>・<極性> — <注>」の形で組む。live の名札と意味は `crates/folio/src/face_labels.rs` の 2 つの関数 mechanism_live_label と mechanism_live_meaning が、組み立て時に憲法の値域から導出した型 MechanismLive の 5 値（now・M0・delivery-0・M1・adr）から引く。この 2 関数を呼ぶ所は face_constitution.rs の 1 か所と、`crates/folio/src/face.rs` の単体の歯 face_labels_are_frozen_needles_for_the_string_tables（名札の表を字で凍結する歯）だけである。

| live の値 | 今の名札 | 今の意味 |
| --- | --- | --- |
| now | いま動く | 今の folio に在る |
| M0 | M0 で動く | M0 = 要件書の scope の 作る の側に在る段 |
| delivery-0 | 便 0 で動く | 便 0 = 最初の便の段 |
| M1 | M1 で動く | M1 = 要件書の scope_m1 の 作る の側に在る段 |
| adr | 判断の記録の欄の決まりの後 | 判断の記録の欄の決まりが定まった後 |

2. **実の面の 6 本は偽の字。** 実の憲法の 27 条の機構の組は、build-check と now が 15・reject と now が 3・human-review と now が 3・reject と M1 が 3・build-check と M1 が 3 である。live が M1 の 6 本（P-11・P-13・P-15・P-17・P-18・A-3）の小窓は「M1 で動く（M1 = 要件書の scope_m1 の 作る の側に在る段）」と出て、M1 の段を過ぎた今「もう動いている」と読める（ADR-23 の帰結）。第 1.4 版の規則は、この 6 本を機構がまだ無い条として床の判定に数えず面に表に出す、と書く。
3. **面の凍結 anchor。** 面の凍結 anchor `tests/fixtures/face/expected.html`（参考値 25,665 byte）は、fixture の憲法（3 条）から生成した憲法の面で、機構の小窓は build-check と M0 が 1・human-review と now が 1・reject と delivery-0 が 1 である。凍結 anchor を読む歯は 3 本で、`crates/folio/tests/face.rs` の face_write_matches_the_frozen_fixture・`crates/folio/tests/badge.rs` の badge_faces_without_the_mark_match_the_seven_frozen_fixtures・`crates/folio/tests/site.rs` の site_write_matches_the_frozen_fixture である（crates の中でこの凍結 anchor の file 名を読む歯の file はこの 3 本だけ・`git grep` の実測）。
4. **面の歯。** `crates/folio/tests/face_constitution.rs`（参考値 14 本）は、歯の側の名札の表（LIVE_LABELS・5 値）で実の面の機構の小窓の本文を走査し、名札の数を正本の live の値の数と比べる歯 f84_mechanism_chip_explains_the_stage を持つ。走査は小窓の本文の全体（機構の注〔「 — 」の後〕を含む）に掛かる。
5. **base の歯と床（参考値）。** workspace の nextest 861 / 861・床 4 本 rc 0。`git grep -n 'f132_' -- crates` は 0 件。

### (b) 直す先 — live が now でない 4 値の名札を 1 つの字に、意味の括弧に段の値を

1. **名札（face_labels.rs）。** mechanism_live_label は now に「いま動く」を返し（不変）、M0・delivery-0・M1・adr の 4 値には同じ名札「機構がまだ無い」を返す。
2. **意味（face_labels.rs）。** mechanism_live_meaning は now に「今の folio に在る」を返し（不変）、4 値には「床は判定しない・憲法の段の値は <値>」を返す（<値> は M0・delivery-0・M1・adr の正本の字）。実の面の 6 本の小窓の頭は、たとえば P-11 で「機械が拒む・機構がまだ無い（床は判定しない・憲法の段の値は M1）・…」になる。
3. **字の選び。** 「まだ分からない」の字は使わない（ADR-23 決定 (4)・床の 3 値の 1 つと取り違えるため）。段の値は正本の字のまま括弧に残し、機構が着地して now に改めるまで、条がどの段の約束を負っていたかを面から読めるようにする（ADR-23 決定 (1) ⑤ が段の値と意味を変えないことと揃える）。名札は live の値だけで引き、種別の値は見ない（(d) の 3）。
4. **組み立ての口は変えない。** face_constitution.rs の組み方・部品・名札（class）・機械のための面（「live: M1」の字）は変えない。変わるのは 2 関数の返す字と、2 関数の説明の注だけである。
5. **凍結 anchor を合わせる。** `tests/fixtures/face/expected.html` の 2 か所の小窓の字を、生成器の出力に合わせる。

| 条（fixture） | 今の字 | 後の字 |
| --- | --- | --- |
| P-1（build-check・M0） | 生成時の検査・M0 で動く（M0 = 要件書の scope の 作る の側に在る段） | 生成時の検査・機構がまだ無い（床は判定しない・憲法の段の値は M0） |
| N-1（reject・delivery-0） | 機械が拒む・便 0 で動く（便 0 = 最初の便の段） | 機械が拒む・機構がまだ無い（床は判定しない・憲法の段の値は delivery-0） |

   後の凍結 anchor は参考値 25,705 byte（+40）・sha256 の頭 248b6df3ccc62014。起草役は、字の置き換え（python）で作った file と、差分を当てた binary で `folio face --face constitution --dir tests/fixtures/face --write` が出した file が byte 一致することを確かめた（2 実装）。ほかの面の凍結 anchor は変わらない。
6. **folio2 自身の面。** `folio build --dir design-intent --out <置き場> --write` の出力は、憲法の面の 6 行（6 本の条の小窓）だけが変わり、ほかの file は 1 byte も変わらない（起草役の実測・file の大きさの和は参考値 1,875,478 byte から 1,875,484 byte）。床の結果・folio check の出力は変わらない。

### (c) 歯（新しく 1 本・直す 2 本・どれも `crates/folio/tests/face_constitution.rs`）

関数名は f132_ で始める（verify の絞り込みの語・base で 0 件）。

1. **f132_articles_without_a_mechanism_say_so（新しい歯）。** 実の design-intent から組んだ憲法の面の機構の小窓の本文のうち、機構の注（「 — 」の後）の前を頭とする。正本の憲法の各条の live のうち now でない値 v ごとに、頭に「機構がまだ無い（床は判定しない・憲法の段の値は v）」を含む小窓の数が、正本の v の条の数と等しい（base の実の正本では M1 の 6 本）。どの頭にも「まだ分からない」の字と古い名札 4 つ（M0 で動く・便 0 で動く・M1 で動く・判断の記録の欄の決まりの後）が無い。now でない条が正本に 1 本も無ければ、前提が崩れたとして落ちる。本数は正本から数え、歯に 6 を書かない（便 43・44 の向き）。**base では名札の数が 0 で落ちる＝RED**（起草役の実測）。
2. **歯の側の名札の表 LIVE_LABELS を直す。** 4 値の名札を「機構がまだ無い」にする（now は不変）。
3. **f84_mechanism_chip_explains_the_stage の走査を名札の部分に限る。** 名札の数え上げを、小窓の本文のうち機構の注（「 — 」の後）の前だけに掛ける。実の条 P-18 の機構の注は「機構がまだ無い条」の字を持つので、全体を走査すると名札が 1 つ多く数えられて落ちる（起草役の実測）。歯が確かめること（名札の数が正本の live の値の数と揃う・名札の後に意味の括弧が続く）は変えない。**表だけを直した写しを base に当てると f84 も落ちる。**

単体の歯 face_labels_are_frozen_needles_for_the_string_tables（`crates/folio/src/face.rs`）の名札と意味の表を (b) の 1・2 の字へ直す（歯の数は不変・write-set の face.rs）。

### (d) 採らなかった形

1. **名札を「まだ分からない」にする。** ADR-23 決定 (4) が禁じる（床の 3 値の 1 つと読まれ、床が判定しないことと食い違う）。
2. **段の値を名札から消す。** 条がどの段の約束を負っていたかが面から消え、機械のための面の「live: M1」とも繋がらない。意味の括弧に残した。
3. **種別が reject か build-check のときだけ名札を変える。** 第 1.4 版の規則が数えるのは reject と build-check だが、human-review と none の条で live が now でない組は、実の正本にも fixture にも 0 本である。種別で分けると名札の関数が 2 つの値を読むことになり、表が閉じた 1 次元でなくなる。名札は live の値だけで引く（(i) の 2）。
4. **面の注に一覧の節を足す（folio check の 1 行と同じ字を面の頭に並べる）。** 面に新しい部品か節が要り、部品目録（P-2.4）の判断の記録が前置になる。条ごとの小窓で規則の「面の機構の欄に条ごとに表に出す」を満たせる。
5. **便 131 と束ねる。** ADR-23 決定 (3) が「面はこの便では触らない」と決め、承認要求の欄 17 が別の便とした。

### (e) 既存の歯のうち落ちるもの・凍結 anchor が動くか

1. **落ちる既存の歯は 5 本で、どれも本便の write-set で直す。** 面の凍結 anchor と byte 一致を見る 3 本（face.rs・badge.rs・site.rs の (a) の 3 の歯）は、凍結 anchor を (b) の 5 のとおり直せば緑に戻る（本文は変えない）。f84 は (c) の 3 で直す。単体の歯 face_labels_are_frozen_needles_for_the_string_tables は (c) の末尾で直す。起草役の写しで、本便の差分を base に当てて workspace の nextest は 862 / 862（base 861 + 歯 1）・clippy 0 警告。
2. **動く凍結 anchor は 1 本。** `tests/fixtures/face/expected.html` の 2 か所だけ（(b) の 5）。床の凍結の土台・生成区間の写し・値域の anchor・ほかの面の凍結 anchor は 1 byte も変えない。
3. **判断の記録 ADR-5 の撤退条件 ②（見た目だけを直す便の数え）に当たらない。** 本便は面の字の意味を正本の規則（第 1.4 版）に合わせる便で、見た目（部品・色・余白・並び・名札 class）を直さない。撤退条件 ② の数えに名指した契約の文書（便 15・16・20・27・28・35・36）に、憲法の面の機構の小窓を直した便は無い（起草役の `git grep` の実測）。持ち主の walk（面の承認）は求めない。

### (f) 大きさ・verify と done の対応

1. **write-set の印。** 7 本とも印なし（書き換える 4 本 = `crates/folio/src/face_labels.rs`・`crates/folio/src/face.rs`・`crates/folio/tests/face_constitution.rs`・`tests/fixtures/face/expected.html`／本文不変の 3 本 = `crates/folio/tests/` の下の face.rs・badge.rs・site.rs・verify の scope）。
2. **余地（CapHeadroom）。** 測るのは `crates/folio/src/` の下の 2 本。各行を ceil(字数 / 120) で数えて足す（空行は 1・`wc -l` ではない）。起草役は python と awk の 2 実装で数え、一致した。

| file | base の正規化行数（参考値） | 余地 = 1500 − 正規化行数 | 起草役の模擬の後 |
| --- | ---: | ---: | ---: |
| `crates/folio/src/face_labels.rs` | 255 | 1,245 | 256（+1） |
| `crates/folio/src/face.rs` | 1,005 | 495 | 1,005（±0） |

   2 本とも余地は S の見積 100 を超える。
3. **size は S。** src の増分は数行の見積。
4. **verify は 7 行**で、done の 7 つの塊と 1 対 1 に揃える。
   1. `cargo nextest run -p folio --test face_constitution f132_` = (c) の 1 の歯 1 本。
   2. `cargo nextest run -p folio --test face_constitution` = 面の憲法の歯の全部（f84 を含む・参考値 15 本）。
   3. `cargo nextest run -p folio --bin folio face_labels_are_frozen_needles` = 名札の表の単体の歯 1 本。
   4. `cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture` = 面の凍結 anchor との byte 一致。
   5. `cargo nextest run -p folio --test badge badge_faces_without_the_mark` = 名札の無い面の凍結 anchor 7 本との byte 一致。
   6. `cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture` = 組み立ての出力の凍結 anchor との byte 一致。
   7. `cargo clippy --workspace --all-targets -- -D warnings` = 0 警告。
5. **verify の歯の file と write-set。** `--test` で名指す歯の file は face_constitution・face・badge・site の 4 本で、どれも write-set に在る。`--bin folio` の絞り込みの語 face_labels_are_frozen_needles を関数名に持つのは `crates/folio/src/face.rs` の 1 本だけで、write-set に在る。絞り込みの語 f132_ を関数名に持つ file は face_constitution.rs だけで、src には置かない。

### (g) 門と受付

1. **門。** 本便は design-intent の下を 1 本も書き換えないので天井の門の対象外で、実測は 0（通す）（§0 の 門）。
2. **受付の順。** 便 131 の後に逐次に受け付ける（§0 の 並行の便との重なり）。

### (h) 数え直す手順（誰でも撃ち直せる形・規則の表の行 D-13）

起草の記録は持ち主の home の下の `.local/share/folio2/handoff-2026-09-25/d131-draft.md`、模擬の差分は同じ dir の d131-scripts（repo には入れない）。

1. 模擬: 差分 b.patch を base に当て、workspace の nextest（862 / 862）・clippy・床 4 本・`folio build --write` の出力の差（憲法の面の 6 行だけ）。便 131 の差分 a.patch と両方を当てても 866 / 866。
2. RED: 歯の file `crates/folio/tests/face_constitution.rs` だけを base に当てると f132_ と f84 が落ちる。
3. 凍結 anchor: 差分を当てた binary で `folio face --face constitution --dir tests/fixtures/face --write` を撃ち、出力を凍結 anchor と byte で比べる（(b) の 5）。
4. 余地: 持ち主の home の下の `.local/share/folio2/handoff-2026-09-24/d119-draft-lines.py` を repo の根で当てる（便 119 の script・同じ式）。

### (i) 本便が運ばないもの・言えないこと・撤退条件

1. **運ばないもの。** folio check の 1 行（便 131）。憲法の schema 節の段の意味（schema.mechanism_live_meaning の「M1 = M1 で実在する」）と 6 本の条の段の値（ADR-23 決定 (1) ⑤ で変えない）。6 本の条の機構そのもの。機械のための面の字。ほかの面。部品目録と部品の名札（class）。台帳への記帳（席）。外部 crate。
2. **言えないこと。** 次の天井の周が、新しい名札を第 1.4 版の規則の「面の機構の欄に条ごとに表に出す」の充足と読むかは、確率的な審査なので言えない（ADR-23 の撤退条件 (3)）。種別が human-review か none で live が now でない条が将来できたとき、その名札も「機構がまだ無い」になる（規則が数えるのは reject と build-check だけ）。今は実の正本にも fixture にも 0 本で、できたときに名札の字を問い直す（(d) の 3）。この問い直しは散文だけに置かず、席が台帳の控えに起こす。便 131 の行の側は、同じ組を便 131 の歯 1 が決定的に数える。
3. **撤退条件。** (1) 本便の後に (e) の 1 の 4 本の外の既存の歯が 1 本でも落ちたら、その歯の本文も fixture も直さずに止めて席へ返す。(2) 本便の後に `folio build` の出力が憲法の面の機構の小窓の外で 1 byte でも変わったら、止めて席へ返す。(3) 受付の時点の main で、face_labels.rs の 2 関数・face_constitution.rs の小窓の組み方・面の凍結 anchor・憲法の条の機構の欄が base と違っていたら、base を取り直して (a)(b) の字と凍結 anchor を作り直してから運ぶ。(4) 本便が着地できない（契約の審査で落ちる・行 R-7 の回数を超えて失敗する）ときは、席が ADR-23 決定 (4) の順（2 本の着地の後に周を組む）を持ち主に問い直す。

## 2. 範囲

- 入れる: `crates/folio/src/face_labels.rs` の 2 関数の字と注。`crates/folio/src/face.rs` の単体の歯の表。`crates/folio/tests/face_constitution.rs` の歯の側の名札の表と f84 の走査の範囲と新しい歯 f132_ 1 本。`tests/fixtures/face/expected.html` の 2 か所。
- 入れない: face_constitution.rs（src）の組み方・機械のための面・部品と名札（class）・ほかの面と凍結 anchor・folio check・設計文書・新しい dir・外部 crate・台帳への記帳。

## 3. 部品

| id | 名 | 役 |
| --- | --- | --- |
| label | live の名札と意味 | `crates/folio/src/face_labels.rs` の mechanism_live_label と mechanism_live_meaning（now でない 4 値を 1 つの名札に・段の値は意味の括弧に） |
| needles | 名札の表の単体の歯 | `crates/folio/src/face.rs` の face_labels_are_frozen_needles_for_the_string_tables |
| anchor | 面の凍結 anchor | `tests/fixtures/face/expected.html` の 2 か所 |
| teeth | 歯 | face_constitution.rs の f132_ 1 本と LIVE_LABELS と f84 の走査の範囲 |

## 4. 検査（歯）

§1 (c)(e)(f) のとおり。共通の検証は `.vessel.toml` の common-verify（workspace 全体の nextest と clippy）。

## 5. 依存

- 外部 crate は増やさない。新しい dir は無い。host に要る命令は無い。
- 前提の着地: ADR-23 の発効の取り込み（憲法 第 1.4 版）。
- 並行の便: 便 131（行 `ed`）と write-set が重ならない（§0）。受付は便 131 → 本便の逐次。
- 本便の着地の後に席が見ること: 台帳の本便の件を閉じる。便 131 と本便の 2 本が着地したら、本流で天井の周を組む（ADR-23 決定 (4)）。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "ee"
title = "判断の記録 ADR-23（憲法 第 1.4 版・発効 2026-09-25・裁定 id = 台帳 f2-648 notes 2026-09-25 07:03 JST）の決定 (4) の面の名札の便（承認要求の承認の後の欄 17）: 憲法の面の条の機構の小窓で、live が now でない 4 値（M0・delivery-0・M1・adr）の名札を 機構がまだ無い に、意味を 床は判定しない・憲法の段の値は <値> にする（crates/folio/src/face_labels.rs の mechanism_live_label と mechanism_live_meaning・まだ分からない の字は使わない・now は不変）。名札の表の単体の歯（crates/folio/src/face.rs）と歯の側の名札の表を直し、f84 の名札の数え上げを機構の注の前に限り、面の凍結 anchor tests/fixtures/face/expected.html の 2 か所を生成器の出力に合わせる。組み立ての口・部品・名札 class・機械のための面・folio check・設計文書は変えない。歯は f132_ の 1 本（実の面の now でない条の名札の数が正本の値ごとの数と等しく、まだ分からない と古い名札 4 つが無い）"
req = ["FR4"]
section = "1"
write-set = ["crates/folio/src/face_labels.rs", "crates/folio/src/face.rs", "crates/folio/tests/face_constitution.rs", "tests/fixtures/face/expected.html", "crates/folio/tests/face.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/site.rs"]
verify = ["cargo nextest run -p folio --test face_constitution f132_", "cargo nextest run -p folio --test face_constitution", "cargo nextest run -p folio --bin folio face_labels_are_frozen_needles", "cargo nextest run -p folio --test face face_write_matches_the_frozen_fixture", "cargo nextest run -p folio --test badge badge_faces_without_the_mark", "cargo nextest run -p folio --test site site_write_matches_the_frozen_fixture", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "face_constitution の f132_ の歯 1 本（実の design-intent の憲法の面で、live が now でない値ごとに 機構がまだ無い（床は判定しない・憲法の段の値は <値>）の名札を持つ小窓の数が正本のその値の条の数と等しく、どの小窓の名札の部分にも まだ分からない と古い名札 4 つが無い）が緑、face_constitution の歯の全部（名札の数え上げを機構の注の前に限った f84 を含む）が緑、名札の表の単体の歯が §1 (b) の 1・2 の字で緑、§1 (b) の 5 の 2 か所を直した面の凍結 anchor との byte 一致の歯（face）が緑、名札の無い面の凍結 anchor との byte 一致の歯（badge）が緑、組み立ての出力の凍結 anchor との byte 一致の歯（site）が緑、clippy が 0 警告で、workspace の nextest が全部緑で CI が通り、着地の後の main で folio check --dir design-intent が合格（違反 0・まだ分からない 0）を返し、folio build の出力は憲法の面の 6 本の条の機構の小窓の外で変わらない"
<!-- contracts:end -->
