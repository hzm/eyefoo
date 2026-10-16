use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::constants::SUSPEND_THRESHOLD_SECS;
#[cfg(test)]
use crate::constants::MAX_POSTPONES;
use crate::state::AppSettings;

#[derive(Clone)]
pub struct TimerManager {
    state: Arc<AtomicU64>,
    postponed: Arc<AtomicU64>,
    /// true = 暂停中，false = 运行中（原来错误地用了 AtomicU64）
    is_paused: Arc<AtomicBool>,
    paused_remaining: Arc<AtomicU64>,
    /// 退出休息模式后的过渡期为 true，用于让 show_forced_rest_window 拦截
    /// 事件队列中残留的触发事件；下一次真实触发（提醒或强制休息）时清除
    pub is_exiting_rest: Arc<AtomicBool>,
    /// 防止同一周期重复触发
    has_triggered: Arc<AtomicBool>,
    /// 防抖：避免极短时间内重复触发强制休息
    last_forced_rest: Arc<std::sync::Mutex<Option<Instant>>>,
    /// 提醒发出的时间，用于 Rust 侧兜底：前端倒计时失效时超时强制休息
    reminder_since: Arc<std::sync::Mutex<Option<Instant>>>,
    settings: Arc<std::sync::Mutex<AppSettings>>,
}

impl TimerManager {
    pub fn new(settings: Arc<std::sync::Mutex<AppSettings>>) -> Self {
        let work_duration = settings.lock().unwrap().work_duration_secs();
        Self {
            state: Arc::new(AtomicU64::new(work_duration)),
            postponed: Arc::new(AtomicU64::new(0)),
            is_paused: Arc::new(AtomicBool::new(false)),
            paused_remaining: Arc::new(AtomicU64::new(0)),
            is_exiting_rest: Arc::new(AtomicBool::new(false)),
            has_triggered: Arc::new(AtomicBool::new(false)),
            last_forced_rest: Arc::new(std::sync::Mutex::new(None)),
            reminder_since: Arc::new(std::sync::Mutex::new(None)),
            settings,
        }
    }

