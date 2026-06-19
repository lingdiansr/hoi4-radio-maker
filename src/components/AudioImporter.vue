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

    <v-expand-transition>
      <div v-if="audioStore.importing" class="progress-wrap mt-3">
        <v-progress-linear color="primary" height="6" rounded indeterminate />
        <div class="text-mono text-caption text-secondary mt-1">正在导入…</div>
      </div>
    </v-expand-transition>

    <v-expand-transition>
      <div v-if="result" class="result-wrap mt-3">
        <v-alert
          type="success"
          variant="tonal"
          density="compact"
          rounded="lg"
          class="result-alert"
          :icon="false"
        >
          <div class="d-flex align-center gap-3">
            <v-icon color="success">mdi-check-circle</v-icon>
            <div>
              <div class="text-body font-weight-medium">导入完成</div>
              <div class="text-mono text-caption text-secondary">
                新增 {{ result.created.length }} 首 · 已存在 {{ result.existing.length }} 首
              </div>
            </div>
            <v-spacer />
            <v-btn
              icon="mdi-close"
              variant="text"
              size="small"
              density="compact"
              @click="result = null"
            />
          </div>
        </v-alert>
      </div>
    </v-expand-transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useAudioStore } from '@/stores/audio'
import { logger } from '@/utils/logger'
import type { BatchImportResult } from '@/stores/audio'

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
const result = ref<BatchImportResult | null>(null)

const buttonLabel = computed(() => {
  return props.mode === 'global' ? '导入到音频库' : '导入音频'
})

async function selectFiles() {
  result.value = null
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

  try {
    let res: BatchImportResult
    if (props.mode === 'global') {
      res = await audioStore.importGlobalBatch(selected)
    } else if (props.projectId) {
      res = await audioStore.importBatch(props.projectId, selected)
    } else {
      logger.warn('audio importer: project mode selected but no projectId provided')
      return
    }
    result.value = res
    logger.info(
      `audio importer: import finished, created=${res.created.length} existing=${res.existing.length}`
    )
    emit('imported', res)
  } catch (err) {
    logger.error(`audio importer: import failed: ${err}`)
    result.value = null
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
  width: 220px;
}

.result-wrap {
  width: 280px;
}

.result-alert {
  background: rgba(76, 175, 80, 0.1) !important;
  border: 1px solid rgba(76, 175, 80, 0.3);
}
</style>
