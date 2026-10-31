import React from 'react';

export type MercuryButtonVariant =
  | 'primary'
  | 'glass-dark'
  | 'tint-dark'
  | 'tint-light'
  | 'outline-light'
  | 'small'
  | 'icon-round';

export interface MercuryButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: MercuryButtonVariant;
  icon?: React.ReactNode;
  trailingIcon?: React.ReactNode;
  active?: boolean;
}

export const MercuryButton: React.FC<MercuryButtonProps> = ({
  variant = 'primary',
  icon,
  trailingIcon,
  active = false,
  children,
  className = '',
  disabled,
  ...props
}) => {
  const baseClasses =
    'inline-flex items-center justify-center font-sans select-none transition-all duration-150 cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed outline-none';

  let variantClasses = '';
  switch (variant) {
    case 'primary':
      variantClasses =
        'h-[40px] px-5 rounded-[32px] bg-[#5266eb] hover:bg-[#4355d6] active:bg-[#3848b8] text-white mercury-ui font-[420] shadow-sm';
      break;

    case 'glass-dark':
      variantClasses =
        'h-[40px] px-5 rounded-[40px] bg-[rgba(175,178,206,0.20)] hover:bg-[rgba(175,178,206,0.30)] border border-[rgba(175,178,206,0.36)] text-[#ededf3] mercury-ui font-[420]';
      break;

    case 'tint-dark':
      variantClasses =
        'h-[40px] px-5 rounded-[40px] bg-[rgba(156,180,232,0.20)] hover:bg-[rgba(156,180,232,0.30)] text-[#ededf3] mercury-ui font-[420]';
      break;

    case 'tint-light':
      variantClasses =
        'h-[40px] px-5 rounded-[40px] bg-[rgba(82,102,235,0.10)] hover:bg-[rgba(82,102,235,0.18)] text-[#28253b] dark:text-[#ededf3] mercury-ui font-[420]';
      break;

    case 'outline-light':
      variantClasses =
        'h-[40px] px-5 rounded-[40px] bg-[rgba(112,115,147,0.10)] hover:bg-[rgba(112,115,147,0.18)] border border-[rgba(112,115,147,0.22)] text-[#28253b] dark:text-[#ededf3] mercury-ui font-[420]';
      break;

    case 'small':
      variantClasses =
        'h-[32px] px-4 rounded-[40px] bg-[rgba(82,102,235,0.10)] hover:bg-[rgba(82,102,235,0.18)] text-[#28253b] dark:text-[#ededf3] mercury-ui-sm font-[420]';
      break;

    case 'icon-round':
      variantClasses =
        'w-[36px] h-[36px] rounded-full p-1.5 bg-[rgba(112,115,147,0.10)] hover:bg-[rgba(112,115,147,0.20)] text-[#28253b] dark:text-[#ededf3] transition';
      break;
  }

  const activeClasses = active
    ? 'ring-2 ring-[#5266eb]/50 bg-[rgba(82,102,235,0.20)] text-[#5266eb]'
    : '';

  return (
    <button
      className={`${baseClasses} ${variantClasses} ${activeClasses} ${className}`}
      disabled={disabled}
      {...props}
    >
      {icon && <span className={children ? 'mr-2 flex items-center' : 'flex items-center'}>{icon}</span>}
      {children}
      {trailingIcon && <span className="ml-2 flex items-center">{trailingIcon}</span>}
    </button>
  );
};
