# 係の規律の参照（agent-discipline）

前置きの手引き SKILL.md の決まりの手順と前例である。席の手元の手引き（起草役の共通の守り）の段を、判断の記録 ADR-59 の決定 (6) でここへ移した。検証の群の手順は判断の記録 ADR-61 の決定 (9)。この file も天井で、決まりの正本は門と歯である。

記号: <置き場> は起草の置き場（repo の git config の鍵 tsuzuri.draftsdir）、<名> は起こしの名、<群 id> は群の id。

## 1. 置き場

- 書くのは自分の dir <置き場>/<名>/ の下だけ。成果（notes.md・script・差の file）は <名>/w/ の下に置き、写しで commit しない。
- 試作の写しは <置き場> の直下に clone する（git clone -q --local <repo> <置き場>/try-<名>）。床の確かめの写しは <置き場>/try-<名>-fl。写しから push しない。器の掃除は、直下の写しの target などを書きが 6 時間無くなった後に消す。
- CARGO_TARGET_DIR は <置き場>/<名>/target の絶対 path にする（相対だと xtask の面の組み立てが crates/target に書き、git add -A で commit に混ざる）。ほかの係の target を使わない（古い rmeta で clippy が偽の赤を出す）。済んだ係の target は消さず、w/retired/ の下へ mv する。
- 動いている board の server を止めない。測るなら loopback の別の port で写しの tz を起こし、自分の pid だけを kill する。

## 2. 組み（SSD の書きを抑える）

- /dev/shm などの tmpfs で組まない（頁が席の cgroup に数えられ、席ごと OOM で落ちる）。
- 床の写しは最初の組みから CARGO_INCREMENTAL=0 で撃ち、同じ target で途中から切り替えない（別の hash で組み直し、実行 file が 2 重になる）。編集を繰り返す試作の写しは incremental のまま。
- 試作の途中は cargo nextest run -p <pkg> --test <自分の歯の file> の範囲で撃つ。workspace の全部の組み（cargo run -q -p xtask -- check・nextest --workspace）は行ごとに終わりの 1 回にする。噛みも --test 1 本の範囲で撃つ。
- 撃つ前に自分が触った file を touch する（在る file だけ）。cargo fmt -p は約 150 file を書き替えるので、rustfmt <自分の file> だけを撃つ。
- Rust を書かない係は cargo を撃たず、床は席の pin の tz で確かめる。

## 3. 契約の行の書き方

- 形は置くノートの既存の行に合わせる（title・req・write-set・verify・size・growth・done・depends と節）。
- done の各項は、名指す歯が実際に測る字だけを書き、測らない字は節に書く。歯の名は試作の fn の名と grep で照らす。done の項の中で (n) の印を書かない（ほかの項は「最後の項」などと書く）。
- 行の title と done に、コロンと空白の並び・空白と # の並び・二重引用符・逆斜線・逆引用符を書かない（folio derive が断るか YAML の注で切れる）。JSON は鍵と字の値の囲みを省いた形と、その戻し方を節に書く。
- verify の 1 行目は git apply --reverse --check docs/design/patch/<行>.patch、続けて clippy（-p <歯の crate> --all-targets --no-deps -- -D warnings）と、done の歯を撃つ cargo nextest run -p <pkg> --test <file> --no-tests=fail <接頭辞>_ の行。cargo run -q -p xtask -- check は器の共通 verify が撃つので書かない。
- verify の filter が当たる歯の file は全部 write-set に入れる。書かない着地済みの file は置き場だけの =<path> で足す（無いと受付が teeth-outside-write-set で断る）。
- depends は同じノートの行だけ（ほかのノートの行は folio check が違反にする）。便の順は toml の depends の欄で決まる。
- 大きさは 1 行 M を目安にし、少し越えても M のまま。gate に渡す diff は 150000 byte まで。crate の src の file の 1500 行の上限の余地が見込みより小さい時は growth の欄を足す。
- ノートの行は 32 まで（条 P-28）。計画だけの行を契約の行へ移す時は、同じ commit で規則の行 R-33 の値を下げる。
- 節が大きい行は、頭の段に done の各項と verify の行の対応を書く。審査役は Read・Grep・Glob だけで読むので、節と done に「審査が撃って確かめる」前提の字を書かない。

## 4. 差の file の形

- 行の試作は、差を docs/design/patch/<行>.patch に置く。切り方は、当てる main に前の行の未着地の差を当てた土台の commit の上に試作の commit を作り、git diff --no-color --full-index <土台> <試作> -- <write-set の file>。
- 確かめは、土台の木で git apply --check、試作の木で git apply --reverse --check がどちらも rc 0。差の file は write-set に足さない。
- 節の段は、在りか（差を切った commit）・なぜ・形・実装の約束（便は差を git apply で当てるだけで、差の外の字を書かず、差の file も書き替えない。当たらない時は手で書かずに止まる）・done と verify の対応・歯と噛み・範囲の外。
- 同じ file を書く未置きの行が並ぶ時は、hunk の文脈を 4 行以上離し、一時の index（GIT_INDEX_FILE）で git apply --cached を両順に照らす。
- main が差の触れる file を動かすと差は当たらなくなる。席は置く直前に当たりを撃ち直す。

## 5. 歯の置き方（段の束ね・判断の記録 ADR-32）

