import test from 'node:test'
import assert from 'node:assert/strict'
import { compareVersions, isInstallerForBuild } from '../src/updater.js'

test('compares release versions numerically', () => {
  assert.equal(compareVersions('v26.0.1', '26.0.0'), 1)
  assert.equal(compareVersions('26.0.0', 'v26.0.0'), 0)
  assert.equal(compareVersions('25.12.9', '26.0.0'), -1)
  assert.equal(compareVersions('26.1', '26.0.9'), 1)
})

test('selects an installer matching the current build edition', () => {
  assert.equal(isInstallerForBuild('CyreneCompass_26.1.0_x64-setup.exe', 'uiaccess'), true)
  assert.equal(isInstallerForBuild('CyreneCompass_26.1.0_x64-standard-setup.exe', 'uiaccess'), false)
  assert.equal(isInstallerForBuild('CyreneCompass_26.1.0_x64-standard-setup.exe', 'standard'), true)
  assert.equal(isInstallerForBuild('CyreneCompass_26.1.0_x64-setup.exe', 'standard'), false)
})
