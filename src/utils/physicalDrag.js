export const DRAG_THRESHOLD = 5

// 统一的“屏幕增量”拖动模型：窗口从上次已应用位置跟随指针的绝对屏幕位移，
// 乘 DPI 缩放得到物理像素。鼠标与触屏共用——触屏不用 clientX/Y 相对窗口坐标，
// 否则串行队列滞后消费旧采样时会用旧坐标套新窗口位置，导致窗口回退/不跟手。
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
