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
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useAudioStore, type BatchImportResult } from '@/stores/audio'
import { invokeCommand } from '@/api/client'
import { logger } from '@/utils/logger'

const props = withDefaults(defineProps<{
  mode?: 'global' | 'project'
  projectId?: string
}>(), {
  mode: 'global',
})

const emit = defineEmits<{
  (e: 'imported', result: BatchImportResult): void
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

  if (!selected || !Array.isArray(selected) || selected.length === 0) {
    logger.info('audio importer: no files selected')
    return
  }

  logger.info(`audio importer: selected ${selected.length} file(s), mode=${props.mode}`)

  audioStore.importing = true

  try {
    const args: Record<string, unknown> = {
      paths: selected,
    }
    if (props.mode === 'project' && props.projectId) {
      args.projectId = props.projectId
    }

    const res = await invokeCommand<BatchImportResult>('import_audio_batch', args)
    if (!res) return

    logger.info(
      `audio importer: import finished, created=${res.created.length} existing=${res.existing.length} failed=${res.failed.length}`
    )

    if (props.mode === 'global') {
      await audioStore.loadAllAudio()
    } else if (props.projectId) {
      await audioStore.loadAudio(props.projectId)
    }

    emit('imported', res)
  } catch (err) {
    logger.error(`audio importer: import failed: ${JSON.stringify(err)}`)
  } finally {
    audioStore.importing = false
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
</style>
