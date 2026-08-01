import test from 'node:test'
import assert from 'node:assert/strict'
import { compareVersions } from '../src/updater.js'

test('compares release versions numerically', () => {
  assert.equal(compareVersions('v26.0.1', '26.0.0'), 1)
  assert.equal(compareVersions('26.0.0', 'v26.0.0'), 0)
  assert.equal(compareVersions('25.12.9', '26.0.0'), -1)
  assert.equal(compareVersions('26.1', '26.0.9'), 1)
})
