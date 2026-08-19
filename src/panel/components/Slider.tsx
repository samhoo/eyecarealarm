import { useEffect, useRef, useState } from "react";

const THUMB = 14;

interface SliderProps {
  value: number;
  ariaLabel: string;
  /** Fired once per user gesture: pointer release / keyboard step. */
  onCommit: (v: number) => void;
}

/**
 * Custom 0-100 slider (green fill + gray track). Visual state is local while
 * dragging so the knob tracks the pointer without waiting for the IPC
 * roundtrip; `onCommit` fires only when the gesture ends.
 */
export function Slider({ value, ariaLabel, onCommit }: SliderProps) {
  const clamp = (v: number) => Math.max(0, Math.min(100, Math.round(v)));
  const [local, setLocal] = useState(clamp(value));
  const draggingRef = useRef(false);
  const wrapRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!draggingRef.current) setLocal(clamp(value));
  }, [value]);

  const valueFromX = (clientX: number): number | null => {
    const wrap = wrapRef.current;
    if (!wrap) return null;
    const rect = wrap.getBoundingClientRect();
    const usable = rect.width - THUMB;
    if (usable <= 0) return null;
    return clamp(Math.round(((clientX - rect.left - THUMB / 2) / usable) * 100));
  };
  const setFromX = (clientX: number) => {
    const v = valueFromX(clientX);
    if (v !== null) setLocal(v);
  };

  const thumbLeft = `calc((100% - ${THUMB}px) * ${local / 100})`;
  const fillWidth = local === 0 ? 0 : `calc((100% - ${THUMB}px) * ${local / 100} + ${THUMB / 2}px)`;

  return (
    <>
      <span className="pct">{local}%</span>
      <div
        ref={wrapRef}
        className="slider-wrap"
        role="slider"
        tabIndex={0}
        aria-label={ariaLabel}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={local}
        onPointerDown={(e) => {
          e.preventDefault();
          draggingRef.current = true;
          e.currentTarget.setPointerCapture(e.pointerId);
          setFromX(e.clientX);
        }}
        onPointerMove={(e) => {
          if (draggingRef.current) setFromX(e.clientX);
        }}
        onPointerUp={(e) => {
          if (!draggingRef.current) return;
          draggingRef.current = false;
          e.currentTarget.releasePointerCapture(e.pointerId);
          const v = valueFromX(e.clientX);
          if (v !== null) {
            setLocal(v);
            onCommit(v);
          } else {
            onCommit(local);
          }
        }}
        onPointerCancel={() => {
          draggingRef.current = false;
        }}
        onKeyDown={(e) => {
          const step = e.shiftKey ? 10 : 1;
          let v: number | null = null;
          if (e.key === "ArrowRight" || e.key === "ArrowUp") v = clamp(local + step);
          else if (e.key === "ArrowLeft" || e.key === "ArrowDown") v = clamp(local - step);
          if (v !== null && v !== local) {
            e.preventDefault();
            setLocal(v);
            onCommit(v);
          }
        }}
      >
        <div className="slider-track" />
        <div className="slider-fill" style={{ width: fillWidth }} />
        <div className="slider-thumb" style={{ left: thumbLeft }} />
      </div>
    </>
  );
}
