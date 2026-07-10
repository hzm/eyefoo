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
  var status = document.getElementById("settings-status");

  function setStatus(message, isError) {
    status.textContent = message;
    status.className = "settings-status" + (isError ? " error" : "");
  }

  function readNumber(input) {
    return Number.parseInt(input.value, 10);
  }

  window.__TAURI__.core.invoke("get_settings")
    .then(function (settings) {
      Object.keys(fields).forEach(function (key) {
        fields[key].value = settings[key];
      });
      soundEnabled.checked = settings.sound_enabled;
      autostartEnabled.checked = settings.autostart_enabled;
    })
    .catch(function (error) { setStatus(String(error), true); });

  saveButton.addEventListener("click", function () {
    var settings = {};
    Object.keys(fields).forEach(function (key) {
      settings[key] = readNumber(fields[key]);
    });
    settings.sound_enabled = soundEnabled.checked;
    settings.autostart_enabled = autostartEnabled.checked;

    saveButton.disabled = true;
    window.__TAURI__.core.invoke("set_settings", { settings: settings })
      .then(function () {
        return window.__TAURI__.core.invoke("set_autostart_enabled", {
          enabled: autostartEnabled.checked
        });
      })
      .then(function () { setStatus("设置已保存"); })
      .catch(function (error) { setStatus(String(error), true); })
      .finally(function () { saveButton.disabled = false; });
  });
})();
