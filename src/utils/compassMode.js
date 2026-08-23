// 桌面罗盘模式：后端方向（0=上，顺时针 45° 递增）与 3×3 网格下标之间的映射。
// 网格下标顺序为 CSS grid 自动放置顺序：
// 0=左上 1=上 2=右上 3=左 4=右 5=左下 6=下 7=右下（中央为关闭/返回按钮）。

// 方向 → 网格下标：DIR_TO_INDEX[dir] = 该方向对应的网格格位。
export const DIR_TO_INDEX = [1, 2, 4, 7, 6, 5, 3, 0]

// 环上槽位 i（角度 i*45°，0=正上，顺时针）显示网格下标 DIR_TO_INDEX[i] 的选项，
// 使环的方位与 3×3 罗盘一致（正上方显示“上”格、右上显示“右上”格……）。
export function directionSlotItems(items) {
  return DIR_TO_INDEX.map(gridIndex => items[gridIndex])
}

// 网格下标 → 方向（DIR_TO_INDEX 的逆映射，用于校验排列完整）。
export function gridIndexToDirection(gridIndex) {
  return DIR_TO_INDEX.indexOf(gridIndex)
}
