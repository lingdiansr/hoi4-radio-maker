<template>
  <v-btn prepend-icon="mdi-music-note-plus" @click="selectFiles">
    导入音频
  </v-btn>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits<{
  (e: 'import', paths: string[]): void
}>()

async function selectFiles() {
  const selected = await open({
    multiple: true,
    filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg'] }],
  })
  if (selected && Array.isArray(selected)) {
    emit('import', selected)
  }
}
</script>
