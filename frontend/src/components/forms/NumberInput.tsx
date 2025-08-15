interface NumberInputProps {
  id?: string
  label?: string
  value: number
  onChange: (value: number) => void
  min?: number
  max?: number
  step?: number
  className?: string
  placeholder?: string
}

function NumberInput({
  id,
  label,
  value,
  onChange,
  min,
  max,
  step = 1,
  className = '',
  placeholder
}: NumberInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseInt(e.target.value) || 0
    onChange(newValue)
  }

  return (
    <div className="d-flex align-items-center">
      {label && (
        <label htmlFor={id} className="form-label me-2 mb-0 google-label">
          {label}:
        </label>
      )}
      <input
        id={id}
        type="number"
        className={`form-control form-control-sm google-input ${className}`}
        value={value}
        onChange={handleChange}
        min={min}
        max={max}
        step={step}
        placeholder={placeholder}
        style={{ width: '100px' }}
      />
    </div>
  )
}

export default NumberInput
