interface StatusBadgeProps {
  status: 'success' | 'danger' | 'warning' | 'info' | 'primary' | 'secondary'
  children: React.ReactNode
  className?: string
}

function StatusBadge({ status, children, className = '' }: StatusBadgeProps) {
  return (
    <span className={`badge bg-${status} ${className}`}>
      {children}
    </span>
  )
}

export default StatusBadge
