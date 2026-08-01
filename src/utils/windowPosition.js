const I32_MIN = -2147483648
const I32_MAX = 2147483647

export function positionForRestore(value) {
  const x = Number(value?.x)
  const y = Number(value?.y)
  const valid = Number.isInteger(x) && Number.isInteger(y) && x >= I32_MIN && x <= I32_MAX && y >= I32_MIN && y <= I32_MAX
  return valid ? { x, y } : { x: I32_MAX, y: I32_MAX }
}
