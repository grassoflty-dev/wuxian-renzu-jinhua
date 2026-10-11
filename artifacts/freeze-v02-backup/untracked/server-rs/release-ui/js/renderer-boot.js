/*
 * 无限人族进化 · 渲染器引导层（Babylon-only）
 *
 * 变更（Babylon-only 迁移）：整个游戏只使用 Babylon.js 第三人称渲染器
 * （window.TpsRenderer）。Three.js 等距渲染器（js/renderer.js）与
 * hive-side2d 适配层均已从页面加载链移除，本引导层不再提供任何回退候选：
 *   - 加载顺序（index.html）：
 *     vendor/babylon.js -> js/tps-renderer.js -> js/renderer-boot.js -> js/app.js
 *   - Babylon 未加载 / TpsRenderer 缺失 / init 抛异常时，页面显示"重试画面"
 *     入口（app.js 的 rendererFailed 流程），绝不静默换用旧渲染器。
 *
 * app.js 仍然只看到同一个 window.CampaignRenderer 契约对象。
 */
(function installRendererBoot(global) {
  "use strict";

  function log(message, detail) {
    if (global.console && global.console.warn) {
      if (detail === undefined) global.console.warn("[renderer-boot] " + message);
      else global.console.warn("[renderer-boot] " + message, detail);
    }
  }

  function safeDispose(chain) {
    if (!chain) return;
    try { if (typeof chain.dispose === "function") chain.dispose(); } catch (error) { /* 释放失败不阻断启动 */ }
  }

  var current = null;
  var failed = false;

  var tpsApi = global.TpsRenderer && typeof global.TpsRenderer.init === "function" ? global.TpsRenderer : null;

  function start(canvas, options) {
    if (!tpsApi) throw new Error("TpsRenderer 不可用：本游戏仅支持 Babylon.js 渲染（vendor/babylon.js + js/tps-renderer.js 必须先于本文件加载）");
    if (failed) throw new Error("Babylon TPS 渲染器此前初始化失败，请使用“重试画面”重新初始化");
    var result = tpsApi.init(canvas, options);
    var delegate = result && typeof result === "object" ? result : tpsApi;
    if (!delegate || typeof delegate !== "object") throw new Error("TpsRenderer.init 未返回渲染器对象");
    if (delegate === api || delegate === boot) throw new Error("TpsRenderer.init 返回了引导层自身（自引用被拒绝）");
    current = { name: "tps", api: tpsApi, delegate: delegate };
    return current;
  }

  function activeName() {
    return current ? "tps" : "none";
  }

  var boot = {
    canvas: null,
    options: null,
    get renderer() { return current ? current.delegate : null; },
    get backend() { return activeName(); },
    init: function (canvas, options) {
      if (!canvas) throw new Error("CampaignRenderer.init 需要一个 canvas");
      if (current) { safeDispose(current.delegate); current = null; }
      boot.canvas = canvas;
      boot.options = options || {};
      failed = false;
      start(canvas, boot.options);
      return api;
    },
    setData: delegateMethod("setData"),
    setEffect: delegateMethod("setEffect"),
    setPlayerState: delegateMethod("setPlayerState"),
    resize: delegateMethod("resize"),
    setQuality: delegateMethod("setQuality"),
    targetAt: delegateMethod("targetAt", null),
    debug: function () {
      var detail = null;
      try { detail = call("debug", []); } catch (error) { log("debug() 失败", error); }
      return {
        mode: "renderer-boot",
        backend: activeName(),
        tps_available: !!tpsApi,
        babylon_loaded: !!global.BABYLON,
        failed: failed ? ["tps"] : [],
        delegate: detail || null,
      };
    },
    dispose: function () {
      if (current) safeDispose(current.delegate);
      current = null;
      failed = false;
    },
  };

  function call(method, args) {
    if (!current) return undefined;
    var owner = current;
    try {
      var fn = owner.delegate && owner.delegate[method];
      if (typeof fn !== "function") return undefined;
      return fn.apply(owner.delegate, args);
    } catch (error) {
      // Babylon-only：渲染器运行期异常不再回退到旧渲染器，统一交给 app.js 的
      // rendererFailed / “重试画面”流程处理。
      failed = true;
      log("Babylon TPS 渲染器 " + method + "() 运行期异常", error);
      throw error;
    } finally {
      void owner;
    }
  }

  function delegateMethod(method, fallbackValue) {
    return function () {
      var value = call(method, Array.prototype.slice.call(arguments));
      return value === undefined ? fallbackValue : value;
    };
  }

  var api = {
    init: boot.init,
    setData: boot.setData,
    setEffect: boot.setEffect,
    setPlayerState: boot.setPlayerState,
    resize: boot.resize,
    dispose: boot.dispose,
    debug: boot.debug,
    targetAt: boot.targetAt,
    setQuality: boot.setQuality,
  };

  global.RendererBoot = boot;
  global.CampaignRenderer = api;
})(typeof window !== "undefined" ? window : this);
