import test from 'node:test'
import assert from 'node:assert/strict'
import { compareVersions, isInstaller } from '../src/updater.js'

test('compares release versions numerically', () => {
  assert.equal(compareVersions('v26.0.1', '26.0.0'), 1)
  assert.equal(compareVersions('26.0.0', 'v26.0.0'), 0)
  assert.equal(compareVersions('25.12.9', '26.0.0'), -1)
  assert.equal(compareVersions('26.1', '26.0.9'), 1)
})

test('selects only the public Windows x64 installer name', () => {
  assert.equal(isInstaller('CyreneCompass_26.1.0_x64-setup.exe'), true)
  assert.equal(isInstaller('CyreneCompass_26.1.0_x64-standard-setup.exe'), false)
  assert.equal(isInstaller('CyreneCompass_26.1.0_arm64-setup.exe'), false)
})
