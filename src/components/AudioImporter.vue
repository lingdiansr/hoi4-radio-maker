<template>
  <div class="audio-importer">
    <v-btn
      color="primary"
      prepend-icon="mdi-music-note-plus"
      class="import-btn"
      :loading="audioStore.importing"
      @click="selectFiles"
    >
      {{ buttonLabel }}
    </v-btn>

    <div v-if="audioStore.importing" class="progress-wrap mt-3">
      <v-progress-linear color="primary" height="6" rounded indeterminate />
      <div class="text-mono text-caption text-secondary mt-1">正在导入…</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useAudioStore } from '@/stores/audio'

const props = withDefaults(defineProps<{
  mode?: 'global' | 'project'
  projectId?: string
}>(), {
  mode: 'global',
})

const emit = defineEmits<{
  (e: 'imported', result: { created: any[]; existing: any[] }): void
}>()

const audioStore = useAudioStore()

const buttonLabel = computed(() => {
  return props.mode === 'global' ? '导入到音频库' : '导入音频'
})

async function selectFiles() {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
      {
        name: 'Audio',
        extensions: ['mp3', 'flac', 'wav', 'ogg', 'm4a', 'aac', 'wma'],
      },
    ],
  })

  if (!selected || !Array.isArray(selected) || selected.length === 0) return

  if (props.mode === 'global') {
    const result = await audioStore.importGlobalBatch(selected)
    emit('imported', result)
  } else if (props.projectId) {
    const result = await audioStore.importBatch(props.projectId, selected)
    emit('imported', result)
  }
}
</script>

<style scoped>
.audio-importer {
  display: inline-flex;
  flex-direction: column;
}

.import-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.progress-wrap {
  width: 200px;
}
</style>
