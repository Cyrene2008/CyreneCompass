import test from 'node:test'
import assert from 'node:assert/strict'
import {
  createLatestMoveQueue,
  createPointerMoveState,
  exceedsDragThreshold,
  pointerDragTarget
} from '../src/utils/physicalDrag.js'

test('incremental drag target follows pointer delta in physical pixels', () => {
  assert.deepEqual(
    pointerDragTarget({ x: 500, y: 240 }, { x: 100, y: 80 }, { x: 112, y: 74 }, 1.5),
    { x: 518, y: 231 }
  )
})

test('touch uses the same screen-delta model as mouse', () => {
  assert.deepEqual(
    pointerDragTarget({ x: 660, y: 330 }, { x: 70, y: 45 }, { x: 30, y: 25 }, 1.5),
    { x: 600, y: 300 }
  )
})

test('drag target always moves forward even when samples are consumed late', () => {
  let applied = { x: 100, y: 100 }
  let previousScreen = { x: 200, y: 200 }
  const screens = [{ x: 260, y: 210 }, { x: 320, y: 220 }, { x: 400, y: 250 }]
  for (const screen of screens) {
    applied = pointerDragTarget(applied, previousScreen, screen, 1)
    previousScreen = screen
  }
  assert.deepEqual(applied, { x: 300, y: 150 })
})

test('drag target uses strict physical deltas at 200 percent scaling', () => {
  assert.deepEqual(
    pointerDragTarget({ x: 100, y: 200 }, { x: 20, y: 20 }, { x: 70, y: 45 }, 2),
    { x: 200, y: 250 }
  )
})

test('drag threshold remains strictly greater than five pixels', () => {
  assert.equal(exceedsDragThreshold({ x: 10, y: 10 }, { x: 15, y: 7 }), false)
  assert.equal(exceedsDragThreshold({ x: 10, y: 10 }, { x: 16, y: 10 }), true)
})

test('move queue is serial and coalesces pending events to the latest sample', async () => {
  const processed = []
  let active = 0
  let maxActive = 0
  let releaseFirst
  const firstGate = new Promise(resolve => { releaseFirst = resolve })
  const queue = createLatestMoveQueue(async value => {
    active += 1
    maxActive = Math.max(maxActive, active)
    processed.push(value)
    if (value === 1) await firstGate
    active -= 1
  })

  queue.push(1)
  await Promise.resolve()
  queue.push(2)
  queue.push(3)
  releaseFirst()
  await queue.flush()

  assert.deepEqual(processed, [1, 3])
  assert.equal(maxActive, 1)
})

test('pointer queue observes a drag controller attached after pointer creation', async () => {
  const moved = []
  const pointer = createPointerMoveState({ id: 7 })
  pointer.drag = Promise.resolve({ move: sample => moved.push(sample) })
  pointer.queue.push({ x: 42, y: 24 })
  await pointer.queue.flush()
  assert.deepEqual(moved, [{ x: 42, y: 24 }])
})
