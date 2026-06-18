<template>
  <div class="audio-importer">
    <v-btn
      color="primary"
      prepend-icon="mdi-music-note-plus"
      class="import-btn"
      :loading="audioStore.importing"
      @click="selectFiles"
    >
      导入音频
    </v-btn>

    <div v-if="audioStore.importProgress" class="progress-wrap mt-4">
      <v-progress-linear
        :model-value="progressPercent"
        color="primary"
        height="8"
        rounded
      />
      <div class="text-mono text-caption text-secondary mt-1">
        处理中 {{ audioStore.importProgress.processed }} / {{ audioStore.importProgress.total }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useAudioStore } from '@/stores/audio'

const props = defineProps<{
  projectId: string
}>()

const audioStore = useAudioStore()

const progressPercent = computed(() => {
  const p = audioStore.importProgress
  if (!p || p.total === 0) return 0
  return (p.processed / p.total) * 100
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

  await audioStore.importBatch(props.projectId, selected)
}
</script>

<style scoped>
.audio-importer {
  display: inline-flex;
  flex-direction: column;
}

.import-btn {
  text-transform: none;
}

.progress-wrap {
  width: 240px;
}
</style>
