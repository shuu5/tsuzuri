# 係の規律の参照（agent-discipline）

前置きの手引き SKILL.md の決まりの手順と前例である。席の手元の手引き（起草役の共通の守り）の段を、判断の記録 ADR-59 の決定 (6) でここへ移した。検証の群の手順は判断の記録 ADR-61 の決定 (9)。この file も天井で、決まりの正本は門と歯である。判断の記録 ADR-63 の決定 (2) で、席の起草の決まりの文はこの file だけに置き、頼みの文はこの file を名指す。

記号: <置き場> は起草の置き場（repo の git config の鍵 tsuzuri.draftsdir）、<名> は起こしの名、<群 id> は群の id。

## 1. 置き場

- 書くのは自分の dir <置き場>/<名>/ の下だけ。成果（notes.md・script・差の file）は <名>/w/ の下に置き、写しで commit しない。
- 試作の写しは <置き場> の直下に clone する（git clone -q --local <repo> <置き場>/try-<名>）。床の確かめの写しは <置き場>/try-<名>-fl。写しから push しない。器の掃除は、直下の写しの target などを書きが 6 時間無くなった後に消す。
- 本物の repo には一時の物も書かない。一時の index（GIT_INDEX_FILE）の照らしも写しの中で撃つ。
- TMPDIR は /tmp/<名>-t にする（係の dir の下は repo の中で歯が落ち、長い path は tmux の socket の path の上限で落ちる）。
- CARGO_TARGET_DIR は <置き場>/<名>/target の絶対 path にする（相対だと xtask の面の組み立てが crates/target に書き、git add -A で commit に混ざる）。ほかの係の target を使わない（古い rmeta で clippy が偽の赤を出す）。例外は、席が頼みの文で同じ写しと target を名指した、別々の tests/<file>.rs を割る係どうしで、git の書きの命令を撃たず、撃つ cargo は自分の file の --test だけにする（src を割る行は共有しない）。済んだ係の target は消さず、w/retired/ の下へ mv する。
- 動いている board の server を止めない。測るなら loopback の別の port で写しの tz を起こし、自分の pid だけを kill する。

## 2. 組み（SSD の書きを抑える）

- /dev/shm などの tmpfs で組まない（頁が席の cgroup に数えられ、席ごと OOM で落ちる）。
- 床の写しは最初の組みから CARGO_INCREMENTAL=0 で撃ち、同じ target で途中から切り替えない（別の hash で組み直し、実行 file が 2 重になる）。編集を繰り返す試作の写しは incremental のまま。
- 試作の途中は cargo nextest run -p <pkg> --test <自分の歯の file> の範囲で撃つ。workspace の全部の組み（cargo run -q -p xtask -- check・nextest --workspace）は行ごとに終わりの 1 回にする。噛みも --test 1 本の範囲で撃つ。
- 撃つ前に自分が触った file を touch する（在る file だけ）。同じ target で 2 つの木（写し・worktree）を撃ったら、木の間で違う file（git diff --name-only）と git ls-files の build.rs を全部 touch してから撃つ（cargo は path の crate を相対の path で見分け、build.rs は path の字を焼くので、別の木の組みを新しいと見る）。clippy だけが落ちて nextest が通る時と、変異が噛まない時は、まずこれを疑う。
- 器の入れ子の binary（<target>/nested/scribe2）は、別の写しで xtask を組むと上書きされる。試作の binary で撃つ前に strings <binary> | grep -c <足した字> で中身を照らし、0 なら試作の写しで組み直す。
- cargo fmt -p は約 150 file を書き替えるので、rustfmt <自分の file> だけを撃つ。着地済みの file は、main の字を一時の file に写して rustfmt --edition 2024 --check <一時の file> の rc で形かを見る（stdin の --check は常に rc 0）。形でない file には掛けず、足す行だけを rustfmt の形で手で書く。
- Rust を書かない係は cargo を撃たず、床は席の pin の tz で確かめる。

## 3. 契約の行の書き方

