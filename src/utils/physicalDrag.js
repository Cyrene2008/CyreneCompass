export const DRAG_THRESHOLD = 5

// Window APIs use physical pixels; convert pointer deltas once at this boundary.
export function pointerDragTarget(previousTarget, previousScreen, currentScreen, scaleFactor) {
  return {
    x: Math.round(previousTarget.x + (currentScreen.x - previousScreen.x) * scaleFactor),
    y: Math.round(previousTarget.y + (currentScreen.y - previousScreen.y) * scaleFactor)
  }
}

export function exceedsDragThreshold(start, current, threshold = DRAG_THRESHOLD) {
  return Math.max(Math.abs(current.x - start.x), Math.abs(current.y - start.y)) > threshold
}

export function createLatestMoveQueue(worker) {
  let pending
  let running = null

  const start = () => {
    if (running) return running
    running = (async () => {
      while (pending !== undefined) {
        const next = pending
        pending = undefined
        await worker(next)
      }
    })().finally(() => {
      running = null
      if (pending !== undefined) start()
    })
    return running
  }

  return {
    push(value) {
      pending = value
      start()
    },
    async flush() {
      while (running || pending !== undefined) {
        start()
        if (running) await running
      }
    },
    clear() {
      pending = undefined
    }
  }
}

export function createPointerMoveState(initialState) {
  const pointer = { ...initialState, drag: null, queue: null }
  pointer.queue = createLatestMoveQueue(async sample => {
    const drag = await pointer.drag
    await drag.move(sample)
  })
  return pointer
}
