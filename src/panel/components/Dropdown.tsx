import { useEffect, useRef, useState } from "react";

export interface DropdownOption {
  id: string;
  name: string;
  /** 悬停提示；缺省时回退为 name。 */
  title?: string;
}

interface DropdownProps {
  value: string;
  options: DropdownOption[];
  onPick: (id: string) => void;
}

/** Custom dropdown with keyboard navigation; menu flips up near the panel bottom. */
export function Dropdown({ value, options, onPick }: DropdownProps) {
  const [open, setOpen] = useState(false);
  const [up, setUp] = useState(false);
  const rootRef = useRef<HTMLDivElement>(null);
  const selectRef = useRef<HTMLButtonElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  const selected = options.find((o) => o.id === value);
  const label = selected ? selected.name : value;

  const close = () => {
    setOpen(false);
    setUp(false);
  };

  // Close on outside click / Escape anywhere.
  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (rootRef.current && !rootRef.current.contains(e.target as Node)) close();
    };
    const onDocKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") close();
    };
    document.addEventListener("click", onDocClick);
    document.addEventListener("keydown", onDocKey);
    return () => {
      document.removeEventListener("click", onDocClick);
      document.removeEventListener("keydown", onDocKey);
    };
  }, [open]);

  const openMenu = () => {
    setOpen(true);
    // Flip up if the menu would overflow the panel bottom.
    requestAnimationFrame(() => {
      const panel = rootRef.current?.closest(".panel");
      const menu = menuRef.current;
      if (panel && menu) {
        const pr = panel.getBoundingClientRect();
        const mr = menu.getBoundingClientRect();
        setUp(mr.bottom > pr.bottom - 2);
      }
      const target =
        menuRef.current?.querySelector<HTMLButtonElement>("button.selected") ??
        menuRef.current?.querySelector<HTMLButtonElement>("button");
      target?.focus();
    });
  };

  const pick = (id: string) => {
    onPick(id);
    close();
    selectRef.current?.focus();
  };

  const onItemKeyDown = (e: React.KeyboardEvent<HTMLButtonElement>, id: string) => {
    const menu = menuRef.current;
    if (!menu) return;
    const items = Array.from(menu.querySelectorAll<HTMLButtonElement>("button"));
    const idx = items.indexOf(e.currentTarget);
    if (e.key === "ArrowDown") {
      e.preventDefault();
      items[(idx + 1) % items.length].focus();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      items[(idx - 1 + items.length) % items.length].focus();
    } else if (e.key === "Home") {
      e.preventDefault();
      items[0].focus();
    } else if (e.key === "End") {
      e.preventDefault();
      items[items.length - 1].focus();
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      pick(id);
    } else if (e.key === "Escape") {
      e.preventDefault();
      close();
      selectRef.current?.focus();
    } else if (e.key === "Tab") {
      close();
    }
  };

  return (
    <div ref={rootRef} className={`dropdown${open ? " open" : ""}${up ? " up" : ""}`}>
      <button
        ref={selectRef}
        type="button"
        className="select"
        title={label}
        onClick={(e) => {
          e.stopPropagation();
          if (open) close();
          else openMenu();
        }}
        onKeyDown={(e) => {
          if (e.key === "ArrowDown" || e.key === "ArrowUp") {
            e.preventDefault();
            if (!open) openMenu();
          } else if (e.key === "Escape") {
            close();
          }
        }}
      >
        <span className="select-label">{label}</span>
        <svg viewBox="0 0 8 8">
          <path d="M1 2.5 4 5.5 7 2.5" fill="none" stroke="currentColor" strokeWidth="1.2" />
        </svg>
      </button>
      <div ref={menuRef} className="menu">
        {options.map((opt) => (
          <button
            key={opt.id}
            type="button"
            role="option"
            aria-selected={opt.id === value}
            className={opt.id === value ? "selected" : undefined}
            title={opt.title ?? opt.name}
            tabIndex={open ? 0 : -1}
            onClick={(e) => {
              e.stopPropagation();
              pick(opt.id);
            }}
            onKeyDown={(e) => onItemKeyDown(e, opt.id)}
          >
            {opt.name}
          </button>
        ))}
      </div>
    </div>
  );
}
