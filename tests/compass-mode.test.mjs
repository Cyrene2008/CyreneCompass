import test from 'node:test'
import assert from 'node:assert/strict'
import { DIR_TO_INDEX, directionSlotItems, gridIndexToDirection } from '../src/utils/compassMode.js'

test('DIR_TO_INDEX covers every direction exactly once', () => {
  assert.deepEqual([...DIR_TO_INDEX].sort((a, b) => a - b), [0, 1, 2, 3, 4, 5, 6, 7])
})

test('each direction maps to the matching grid cell', () => {
  // 方向（0=上，顺时针）→ 网格下标（0=左上 1=上 2=右上 3=左 4=右 5=左下 6=下 7=右下）
  assert.equal(DIR_TO_INDEX[0], 1) // 上
  assert.equal(DIR_TO_INDEX[1], 2) // 右上
  assert.equal(DIR_TO_INDEX[2], 4) // 右
  assert.equal(DIR_TO_INDEX[3], 7) // 右下
  assert.equal(DIR_TO_INDEX[4], 6) // 下
  assert.equal(DIR_TO_INDEX[5], 5) // 左下
  assert.equal(DIR_TO_INDEX[6], 3) // 左
  assert.equal(DIR_TO_INDEX[7], 0) // 左上
})

test('ring slot i shows the item of the grid cell in that direction', () => {
  const items = Array.from({ length: 8 }, (_, i) => ({ id: `grid-${i}`, label: `格${i}` }))
  const slots = directionSlotItems(items)
  // 槽 0（正上）显示网格“上”格（下标 1），槽 1（右上）显示“右上”格（下标 2）……
  assert.equal(slots[0].id, 'grid-1')
  assert.equal(slots[1].id, 'grid-2')
  assert.equal(slots[2].id, 'grid-4')
  assert.equal(slots[3].id, 'grid-7')
  assert.equal(slots[4].id, 'grid-6')
  assert.equal(slots[5].id, 'grid-5')
  assert.equal(slots[6].id, 'grid-3')
  assert.equal(slots[7].id, 'grid-0')
})

test('direction and grid index round-trip', () => {
  for (let dir = 0; dir < 8; dir += 1) {
    assert.equal(gridIndexToDirection(DIR_TO_INDEX[dir]), dir)
  }
})