- 形は置くノートの既存の行に合わせる（title・req・write-set・verify・size・growth・done・depends と節）。
- done の各項は、名指す歯が実際に測る字だけを書き、測らない字は節に書く。歯の名は試作の fn の名と grep で照らす。done の項の中で (n) の印を書かない（ほかの項は「最後の項」などと書く）。
- 不在を見る歯（見張り）には「無い」だけを、等しさを見る歯には「一致する」だけを言わせ、替え先の一覧・変わらない行・読む順は節に移す。見張りは等しさでなく包み（⊇）で見て、字の残りを見る歯はその歯の file 自身を除くと done と歯の両方に書く。
- 置き場を「impl X の中」と名指す時は、X を base の宣言と grep で照らす。欄の名を一括で置き換えた後は、git diff <base> の - と + の fn の行で歯の名が壊れていないかを見る。
- done が節の字や値を言う時は、歯が比べる字そのもの（値の具体）と、数を決める着地済みの fn の数え方を節に書く（材料が節に無いと審査が section-material-missing で落とす）。write-set が 30 項を越える行は審査の材料から file の字が落ちるので、節に今の字の段と試作と噛みの段を置く。
- done に行の番号と行の数を持たせない。後の行の節には、前の行の試作の行の数・行の番号・試作と同じという断言・前の行の fixture から計算した数を書かず、前の行の着地の後に着地した字で当て直す。字を数える歯は doc と注の行を除いて数える。
- 名が verify の filter の語を含まないことを測る歯（own_names）を持つ行は、節に語の一覧（main の commit・語の数 N・空白で区切る）を字で置き、done は歯の file がその N 語を字のまま持って数 N を断言する形にする。
- 行の title と done に、コロンと空白の並び・空白と # の並び・二重引用符・逆斜線・逆引用符を書かない（folio derive が断るか YAML の注で切れる）。JSON は鍵と字の値の囲みを省いた形と、その戻し方を節に書く。
- verify の 1 行目は git apply --reverse --check docs/design/patch/<行>.patch、続けて clippy（-p <歯の crate> --all-targets --no-deps -- -D warnings）と、done の歯を撃つ cargo nextest run -p <pkg> --test <file> --no-tests=fail <接頭辞>_ の行。cargo run -q -p xtask -- check は器の共通 verify が撃つので書かない。
- verify の filter が当たる歯の file は全部 write-set に入れる。書かない着地済みの file は置き場だけの =<path> で足す（無いと受付が teeth-outside-write-set で断る）。
- write-set の印は 6 つ（素の path は在る file・末の / は dir・+ は新しい file・- は縮むが残る file・~ は着地で消える file・= は置き場だけ）。git mv で移す行は、移す元を ~、移す先を + で書く（- の file が無くなると行が永久に解けない）。
- 前の行が足す file を =<path> で名指す行は、前の行の着地の後に置く（受付は置く時の base で = を解き、depends では足りない）。床と preflight は、前の行の差を着地の形で commit した branch で撃つ。
- depends は同じノートの行だけ（ほかのノートの行は folio check が違反にする）。便の順は toml の depends の欄で決まる。
- 行は数珠つなぎにしない。割る時は、まず互いに depends を持たない切り方（触る file が交わらない・交わっても差が両順で当たる）を探す。depends は、後の行の code が前の行の code を呼ぶか前の行の歯を書き替える時だけに書き、書いた時はなぜ並べられないかを節に 1 文で書く。
- 置き場の物差しの順は、領域（domain）の親に置く、交わる時は差の塊の文脈を離して両順で当てる（判断の記録 ADR-60）、それでも当たらなければ depends を書く。交わりのために置き場・名・並びを曲げた行は、節に「置き場の妥協」の 1 文を書く。
- 大きさは 1 行 M を目安にし、少し越えても M のまま。gate に渡す diff は 150000 byte まで（gate は移動を畳むので、git diff を --no-renames を付けずに測る）。crate の src の file の 1500 行の上限の余地が見込みより小さい時は growth の欄を足す。
- ノートの行は 32 まで（条 P-28）。行の数は contracts/<ノート>.toml の id = の行で数える（適用の script の節の番号は行の数でない）。計画だけの行を契約の行へ移す時は、同じ commit で規則の行 R-33 の値を下げる。
- 節が大きい行は、頭の段に done の各項と verify の行の対応を書く。審査役は Read・Grep・Glob だけで読むので、節と done に「審査が撃って確かめる」前提の字を書かない。
- 行の節（契約表の goal に写る字）に歯の本数を書かない（本数の正本は欄 done-teeth・条 P-2.3）。
- 根の src の module の頭の doc に行の id を書かない（行の id は契約と commit が持つ）。
- 器の案内の行（help）の幅は、器の入れ子の歯（接頭辞 cli_help_）が落とす。

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
- 実装の source の字を照らす歯を足す時は、振る舞いで測れない訳を節に 1 文書く。
- 新しい歯の file は、頭の //! の行の続きのすぐ次の行に #![cfg(test)] を置き、間に空の行を挟まない。
- 歯の file を path で読む歯は、tests/ の直下に固定せず、名で探すか include_str! で読む。
- 歯の fn も clippy の cognitive_complexity と too_many_lines で落ちる。歯を直したら clippy を先に撃つ。
- 子の環境を比べる歯は、名の列を先に assert_eq し、名が合った後にだけ値を比べる（値ごとだと、落ちた時に係の環境の値が出力に出る）。噛みの log は left と right の行を伏せる。

## 6. 否定の見本と噛み

