//! 読みの合流（便 e-coalesce）: 走っている読みの間に届いた呼び出しは新しい読みを始めず、
//! 走っている読みの終わりを待って同じ結果（読めた値か、読めない）を受ける。
//! 走っている読みが無ければ新しい読みを始める。読みが終われば結果は持ち回さない
//! （次の呼び出しは新しい読みを始める）。待つのは呼ぶ側が渡す上限までで、越えたら読めない。
//! 分け合うのは同じ `Coalesce` とその clone だけ（`new` で作った別の値とは分け合わない）。

use std::fmt;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

/// 合流して待つ要求が、読みの上限に足して待つ時間。
pub const GRACE: Duration = Duration::from_secs(1);

/// 1 本の読みの合流の場（clone は同じ場を分け合う）。
pub struct Coalesce<T> {
    running: Arc<Mutex<Option<Arc<Flight<T>>>>>,
}

/// 走っている 1 本の読み（終われば結果を置いて待つ側を起こす）。
struct Flight<T> {
    /// 読みの結果（読みの途中は None・終われば Some）。
    result: Mutex<Option<Option<T>>>,
    done: Condvar,
}

impl<T> Coalesce<T> {
    pub fn new() -> Coalesce<T> {
        Coalesce {
            running: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> Default for Coalesce<T> {
    fn default() -> Coalesce<T> {
        Coalesce::new()
    }
}

impl<T> Clone for Coalesce<T> {
    fn clone(&self) -> Coalesce<T> {
        Coalesce {
            running: Arc::clone(&self.running),
        }
    }
}

impl<T> fmt::Debug for Coalesce<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Coalesce")
    }
}

impl<T: Clone> Coalesce<T> {
    /// 走っている読みが在ればその終わりを `wait` まで待って同じ結果を返し（越えたら None）、
    /// 無ければ `read` を撃って結果を返す（待っていた呼び出しにも同じ結果を渡す）。
    pub fn share(&self, wait: Duration, read: impl FnOnce() -> Option<T>) -> Option<T> {
        let lead = {
            let mut running = lock(&self.running);
            match &*running {
                Some(flight) => Err(Arc::clone(flight)),
                None => {
                    let flight = Arc::new(Flight {
                        result: Mutex::new(None),
                        done: Condvar::new(),
                    });
                    *running = Some(Arc::clone(&flight));
                    Ok(Lead {
                        running: &self.running,
                        flight,
                        out: None,
                    })
                }
            }
        };
        match lead {
            Ok(mut lead) => {
                lead.out = read();
                lead.out.clone()
            }
            Err(flight) => {
                let result = lock(&flight.result);
                let (result, _) = flight
                    .done
                    .wait_timeout_while(result, wait, |r| r.is_none())
                    .unwrap_or_else(|e| e.into_inner());
                result.clone().flatten()
            }
        }
    }
}

/// 読みを始めた呼び出しの持ち分。落ちる時に場を空け（次の呼び出しは新しい読みを始める）、
/// 待つ側に結果を渡す（読みが panic で落ちても、待つ側は読めないを受ける）。
struct Lead<'a, T> {
    running: &'a Mutex<Option<Arc<Flight<T>>>>,
    flight: Arc<Flight<T>>,
    out: Option<T>,
}

impl<T> Drop for Lead<'_, T> {
    fn drop(&mut self) {
        *lock(self.running) = None;
        *lock(&self.flight.result) = Some(self.out.take());
        self.flight.done.notify_all();
    }
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
