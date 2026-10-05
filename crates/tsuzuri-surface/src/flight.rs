//! 口ごとの読みの印の決め方（net が使い host でも組む）。
//! 読み直しの呼びは読みの途中なら 1 回にまとめ（読みの後にもう 1 回読む）、
//! 見ている部品が無い間の呼びは部品が戻ったときの 1 回にまとめる。

/// 知らせの接続が開くのを待ってから最初の読みを撃つ上限（ミリ秒）。
pub const OPEN_WAIT_MS: u64 = 1000;

/// 1 つの口の読みの印。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flight {
    /// 見ている部品の数。
    users: usize,
    /// 読みの途中か。
    busy: bool,
    /// 読みの途中に呼びが来たか（読みの後にもう 1 回読む）。
    again: bool,
    /// 見ている部品が無い間に呼びが来たか（部品が戻ったときに読む）。
    missed: bool,
    /// path が変わった回数（前の path の読みを見分ける）。
    round: u32,
}

impl Flight {
    /// 見ている部品を 1 つ足す。0 から 1 になり、見ていない間に呼びが来ていれば読みを始めるか（真なら読む）。
    pub fn attach(&mut self) -> bool {
        self.users += 1;
        if self.users == 1 && self.missed {
            self.missed = false;
            return self.start();
        }
        false
    }

    /// 見ている部品を 1 つ引く（0 より下げない）。
    pub fn detach(&mut self) {
        self.users = self.users.saturating_sub(1);
    }

    /// 読み直しの呼び（真なら読む）。見ている部品が無ければ部品が戻るまで待つ。
    pub fn call(&mut self) -> bool {
        if self.users == 0 {
            self.missed = true;
            return false;
        }
        self.start()
    }

    /// 読みを始めるか（真なら読む）。読みの途中なら読みの後にもう 1 回読む印を付ける。
    pub fn start(&mut self) -> bool {
        if self.busy {
            self.again = true;
            return false;
        }
        self.busy = true;
        true
    }

    /// 読みの鎖の終わり（真なら読みの途中のまま続けてもう 1 回読む）。
    pub fn end(&mut self) -> bool {
        if self.again {
            self.again = false;
            if self.users >= 1 {
                return true;
            }
            self.missed = true;
        }
        self.busy = false;
        false
    }

    /// path が変わった（読みの途中を捨てて round を 1 進め、その round を返す）。
    pub fn renew(&mut self) -> u32 {
        self.busy = false;
        self.again = false;
        self.round += 1;
        self.round
    }

    /// 見ている部品の数。
    pub fn users(&self) -> usize {
        self.users
    }

    /// 読みの途中か。
    pub fn busy(&self) -> bool {
        self.busy
    }

    /// path が変わった回数。
    pub fn round(&self) -> u32 {
        self.round
    }
}
