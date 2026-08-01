<template>
  <button
    type="button"
    class="fluent-switch"
    :class="{ checked: modelValue, disabled }"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    @click="$emit('update:modelValue', !modelValue); $emit('change', !modelValue)"
  >
    <span class="switch-track"><span class="switch-thumb" /></span>
  </button>
</template>

<script setup>
defineProps({ modelValue: { type: Boolean, default: false }, disabled: { type: Boolean, default: false } })
defineEmits(['update:modelValue', 'change'])
</script>

<style scoped>
.fluent-switch { width: 42px; height: 24px; padding: 0; background: transparent; border: 0; flex: 0 0 auto; }
.switch-track { position: relative; display: block; width: 40px; height: 20px; border: 1px solid var(--text-muted); border-radius: 999px; background: transparent; transition: background .16s ease, border-color .16s ease; }
.switch-thumb { position: absolute; left: 3px; top: 3px; width: 12px; height: 12px; border-radius: 50%; background: var(--text-muted); transition: transform .18s cubic-bezier(.1,.9,.2,1), background .16s ease; }
.fluent-switch.checked .switch-track { background: var(--accent); border-color: var(--accent); }
.fluent-switch.checked .switch-thumb { transform: translateX(20px); background: #fff; }
.fluent-switch:hover .switch-track { border-color: var(--accent); }
.fluent-switch:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; border-radius: 999px; }
.fluent-switch.disabled { opacity: .45; cursor: default; }
</style>
