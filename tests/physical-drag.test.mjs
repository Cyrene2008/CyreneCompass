import test from 'node:test'
import assert from 'node:assert/strict'
import {
  clientOffsetToPhysical,
  createLatestMoveQueue,
  createPointerMoveState,
  exceedsDragThreshold,
  mouseDragTarget,
  pointerAbsolutePhysical,
  touchDragTarget
} from '../src/utils/physicalDrag.js'

test('restores the absolute touch point from physical window position', () => {
  assert.deepEqual(
    pointerAbsolutePhysical({ x: 1200, y: 400 }, { x: 80, y: 36 }, 1.5),
    { x: 1320, y: 454 }
  )
})

test('touch movement is not halved at 200 percent scaling', () => {
  const grabOffset = clientOffsetToPhysical(20, 20, 2)
  const target = touchDragTarget(
    { x: 100, y: 200 },
    { x: 70, y: 45 },
    2,
    grabOffset
  )
  assert.deepEqual(target, { x: 200, y: 250 })
})

test('touch target remains stable after the window has already moved', () => {
  const grabOffset = clientOffsetToPhysical(30, 25, 1.5)
  const first = touchDragTarget({ x: 600, y: 300 }, { x: 70, y: 45 }, 1.5, grabOffset)
  const second = touchDragTarget(first, { x: 30, y: 25 }, 1.5, grabOffset)
  assert.deepEqual(first, { x: 660, y: 330 })
  assert.deepEqual(second, first)
})

test('mouse movement remains incremental in physical pixels', () => {
  assert.deepEqual(
    mouseDragTarget({ x: 500, y: 240 }, { x: 100, y: 80 }, { x: 112, y: 74 }, 1.5),
    { x: 518, y: 231 }
  )
})

test('delayed processing uses the window position captured with the event', () => {
  const grabOffset = clientOffsetToPhysical(20, 20, 1.5)
  const eventWindowPosition = { x: 300, y: 180 }
  const target = touchDragTarget(eventWindowPosition, { x: 60, y: 40 }, 1.5, grabOffset)
  assert.deepEqual(target, { x: 360, y: 210 })
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
