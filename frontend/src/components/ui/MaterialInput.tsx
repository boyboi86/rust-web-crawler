interface MaterialInputProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'email' | 'url' | 'number'
  placeholder?: string
  required?: boolean
  className?: string
}

function MaterialInput({
  id,
  label,
  value,
  onChange,
  type = 'text',
  placeholder = ' ',
  required = false,
  className = ''
}: MaterialInputProps) {
  return (
    <div className={`material-group ${className}`}>
      <input
        id={id}
        type={type}
        className="material-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        required={required}
      />
      <label htmlFor={id} className="material-label">
        {label}
      </label>
      <span className="material-bar"></span>
    </div>
  )
}

export default MaterialInput
