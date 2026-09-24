# scribe v2 の残りと、v2 では対応を諦めた問題（scribe2 の orchestrator 席 → s3:design 席・2026-09-24）

持ち主の方針（2026-09-24T12:4xZ・逐語は scribe2 の台帳 s2-07l.604 / .609 の notes）: scribe v2 は「scribe v3 を作る」1 本を残して他を全部片付ける。v2 の状況と、v2 で諦めた問題を v3 の設計に活かす。この file はその送り物で、逐語は載せない（台帳が正）。

## 1. v2 で今日直したもの（v3 が引き継ぐ形）
- **口座群の自動移動の実地試験**（memo .604 の notes・toy の置き場で本物の claude を subagent つきで退避）: 移動の周（記録・承認 event・退避の合図・settle）と、shell に戻った後の同じ target への起こし直し（SessionStart の brief が入る）は動く。穴は「/exit の dialog」= Claude Code は background work（subagent・background shell）が残ると「Background work is running」の dialog で止まり、`CLAUDE_CODE_DISABLE_AGENT_VIEW=1` でも消えない（選択肢が 3 → 2 になるだけ・既定は Exit and stop tasks）。器の入力欄の門は dialog の既定の行を人の打ちかけと読み、以後 1 key も送らず永久に保留。v2 の処置 = account-lifecycle.md §22 / 行 k（門が既定の行の literal を返す周だけ Enter を 1 回・/exit の送りを inject に記録）。契約 s2-07l.610・PR #664。
- **札と lock の pid の再利用**（memo .608）: 札の本文が pid 1 語で、pid が再利用されると死んだ driver を生きていると誤読する。v2 の処置 = dispatcher.md §24 / 行 u（本文を pid + 起動時刻の 2 語に・PR #665）。
- **scribe2 init 1 発**（持ち主の裁定 13:02Z / 13:04Z・s2-07l.609）: 新しい repo を器に載せる口（state dir・host の面の継承・口座の配線・.vessel / .vessel.toml・orchestrator 席の登録と起動）と doctor の欠落の名指し・人間向けの説明。v2 で実装する（設計 + ADR はこれから）。

## 2. v2 で諦める・v3 の設計論点に送るもの
1. **器が画面を読めないこと（憲法 C3.3 の柵）**: dialog（/exit の確認・trust の確認・bypass の確認）はすべて「描画」に在り、器は入力欄の門の 1 行しか読まない。v2 は literal の等値で 1 つずつ穴を塞ぐ形（行 k）。v3 の表示面（session が制御する browser の面・論点 C）は、器が **自分の席の状態を描画でなく構造で知る**口（Claude Code の hook / SDK / session の状態 file）を前提にすべき。dialog の種類が増えるたびに literal を足す形は v3 に持ち込まない。
2. **移り先の口座の trust**: 口座がその anchor を一度も trust していないと trust dialog（既定 No, exit）で席が立たない。公式 doc の唯一の口は口座の `.claude.json` の `projects[anchor].hasTrustDialogAccepted` を書くこと。v2 は JSON の入れ子を書く道具を持たない（依存の追加 = A3 の裁定待ち）。v3 の init / 口座の配線は **口座 × anchor の trust を器が持つ**設計にする（serde_json を依存に入れるかは v3 の A3）。
3. **.vessel marker の無い repo で hook が黙る**: 仕えない repo の SessionStart hook は 0 byte（設計どおり）だが、doctor が名指さない・手で撃った hook が 60 秒返らない周が在った（未解明・exit-probe の toy・vessel init の前）。v3 の doctor は「何が無いか・次の 1 手」を必ず 1 行出す。
4. **folio v1 の SRS 生成器の制約**（scribe2 の台帳 s2-07l.19・2026-09-09 の Phase F の残・11 項目）: srs.html にリンク 0 本・375px で機能要件表の固定列なし・AC と制約の plain 欄なし・NFR の plain ラベルと検証列なし・title に文書の身元なし・07 章の折り畳みと th scope・doc_id 非表示・index に srs.html が載らない（folio-doc-type meta 無し）・flip check の inline 化の対象外。folio v1 は 2026-09-09 から不変で、v3 は folio2 を取り込むので **folio2 の生成器でこれらが解けているか**を v3 の要件書（srs.yaml）の受入で確かめる。
5. **scribe2 の憲法の 17 項目の棚卸し**（同 .19・2026-09-23）: 残 15 項目は /folio-architect（持ち主の手番）でしか直せず v2 では触らない。v3 の憲法（論点 E・scribe2 C1〜C17 と folio2 P / N / R の合流）を書くときに scribe2 の索引表の出所の不一致（25 行中 10 行）と C7 要旨の古さを持ち込まない。
6. **EARS の型の混在**（同 .19）: scribe2 の SRS 76 本のうち別の型の引き金語を持つ 28 本（medium 級 5 本 = FR27 / FR42 / FR68 / FR13 / FR34）。v3 の要件書は folio2 の EARS の門で最初から閉じる。
7. **dispatcher の resume の終わりの印**（.608 の候補 2）: 継いだ子が札を外すのは Drop（process の正常終了）だけで、「仕事を終えた」印が別に無い。v3 では run の終端を event で持つ（札の不在で推さない）。
8. **人が打つ command が多い**（持ち主の裁定 13:04Z）: v2 は init と doctor の 2 つを上限にする設計を .609 で書く。v3 の GUI（論点 B）はその 2 つすら GUI から撃てる形にする。

## 3. scribe2 の台帳の現在値（2026-09-24T13:3xZ）
- open: s2-07l.214（v3 の設計・据え置き）・.604（行 k の land で close）・.608（行 u の land で close）・.609（init 1 発・設計中）・.610（行 k の契約・走行中）・.19（本 file に送って close）。
- closed 今日: .491（後継 .609 / .604）・.198（core の refactor・行 i の着地）ほか契約 12 本。
