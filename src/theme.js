export const DEFAULT_ACCENT = '#ea5ec1'

const clamp = (v, min, max) => Math.min(max, Math.max(min, v))
const normalizeHex = (value, fallback = DEFAULT_ACCENT) => {
  const raw = String(value || '').trim().replace(/^#/, '')
  if (/^[0-9a-f]{3}$/i.test(raw)) return `#${raw.split('').map(c => c + c).join('').toLowerCase()}`
  if (/^[0-9a-f]{6}$/i.test(raw)) return `#${raw.toLowerCase()}`
  return fallback
}
const hexToRgb = value => { const h = normalizeHex(value).slice(1); return { r: parseInt(h.slice(0, 2), 16), g: parseInt(h.slice(2, 4), 16), b: parseInt(h.slice(4, 6), 16) } }
const rgbToHex = ({ r, g, b }) => `#${[r, g, b].map(x => Math.round(clamp(x, 0, 255)).toString(16).padStart(2, '0')).join('')}`
const rgbToHsl = ({ r, g, b }) => {
  const a = [r, g, b].map(v => v / 255); const max = Math.max(...a); const min = Math.min(...a); const l = (max + min) / 2
  if (max === min) return { h: 0, s: 0, l }
  const d = max - min; const s = l > .5 ? d / (2 - max - min) : d / (max + min); let h
  if (max === a[0]) h = (a[1] - a[2]) / d + (a[1] < a[2] ? 6 : 0); else if (max === a[1]) h = (a[2] - a[0]) / d + 2; else h = (a[0] - a[1]) / d + 4
  return { h: h * 60, s, l }
}
const hslToHex = ({ h, s, l }) => {
  const hue = ((h % 360) + 360) % 360; const sat = clamp(s, 0, 1); const light = clamp(l, 0, 1); const c = (1 - Math.abs(2 * light - 1)) * sat; const x = c * (1 - Math.abs((hue / 60) % 2 - 1)); const m = light - c / 2
  const p = hue < 60 ? [c, x, 0] : hue < 120 ? [x, c, 0] : hue < 180 ? [0, c, x] : hue < 240 ? [0, x, c] : hue < 300 ? [x, 0, c] : [c, 0, x]
  return rgbToHex({ r: (p[0] + m) * 255, g: (p[1] + m) * 255, b: (p[2] + m) * 255 })
}
const rgba = (hex, alpha) => { const { r, g, b } = hexToRgb(hex); return `rgba(${r}, ${g}, ${b}, ${alpha})` }
export function createThemeVariables(accentValue, dark = false, neutral = false) {
  const base = rgbToHsl(hexToRgb(accentValue)); const s = clamp(base.s, .48, .9); const accent = hslToHex({ h: base.h, s, l: dark ? clamp(base.l, .58, .72) : clamp(base.l, .4, .56) }); const surface = hslToHex({ h: base.h, s: clamp(base.s * .18, .08, .22), l: dark ? .12 : .97 })
  return {
    '--accent': accent, '--accent-light': hslToHex({ h: base.h, s, l: dark ? .8 : .72 }), '--accent-dark': hslToHex({ h: base.h, s, l: dark ? .42 : .34 }), '--accent-hover': hslToHex({ h: base.h, s, l: dark ? .65 : .45 }), '--accent-50': rgba(accent, dark ? .16 : .1), '--accent-200': rgba(accent, dark ? .28 : .22),
    '--bg-base': neutral ? (dark ? '#202020' : '#f3f3f3') : surface,
    '--bg-card': neutral ? (dark ? 'rgba(44,44,44,.58)' : 'rgba(255,255,255,.62)') : (dark ? 'rgba(38,28,36,.56)' : 'rgba(255,255,255,.6)'),
    '--bg-card-solid': neutral ? (dark ? '#2c2c2c' : '#ffffff') : (dark ? '#30232c' : '#fffafd'),
    '--bg-hover': neutral ? (dark ? '#383838' : '#e8e8e8') : (dark ? '#3b2c38' : '#f9eef6'),
    '--text-primary': neutral ? (dark ? '#ffffff' : '#1b1b1b') : (dark ? '#fff5fb' : '#351a2c'),
    '--text-secondary': neutral ? (dark ? '#d6d6d6' : '#424242') : (dark ? '#e0c9d8' : '#6e4b60'),
    '--text-muted': neutral ? (dark ? '#a0a0a0' : '#707070') : (dark ? '#ad8c9f' : '#9d738c'),
    '--border-default': neutral ? (dark ? 'rgba(255,255,255,.14)' : 'rgba(0,0,0,.14)') : rgba(accent, dark ? .22 : .18),
    '--border-subtle': neutral ? (dark ? 'rgba(255,255,255,.08)' : 'rgba(0,0,0,.08)') : rgba(accent, dark ? .12 : .1),
    '--border-strong': neutral ? (dark ? 'rgba(255,255,255,.22)' : 'rgba(0,0,0,.22)') : rgba(accent, dark ? .34 : .28),
    '--shadow': 'none'
  }
}
