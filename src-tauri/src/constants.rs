/// 工作时长（秒），到期后触发休息提醒
pub const WORK_DURATION_SECS: u64 = 44 * 60;

/// 强制休息屏幕持续时长（秒）
pub const REST_DURATION_SECS: u64 = 3 * 60;

/// 最大可推迟次数，超出后强制休息
pub const MAX_POSTPONES: u64 = 3;

/// 提醒弹窗倒计时自动触发强制休息的秒数（仅用于文档，实际计时逻辑在 reminder.js）
#[allow(dead_code)]
pub const REMINDER_AUTO_DISMISS_SECS: u64 = 60;
