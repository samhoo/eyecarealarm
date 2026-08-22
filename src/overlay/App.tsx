import { useCallback, useEffect, useRef, useState } from "react";
import { onOverlayClose, onOverlayInputReady, onOverlayStart, overlayExit } from "../shared/ipc";
import { t } from "../shared/i18n";
import { TIMELINE, type OverlayPayload } from "../shared/types";

/**
 * 文字1（warmupText）：0-2s 渐显，2-4s 全显，4-5s 渐隐（1s）。
 * 文字2（restText）：text2Start(5)-6s 渐显（1s），其中 5s 与文字1 渐隐交叉淡化；第 8s 屏蔽输入。
 */
const TEXT1_IN_END = 2;
const TEXT1_HOLD_END = TIMELINE.warmupEnd - 1;
const TEXT2_FADE = 1;

export default function App() {
  const [payload, setPayload] = useState<OverlayPayload | null>(null);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [canExit, setCanExit] = useState(false);
  const [fadeMs, setFadeMs] = useState<number | null>(null);
  const exitedRef = useRef(false);
  const closedRef = useRef(false);
  const canExitRef = useRef(false);
  const dimRef = useRef<HTMLDivElement>(null);
  const text1Ref = useRef<HTMLParagraphElement>(null);
  const text2Ref = useRef<HTMLParagraphElement>(null);
  const rafRef = useRef(0);

  /** 只启动一次：onOverlayStart 事件与 getSettings 快照，先到者为准 */
  /** 窗口常驻复用：每次 overlay-start 事件都重置并重新跑完整时间线 */
  const start = useCallback((p: OverlayPayload) => {
    exitedRef.current = false;
    closedRef.current = false;
    canExitRef.current = false;
    setCanExit(false);
    setFadeMs(null);
    setCountdown(null);
    setPayload(p); // 新对象 → rAF effect cleanup 后重跑，时间线归零
  }, []);

  /** Escape / 退出按钮仅在正式阶段开始后可响应。 */
  const exit = useCallback(() => {
    if (!canExitRef.current || exitedRef.current) return;
    exitedRef.current = true;
    void overlayExit();
  }, []);

  useEffect(() => {
    let disposed = false;
    const unlistens: Array<() => void> = [];
    onOverlayStart(start).then((u) => {
      if (disposed) u();
      else unlistens.push(u);
    });
    onOverlayInputReady(() => {
      canExitRef.current = true;
      setCanExit(true);
    }).then((u) => {
      if (disposed) u();
      else unlistens.push(u);
    });
    onOverlayClose((ms) => {
      if (closedRef.current) return;
      closedRef.current = true;
      setFadeMs(ms);
    }).then((u) => {
      if (disposed) u();
      else unlistens.push(u);
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        exit();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      disposed = true;
      unlistens.forEach((u) => u());
      window.removeEventListener("keydown", onKey);
    };
  }, [start, exit]);

  /* 时间线 rAF 驱动：透明度逐帧写入 DOM（60fps 无跳变），倒计时数字每秒走 setState */
  useEffect(() => {
    if (!payload) return;
    /* 设置值为透明度 %：100 = 全透明 → 黑幕不透明度 = (100 - overlay_opacity) / 100 */
    const dimTarget = (100 - payload.overlay_opacity) / 100;
    const naturalEnd = TIMELINE.blockStart + payload.rest_sec;
    const stopAt = naturalEnd + TIMELINE.fadeOutMs / 1000 + 0.5;
    const t0 = performance.now();

    const tick = (now: number) => {
      const tSec = (now - t0) / 1000;

      const dim = dimRef.current;
      if (dim) {
        /* 0-warmupEnd：0 线性升至目标值，之后保持。
           上限 0.999：macOS 12 WKWebView 在透明窗口上会丢失完全不透明
           (alpha=1) 的大背景层（遮罩透明度设 0% 时 dimTarget=1 会隐形）。 */
        dim.style.opacity = String(
          Math.min(0.999, dimTarget, dimTarget * (tSec / TIMELINE.warmupEnd))
        );
      }

      const p1 = text1Ref.current;
      if (p1) {
        let o = 0;
        if (tSec < TEXT1_IN_END) o = tSec / TEXT1_IN_END;
        else if (tSec < TEXT1_HOLD_END) o = 1;
        else if (tSec < TIMELINE.warmupEnd)
          o = (TIMELINE.warmupEnd - tSec) / (TIMELINE.warmupEnd - TEXT1_HOLD_END);
        p1.style.opacity = String(Math.min(1, Math.max(0, o)));
      }

      const p2 = text2Ref.current;
      if (p2) {
        const o =
          tSec < TIMELINE.text2Start ? 0 : (tSec - TIMELINE.text2Start) / TEXT2_FADE;
        p2.style.opacity = String(Math.min(1, Math.max(0, o)));
      }

      /* blockStart 起：n 从 rest_sec 逐秒降到 0。 */
      if (tSec >= TIMELINE.blockStart) {
        const n = Math.max(0, Math.ceil(payload.rest_sec - (tSec - TIMELINE.blockStart)));
        setCountdown((prev) => (prev === n ? prev : n));
      }

      if (tSec < stopAt) rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(rafRef.current);
  }, [payload]);

  if (!payload) return null;
  const msg = t(payload.lang);
  const n = countdown ?? payload.rest_sec;
  /* exitBtn 是整句文案；按数字拆分以便倒计时数字用 accent 绿 */
  const [pre, post = ""] = msg.exitBtn(n).split(String(n));

  return (
    <div
      className="veil"
      style={
        fadeMs != null
          ? { opacity: 0, transition: `opacity ${fadeMs}ms ease` }
          : undefined
      }
    >
      <div className="veil-dim" ref={dimRef} />
      <div className="veil-text">
        <p className="phase" ref={text1Ref}>
          {msg.warmupText}
        </p>
        <p className="phase" ref={text2Ref}>
          {msg.restText(payload.rest_sec)}
        </p>
      </div>
      <button
        className={`exit-btn${canExit ? " is-ready" : ""}`}
        type="button"
        onClick={exit}
        tabIndex={canExit ? 0 : -1}
        aria-hidden={!canExit}
      >
        {pre}
        <span className="cd">{n}</span>
        {post}
      </button>
    </div>
  );
}
