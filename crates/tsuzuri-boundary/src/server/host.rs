//! host の負荷と書きの読み（file を読むだけ・書かない）。
//! 読むのは root なしで読める kernel の file（負荷・動いている cpu の範囲・PSI の 3 種・meminfo）と、server 自身の
//! cgroup の親の dir の下の scope の memory.current と memory.max と、host の面（state dir の host.toml）の
//! 書きの測りの表が名指す装置の stat と摩耗の記録（smartctl の JSON・root の timer が書く）。装置の path は
//! code に焼かず表から読む（条 N-7）。値にするのは中核の `hostload` で、ここは字を集めるだけ。
//! 書きの速さは前の読みとの差で、差の間が `RATE_GAP` に満たない読みは前の速さを返す（口と見張りが続けて読んでも揺れない）。
//! 見張り（`Host::watch`）は受け手が居る周だけ `HOST_POLL` ごとに読み、面が読む中身（組んだ時刻を除く電文）が
//! 前と違う時だけ host の種類（`ChangeKind::Host`）の board-changed を送る（受け手が居なければ読まない・規則の行 R-34）。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tsuzuri_contract::board::Reading;
use tsuzuri_contract::host::HostDoc;
use tsuzuri_contract::surface::ChangeKind;
use tsuzuri_core::hostload::{
    DeviceTexts, HostTexts, ScopeTexts, budget_rows, host_doc, rate, sectors_written,
};

use super::events::{Hub, now};
use super::seat::HOST_TOML;

/// 見張りの読みの間隔（受け手が居る周だけ読む）。
pub const HOST_POLL: Duration = Duration::from_secs(10);

/// 書きの速さを測り直す最短の間（これより短い間の読みは前の速さを返す）。
pub const RATE_GAP: Duration = Duration::from_secs(5);

/// kernel の file（根からの相対・負荷・cpu の範囲・PSI の cpu と memory と io・meminfo の順）。
pub const KERNEL: [&str; 6] = [
    "proc/loadavg",
    "sys/devices/system/cpu/online",
    "proc/pressure/cpu",
    "proc/pressure/memory",
    "proc/pressure/io",
    "proc/meminfo",
];

/// 自分の cgroup の file と cgroup の根（根からの相対）。
pub const SELF_CGROUP: &str = "proc/self/cgroup";
pub const CGROUP_ROOT: &str = "sys/fs/cgroup";

/// 装置ごとの前の読み（区の数・読んだ ms・その時の速さ）。
type Samples = BTreeMap<String, (u64, u64, Reading<u64>)>;

/// host の読みの出所（kernel の file の根・state dir・前の読み）。
pub struct Host {
    root: PathBuf,
    state_dir: Option<PathBuf>,
    origin: Instant,
    gap: Duration,
    samples: Mutex<Samples>,
}

impl Host {
    /// 本物の根（/）で読む。
    pub fn new(state_dir: Option<&Path>) -> Host {
        Host::with_root(Path::new("/"), state_dir)
    }

    /// kernel の file を `root` の下で読む（歯は一時の dir に写しの木を置く）。
    pub fn with_root(root: &Path, state_dir: Option<&Path>) -> Host {
        Host {
            root: root.to_path_buf(),
            state_dir: state_dir.map(Path::to_path_buf),
            origin: Instant::now(),
            gap: RATE_GAP,
            samples: Mutex::new(Samples::new()),
        }
    }

    /// 書きの速さを測り直す最短の間を替える（歯が短い間で測る）。
    #[must_use]
    pub fn with_gap(mut self, gap: Duration) -> Host {
        self.gap = gap;
        self
    }

    fn read(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.root.join(rel)).ok()
    }

    /// server 自身の cgroup の親の dir の下の子の dir の名と memory の 2 つの字（親が読めなければ None）。
    fn scopes(&self) -> Option<Vec<ScopeTexts>> {
        let line = self.read(SELF_CGROUP)?;
        let own = line.lines().find_map(|l| l.strip_prefix("0::"))?.trim();
        let parent = Path::new(own).parent()?.strip_prefix("/").ok()?;
        let dir = self.root.join(CGROUP_ROOT).join(parent);
        let mut out: Vec<ScopeTexts> = fs::read_dir(&dir)
            .ok()?
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| {
                let path = e.path();
                let read = |f: &str| fs::read_to_string(path.join(f)).ok();
                ScopeTexts {
                    name: e.file_name().to_string_lossy().into_owned(),
                    current: read("memory.current"),
                    max: read("memory.max"),
                }
            })
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Some(out)
    }

    /// 装置の速さ（前の読みから間（既定は `RATE_GAP`）以上たてば測り直して控え、満たなければ前の速さ）。
    fn device_rate(&self, name: &str, stat: &str) -> Reading<u64> {
        let Some(sectors) = fs::read_to_string(stat)
            .ok()
            .and_then(|t| sectors_written(&t))
        else {
            return Reading::Unknown;
        };
        let at = u64::try_from(self.origin.elapsed().as_millis()).unwrap_or(u64::MAX);
        let gap = u64::try_from(self.gap.as_millis()).unwrap_or(u64::MAX);
        let mut samples = self
            .samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = samples.get(name).cloned();
        if let Some((_, was, ref held)) = before
            && at.saturating_sub(was) < gap
        {
            return held.clone();
        }
        let now = rate(before.map(|(s, a, _)| (s, a)), (sectors, at));
        samples.insert(name.to_string(), (sectors, at, now.clone()));
        now
    }

    /// 電文の材料の字（host の面が無いか読めなければ装置は空）。
    pub fn texts(&self) -> HostTexts {
        let [loadavg, online, psi_cpu, psi_memory, psi_io, meminfo] = KERNEL.map(|k| self.read(k));
        let face = self
            .state_dir
            .as_ref()
            .and_then(|d| fs::read_to_string(d.join(HOST_TOML)).ok())
            .unwrap_or_default();
        let devices = budget_rows(&face)
            .into_iter()
            .map(|row| DeviceTexts {
                rate: self.device_rate(&row.name, &row.stat),
                wear: row.wear.map(|p| fs::read_to_string(p).ok()),
                name: row.name,
            })
            .collect();
        HostTexts {
            loadavg,
            online,
            psi_cpu,
            psi_memory,
            psi_io,
            meminfo,
            scopes: self.scopes(),
            devices,
        }
    }

    /// 口の電文（今の時刻で組む）。
    pub fn doc(&self) -> HostDoc {
        host_doc(&self.texts(), now())
    }

    /// host の見張りを始める（Hub が落ちれば止まる）。`poll` ごとに受け手が居る周だけ読み、組んだ時刻を除く
    /// 電文が前に送った周と違えば host の種類の board-changed を 1 件送る（受け手が 0 人の周は読まず、前の中身を忘れる）。
    pub fn watch(self: &Arc<Self>, hub: &Arc<Hub>, poll: Duration) {
        let host = Arc::clone(self);
        let weak = Arc::downgrade(hub);
        thread::spawn(move || {
            let mut seen: Option<HostDoc> = None;
            loop {
                thread::sleep(poll);
                let Some(hub) = weak.upgrade() else {
                    return;
                };
                if hub.listeners() == 0 {
                    seen = None;
                    continue;
                }
                let mut doc = host.doc();
                let at = doc.at;
                doc.at = 0;
                if seen.as_ref() == Some(&doc) {
                    continue;
                }
                seen = Some(doc);
                hub.board_changed(&[ChangeKind::Host], at);
            }
        });
    }
}
