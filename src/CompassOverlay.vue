<template>
  <div class="overlay-root" :class="{ dark: settings.dark }" :style="rootStyle" @contextmenu.prevent>
    <div v-if="visible" ref="stageRef" class="radial-stage">
      <div class="dial" :style="dialStyle">
        <span class="dial-outline"></span>
        <span class="dial-inner-line"></span>
        <span v-if="activeIndex >= 0" class="dial-glow" :style="glowStyle"></span>
        <button v-for="(item, index) in slotItems" :key="index" class="dial-item" :class="{ active: activeIndex === index, empty: !item.label, submenu: !!item.children?.length }" :style="slotStyle(index)" :aria-label="item.label || ''">
          <span class="dial-item-icon"><img v-if="item.iconMode === 'custom' && item.customIcon" :src="item.customIcon" alt="" /><img v-else-if="item.iconMode !== 'iconify' && item.systemIcon" :src="item.systemIcon" alt="" /><Icon v-else :icon="item.icon || 'fluent:app-generic-24-regular'" /></span>
          <Icon v-if="item.children?.length" class="dial-chevron" icon="fluent:chevron-right-16-regular" />
        </button>
        <div class="dial-center">
          <Icon class="dial-center-emblem" icon="fluent:compass-northwest-24-filled" />
          <span class="dial-center-label">{{ activeItemLabel }}</span>
        </div>
      </div>
      <div class="radial-hint">{{ activeIndex >= 0 ? t('releaseToLaunch') : t('moveToSelect') }}</div>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Icon, addCollection } from '@iconify/vue'
import { gsap } from 'gsap'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import fluentIcons from './fluent-icons.json'
import { createThemeVariables, DEFAULT_ACCENT } from './theme'
import { translate } from './i18n'
import { DIR_TO_INDEX, directionSlotItems } from './utils/compassMode'

addCollection(fluentIcons)

async function loadSettings() {
  let saved = null
  try { saved = await invoke('load_settings') } catch {}
  if (!saved) { try { saved = JSON.parse(localStorage.getItem('cyrene-compass-settings') || 'null') } catch {} }
  return { theme: 'peach', accent: DEFAULT_ACCENT, dark: false, language: 'zh', compassSize: 560, uiScale: 100, fontFamily: 'MiSans', items: [], ...(saved || {}) }
}

const settings = ref(loadSettings())
const items = computed(() => settings.value.items || [])
// 环上槽位 i（角度 i*45°，0=正上）显示该方向对应网格格位的选项
const slotItems = computed(() => directionSlotItems(items.value).map(item => item || { id: 'empty', label: '' }))
const visible = ref(false)
const activeIndex = ref(-1)
const stageRef = ref()
let closing = false
let pulseTween = null
const cleanups = []
const t = key => translate(settings.value.language, key)

const accent = computed(() => settings.value.theme === 'fluent' ? '#0078d4' : settings.value.theme === 'peach' ? DEFAULT_ACCENT : settings.value.accent)
const rootStyle = computed(() => ({ ...createThemeVariables(accent.value, settings.value.dark, settings.value.theme === 'fluent'), '--font-ui': `'${settings.value.fontFamily}', 'HarmonyOS', 'Segoe UI Variable', sans-serif` }))
// 外环直径（含图标），基于罗盘尺寸与视口安全上限
const radius = computed(() => { const base = settings.value.compassSize * 0.32 * (settings.value.uiScale / 100); const viewportCap = Math.min(window.innerWidth, window.innerHeight) * 0.36; return Math.min(310, Math.max(120, base), viewportCap) })
// 图标直径
const slotSize = computed(() => Math.min(84, Math.max(46, radius.value * 0.4)))
const dialStyle = computed(() => ({ '--dial-size': `${radius.value * 2}px`, '--slot-size': `${slotSize.value}px`, '--slot-offset': `${Math.round(radius.value * 0.78)}px`, '--glow-radius': `${Math.round(radius.value * 0.9)}px` }))
const slotStyle = index => ({ '--slot-angle': `${index * 45}deg` })
const glowStyle = computed(() => ({ '--glow-angle': `${activeIndex.value * 45}deg` }))
// 中央枢纽显示的当前选中应用名（未选中显示默认罗盘名）
const activeItem = computed(() => activeIndex.value >= 0 ? slotItems.value[activeIndex.value] : null)
const activeItemLabel = computed(() => activeItem.value?.label || t('dialCenter'))

async function openCompass() {
  if (closing) return
  settings.value = await loadSettings() // 每次展开重新读取最新配置
  activeIndex.value = -1
  visible.value = true
  nextTick(() => {
    if (!stageRef.value) return
    gsap.fromTo(stageRef.value, { opacity: 0, scale: 0.55 }, { opacity: 1, scale: 1, duration: 0.24, ease: 'back.out(1.4)', overwrite: 'auto' })
    const icons = stageRef.value.querySelectorAll('.dial-item-icon')
    gsap.fromTo(icons, { opacity: 0, scale: 0.3 }, { opacity: 1, scale: 1, duration: 0.3, stagger: 0.03, delay: 0.04, ease: 'back.out(2.4)', overwrite: 'auto' })
  })
}

// 后端方向 dir（0=上，顺时针）直接对应环上槽位 index；空位不高亮
function onDirection(dir) {
  const target = dir >= 0 && dir <= 7 && items.value[DIR_TO_INDEX[dir]]?.label ? dir : -1
  const changed = target !== activeIndex.value
  activeIndex.value = target
  if (changed && target >= 0 && stageRef.value) {
    const icon = stageRef.value.querySelectorAll('.dial-item-icon')[target]
    if (icon) { pulseTween?.kill(); pulseTween = gsap.fromTo(icon, { scale: 1.22 }, { scale: 1, duration: 0.2, ease: 'back.out(2)', overwrite: 'auto' }) }
  }
}

function closeCompass() {
  closing = true
  visible.value = false
  activeIndex.value = -1
  pulseTween?.kill()
  setTimeout(() => { closing = false }, 250)
}

async function onRelease(dir) {
  closeCompass()
  if (dir < 0 || dir > 7) return
  const item = items.value[DIR_TO_INDEX[dir]]
  if (!item || !item.label) return
  if (item.children?.length) {
    invoke('open_submenu_compass', { itemId: item.id }).catch(() => {})
    return
  }
  invoke('execute_action', { target: item.target || '', elevated: !!item.elevated, script: !!item.script }).catch(() => {})
}

onMounted(async () => {
  cleanups.push(await listen('compass-mode-open', () => openCompass()))
  cleanups.push(await listen('compass-mode-dir', e => onDirection(e.payload)))
  cleanups.push(await listen('compass-mode-release', e => onRelease(e.payload)))
})
onBeforeUnmount(() => { cleanups.forEach(fn => fn()) })
</script>