- 「読まない・断る」を言う done の項の否定の見本は、正しい見本から測る句を 1 つだけ外す（ほかの欄は全部正しくする）。別の理由で先に断られる見本は空振りである。
- 否定の句ごとに、src で最初に断る検めを照らしてから見本を書き、断りの字も照らす。句の型ごとに見本を置く。順は 2 つ以上で入力・数・字・逆の順がどれも違う並び、重ねないは同じ物を 2 度、縁は N と N+1 と値そのもの、「だけ」は外れる物。
- 判じの呼び（match の式）を字で見る歯は、腕（Ok(false) => continue など）も字で見る（呼びだけだと腕を替える変異が通る）。噛みは、log の panicked の行でその句の断言で落ちたことを照らす（歯の名が落ちただけでは足りない）。
- 置き場の字（の後に・1 度だけ）を file 全体の字で測ると、審査が vacuous-assert で落とす。関数か塊の中で照らし、数を断言する。
- 働きを通らない照らし（足す物が 0 の時に通る等式など）は審査で落ちる。歯は足す物の在る見本で働きを通す。
- 審査の FAIL を直す時は、指された項だけでなく done の全部の項について、外す・替える変異が名指しの歯で落ちるかの表を作る（審査は 1 周で全部の項を指さない）。数の上限・字の切り・既定の値・順・空・読めない時は、fixture がその縁を通るかを見る。
- 噛みは句ごとに撃つ。その句を見ない変異で、名指しの歯が rc 100 で落ちる（nextest の rc 101 は組み立ての失敗で噛みでない）。表（句・歯・変異・rc）を notes.md に置く。噛みの道具は終わりに書き戻すので、噛みの前に試作を commit する。
- 噛みで rc 0 が出たら、歯を足す前に、変異を当てた関数を歯の撃ちの入口から呼びの鎖で通るかを読む。表には当てた関数と入口を並べ、生き残りも消さずに残す。
- 噛みの script には main の守り（__name__ の照らし）を置き、変異の一覧は script を撃たずに字から読む（runpy などで読むと噛みが頭から走る）。

## 7. 床と受付の照らし

- 床は頼みが名指す席の pin の tz で、床の写しの根で tz schema --write、(cd design-intent && tz derive --dir . --out ../contracts --write)（--out は --dir からの相対）、tz check、derive --check、tz check --freeze-adrs を撃つ。合否は終了 code で判じる（0 は合格・1 は不合格・2 はまだ分からない・字で判じない）。
- 適用の script（python）は前の版を照らし、ノートと計画の版を上げ、書いた字を YAML で読み直す。外れたら何も書かずに止まる。
- 適用の script は、節の番号と版をその時の字から +1 で決め、固い字で照らさない（同じノートに並ぶ行が先に置かれる）。srs.yaml は PyYAML の safe_load で読めないので、版は見出しの version の行を正規表現で読む。
- 足した行ごとに scribe2 pipe preflight --state-dir <空の state dir> --repo <床の写し> --design contracts/<ノート>.toml#<行> --bead <在る bead> を撃ち、断りの理由を 0 にする。
- 出す物に差の file（.patch）を持つ係は、終える前に出力の dir に床の記録 floor.tsv を置く（判断の記録 ADR-63 の決定 (4)）。差の file ごとに 1 行で、欄はタブで区切った 6 つ（行 id・preflight の rc・tz check の rc・derive --check の rc・器の表の検査の rc・verify の 2 行目から最後の行の rc を , で繋いだ字）。# で始まる行は頭として読まれない。
- 器の表の検査の rc は、床の写しの根で撃った scribe2 contracts check --repo . --base main の出力のうち、行の頭（contracts/<ノート>.toml の [[contract]] の行）を名指す断りが無ければ 0・在れば 1 と書く（着地済みの行の断りで検査全体の rc は今いつも 1）。床の写しの行は branch に commit し、main を動かさない（preflight は HEAD を読み、--base main は当てる前の木を指す）。
- 終える前の門は、床の写し <置き場>/try-<名>-fl で安い 4 本（preflight は空の state dir <名>/floor-state）を撃ち直して記録と比べる。記録が無い・行が欠ける・rc が 0 でない・撃ち直しと違う・撃ち直せない時は 1 度目の終わりを止め、判じを出力の dir の floor-gate.tsv に足す。記録の合格は完成でも審査の合格でもない。

## 8. 出す物と公開

- 出す物は、notes.md（調べ・実測・未確か・席への問い・印は V 確かめた・D 記録から・I 見立て・U 分からない）、適用の script、commit の文の案、差の file、席が撃つ命令の列。出す物の file の名に report を使わない。
- 持ち主への問いが要るなら、概要・技術・理由・推奨の閉じた問いにする。数は完全な記録からだけ書く。
- repo は公開である。host の名・網の住所・口座の名・ほかの project の名・持ち主の逐語を書かない。
- 出す物の file の名に findings も使わない（report と同じく harness が断る）。最後の返事（席への知らせ）は 10 行以内の日本語で、要点と出力の path を書く。

