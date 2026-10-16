/**
 * rest.js — 强制休息界面逻辑
 *
 * 设计原则：
 * - 纯事件驱动（Rust 每秒 emit rest-tick），彻底去除 200ms 高频轮询
 * - 仅在页面重新可见时做一次 IPC 同步（get_rest_remaining）
 * - 不使用 window.* 全局变量
 * - 通过 restart-polling 事件接收 Rust 的唤醒信号，取代 window.eval 调用
 */
(function () {
  "use strict";

  var countdownEl = document.getElementById("countdown");
  var exitBtn = document.getElementById("btn-exit");
  var exitHintEl = document.getElementById("exit-hint");
  var REST_DURATION_SECS = 180;
  var EXIT_UNLOCK_AFTER_SECS = 30;

  function formatTime(s) {
    var m = Math.floor(s / 60);
    var sec = s % 60;
    return (m < 10 ? "0" : "") + m + ":" + (sec < 10 ? "0" : "") + sec;
  }

  function setDisplay(seconds) {
    if (countdownEl) {
      countdownEl.textContent = formatTime(Math.max(0, seconds));
    }
    var elapsed = REST_DURATION_SECS - seconds;
    var unlocked = elapsed >= EXIT_UNLOCK_AFTER_SECS;
    if (exitBtn) {
      exitBtn.hidden = !unlocked;
      exitBtn.setAttribute("aria-hidden", unlocked ? "false" : "true");
    }
    if (exitHintEl) {
      exitHintEl.textContent = unlocked
        ? ""
        : EXIT_UNLOCK_AFTER_SECS - elapsed + " 秒后可在右下角退出";
    }
  }

  /** 单次 IPC 同步，用于页面重新可见或窗口被唤醒时校正显示 */
  function resync() {
    window.__TAURI__.core
      .invoke("get_rest_remaining")
      .then(function (remaining) {
        setDisplay(remaining);
      })
      .catch(function () {});
  }

  // ── Tauri 事件监听 ────────────────────────────────────────────────────────

  // Rust 每秒 emit 一次，驱动倒计时显示（取代 200ms 轮询）
  window.__TAURI__.event.listen("rest-tick", function (event) {
    var p = event.payload;
    if (typeof p === "string") {
      try { p = JSON.parse(p); } catch (e) {}
    }
    if (p && typeof p.remaining === "number" && p.remaining >= 0) {
      setDisplay(p.remaining);
    } else if (typeof p === "number" && p >= 0) {
      setDisplay(p); // 兼容旧格式（防万一）
    }
  });

  // 休息开始时初始化显示
  window.__TAURI__.event.listen("rest-started", function (event) {
    var p = event.payload;
    if (typeof p === "string") {
      try { p = JSON.parse(p); } catch (e) {}
    }
    if (p && typeof p.remaining === "number") {
      if (typeof p.duration === "number" && p.duration > 0) {
        REST_DURATION_SECS = p.duration;
      }
      if (typeof p.exit_unlock === "number" && p.exit_unlock >= 0) {
        EXIT_UNLOCK_AFTER_SECS = p.exit_unlock;
      }
      setDisplay(p.remaining);
    }
    resync();
  });

  // Rust 调用 window.emit("restart-polling", ()) 来唤醒此页面同步状态
  window.__TAURI__.event.listen("restart-polling", function () {
    resync();
  });

  // 休息结束时清空显示，防止下次弹出时闪出上次的残留数字
  window.__TAURI__.event.listen("rest-ended", function () {
    if (countdownEl) countdownEl.textContent = "";
    if (exitBtn) exitBtn.hidden = true;
    if (exitHintEl) exitHintEl.textContent = "";
  });

  // 休息自然结束由 Rust 倒计时线程直接收尾（finish_rest），无需前端回调

  // ── 页面可见性变化时重新同步（如副屏唤醒、切换回来等）──────────────────────

  document.addEventListener("visibilitychange", function () {
    if (!document.hidden) resync();
  });

  window.addEventListener("focus", resync);
  window.addEventListener("pageshow", resync);

  // ── 按钮 ─────────────────────────────────────────────────────────────────

  exitBtn.addEventListener("click", function () {
    window.__TAURI__.core.invoke("exit_forced_rest").catch(console.error);
  });

  // 初始同步
  resync();
})();
