# 設計: 便 83 — 面の天井の名札を束の置き場でなく天井の印から読み、名札に説明の小窓を付ける（台帳 f2-648.113・天井 17 周目 読みやすさ F-5・FR18 / FR4）

- 要件: FR18（所見の形と天井の 3 値を床で数え、観点ごとの 3 値と日付を面に天井の名札として出す）/ FR4（人が読むページを 1 つの生成器から出す）
- 条: P-6.3（同じ内容を 2 つの面が持つとき一方を正本とし他方は導出する）/ P-4.1（検査・生成が実行できなかった結果を異常なしとして扱わない）/ P-4.2（判定できないものは まだ分からない として表に出す）/ P-2.1（人が読むページはすべて 1 つの生成器から出力する）
- 出所: 台帳 f2-648.113（天井の 16 周目 実態 F-5 で起票）「面の天井の名札（folio build --ceiling が束の置き場を読む形）を、生成物の印 design-intent/preview/ceiling-stamp.yaml を読む形に変える」。あわせて一括 10 の仕分け C の 17 周目 読みやすさ F-5「面の頭の天井の名札に説明の吹き出しか用語集への導線を付ける」。
- 位置: **便 87（face.rs の名札の切り出し）の後**。初版は受付の cap で断られた（器は crates/folio/src/face.rs の余地を 93 行と測り、size S の見積 100 行に届かない）。便 87 が face.rs の 480 行から 787 行（名札と名札の表・308 行）を crates/folio/src/face_labels.rs へそのまま移し、余地を約 398 行に空ける。本便は便 87 の着地の後に受け付ける。
- 置き場: この文書は folio2 の設計ノート。契約表は末尾の区間。審査の材料は行 cf が指す §1 だけなので、判定に要る材料は §1 に全部置く。write-set は手書き。
- 改訂 b（2026-09-22）: 便 87 を前提にし、実測を器の行数の式（空行を含む全行 + 120 字を超える行の折り返し）で測り直した。運ぶ中身は初版と同じ。
- 門: 本便は design-intent の下の正本を 1 file も書き換えない（触るのは crates/folio/ と tests/fixtures/face/ だけ・印 design-intent/preview/ceiling-stamp.yaml も触らない）ので、天井の門（folio ceiling --gate）は 通す を返す。

## 1. 目的と中身

天井の結果は今、2 つの経路で面と門に届く。門（folio ceiling --gate）は印 design-intent/preview/ceiling-stamp.yaml を読む。面の天井の名札は印を読まず、旗 --ceiling が指す束の置き場を歩いて観点ごとに数え直す。同じ事実を 2 つの口が別々に作っており、P-6.3 の「一方を正本とし他方は導出する」に反する。旗を渡し忘れれば名札は 未実施 になり、旗が古い周の置き場を指せば名札だけが古い周を映す。本便は名札の出所を印 1 つに寄せ、旗を外す。あわせて、名札が何を言っているのか面から分からない（3 値の意味も、観点の 4 つが何かも面に無い）ので、名札に説明の小窓を 1 つ付けて用語集へ導く。

orchestrator 席の実測（2026-09-22・main bb80fef・便 79 / 80 / 81 の着地後。便 87 の切り出しより前の行の番号）:

