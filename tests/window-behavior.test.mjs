import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'

const appSource = readFileSync(new URL('../src/App.vue', import.meta.url), 'utf8')
const styleSource = readFileSync(new URL('../src/styles.css', import.meta.url), 'utf8')
const nativeSource = readFileSync(new URL('../src-tauri/src/lib.rs', import.meta.url), 'utf8')

test('compass mode switches do not animate native window dimensions', () => {
  const resizeBody = appSource.match(/async function resizeWindow\(next\)\{([\s\S]*?)\nfunction openCompass/)?.[1]
  assert.ok(resizeBody)
  assert.doesNotMatch(resizeBody, /animate:true/)
  assert.match(resizeBody, /animate:false/)
  assert.doesNotMatch(nativeSource, /for frame in 1\.\.=6/)
})

test('touch surfaces suppress browser boundary overscroll without disabling settings scrolling', () => {
  assert.match(styleSource, /html, body, #app, \.app-shell\s*\{[^}]*overscroll-behavior:\s*none/)
  assert.match(styleSource, /\.settings-content\s*\{[^}]*overscroll-behavior:\s*none/)
  assert.match(styleSource, /\.icon-grid\s*\{[^}]*overscroll-behavior:\s*none/)
  assert.match(styleSource, /\.compass-stage\s*\{[^}]*touch-action:\s*none[^}]*overscroll-behavior:\s*none/)
  assert.match(styleSource, /\.settings-content\s*\{[^}]*overflow:\s*auto/)
})

test('overlay windows remain tool windows and are restored after Show Desktop', () => {
  assert.match(nativeSource, /WS_EX_APPWINDOW/)
  assert.match(nativeSource, /IsIconic/)
  assert.match(nativeSource, /IsWindowVisible/)
  assert.match(nativeSource, /SW_SHOWNA/)
})
