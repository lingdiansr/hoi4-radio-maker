<template>
  <div class="path-field">
    <v-text-field
      :model-value="modelValue"
      :label="label"
      :placeholder="placeholder"
      :prepend-inner-icon="prependInnerIcon"
      :rules="rules"
      variant="outlined"
      density="comfortable"
      class="path-input"
      hide-details="auto"
      @update:model-value="$emit('update:modelValue', $event)"
    >
      <template #append-inner>
        <v-btn
          variant="text"
          density="comfortable"
          size="small"
          :icon="pickerMode === 'file' ? 'mdi-file-search' : 'mdi-folder-open'"
          class="browse-btn"
          @click="browse"
        />
      </template>
    </v-text-field>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

interface Props {
  modelValue: string
  label: string
  placeholder?: string
  prependInnerIcon?: string
  pickerMode?: 'file' | 'directory'
  rules?: ((v: string) => true | string)[]
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: '',
  prependInnerIcon: 'mdi-folder-outline',
  pickerMode: 'directory',
  rules: () => [],
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

async function browse() {
  const selected = await open({
    directory: props.pickerMode === 'directory',
    multiple: false,
  })
  if (selected && typeof selected === 'string') {
    emit('update:modelValue', selected)
  }
}
</script>

<style scoped>
.path-field {
  display: flex;
  align-items: center;
  gap: 8px;
}

.path-input :deep(.v-field__append-inner) {
  padding-top: 0;
  padding-bottom: 0;
}

.browse-btn {
  color: #c4b5a0;
  transition: color 0.2s ease, background 0.2s ease;
}

.browse-btn:hover {
  color: #ffb020;
  background: rgba(255, 176, 32, 0.1);
}
</style>
