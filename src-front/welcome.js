/**
 * welcome.js — 首次启动引导页
 */
(function () {
  "use strict";

  document.getElementById("btn-got-it").addEventListener("click", function () {
    window.__TAURI__.webviewWindow
      .getCurrentWebviewWindow()
      .close()
      .catch(function (error) {
        console.error("关闭引导窗口失败:", error);
      });
  });
})();
