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
  className = ''
}: NumberInputProps) {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseInt(e.target.value) || 0
    onChange(newValue)
  }

  return (
    <div className="material-group">
      <input
        id={id}
        type="number"
        className={`material-input ${className}`}
        value={value}
        onChange={handleChange}
        min={min}
        max={max}
        step={step}
        placeholder=" "
      />
      {label && (
        <label htmlFor={id} className="material-label">
          {label}
        </label>
      )}
      <span className="material-bar"></span>
    </div>
  )
}

export default NumberInput
