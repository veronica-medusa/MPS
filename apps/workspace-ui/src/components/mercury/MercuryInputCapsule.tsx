import React, { useRef } from 'react';
import { ArrowRight, Sparkles } from 'lucide-react';
import { MercuryButton } from './MercuryButton';

export interface MercuryInputCapsuleProps {
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onSubmit?: (val: string) => void;
  placeholder?: string;
  leadingIcon?: React.ReactNode;
  actionLabel?: string;
  actionIcon?: React.ReactNode;
  disabled?: boolean;
  className?: string;
  inputClassName?: string;
  autoFocus?: boolean;
}

export const MercuryInputCapsule: React.FC<MercuryInputCapsuleProps> = ({
  value,
  onChange,
  onSubmit,
  placeholder = 'Type a command, formula, or ask WPS AI...',
  leadingIcon,
  actionLabel = 'Execute',
  actionIcon,
  disabled = false,
  className = '',
  inputClassName = '',
  autoFocus = false,
}) => {
  const inputRef = useRef<HTMLInputElement | null>(null);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (onSubmit && value.trim()) {
        onSubmit(value);
      }
    }
  };

  const handleActionClick = () => {
    if (onSubmit && value.trim()) {
      onSubmit(value);
    } else {
      inputRef.current?.focus();
    }
  };

  return (
    <div
      className={`relative flex items-center h-[46px] w-full rounded-full bg-white dark:bg-[#1e1e2a] border border-[rgba(112,115,147,0.22)] shadow-[0_2px_12px_rgba(40,37,59,0.05),0_1px_2px_rgba(40,37,59,0.04)] transition-all focus-within:border-[#5266eb] focus-within:ring-2 focus-within:ring-[#5266eb]/20 p-[3px] ${className}`}
    >
      {/* Optional Leading Icon / Indicator */}
      {leadingIcon && (
        <div className="pl-3.5 pr-1 flex items-center text-[#554e7b] dark:text-[#9cb4e8]">
          {leadingIcon}
        </div>
      )}

      {/* Text Input */}
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={onChange}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        autoFocus={autoFocus}
        className={`flex-1 h-full px-4 rounded-l-[32px] bg-transparent text-[#28253b] dark:text-[#ededf3] text-[15px] font-[400] placeholder-[#70707d] dark:placeholder-[#9cb4e8]/60 outline-none select-text ${inputClassName}`}
      />

      {/* Fused Trailing Primary Action (Single Solid #5266eb Primary Button) */}
      <MercuryButton
        variant="primary"
        onClick={handleActionClick}
        disabled={disabled || !value.trim()}
        className="!h-[40px] !px-4 !rounded-[32px] shrink-0 text-sm"
      >
        <span>{actionLabel}</span>
        {actionIcon ? (
          <span className="ml-1.5 flex items-center">{actionIcon}</span>
        ) : (
          <ArrowRight className="w-3.5 h-3.5 ml-1.5" />
        )}
      </MercuryButton>
    </div>
  );
};
