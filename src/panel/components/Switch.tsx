interface SwitchProps {
  checked: boolean;
  onToggle: () => void;
  ariaLabel?: string;
}

export function Switch({ checked, onToggle, ariaLabel }: SwitchProps) {
  return (
    <button
      type="button"
      className="switch"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      onClick={onToggle}
    />
  );
}
