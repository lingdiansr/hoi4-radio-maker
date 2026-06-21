<template>
  <v-slide-y-reverse-transition>
    <v-card
      v-if="progress.visible"
      class="import-progress-panel"
      variant="flat"
      rounded="lg"
    >
      <div class="panel-accent" />

      <v-card-title class="panel-header pa-4 pb-2">
        <div class="d-flex align-center gap-3 w-100">
          <v-icon color="primary" size="24">mdi-import</v-icon>
          <div class="flex-grow-1">
            <div class="text-mono text-caption text-secondary">IMPORT PROGRESS</div>
            <div class="text-body font-weight-medium">导入音频</div>
          </div>
          <div class="text-mono text-caption text-secondary">
            {{ completedCount }} / {{ progress.files.length }}
          </div>
          <v-btn
            v-if="!progress.isActive"
            icon="mdi-close"
            variant="text"
            density="comfortable"
            size="small"
            @click="progress.dismiss"
          />
        </div>
      </v-card-title>

      <v-card-text class="pa-4 pt-2">
        <div v-if="progress.phase" class="phase-row d-flex align-center justify-space-between mb-2">
          <div class="text-mono text-caption text-secondary">{{ phaseLabel }}</div>
          <div class="text-mono text-caption">
            {{ progress.phaseCurrent }} / {{ progress.phaseTotal }}
          </div>
        </div>

        <v-progress-linear
          :model-value="overallProgress"
          color="primary"
          height="6"
          rounded
          class="mb-3"
          :indeterminate="progress.isActive && overallProgress === 0"
        />

        <div class="file-list">
          <div
            v-for="file in visibleFiles"
            :key="file.path"
            class="file-item d-flex align-center gap-3 py-1"
          >
            <v-icon :color="statusMeta(file.status).color" size="18">
              {{ statusMeta(file.status).icon }}
            </v-icon>
            <div class="file-name text-body-2 text-truncate flex-grow-1">
              {{ file.name }}
            </div>
            <v-chip
              :color="statusMeta(file.status).color"
              variant="tonal"
              size="x-small"
              class="status-chip"
            >
              {{ statusMeta(file.status).label }}
            </v-chip>
          </div>
        </div>

        <div v-if="failedFiles.length > 0" class="failed-section mt-3">
          <div class="text-mono text-caption text-error mb-1">FAILED</div>
          <div
            v-for="file in failedFiles"
            :key="file.path"
            class="failed-item text-caption"
          >
            <v-icon size="14" color="error" class="mr-1">mdi-alert-circle</v-icon>
            {{ file.name }}: {{ file.message }}
          </div>
        </div>
      </v-card-text>

      <v-divider opacity="0.2" />

      <v-card-actions class="pa-4">
        <div v-if="progress.result" class="result-summary text-body-2 d-flex align-center gap-3 flex-wrap">
          <span v-if="progress.cancelled" class="text-warning">已取消</span>
          <span class="text-success">新增 {{ progress.result.created.length }}</span>
          <span class="text-secondary">·</span>
          <span class="text-secondary">已存在 {{ progress.result.existing.length }}</span>
          <span v-if="progress.result.failed.length > 0" class="text-secondary">·</span>
          <span v-if="progress.result.failed.length > 0" class="text-error">
            失败 {{ progress.result.failed.length }}
          </span>
          <span v-if="progress.cancelled" class="text-caption text-secondary">已导入的文件已保留</span>
        </div>
        <v-spacer />
        <v-btn
          v-if="progress.isActive"
          color="error"
          variant="text"
          size="small"
          class="action-btn"
          :loading="cancelling"
          @click="onCancel"
        >
          取消导入
        </v-btn>
        <v-btn
          v-else
          color="primary"
          variant="text"
          size="small"
          class="action-btn"
          @click="progress.dismiss"
        >
          完成
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-slide-y-reverse-transition>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useImportProgressStore, type ImportFileStatusType } from '@/stores/importProgress'

const progress = useImportProgressStore()
const cancelling = ref(false)

const phaseLabels: Record<string, string> = {
  hashing: '计算哈希',
  checking: '查重比对',
  transcoding: '转码压缩',
  persisting: '写入数据库',
}

const phaseLabel = computed(() => {
  return progress.phase ? phaseLabels[progress.phase] || progress.phase : ''
})

const completedCount = computed(() => {
  return progress.files.filter((f) =>
    ['existing', 'created', 'error'].includes(f.status)
  ).length
})

const overallProgress = computed(() => {
  if (progress.files.length === 0) return 0
  return (completedCount.value / progress.files.length) * 100
})

const visibleFiles = computed(() => {
  // Show active / recently finished files first, then pending.
  return [...progress.files].sort((a, b) => {
    const order: Record<ImportFileStatusType, number> = {
      error: 0,
      transcoding: 1,
      hashing: 2,
      created: 3,
      existing: 4,
      pending: 5,
    }
    return order[a.status] - order[b.status]
  })
})

const failedFiles = computed(() => {
  return progress.files.filter((f) => f.status === 'error')
})

function statusMeta(status: ImportFileStatusType) {
  switch (status) {
    case 'pending':
      return { icon: 'mdi-clock-outline', color: 'secondary', label: '等待中' }
    case 'hashing':
      return { icon: 'mdi-fingerprint', color: 'primary', label: '哈希' }
    case 'existing':
      return { icon: 'mdi-check-circle', color: 'secondary', label: '已存在' }
    case 'transcoding':
      return { icon: 'mdi-cog-outline', color: 'primary', label: '转码中' }
    case 'created':
      return { icon: 'mdi-check-circle', color: 'success', label: '已导入' }
    case 'error':
      return { icon: 'mdi-alert-circle', color: 'error', label: '失败' }
    default:
      return { icon: 'mdi-help-circle', color: 'secondary', label: status }
  }
}

async function onCancel() {
  cancelling.value = true
  try {
    await progress.cancel()
  } finally {
    cancelling.value = false
  }
}
</script>

<style scoped>
.import-progress-panel {
  position: fixed;
  right: 24px;
  bottom: 24px;
  width: 420px;
  max-width: calc(100vw - 48px);
  background: #1a1714;
  border: 1px solid rgba(74, 66, 56, 0.5);
  overflow: hidden;
  z-index: 2000;
}

.panel-accent {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #ffb020 0%, rgba(255, 176, 32, 0.3) 100%);
}

.panel-header {
  padding-top: 20px;
}

.file-list {
  max-height: 220px;
  overflow-y: auto;
}

.file-item {
  min-height: 28px;
}

.file-name {
  opacity: 0.9;
}

.status-chip {
  font-family: var(--font-mono);
  letter-spacing: 0.02em;
}

.failed-section {
  border-top: 1px solid rgba(244, 67, 54, 0.2);
  padding-top: 8px;
}

.failed-item {
  color: #ef9a9a;
  word-break: break-all;
}

.result-summary {
  font-family: var(--font-mono);
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

@media (max-width: 600px) {
  .import-progress-panel {
    right: 12px;
    bottom: 12px;
    width: calc(100vw - 24px);
  }
}
</style>