    pub fn start_timer(&self, app: AppHandle) {
        let state = self.state.clone();
        let postponed = self.postponed.clone();
        let is_paused = self.is_paused.clone();
        let paused_remaining = self.paused_remaining.clone();
        let is_exiting_rest = self.is_exiting_rest.clone();
        let has_triggered = self.has_triggered.clone();
        let last_forced_rest = self.last_forced_rest.clone();
        let reminder_since = self.reminder_since.clone();
        let settings = self.settings.clone();

        thread::spawn(move || {
            let mut last_tick = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(1));
                let now = Instant::now();
                let elapsed = now.duration_since(last_tick);
                last_tick = now;
                // 单次 tick 远超 1 秒说明系统休眠或进程被长时间挂起，这段时间不计入计时
                let elapsed_secs = if elapsed > Duration::from_secs(SUSPEND_THRESHOLD_SECS) {
                    0
                } else {
                    elapsed.as_secs().max(1)
                };

                if is_paused.load(Ordering::SeqCst) {
                    let current = paused_remaining.load(Ordering::SeqCst);
                    if current > 0 {
                        let next = current.saturating_sub(elapsed_secs);
                        paused_remaining.store(next, Ordering::SeqCst);
                        if next == 0 {
                            is_paused.store(false, Ordering::SeqCst);
                        }
                    }
                    let pr = paused_remaining.load(Ordering::SeqCst);
                    let _ = app.emit(
                        "timer-tick",
                        serde_json::json!({
                            "remaining": state.load(Ordering::SeqCst),
                            "postponed": postponed.load(Ordering::SeqCst),
                            "paused": true,
                            "paused_remaining": pr
                        }),
                    );
                } else {
                    let before = state.load(Ordering::SeqCst);
                    if before > 0 {
                        state.store(before.saturating_sub(elapsed_secs), Ordering::SeqCst);
                    }
                    let remaining = state.load(Ordering::SeqCst);
                    let p = postponed.load(Ordering::SeqCst);
                    let max_postpones = settings.lock().unwrap().max_postpones;
                    let _ = app.emit(
                        "timer-tick",
                        serde_json::json!({
                            "remaining": remaining,
                            "postponed": p,
                            "paused": false,
                            "paused_remaining": 0
                        }),
                    );

                    if remaining == 0 && !has_triggered.load(Ordering::SeqCst) {
                        has_triggered.store(true, Ordering::SeqCst);
                        // 能走到这里说明是全新周期的真实触发，距上次退出休息至少
                        // 已过一个工作周期，退出过渡期早已结束，清除标志。
                        // 退出后毫秒级到达的残留事件仍由 show_forced_rest_window
                        // 里的 is_exiting_rest 检查拦截，不受影响。
                        is_exiting_rest.store(false, Ordering::SeqCst);
                        if p >= max_postpones {
                            // 直接进入强制休息，不存在挂起的提醒，清掉兜底计时
                            *reminder_since.lock().unwrap() = None;
                            let should_emit = {
                                let mut last = last_forced_rest.lock().unwrap();
                                let now = Instant::now();
                                let can_emit = last.map_or(true, |inst| {
                                    now.duration_since(inst) >= Duration::from_secs(1)
                                });
                                if can_emit {
                                    *last = Some(now);
                                }
                                can_emit
                            };
                            if should_emit {
                                let _ = app.emit("forced-rest-triggered", serde_json::json!({}));
                            }
                        } else {
                            *reminder_since.lock().unwrap() = Some(now);
                            let _ = app.emit("reminder-triggered", serde_json::json!({}));
                        }
                    } else if remaining == 0 {
                        // 兜底：提醒已发出（reminder_since 非空）但前端倒计时未生效
                        // （WebView 节流、推迟被拒导致 JS 计时中断等），超过
                        //  reminder_timeout_seconds 后由 Rust 直接触发强制休息，
                        // 避免计时永远停在 00:00
                        let timeout = settings.lock().unwrap().reminder_timeout_seconds;
                        let due = reminder_since.lock().unwrap().map_or(false, |since| {
                            now.duration_since(since).as_secs() >= timeout
                        });
                        if due {
                            *reminder_since.lock().unwrap() = None;
                            let should_emit = {
                                let mut last = last_forced_rest.lock().unwrap();
                                let can_emit = last.map_or(true, |inst| {
                                    now.duration_since(inst) >= Duration::from_secs(1)
                                });
                                if can_emit {
                                    *last = Some(now);
                                }
                                can_emit
                            };
                            if should_emit {
                                let _ = app.emit("forced-rest-triggered", serde_json::json!({}));
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn postpone(&self, minutes: u64) {
        // 仅在计时到期（提醒弹出）时接受推迟；提醒隐藏后的重复点击直接忽略，
        // 避免双击一次消耗两次推迟次数
        if self.state.load(Ordering::SeqCst) != 0 {
            return;
        }
        let max_postpones = self.settings.lock().unwrap().max_postpones;
        if self.postponed.load(Ordering::SeqCst) < max_postpones {
            self.postponed.fetch_add(1, Ordering::SeqCst);
            self.state.store(minutes * 60, Ordering::SeqCst);
            self.has_triggered.store(false, Ordering::SeqCst);
        }
    }

    pub fn pause_for(&self, minutes: u64) {
        // A visible reminder means the timer is at zero and has already
        // emitted its trigger. Pausing at this point skips that reminder and
        // starts a fresh work cycle after the pause.
        if self.state.load(Ordering::SeqCst) == 0 {
            let work_duration = self.settings.lock().unwrap().work_duration_secs();
            self.state.store(work_duration, Ordering::SeqCst);
            self.postponed.store(0, Ordering::SeqCst);
            self.has_triggered.store(false, Ordering::SeqCst);
        }
        self.is_paused.store(true, Ordering::SeqCst);
        self.paused_remaining.store(minutes * 60, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.paused_remaining.store(0, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    /// 调试用途：令计时立即到期，走与真实触发完全一致的 reminder-triggered 路径
    #[cfg(debug_assertions)]
    pub fn force_expire(&self) {
        self.state.store(0, Ordering::SeqCst);
        self.has_triggered.store(false, Ordering::SeqCst);
    }

    /// 重置为全新工作周期。注意：不清除 is_exiting_rest，
    /// 该标志由 trigger_forced_rest / rest_now 显式清除，或由计时线程
    /// 在下一次真实触发时清除，以保护退出过渡期。
    pub fn reset_cycle(&self) {
        *self.last_forced_rest.lock().unwrap() = None;
        let work_duration = self.settings.lock().unwrap().work_duration_secs();
        self.state.store(work_duration, Ordering::SeqCst);
        self.postponed.store(0, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.has_triggered.store(false, Ordering::SeqCst);
    }

    /// 退出强制休息模式：设置 is_exiting_rest 防止事件队列残留触发，然后重置周期
    pub fn exit_rest_mode(&self) {
        self.is_exiting_rest.store(true, Ordering::SeqCst);
        self.reset_cycle();
    }

    pub fn get_remaining(&self) -> u64 {
        self.state.load(Ordering::SeqCst)
    }

    pub fn get_postponed(&self) -> u64 {
        self.postponed.load(Ordering::SeqCst)
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn paused_remaining(&self) -> u64 {
        self.paused_remaining.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> TimerManager {
        TimerManager::new(Arc::new(std::sync::Mutex::new(AppSettings::default())))
    }

    #[test]
    fn test_new_timer_has_work_duration() {
        let m = manager();
        assert_eq!(m.get_remaining(), 44 * 60);
        assert_eq!(m.get_postponed(), 0);
    }

    #[test]
    fn test_postpone_5min() {
        let m = manager();
        m.state.store(0, Ordering::SeqCst);
        m.postpone(5);
        assert_eq!(m.get_remaining(), 5 * 60);
        assert_eq!(m.get_postponed(), 1);
    }

    #[test]
    fn test_postpone_8min() {
        let m = manager();
        m.state.store(0, Ordering::SeqCst);
        m.postpone(8);
        assert_eq!(m.get_remaining(), 8 * 60);
        assert_eq!(m.get_postponed(), 1);
    }

    #[test]
    fn test_postpone_ignored_while_timer_running() {
        // 计时未到期时推迟无效，防止提醒隐藏后的重复点击消耗推迟次数
        let m = manager();
        m.postpone(5);
        assert_eq!(m.get_postponed(), 0);
        assert_eq!(m.get_remaining(), 44 * 60);
    }

    #[test]
    fn test_max_postpones_capped() {
        let m = manager();
        for _ in 0..=MAX_POSTPONES {
            m.state.store(0, Ordering::SeqCst);
            m.postpone(5);
        }
        assert_eq!(m.get_postponed(), MAX_POSTPONES);
    }

    #[test]
    fn test_reset_cycle_restores_initial_state() {
        let m = manager();
        m.state.store(0, Ordering::SeqCst);
        m.postpone(5);
        m.state.store(0, Ordering::SeqCst);
        m.postpone(5);
        m.reset_cycle();
        assert_eq!(m.get_remaining(), 44 * 60);
        assert_eq!(m.get_postponed(), 0);
    }

    #[test]
    fn test_reset_cycle_does_not_clear_exiting_rest() {
        // is_exiting_rest 由 trigger_forced_rest 显式清除，reset_cycle 不应动它
        let m = manager();
        m.is_exiting_rest.store(true, Ordering::SeqCst);
        m.reset_cycle();
        assert!(
            m.is_exiting_rest.load(Ordering::SeqCst),
            "is_exiting_rest should survive reset_cycle"
        );
    }

    #[test]
    fn test_pause_stops_get_remaining() {
        let m = manager();
        m.pause_for(30);
        assert!(m.is_paused());
        let initial = m.get_remaining();
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert_eq!(
            m.get_remaining(),
            initial,
            "timer must not tick while paused"
        );
    }

    #[test]
    fn test_resume_clears_pause() {
        let m = manager();
        m.pause_for(30);
        m.resume();
        assert!(!m.is_paused());
        assert_eq!(m.paused_remaining(), 0);
    }

    #[test]
    fn test_pause_after_trigger_starts_fresh_cycle() {
        let m = manager();
        m.state.store(0, Ordering::SeqCst);
        m.has_triggered.store(true, Ordering::SeqCst);
        m.postponed.store(MAX_POSTPONES, Ordering::SeqCst);
        m.pause_for(30);
        assert_eq!(m.get_remaining(), 44 * 60);
        assert_eq!(m.get_postponed(), 0);
        assert!(!m.has_triggered.load(Ordering::SeqCst));
    }
}
