import { ReactNode } from 'react'

interface CardProps {
  children: ReactNode
  title?: string
  className?: string
  header?: ReactNode
  footer?: ReactNode
}

function Card({ children, title, className = '', header, footer }: CardProps) {
  return (
    <div className={`card google-card ${className}`}>
      {(title || header) && (
        <div className="card-header border-0 bg-transparent">
          {header || (
            <h5 className="card-title google-header mb-0">{title}</h5>
          )}
        </div>
      )}
      <div className="card-body">
        {children}
      </div>
      {footer && (
        <div className="card-footer border-0 bg-transparent">
          {footer}
        </div>
      )}
    </div>
  )
}

export default Card
