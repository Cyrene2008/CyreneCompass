<template>
  <div class="field">
    <span>{{ label }}</span>
    <FluentNumberInput
      :model-value="modelValue"
      :unit="unit"
      :min="min"
      :max="max"
      :step="step"
      @update:model-value="$emit('update:modelValue', $event)"
      @change="$emit('change', $event)"
    />
  </div>
</template>

<script setup>
import FluentNumberInput from './FluentNumberInput.vue'
defineProps({ modelValue: { type: Number, required: true }, label: { type: String, required: true }, unit: { type: String, default: '' }, min: { type: Number, required: true }, max: { type: Number, required: true }, step: { type: Number, default: 1 } })
defineEmits(['update:modelValue', 'change'])
</script>