- 新しい行の歯は tests/<接頭辞>.rs の 1 本（段）で足し、群 tests/<群>/ の main.rs と module を書かない。群に module を足す行は、xtask/tests/teeth1/kfold.rs の表 GROUPS も直す。
- 着地した群の module を直す行は、その module の file だけを write-set に置き、verify を -p <pkg> --test <群> --no-tests=fail <接頭辞>_ にする。
- 歯の fn の名は crate の中で一意にする。新しい接頭辞は、契約表の verify の filter の語と群の module の名のどれとも部分の字で重ならない。
- 新しい歯の file は、頭の //! の行の続きのすぐ次の行に #![cfg(test)] を置き、間に空の行を挟まない。
- 歯の file を path で読む歯は、tests/ の直下に固定せず、名で探すか include_str! で読む。
- 歯の fn も clippy の cognitive_complexity と too_many_lines で落ちる。歯を直したら clippy を先に撃つ。

## 6. 否定の見本と噛み

- 「読まない・断る」を言う done の項の否定の見本は、正しい見本から測る句を 1 つだけ外す（ほかの欄は全部正しくする）。別の理由で先に断られる見本は空振りである。
- 置き場の字（の後に・1 度だけ）を file 全体の字で測ると、審査が vacuous-assert で落とす。関数か塊の中で照らし、数を断言する。
- 噛みは句ごとに撃つ。その句を見ない変異で、名指しの歯が rc 100 で落ちる（nextest の rc 101 は組み立ての失敗で噛みでない）。表（句・歯・変異・rc）を notes.md に置く。噛みの道具は終わりに書き戻すので、噛みの前に試作を commit する。

## 7. 床と受付の照らし

- 床は頼みが名指す席の pin の tz で、床の写しの根で tz schema --write、(cd design-intent && tz derive --dir . --out ../contracts --write)（--out は --dir からの相対）、tz check、derive --check、tz check --freeze-adrs を撃つ。合否は終了 code で判じる（0 は合格・1 は不合格・2 はまだ分からない・字で判じない）。
- 適用の script（python）は前の版を照らし、ノートと計画の版を上げ、書いた字を YAML で読み直す。外れたら何も書かずに止まる。
- 足した行ごとに scribe2 pipe preflight --state-dir <空の state dir> --repo <床の写し> --design contracts/<ノート>.toml#<行> --bead <在る bead> を撃ち、断りの理由を 0 にする。

## 8. 出す物と公開

- 出す物は、notes.md（調べ・実測・未確か・席への問い・印は V 確かめた・D 記録から・I 見立て・U 分からない）、適用の script、commit の文の案、差の file、席が撃つ命令の列。出す物の file の名に report を使わない。
- 持ち主への問いが要るなら、概要・技術・理由・推奨の閉じた問いにする。数は完全な記録からだけ書く。
- repo は公開である。host の名・網の住所・口座の名・ほかの project の名・持ち主の逐語を書かない。

## 9. 道具の撃ち方

- gh は頭の語に置き、出力は > か | で受ける。置換・for・heredoc の中の gh と git の字は、読むだけでも器の門が断る。書き換えの script は Write の道具で file に書き、python3 <file> で撃つ。
- 背景で撃った物の終わりの知らせは届かないことがある。log の rc の行と ps で自分で見る（pgrep -f の待ちは自分の bash の行に当たって終わらない）。

## 10. 検証の群（判断の記録 ADR-61）

1. 全文を読む攻めの係（検証役 1 体・頭の予算だけ・群の上限に入らない）が、分からない（U）の主張の一覧 <置き場>/<群 id>/unknowns.tsv を書く（1 行に id・主張・判じられない訳・資料の path をタブで区切る）。
2. 一覧が 3 行未満なら群を起こさない。18 行を越えたら、下書きを割って攻め直すか 1 体の道に戻る（割った下書きは同じ対象なので、群は 7 日に 1 つの塊だけ）。下書きの種類は 問いの本文・判断の記録・消す行・緩める問い の 4 つだけ。
3. 席は群を起こす前に <置き場>/<群 id>/plan.json を置く。欄は type（tsuzuri:verifier）・target（頭の対象と同じ）・draft・attacker・members（name・model・claims・paths）と、在れば ruling。1 体の割りの主張は 6 まで、係どうしで重ねず、和は一覧の全部。
4. 反証役 3 体までを 1 つの返事で並べて起こす。頭は 4 行に 5 行目 群: <群 id> <i>/<k> を足し、予算は token 150000 以下、Agent の呼びの model は計画の値。群の係は続けず、段ごとに新しい係を起こす。
5. 群の合計が止める線（新しい量 405000・読み直し 6750000）に届いたら、新しい群の係を起こさない。同じ対象の群は 7 日に 1 つ（例外は、問いの本文が群の id を名指した裁定の裁定 id を計画の ruling に書く）。
6. 席の流れの道具（Workflow）で係を起こさない（起こしの門が全部断る・使うなら持ち主に問う）。
7. 群の終わりに、問いの本文か判断の記録の概要と推奨が頼る主張の全部と、記録から（D）1 件以上と、確かめた（V）2 件以上を席が照らし直す。外れが 1 件でも在れば群の全部の主張を下げる。

大きな調べは 1 体ずつにする。予算の 100% に届いた調べ係は、調べた所と残りの path を出す物に書く。席は残りを、前の係の終えの印の後に新しい 1 体に頼む（判断の記録 ADR-61 の決定 (3)）。

## 11. 畳みの行（席の手）

- 着地した段の file（群に入っていない tests/ の直下の .rs）が、未着地の行が名指す file を除いて 12 本に達したら、畳みの行を 1 行置く。
- 切り方は、置く直前の main の写しで crate ごとに畳みの道具を撃ち、表 GROUPS も書き直す。差は git diff --no-color --full-index -M で切り、130000 byte を目安に、越えれば crate ごとに割る。
- 確かめは、行の木で verify の行を全部撃ち、xtask の check は全部の畳みを当てた最後の木で 1 回。着地した行の verify の字と差の file は歴史として書き換えない。