- 名札の字を組むのは crates/folio/src/face.rs の ceiling_stamp（369〜406 行）。引数は正本の置き場と、束の置き場（解決済み・無しも可）。crates/folio/src/findings.rs の stamps（89〜107 行）が束を歩いて観点ごとに 3 値・起動の記録の at・要約値の先頭 8 字を数え、viewpoint_names（111〜133 行）が天井の正本 ceiling.yaml の viewpoints の id と名を正本の順で返す。日付は観点の at のうち読めたものの byte 順の最大で、読めなければ 日付なし。置き場が無ければ 4 観点とも まだ分からない で 未実施。
- 名札を site-bar に置くのは同じ file の Frame の head（本便の受付の前の main では 914〜917 行。便 87 が 480 行から 787 行を移すので、この行は約 305 行くり上がる。行の番号でなく関数の名で当たること）で、部品 ceiling-stamp の span に ceiling_stamp の返り値をそのまま入れる。床の名札（freshness-stamp）の直後。
- ceiling_stamp を呼ぶのは 5 面の derive だけ（face_constitution.rs 134 行・face_srs.rs 260 行・face_adr.rs 210 行・face_note.rs 268 行・face_index.rs 284 行）。
- 印を書くのは crates/folio/src/stamp.rs。置き場は同 file 22 行の STAMP_FILE（preview/ceiling-stamp.yaml）。欄は round・at・verdict・sources・faces・viewpoints・refutes・reads の 8 つ（同 file 6 行）。viewpoints の行は id・verdict・findings・stops・bundle・model・effort・at を持ち、bundle は要約値の先頭 8 字である。top-level の at は観点の at の byte 順の最大（同 file 122〜127 行）で、名札が今使っている日付の規則と同じ。
- 印を読む既存の口は crates/folio/src/gate.rs の read_stamp（128〜162 行）1 つだけで、非公開かつ sources と viewpoints の（id・verdict）しか持たない。日付も要約値も読まない。名札に要る欄を読む口は今どこにも無い。
- 旗 --ceiling は crates/folio/src/main.rs の Face（123〜125 行）と Build（164〜166 行）の 2 か所で宣言され、face::run（main.rs 373 行）と site::run（main.rs 416 行）へ渡る。旗を実際に渡す歯は crates/folio/tests/badge.rs の 9 か所だけ（ほかの一致はどれも説明の文の中の言及）。
- 小窓は face.rs の hint（358〜363 行・名札付き）と hint_q（409〜414 行・「?」の形）。この 2 本は便 87 が移す区間より前に在るので行の番号は変わらない。用語集への導線の既存の形は同 file の glossary_chip（便 87 の前は 1010〜1021 行・便 87 で約 305 行くり上がる）で、付録の表 ANNEXES から章の番号を引き、constitution.html のその章へリンクする。ANNEXES は便 87 で crates/folio/src/face_labels.rs へ移るが、face.rs が新しい module を丸ごと再輸出するので、本便は今までどおり face.rs の中から ANNEXES の名で読める（face_labels.rs は 1 byte も触らない）。
- 様式 design-intent/preview/folio.css は 95 行で部品 ceiling-stamp を inline-flex・折り返しなしで定義し、588〜599 行で小窓を position:relative の inline-block、小窓の本体を position:absolute で折り返しありに定義する。小窓を名札の中に置いても様式は 1 byte も要らない。部品 --check は面の class が folio.css に在ることを見る（crates/folio/src/parts.rs 233〜242 行）ので、使う class は既存の hint・hint-btn・hint-body・vh だけにする。
- 天井の 21 周目の印（design-intent/preview/ceiling-stamp.yaml）の実測: 4 観点とも 合格、top-level の at は 2026-09-21T14:08:10Z、観点の bundle は 8dead31e / 22aa40dc / af59959e / 53588227。

### (a) 印を読む口（stamp.rs）

stamp.rs に、名札のために印を読む公開の口を 1 本足す。書く側と読む側が同じ file に並ぶので、欄の名の写しが 2 か所に分かれない。

1. 公開の型 Mark { id, verdict, at, bundle }（どれも文字列）。
2. 公開の関数 marks(dir) -> R<Option<(String, Vec<Mark>)>>。返りの 1 つ目は印の top-level の at。
   - 印の file が無ければ Ok(None)（名札は 未実施 のまま・旗が無かったときと同じ扱い）。
   - file が symlink なら Err。
   - 読めない・parse できない・欄 at が読めない・viewpoints が一覧でない・行の id / verdict / at / bundle のどれかが読めないは Err（面は導出できない・終了コード 2・P-4.1）。エラーの字は印の相対 path と読めなかった欄を含める。
   - 行の順は印の順のまま返す。

### (b) 名札の出所を印にする（face.rs）

