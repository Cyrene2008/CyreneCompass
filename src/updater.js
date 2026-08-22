import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export const CURRENT_VERSION = '26.0.2'

export const updateState = ref({
  checked: false,
  checking: false,
  available: false,
  downloading: false,
  progress: 0,
  version: '',
  url: '',
  fileName: '',
  fileSize: 0,
  releaseUrl: '',
  errorKey: '',
  error: ''
})

const normalizeVersion = value => String(value || '').replace(/^v/i, '').trim()

export function isInstaller(name) {
  return /^cyrenecompass_.+_x64-setup\.exe$/i.test(String(name || ''))
}

export function compareVersions(left, right) {
  const a = normalizeVersion(left).split('.').map(Number)
  const b = normalizeVersion(right).split('.').map(Number)
  for (let index = 0; index < Math.max(a.length, b.length); index += 1) {
    const difference = (a[index] || 0) - (b[index] || 0)
    if (difference) return difference > 0 ? 1 : -1
  }
  return 0
}

export async function checkForUpdates(silent = false) {
  if (updateState.value.checking || updateState.value.downloading) return
  updateState.value.checking = true
  updateState.value.errorKey = ''
  updateState.value.error = ''
  try {
    const release = await invoke('check_update')
    const version = normalizeVersion(release?.tag_name)
    const asset = (release?.assets || []).find(item => isInstaller(item?.name))
    const available = compareVersions(version, CURRENT_VERSION) > 0
    updateState.value = {
      ...updateState.value,
      checked: true,
      checking: false,
      available,
      version,
      url: asset?.browser_download_url || '',
      fileName: asset?.name || '',
      fileSize: Number(asset?.size) || 0,
      releaseUrl: release?.html_url || '',
      errorKey: available && !asset ? 'missingInstaller' : '',
      error: ''
    }
  } catch (error) {
    updateState.value.checked = true
    updateState.value.checking = false
    if (!silent) updateState.value.error = String(error)
  }
}

export async function downloadUpdate() {
  if (!updateState.value.available || !updateState.value.url || updateState.value.downloading) return
  updateState.value.downloading = true
  updateState.value.progress = 0
  updateState.value.errorKey = ''
  updateState.value.error = ''
  let removeProgress
  try {
    removeProgress = await listen('update-download-progress', event => {
      updateState.value.progress = Math.max(0, Math.min(100, Number(event.payload) || 0))
    })
    await invoke('download_and_launch_update', {
      url: updateState.value.url,
      fileName: updateState.value.fileName,
      expectedSize: updateState.value.fileSize
    })
    updateState.value.progress = 100
  } catch (error) {
    updateState.value.error = String(error)
    updateState.value.downloading = false
  } finally {
    if (typeof removeProgress === 'function') removeProgress()
  }
}
