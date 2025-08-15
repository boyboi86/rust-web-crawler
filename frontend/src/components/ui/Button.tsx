import { ReactNode } from 'react'

interface ButtonProps {
  children: ReactNode
  onClick?: () => void
  type?: 'button' | 'submit' | 'reset'
  variant?: 'primary' | 'secondary' | 'success' | 'danger' | 'warning' | 'info' | 'light' | 'dark'
  size?: 'sm' | 'md' | 'lg'
  disabled?: boolean
  className?: string
  startIcon?: ReactNode
  endIcon?: ReactNode
}

function Button({
  children,
  onClick,
  type = 'button',
  variant = 'primary',
  size = 'md',
  disabled = false,
  className = '',
  startIcon,
  endIcon
}: ButtonProps) {
  const sizeClass = size !== 'md' ? `btn-${size}` : ''
  
  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      className={`btn btn-${variant} google-btn ${sizeClass} ${className}`}
    >
      {startIcon && <span className="me-2">{startIcon}</span>}
      {children}
      {endIcon && <span className="ms-2">{endIcon}</span>}
    </button>
  )
}

export default Button