1. ceiling_stamp の引数から束の置き場を外し、ceiling_stamp(dir) -> R<String> にする。
2. 中身は stamp::marks(dir) と findings::viewpoint_names(dir) の 2 つだけを読む。
   - 印が無ければ 今の 未実施 の字（「天井 <b>{名} まだ分からない</b> · …（未実施）」）をそのまま出す。
   - 在れば、印の行の id の列が天井の正本の viewpoints の id の列と順まで同じであることを確かめ、違えば Err（字は 印の観点が天井の正本と違う 旨と、食い違った id を含める）。印と正本の食い違いを黙って通さない（P-4.1）。
   - 字は 「天井 <b>{名} {3 値}</b> · …（{印の at}・束 {bundle を / で繋いだもの}）」。名は正本の逐語（今までどおり）。日付と要約値は印の値をそのまま使う（面の側で数え直さない）。
3. findings.rs の stamps と型 Stamp を外す（読み手が無くなるため。count_viewpoint と viewpoint_names は天井の検査と印の書き手が使い続けるので残す）。

### (c) 旗を外す（main.rs・site.rs・5 面）

1. main.rs の Face と Build から引数 ceiling を外し、face::run と site::run の呼びからも外す。命令は 1 本も増やさず減らさない（tests/check.rs の p1_commands_closed_list が凍結する 11 本は不変）。
2. face::run と site::run の引数から束の置き場を外す。site.rs の先頭の説明（10 行）を、名札の出所が印であることに合わせて書き換える。
3. 5 面の derive の引数から束の置き場を外し、ceiling_stamp(dir) を呼ぶ形にする（1 面 2 行ずつ）。

### (d) 名札の説明の小窓

ceiling_stamp の返り値の末尾に、hint_q(<本体>) を 1 つ足す（名札の字の後ろ・同じ span の中）。本体は面によらず同じ字で、次の 3 つを p で並べる。

1. 「天井は AI が意味を読む検査です。観点ごとに 合格・不合格・まだ分からない の 3 値を出します。1 つでも まだ分からない が在れば合格にしません。」
2. 「括弧の中は、最後に数えた日付と、観点ごとの材料の束の要約値の先頭 8 字です。」
3. 用語集への導線 1 本。行き先と字の組み方は glossary_chip と同じ規則（ANNEXES の付録 語彙 の章の番号を引き、constitution.html のその章へ）。リンクの字は 「用語集（天井の判定の印）」。
   
床の名札（freshness-stamp）には小窓を付けない（本便の範囲の外）。

### (e) 歯（関数名は f83_ で始める。`grep -rn 'fn f83_' crates/folio/tests` は今 0 本＝席が 2026-09-22 に実測。使われている接頭辞の最大は f78_）

crates/folio/tests/badge.rs に置く（天井の名札の歯の置き場）。旗を渡していた既存の 9 か所は、写しの置き場に印を書いてから面を書く形に直す。

1. f83_stamp_comes_from_the_mark: 写しの置き場に印を 1 本書き（4 観点とも 合格・top-level の at と観点ごとの bundle を持つ）、5 面のどれかを書く → 0 ∧ 名札の字が、歯の側で印の値から独立に組んだ 「天井 <b>{名} 合格</b> · …（{at}・束 {8 字}/{8 字}/{8 字}/{8 字}）」 と一致する。本便の前の main では旗が無いので 未実施 になり赤い歯。
2. f83_stamp_says_not_run_without_the_mark: 印を置かない写しでは 4 観点とも まだ分からない ∧ 括弧の中が 未実施（今の字面のまま）。
3. f83_stamp_is_unknown_when_the_mark_is_broken: 印の viewpoints の 1 行から verdict を落とした写しで面を書く → 2（まだ分からない）∧ 標準エラーに印の相対 path と読めなかった欄の名。
4. f83_stamp_refuses_a_mark_that_disagrees_with_the_source: 印の viewpoints の行を 1 つ別の id に替えた写しで面を書く → 2 ∧ 標準エラーに食い違った id。
5. f83_the_ceiling_flag_is_gone: binary に face の旗として --ceiling を渡す → 終了コードが 0 でない ∧ 標準エラーに未知の引数の旨。build でも同じ。
6. f83_five_faces_carry_the_same_hint: 印を置いた写しで 5 面を書き、どの面にも名札の中に小窓が 1 つ在り、小窓の本体の字が 5 面で byte 一致し、その中に 3 値の 3 語と、constitution.html の付録の章へのリンクが在る。リンクの行き先は歯の側で語彙の付録の章の番号から独立に組む。本便の前の main では小窓が 0 個なので赤い歯。
7. 回帰（期待不変・verify の 2 行目）: crates/folio/tests/badge.rs の既存の歯（旗の分岐を印の分岐へ直したもの）・tests/face.rs・tests/site.rs・tests/face_index.rs・tests/face_adr.rs・tests/face_note.rs・tests/check.rs の既存の歯すべて。天井の検査と印と門の歯（tests/ceiling.rs・tests/stamp.rs・tests/gate.rs・tests/findings.rs）は共通の検証がまとめて回す。

