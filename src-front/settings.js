(function () {
  "use strict";

  var fields = {
    work_duration_minutes: document.getElementById("work-duration"),
    rest_duration_minutes: document.getElementById("rest-duration"),
    reminder_timeout_seconds: document.getElementById("reminder-timeout"),
    postpone_5_minutes: document.getElementById("postpone-5"),
    postpone_8_minutes: document.getElementById("postpone-8"),
    max_postpones: document.getElementById("max-postpones"),
    exit_unlock_seconds: document.getElementById("exit-unlock")
  };
  var soundEnabled = document.getElementById("sound-enabled");
  var autostartEnabled = document.getElementById("autostart-enabled");
  var saveButton = document.getElementById("save-settings");
  var resetButton = document.getElementById("reset-settings");
  var status = document.getElementById("settings-status");

  function setStatus(message, isError) {
    status.textContent = message;
    status.className = "settings-status" + (isError ? " error" : "");
  }

  function fillForm(settings) {
    Object.keys(fields).forEach(function (key) {
      fields[key].value = settings[key];
    });
    soundEnabled.checked = settings.sound_enabled;
    autostartEnabled.checked = settings.autostart_enabled;
  }

  function readNumber(input) {
    return Number.parseInt(input.value, 10);
  }

  window.__TAURI__.core.invoke("get_settings")
    .then(fillForm)
    .catch(function (error) { setStatus(String(error), true); });

  resetButton.addEventListener("click", function () {
    resetButton.disabled = true;
    window.__TAURI__.core.invoke("reset_settings")
      .then(function (settings) {
        fillForm(settings);
        setStatus("已恢复默认设置");
      })
      .catch(function (error) { setStatus(String(error), true); })
      .finally(function () { resetButton.disabled = false; });
  });

  saveButton.addEventListener("click", function () {
    var settings = {};
    Object.keys(fields).forEach(function (key) {
      settings[key] = readNumber(fields[key]);
    });
    settings.sound_enabled = soundEnabled.checked;
    settings.autostart_enabled = autostartEnabled.checked;

    saveButton.disabled = true;
    // 先写注册表（唯一可能因外部原因失败的一步），成功后再保存设置文件，
    // 避免注册表未更新但文件已写入 autostart_enabled 的半成功状态
    window.__TAURI__.core.invoke("set_autostart_enabled", {
      enabled: autostartEnabled.checked
    })
      .then(function () {
        return window.__TAURI__.core.invoke("set_settings", { settings: settings });
      })
      .then(function () { setStatus("设置已保存"); })
      .catch(function (error) { setStatus(String(error), true); })
      .finally(function () { saveButton.disabled = false; });
  });
})();
