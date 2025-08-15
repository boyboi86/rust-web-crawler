import { ReactNode } from 'react'

interface FormGroupProps {
  children: ReactNode
  label?: string
  description?: string
  required?: boolean
  className?: string
}

function FormGroup({ children, label, description, required = false, className = '' }: FormGroupProps) {
  return (
    <div className={`mb-3 ${className}`}>
      {label && (
        <label className={`form-label google-label ${required ? 'required' : ''}`}>
          {label}
          {required && <span className="text-danger ms-1">*</span>}
        </label>
      )}
      {children}
      {description && (
        <div className="form-text text-muted">
          {description}
        </div>
      )}
    </div>
  )
}

export default FormGroup