### (f) 凍結の写し

(d) の小窓は 5 面すべての名札に入るので、凍結の写し 7 本を同じ着地で生成し直して置き換える（手で直さない・歯と同じ経路で binary に face を当て、その出力を写す）。写しの置き場 tests/fixtures/face/ は印を持たないので、(b) の直しは写しの名札の字を変えない（未実施 のまま）。

- tests/fixtures/face/expected.html・expected-index.html・expected-index-sheet.html・expected-srs.html・expected-adr.html・expected-note.html・expected-site-adr-2.html

### (g) 大きさ

行数は器の式（空行を含む全行を数え、字数が 120 を超える行は 切り上げ(字数 ÷ 120) − 1 だけ足す）で測る。実測は main bb80fef（便 79 / 80 / 81 の着地後）。

src は 10 本。crates/folio/src/stamp.rs（277・余地 1223・見積は + 約 55 行）／face.rs（今 1407・余地 93 だが、**便 87 の着地で約 1102・余地 約 398** になる・本便の見積は + 約 20 行）／findings.rs（1144・余地 356・約 20 行減る）／main.rs（542・余地 958・約 10 行減る。便 87 で + 1 行）／site.rs（262・余地 1238・約 3 行減る）／face_constitution.rs（1289・余地 211）・face_srs.rs（1398・余地 102）・face_adr.rs（853・余地 647）・face_note.rs（1023・余地 477）・face_index.rs（1378・余地 122）はどれも引数を 1 つ外す差し替えで、**行は 1 行も増えない**（長い行が短くなるだけ）。だから余地 102 の face_srs.rs と余地 122 の face_index.rs も受付を通る。size **S**（src の足し合わせは 100 行に届かない）。外部 crate は増やさない。face_labels.rs（便 87 の新しい module）・部品目録・様式・design-intent の下の正本と印・tests/floor_cases.yaml・vendor/archify/・CI の yml は触らない。

### (h) 本便が運ばないもの

正本 design-intent/srs.yaml の要件 FR18 の注（旗 folio face --ceiling と folio build --ceiling を名指している）。この注は本便の着地で実態と食い違うので、着地の後に席が別の PR で直す（生成区間の外・規範文と平易文は不変・便 77 の §1 (i) と同じ運び）。同じく docs/design/delivery-40.md・delivery-42.md・docs/design/ceiling-gate.md の旗の言及も席が直す。印そのもの（design-intent/preview/ceiling-stamp.yaml）は本便で 1 byte も触らない。床の名札（freshness-stamp）の小窓。天井の検査・印の書き手・門の判定の式。

### (i) 運用の変わる所（席への申し送り）

本便の着地の後、天井の周の手順は「周を回す → folio ceiling --stamp で印を書く → folio build で面を出す」の順になる（面が印を読むので、印を書く前に面を出すと 1 周前の名札が出る）。旗 --ceiling は無くなるので、周回の手順から外す。

## 2. 範囲

