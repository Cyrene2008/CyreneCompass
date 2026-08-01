import test from 'node:test'
import assert from 'node:assert/strict'
import { positionForRestore } from '../src/utils/windowPosition.js'

test('keeps valid saved physical coordinates', () => {
  assert.deepEqual(positionForRestore({ x: -1200, y: 240 }), { x: -1200, y: 240 })
})

test('turns tampered saved coordinates into a forced-reset position', () => {
  const reset = { x: 2147483647, y: 2147483647 }
  assert.deepEqual(positionForRestore({ x: Number.MAX_SAFE_INTEGER, y: 0 }), reset)
  assert.deepEqual(positionForRestore({ x: 'not-a-coordinate', y: 0 }), reset)
  assert.deepEqual(positionForRestore({ x: 10.5, y: 20 }), reset)
  assert.deepEqual(positionForRestore(null), reset)
})
