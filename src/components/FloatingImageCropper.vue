<template>
  <div class="modal-layer" @pointerdown.self="$emit('cancel')">
    <div class="modal cropper-modal" role="dialog" aria-modal="true" :aria-label="title">
      <h2>{{ title }}</h2>
      <div
        ref="viewportRef"
        class="cropper-viewport"
        @pointerdown="startDrag"
        @pointermove="drag"
        @pointerup="endDrag"
        @pointercancel="endDrag"
        @wheel.prevent="onWheel"
      >
        <img
          ref="imageRef"
          :src="source"
          alt=""
          draggable="false"
          :style="imageStyle"
          @load="onImageLoad"
        />
      </div>
      <label class="cropper-zoom">
        <span>{{ zoomLabel }}</span>
        <input v-model.number="zoom" type="range" min="1" max="4" step="0.01" @input="clampPosition" />
      </label>
      <div class="modal-actions">
        <button class="secondary-btn" @click="$emit('cancel')">{{ cancelLabel }}</button>
        <button class="primary-btn" :disabled="!loaded" @click="confirmCrop">{{ confirmLabel }}</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, ref } from 'vue'
import { clampCropOffset, coverScale } from '../utils/floatingBallStyle'

const props = defineProps({
  source: { type: String, required: true },
  title: { type: String, required: true },
  zoomLabel: { type: String, required: true },
  cancelLabel: { type: String, required: true },
  confirmLabel: { type: String, required: true }
})
const emit = defineEmits(['cancel', 'confirm'])
const viewportRef = ref()
const imageRef = ref()
const imageWidth = ref(0)
const imageHeight = ref(0)
const viewportSize = ref(320)
const zoom = ref(1)
const offsetX = ref(0)
const offsetY = ref(0)
const loaded = computed(() => imageWidth.value > 0 && imageHeight.value > 0)
const baseScale = computed(() => coverScale(imageWidth.value, imageHeight.value, viewportSize.value))
const displayWidth = computed(() => imageWidth.value * baseScale.value * zoom.value)
const displayHeight = computed(() => imageHeight.value * baseScale.value * zoom.value)
const imageStyle = computed(() => ({
  width: `${displayWidth.value}px`,
  height: `${displayHeight.value}px`,
  transform: `translate(calc(-50% + ${offsetX.value}px), calc(-50% + ${offsetY.value}px))`
}))
let pointer

function clampPosition() {
  offsetX.value = clampCropOffset(offsetX.value, displayWidth.value, viewportSize.value)
  offsetY.value = clampCropOffset(offsetY.value, displayHeight.value, viewportSize.value)
}
function onImageLoad() {
  imageWidth.value = imageRef.value.naturalWidth
  imageHeight.value = imageRef.value.naturalHeight
  viewportSize.value = viewportRef.value.clientWidth
  zoom.value = 1
  offsetX.value = 0
  offsetY.value = 0
}
function startDrag(event) {
  if (event.pointerType === 'mouse' && event.button !== 0) return
  event.currentTarget.setPointerCapture(event.pointerId)
  pointer = { id: event.pointerId, x: event.clientX, y: event.clientY }
}
function drag(event) {
  if (!pointer || pointer.id !== event.pointerId) return
  offsetX.value += event.clientX - pointer.x
  offsetY.value += event.clientY - pointer.y
  pointer.x = event.clientX
  pointer.y = event.clientY
  clampPosition()
}
function endDrag(event) {
  if (!pointer || pointer.id !== event.pointerId) return
  pointer = null
  if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId)
}
function onWheel(event) {
  zoom.value = Math.max(1, Math.min(4, zoom.value - event.deltaY * 0.0015))
  clampPosition()
}
function confirmCrop() {
  const outputSize = 512
  const factor = outputSize / viewportSize.value
  const canvas = document.createElement('canvas')
  canvas.width = outputSize
  canvas.height = outputSize
  const context = canvas.getContext('2d')
  context.imageSmoothingEnabled = true
  context.imageSmoothingQuality = 'high'
  context.drawImage(
    imageRef.value,
    outputSize / 2 + (offsetX.value - displayWidth.value / 2) * factor,
    outputSize / 2 + (offsetY.value - displayHeight.value / 2) * factor,
    displayWidth.value * factor,
    displayHeight.value * factor
  )
  emit('confirm', canvas.toDataURL('image/png'))
}
</script>
