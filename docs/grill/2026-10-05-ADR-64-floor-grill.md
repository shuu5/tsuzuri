# 反対側からの検証 — 器を書く便の同時 1 本の床を外す時機（判断の記録 ADR-64）

- 日: 2026-10-05
- 検証役: qv66（AI・起草者と別の独立の文脈・読み取りだけ・cargo なし）
- 攻めた物: 係 w254a の判断の記録 ADR-64 の下書き（根を名乗る行 v-roots を v-join を待たずに流し、v-roots の着地の後に追随の撃ち直しを 1 回見てから床を外し、面の外の穴は塞ぐまで席の門で守る）・床を外す行 t-run-cap-off の差と行の字・席の門 gate-scribe2-crates.py。
- 拠り: 問い t3-hub.87.38 と t3-hub.87.39 の本文、判断の記録 ADR-33 の決定 (12)(14)・ADR-63 の決定 (13) と撤退 (8)・ADR-60 の決定 (1)(6)、器の land.rs・train.rs・land/verify.rs・gate/record.rs。
- 条 A-2.3 と P-17.4: 字の上では当たらない（床は規則の行でなく宣言の鍵）。断りを外す変更なので、ADR-60 の決定 (5) の前例に倣い、封の前にこの攻めを置いた。条文の改訂ではない。
- 持ち主の逐語はここに書かない（台帳の問いの notes に在る）。

## 1. 判じ

決定（案 甲）は崩れない。封の前に字を直す所が在る。致命 2・要直し 6・軽い 5。

## 2. 致命

- K1 単独の着地では、組めない物は main を進めた後にしか落ちない。着地の列の候補の木は、列に後ろの便が居る周だけに在る。後ろが居ない周は、撃ち直しを省いた後に squash と CAS で main を進め、主実測が赤を見つけて止まる（自動では戻さない）。下書きの「組めない物は候補の木の検査が main の前に落とし、残るのは審査役だけが拾う食い違い」は、この周に当たらない。→ 結果の欄と決定 (4) の残りの見積もりを「単独の着地では main の赤になりうる」に直す。撤退 (2) は既に main の赤を数える。
- K2 観る 1 回（決定 (3)）は、穴と関わらない理由で撃ち直した周でも満ちる。撃ち直しは base から main の差に根の crates/・Cargo.toml・Cargo.lock・rules/・.vessel.toml が 1 つでも在れば起きる。v-roots の直後に走る便の base は多くが v-roots の前で、差に v-roots 自身の .vessel.toml が入る。列の後ろで着地した便は追随を通らない。→ 差の path が scribe2/crates/ の下を持ち、根の面の 5 項目とほかの根の crate を持たない周に限り、列の後ろと既着地の周を数えず、range と path の一覧を記帳する。

## 3. 要直し

- R1 便の bead を「置いた直後に hold」すると、起票と hold の間にほかの便の終端で起こされる隙が在る。→ 観る前は bead を起こさないか、閉じていない印の bead に blocks の依存で繋いで起こす。
- R2 面の外に残る器の組みの入力は 3 つより多い。器の binary は plugin の 2 file（plugin.json と hooks.json）も組みの時に読み、contracts/schema.toml を器の src と歯が読む。.cargo/config.toml・clippy.toml・rust-toolchain.toml・deny.toml・.config/nextest.toml も組みと lint を変える。器だけの repo の面には .vessel.toml も入っていた。門は crate を書かない行を全部断るので生きている間は守るが、宣言の行 t-scope-on の着地で外れた後は、これらだけを書く行が K1 の道に入りうる。→ 器だけの repo と同じ面にする言い分と残る入力の一覧を本文に書く。門を残すなら持ち主に問う。今の契約表に該当する未着地の行は無い。
- R3 観る 1 回の字のうち、land の出力の regate=skipped は器の定数と合う。verify.jsonl の省きの行は JSON の 1 行（鍵 kind の値 gate・鍵 skipped の値 regate・鍵 reason の値 outside-scope）で、下書きの skipped=regate は code の注の書き方である。land の出力が後から読める所に残るかは分からない。→ verify.jsonl と event で判じる字に置く。
- R4 basis の A-2.3 と note の字を、字の上では当たらず前例に倣って検証を添えた形に揃える。
- R5 v-roots と v-join-xt は着地した。下書きは v-roots を未着地として書く。封の後は替えられないので、事実の字を封の時点に揃える。
- R6 撤退 (2) の数え口（省いた周の印と main の赤の event・窓の始まり）を字で名指す。

## 4. 軽い

- L1 門は契約表の toml が読めない時に通る（fail-open）。読めない時は rc 2 で止める。
- L2 門の base と index の読み（commit → rebase → 門の順で index は HEAD）と、外れる条件の 3 項目は宣言の行 t-scope-on の値と合う。
- L3 新しい歯は字 run-cap を含む行を全部拾うので、後に宣言の注が run-cap を引くと落ちる。今は 0 行。
- L4 消す歯の file の write-set の書き方と、着地済みの行の done-teeth の扱いは席の作法と合う。
- L5 平易の欄は「規則の表と部品の一覧の file だけ」と言うが、門は crate を書かない行を全部断る。

## 5. 崩れなかった所

- 床を外す行の差は、v-roots の後の main に単独で当たり、宣言の行 t-scope-on の差と両順に当たって同じ木になる（一時の index・写しの中で照らした）。
- 契約表の器の行で crate を書かず器の dir を書く行は、着地済みの 1 本だけ。規則の表を書く未着地の行は crate の file も持つ。
- 撃ち直しを省く判じは 1 本の関数で、宣言の根の crate を面に入れる。v-roots の後は器の crate の path で撃ち直す。