- 入れる: 印を読む口 1 本と型 1 つ・名札の出所の差し替え・印と正本の食い違いの断り・旗を外すこと・5 面の呼びの差し替え・説明の小窓 1 つ・歯 6 本と既存の歯の旗の分岐の直し・凍結の写し 7 本の生成し直し。
- 入れない: 正本と印の中身・要件書の注と設計ノートの字・門と天井の検査の判定・命令の増減・床の名札・新しい class と新しい部品・部品目録・様式。

## 3. 部品

| id | 名 | 役 |
|---|---|---|
| mark | 印の読み手 | stamp.rs の Mark と marks（8 欄のうち名札に要る 4 つ） |
| stampline | 名札 | face.rs の ceiling_stamp（印と天井の正本だけを読む） |
| skew | 食い違い | 印の観点の列と天井の正本の列のずれを Err に落とす |
| flag | 旗 | main.rs / site.rs / 5 面から --ceiling を外す |
| hint | 小窓 | 名札の中の hint_q 1 つ（3 値の意味 + 用語集への導線） |
| teeth | 歯 | crates/folio/tests/badge.rs の f83_ 6 本 |
| frozen | 凍結 | tests/fixtures/face/ の写し 7 本の生成し直し |

## 4. 検査（歯）

§1 (e) のとおり。共通の検証は .vessel.toml の common-verify。

## 5. 依存

外部 crate は増やさない。

<!-- contracts:begin -->
schema = 1

[[contract]]
id = "cf"
title = "面の天井の名札の出所を、旗が指す束の置き場から天井の印 preview/ceiling-stamp.yaml の 1 つに寄せ（日付と要約値は印の値をそのまま使い、印の観点の列が天井の正本と違えば まだ分からない に落とす）、旗 --ceiling を face と build から外し、名札に 3 値の意味と用語集への導線を持つ説明の小窓を 1 つ付ける（印と正本は 1 byte も触らない・命令は 11 本のまま・凍結の写し 7 本を生成し直す・台帳 f2-648.113 と天井 17 周目 読みやすさ F-5）"
req = ["FR18", "FR4"]
section = "1"
write-set = ["crates/folio/src/stamp.rs", "crates/folio/src/face.rs", "crates/folio/src/findings.rs", "crates/folio/src/main.rs", "crates/folio/src/site.rs", "crates/folio/src/face_constitution.rs", "crates/folio/src/face_srs.rs", "crates/folio/src/face_adr.rs", "crates/folio/src/face_note.rs", "crates/folio/src/face_index.rs", "crates/folio/tests/badge.rs", "crates/folio/tests/face.rs", "crates/folio/tests/site.rs", "crates/folio/tests/face_index.rs", "crates/folio/tests/face_adr.rs", "crates/folio/tests/face_note.rs", "crates/folio/tests/check.rs", "tests/fixtures/face/expected.html", "tests/fixtures/face/expected-index.html", "tests/fixtures/face/expected-index-sheet.html", "tests/fixtures/face/expected-srs.html", "tests/fixtures/face/expected-adr.html", "tests/fixtures/face/expected-note.html", "tests/fixtures/face/expected-site-adr-2.html"]
verify = ["cargo nextest run -p folio --test badge f83_", "cargo nextest run -p folio --test badge --test face --test site --test face_index --test face_adr --test face_note --test check", "cargo clippy --workspace --all-targets -- -D warnings"]
size = "S"
done = "f83_ の歯 6 本（名札が印の値と一致・印が無ければ未実施・印が壊れていれば まだ分からない・印と正本の観点が違えば まだ分からない・旗が受け付けられない・5 面の小窓が同じ字で用語集へ導く）が緑、tests/badge.rs と tests/face.rs と tests/site.rs と tests/face_index.rs と tests/face_adr.rs と tests/face_note.rs と tests/check.rs の既存の歯が全部緑（命令 11 本の閉じた一覧の歯と、生成し直した凍結の写し 7 本との byte 一致の歯を含む）、clippy が 0 警告で CI が通る"
<!-- contracts:end -->