## 9. 道具の撃ち方と読む範囲

- gh は頭の語に置き、出力は > か | で受ける。置換・for・heredoc の中の gh と git の字は、読むだけでも器の門が断る。書き換えの script は Write の道具で file に書き、python3 <file> で撃つ。
- 背景で撃った物の終わりの知らせは届かないことがある。log の rc の行と ps で自分で見る（pgrep -f の待ちは自分の bash の行に当たって終わらない）。
- 門や hook に断られた命令は、別の命令（git commit-tree・branch -f・update-ref ほか）で回り込まない。断りの字と止まった所を notes.md に書いて席に知らせる。
- 着地の窓が閉じている間は写しの commit も門が断る。試しは commit せず、git write-tree の木の hash で撃つ。
- token を節約する。build・clippy・nextest の出力は file に落とし、tail か grep で落ちた所だけを読む。大きな file は要る範囲だけ読み（offset と limit）、同じ file の全文を何度も読み直さない。
- 読む範囲は頼みの文が名指す（起草係の新しい量の大きな分は、前の係の出す物と設計文書の読みである）。席は頼みの文に、手本にする前の係の出す物の file と節（1〜2 本）・要件と判断の記録の id と節・読む code の file と fn を書く。
- 係は file を丸ごと cat しない。grep -n で行を当て、sed -n で当てた範囲だけを読む。
- 前の係の出す物は、頼みの文が名指した file と節だけを読む。名指しの無い前の係の notes・行の字・命令の列・commit の文の案は、手本にも読まない。
- 要件と判断の記録は、id を grep -A で読む（要件は要件書の id の行から、判断の記録は名指された決定の番号の行から）。要件書と判断の記録の全文を読まない。

## 10. 検証の群（判断の記録 ADR-61）

1. 全文を読む攻めの係（検証役 1 体・頭の予算だけ・群の上限に入らない）が、分からない（U）の主張の一覧 <置き場>/<群 id>/unknowns.tsv を書く（1 行に id・主張・判じられない訳・資料の path をタブで区切る）。
2. 一覧が 3 行未満なら群を起こさない。18 行を越えたら、下書きを割って攻め直すか 1 体の道に戻る（割った下書きは同じ対象なので、群は 7 日に 1 つの塊だけ）。下書きの種類は 問いの本文・判断の記録・消す行・緩める問い の 4 つだけ。
3. 席は群を起こす前に <置き場>/<群 id>/plan.json を置く。欄は type（tsuzuri:verifier）・target（頭の対象と同じ）・draft・attacker・members（name・model・claims・paths）と、在れば ruling。1 体の割りの主張は 6 まで、係どうしで重ねず、和は一覧の全部。
4. 反証役 3 体までを 1 つの返事で並べて起こす。頭は 4 行に 5 行目 群: <群 id> <i>/<k> を足し、予算は token ちょうど 150000、Agent の呼びの model は計画の値。群の係は続けず、段ごとに新しい係を起こす。
5. 群の合計が止める線（新しい量 405000・読み直し 6750000）に届いたら、新しい群の係を起こさない。同じ対象の群は 7 日に 1 つ（例外は、問いの本文が群の id を名指した裁定の裁定 id を計画の ruling に書く）。
6. 席の流れの道具（Workflow）で係を起こさない（起こしの門が全部断る・使うなら持ち主に問う）。
7. 群の終わりに、問いの本文か判断の記録の概要と推奨が頼る主張の全部と、記録から（D）1 件以上と、確かめた（V）2 件以上を席が照らし直す。外れが 1 件でも在れば群の全部の主張を下げる。

大きな調べは 1 体ずつにする。予算の 100% に届いた調べ係は、調べた所と残りの path を出す物に書く。席は残りを、前の係の終えの印の後に新しい 1 体に頼む（判断の記録 ADR-61 の決定 (3)）。

## 11. 畳みの行（席の手）

- 着地した段の file（群に入っていない tests/ の直下の .rs）が、未着地の行が名指す file を除いて 12 本に達したら、畳みの行を 1 行置く。
- 切り方は、置く直前の main の写しで crate ごとに畳みの道具を撃ち、表 GROUPS も書き直す。差は git diff --no-color --full-index -M で切り、130000 byte を目安に、越えれば crate ごとに割る。
- 確かめは、行の木で verify の行を全部撃ち、xtask の check は全部の畳みを当てた最後の木で 1 回。verify に命令 kfold-cap を、書く範囲に持つ群と作る群を全部名指した 1 行で置く（例 cargo run -q -p xtask -- kfold-cap crates/tsuzuri-surface/tests/teeth6 xtask/tests/teeth1）。着地した行の verify の字と差の file は歴史として書き換えない。
