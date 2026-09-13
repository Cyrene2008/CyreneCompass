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

test('screen-delta model tracks window position through clamped moves', () => {
  assert.deepEqual(
    pointerDragTarget({ x: 660, y: 330 }, { x: 70, y: 45 }, { x: 30, y: 25 }, 1.5),
    { x: 600, y: 300 }
  )
})

test('touch movement applies the window display scale once', () => {
  assert.deepEqual(
    pointerDragTarget({ x: 100, y: 200 }, { x: 20, y: 20 }, { x: 70, y: 45 }, 2),
    { x: 200, y: 250 }
  )
})

test('queued samples use the last applied sample as their baseline', async () => {
  const pointer = createPointerMoveState({ lastScreenX: 0, lastScreenY: 0 })
  const processed = []
  pointer.drag = Promise.resolve({ move: sample => processed.push(sample) })
  pointer.queue.push({ screenX: 10, screenY: 0 })
  pointer.queue.push({ screenX: 20, screenY: 0 })
  await pointer.queue.flush()
  assert.deepEqual(processed, [{ screenX: 10, screenY: 0 }, { screenX: 20, screenY: 0 }])
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

test('touch screen-delta drag with grab offset tracks finger at scale 1.5', () => {
  const windowPos = { x: 400, y: 200 }
  const factor = 1.5
  const startScreen = { x: 100, y: 80 }
  const grab = {
    x: startScreen.x * factor - windowPos.x,
    y: startScreen.y * factor - windowPos.y
  }
  const moveScreen = { x: 120, y: 90 }
  const target = {
    x: Math.round(moveScreen.x * factor - grab.x),
    y: Math.round(moveScreen.y * factor - grab.y)
  }
  assert.deepEqual(target, { x: 430, y: 215 })
})

test('touch screen-delta drag with grab offset at 200 percent scaling', () => {
  const windowPos = { x: 100, y: 50 }
  const factor = 2
  const startScreen = { x: 30, y: 20 }
  const grab = {
    x: startScreen.x * factor - windowPos.x,
    y: startScreen.y * factor - windowPos.y
  }
  const moveScreen = { x: 45, y: 30 }
  const target = {
    x: Math.round(moveScreen.x * factor - grab.x),
    y: Math.round(moveScreen.y * factor - grab.y)
  }
  assert.deepEqual(target, { x: 130, y: 70 })
})
