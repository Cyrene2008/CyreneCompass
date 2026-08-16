import test from 'node:test'
import assert from 'node:assert/strict'
import { clampCropOffset, coverScale, dataUrlByteLength, resolveBallRadius } from '../src/utils/floatingBallStyle.js'

test('minimum crop scale covers one pair of viewport edges', () => {
  assert.equal(coverScale(800, 400, 320), 0.8)
  assert.equal(coverScale(400, 800, 320), 0.8)
  assert.equal(coverScale(400, 400, 320), 0.8)
})

test('crop offsets cannot expose empty space', () => {
  assert.equal(clampCropOffset(200, 480, 320), 80)
  assert.equal(clampCropOffset(-200, 480, 320), -80)
  assert.equal(clampCropOffset(20, 320, 320), 0)
})

test('custom radius only applies to text and custom images', () => {
  assert.equal(resolveBallRadius('text', 24), 24)
  assert.equal(resolveBallRadius('image', 24), 24)
  assert.equal(resolveBallRadius('solid', 24), 50)
  assert.equal(resolveBallRadius('image', 80), 50)
})

test('data URL size uses decoded bytes', () => {
  assert.equal(dataUrlByteLength('data:image/png;base64,YWJj'), 3)
  assert.equal(dataUrlByteLength('data:image/png;base64,YWI='), 2)
})
