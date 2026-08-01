<template>
  <div class="number-control">
    <button type="button" :disabled="modelValue <= min" @click="stepBy(-step)" aria-label="Decrease"><Icon icon="fluent:subtract-16-regular" /></button>
    <input :value="modelValue" type="number" :min="min" :max="max" :step="step" @change="commit($event.target.value)" @blur="commit($event.target.value)" />
    <span v-if="unit">{{ unit }}</span>
    <button type="button" :disabled="modelValue >= max" @click="stepBy(step)" aria-label="Increase"><Icon icon="fluent:add-16-regular" /></button>
  </div>
</template>

<script setup>
import { Icon } from '@iconify/vue'
const props = defineProps({ modelValue: { type: Number, required: true }, min: { type: Number, required: true }, max: { type: Number, required: true }, step: { type: Number, default: 1 }, unit: { type: String, default: '' } })
const emit = defineEmits(['update:modelValue', 'change'])
function setValue(value) { const next = Math.min(props.max, Math.max(props.min, Number(value) || props.min)); emit('update:modelValue', next); emit('change', next) }
function stepBy(delta) { setValue(props.modelValue + delta) }
function commit(value) { setValue(value) }
</script>

<style scoped>
.number-control { width: 176px; height: 34px; display: grid; grid-template-columns: 32px 1fr auto 32px; align-items: center; border: 1px solid var(--border-strong); border-radius: 6px; background: var(--bg-card-solid); overflow: hidden; }
.number-control:focus-within { border-color: var(--accent); box-shadow: inset 0 -2px var(--accent); }
.number-control button { width: 32px; height: 32px; display: grid; place-items: center; background: transparent; color: var(--text-secondary); }
.number-control button:hover:not(:disabled) { background: var(--bg-hover); color: var(--accent); }
.number-control button:disabled { opacity: .3; cursor: default; }
.number-control input { min-width: 0; width: 100%; border: 0; outline: 0; background: transparent; color: var(--text-primary); text-align: right; font: inherit; }
.number-control input::-webkit-inner-spin-button { display: none; }
.number-control span { padding-left: 4px; color: var(--text-muted); font-size: 12px; }
</style>
