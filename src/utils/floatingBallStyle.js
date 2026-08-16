export function coverScale(imageWidth, imageHeight, viewportSize) {
  if (imageWidth <= 0 || imageHeight <= 0 || viewportSize <= 0) return 1
  return Math.max(viewportSize / imageWidth, viewportSize / imageHeight)
}

export function clampCropOffset(offset, imageSize, viewportSize) {
  const limit = Math.max(0, (imageSize - viewportSize) / 2)
  return Math.max(-limit, Math.min(limit, offset))
}

export function resolveBallRadius(style, radius) {
  return style === 'text' || style === 'image'
    ? Math.max(0, Math.min(50, Number(radius) || 0))
    : 50
}

export function dataUrlByteLength(dataUrl) {
  const encoded = dataUrl.split(',', 2)[1] || ''
  return Math.max(0, Math.floor(encoded.length * 3 / 4) - (encoded.endsWith('==') ? 2 : encoded.endsWith('=') ? 1 : 0))
}
