interface ToggleProps {
  enabled: boolean
  onChange: (enabled: boolean) => void
  label: string
  description?: string
  disabled?: boolean
}

function Toggle({ enabled, onChange, label, description, disabled = false }: ToggleProps) {
  return (
    <div className="d-flex align-items-center justify-content-between py-2">
      <div className="flex-grow-1">
        <div className="d-flex align-items-center">
          <span className="google-subheader fw-medium">
            {label}
          </span>
          {enabled ? (
            <span className="ms-2 badge bg-success">
              Enabled
            </span>
          ) : (
            <span className="ms-2 badge bg-danger">
              Disabled
            </span>
          )}
        </div>
        {description && (
          <p className="small text-muted mt-1 mb-0">
            {description}
          </p>
        )}
      </div>
      
      <div className="form-check form-switch">
        <input
          className="form-check-input"
          type="checkbox"
          role="switch"
          id={`toggle-${label.replace(/\s+/g, '-').toLowerCase()}`}
          checked={enabled}
          disabled={disabled}
          onChange={(e) => onChange(e.target.checked)}
        />
        <label 
          className="form-check-label visually-hidden" 
          htmlFor={`toggle-${label.replace(/\s+/g, '-').toLowerCase()}`}
        >
          {label}
        </label>
      </div>
    </div>
  )
}

export default Toggle
