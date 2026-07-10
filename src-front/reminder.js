/**
 * reminder.js — 提醒弹窗逻辑
 *
 * 设计原则：
 * - 不使用 window.* 全局变量作为消息总线
 * - 通过 Tauri 事件（activate-reminder）接收激活信号，取代 window.eval 调用
 * - 所有状态局限于此 IIFE 作用域内
 */
(function () {
  "use strict";

  var autoDismissSecs = 60;

  var autoRemaining = autoDismissSecs;
  var isCountingDown = false;
  var postponedCount = 0;
  var maxPostpones = 3;

  var btn5 = document.getElementById("btn-postpone5");
  var btn8 = document.getElementById("btn-postpone8");
  var countEl = document.getElementById("postpone-count");
  var autoEl = document.getElementById("auto-dismiss");

  function updateCount() {
    if (!countEl) return;
    countEl.textContent =
      postponedCount > 0 ? "已推迟 " + postponedCount + "/" + maxPostpones : "";
  }

  function stopTimer() {
    isCountingDown = false;
  }

  function startCountdown() {
    stopTimer();
    autoRemaining = autoDismissSecs;
    isCountingDown = true;
    if (autoEl) autoEl.textContent = autoRemaining;
  }

  // ── Tauri 事件监听 ────────────────────────────────────────────────────────

  // Rust 调用 window.emit("activate-reminder", {postponed}) 来激活倒计时
  window.__TAURI__.event.listen("activate-reminder", function (event) {
    var payload = event.payload || {};
    if (typeof payload === "string") {
      try { payload = JSON.parse(payload); } catch (e) {}
    }
    if (typeof payload.postponed === "number") {
      postponedCount = payload.postponed;
    }
    if (typeof payload.timeout === "number" && payload.timeout > 0) {
      autoDismissSecs = payload.timeout;
    }
    if (typeof payload.max_postpones === "number") {
      maxPostpones = payload.max_postpones;
    }
    if (typeof payload.postpone_5 === "number") {
      btn5.textContent = "推迟 " + payload.postpone_5 + " 分钟";
    }
    if (typeof payload.postpone_8 === "number") {
      btn8.textContent = "推迟 " + payload.postpone_8 + " 分钟";
    }
    updateCount();
    startCountdown();
  });

  // 同步推迟次数并驱动倒计时（timer-tick 每秒广播，不受 WebView 休眠影响）
  window.__TAURI__.event.listen("timer-tick", function (event) {
    var payload = event.payload;
    if (typeof payload === "string") {
      try { payload = JSON.parse(payload); } catch (e) {}
    }
    if (payload && typeof payload.postponed === "number") {
      postponedCount = payload.postponed;
      updateCount();
    }

    if (payload && payload.paused) {
      stopTimer();
      return;
    }

    if (isCountingDown) {
      autoRemaining--;
      if (autoEl) autoEl.textContent = autoRemaining;
      if (autoRemaining <= 0) {
        stopTimer();
        window.__TAURI__.core.invoke("start_forced_rest").catch(console.error);
      }
    }
  });

  // ── 按钮事件 ──────────────────────────────────────────────────────────────

  btn5.addEventListener("click", function () {
    stopTimer();
    window.__TAURI__.core.invoke("postpone_5min").catch(console.error);
  });

  btn8.addEventListener("click", function () {
    stopTimer();
    window.__TAURI__.core.invoke("postpone_8min").catch(console.error);
  });

})();
