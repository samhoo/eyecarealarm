import { useEffect, useRef, useState } from "react";

interface StepperProps {
  value: number;
  min: number;
  max: number;
  ariaLabel: string;
  onCommit: (v: number) => void;
}

/** Numeric input with up/down steppers. Typing commits on blur; arrows step by 1. */
export function Stepper({ value, min, max, ariaLabel, onCommit }: StepperProps) {
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v || min)));
  const [text, setText] = useState(String(value));
  const focusedRef = useRef(false);

  // External value changes sync into the input unless the user is editing.
  useEffect(() => {
    if (!focusedRef.current) setText(String(value));
  }, [value]);

  const commit = (raw: string) => {
    const v = clamp(Number(raw));
    setText(String(v));
    if (v !== value) onCommit(v);
  };
  const step = (dir: 1 | -1) => {
    commit(String(clamp((Number(text) || min) + dir)));
  };

  return (
    <div className="stepper">
      <input
        type="text"
        inputMode="numeric"
        value={text}
        aria-label={ariaLabel}
        onFocus={() => {
          focusedRef.current = true;
        }}
        onChange={(e) => setText(e.target.value)}
        onBlur={(e) => {
          focusedRef.current = false;
          commit(e.target.value);
        }}
        onKeyDown={(e) => {
          if (e.key === "ArrowUp") {
            e.preventDefault();
            step(1);
          } else if (e.key === "ArrowDown") {
            e.preventDefault();
            step(-1);
          }
        }}
      />
      <span className="step-btns">
        <button type="button" aria-label="+" onClick={() => step(1)} tabIndex={-1}>
          <svg viewBox="0 0 8 8">
            <path d="M1 5.5 4 2.5 7 5.5" fill="none" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
        <button type="button" aria-label="-" onClick={() => step(-1)} tabIndex={-1}>
          <svg viewBox="0 0 8 8">
            <path d="M1 2.5 4 5.5 7 2.5" fill="none" stroke="currentColor" strokeWidth="1.2" />
          </svg>
        </button>
      </span>
    </div>
  );
}
