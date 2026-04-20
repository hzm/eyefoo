let postponedCount = 0;

async function initReminder() {
  const btn5 = document.getElementById("btn-postpone5");
  const btn8 = document.getElementById("btn-postpone8");
  const countEl = document.getElementById("postpone-count");

  // Listen for timer-tick to show postpone count
  if (window.__TAURI__) {
    const unlisten = await window.__TAURI__.event.listen("timer-tick", (event) => {
      postponedCount = event.payload.postponed;
      updateCountDisplay(countEl);
    });
  }

  btn5.addEventListener("click", async () => {
    if (window.__TAURI__) {
      await window.__TAURI__.core.invoke("postpone_5min");
    }
  });

  btn8.addEventListener("click", async () => {
    if (window.__TAURI__) {
      await window.__TAURI__.core.invoke("postpone_8min");
    }
  });

  updateCountDisplay(countEl);
}

function updateCountDisplay(el) {
  if (postponedCount > 0) {
    el.textContent = `Postponed ${postponedCount}/3 times`;
  } else {
    el.textContent = "";
  }
}

let restSeconds = 20;

async function initForcedRest() {
  const countdownEl = document.getElementById("countdown");
  const exitBtn = document.getElementById("btn-exit");

  if (window.__TAURI__) {
    await window.__TAURI__.event.listen("timer-tick", (event) => {
      const remaining = event.payload.remaining;
      if (remaining > 0 && remaining <= restSeconds + 1) {
        countdownEl.textContent = remaining;
      }
    });
  }

  exitBtn.addEventListener("click", async () => {
    if (window.__TAURI__) {
      await window.__TAURI__.core.invoke("exit_forced_rest");
    }
  });
}