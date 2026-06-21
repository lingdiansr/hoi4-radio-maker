<template>
  <div class="archive-view">
    <v-card class="archive-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-start pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">GLOBAL ARCHIVE</div>
          <div class="text-display text-h4 archive-title">音频库</div>
          <div class="text-body text-secondary mt-2">
            共 {{ audioStore.allAudioFiles.length }} 条音频 · 已选择 {{ selectedIds.length }} 条
          </div>
        </div>
        <div class="d-flex align-center gap-3">
          <v-btn-toggle v-model="viewMode" density="comfortable" variant="outlined" divided>
            <v-btn value="grid" icon="mdi-view-grid" />
            <v-btn value="list" icon="mdi-view-list" />
          </v-btn-toggle>
          <AudioImporter mode="global" @imported="onImported" />
        </div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6 toolbar-area">
        <v-row>
          <v-col cols="12" md="5">
            <v-text-field
              v-model="search"
              label="搜索音频"
              placeholder="标题、艺术家、哈希…"
              prepend-inner-icon="mdi-magnify"
              variant="outlined"
              density="comfortable"
              hide-details
              class="search-field"
            />
            <v-chip-group
              v-if="allTags.length"
              v-model="selectedTag"
              class="tag-filter mt-3"
            >
              <v-chip
                v-for="tag in allTags"
                :key="tag"
                :value="tag"
                filter
                variant="outlined"
                size="small"
              >
                {{ tag }}
              </v-chip>
            </v-chip-group>
          </v-col>
          <v-col cols="12" md="7" class="d-flex align-center justify-end gap-3 flex-wrap">
            <div class="d-flex align-center gap-3 flex-wrap">
              <v-btn
                variant="text"
                size="small"
                prepend-icon="mdi-select-all"
                @click="selectAll"
              >
                全选
              </v-btn>
              <v-btn
                variant="text"
                size="small"
                prepend-icon="mdi-select-inverse"
                @click="invertSelection"
              >
                反选
              </v-btn>
              <v-btn
                v-if="selectedIds.length"
                variant="text"
                size="small"
                prepend-icon="mdi-select-remove"
                @click="selectedIds = []"
              >
                清除选择
              </v-btn>
            </div>
            <v-divider v-if="selectedIds.length" vertical class="mx-2" />
            <div v-if="selectedIds.length" class="d-flex align-center gap-3 flex-wrap">
              <v-btn
                v-if="selectedIds.length > 1"
                color="primary"
                variant="tonal"
                size="small"
                prepend-icon="mdi-pencil-box-multiple"
                @click="showBatchEdit = true"
              >
                批量编辑 ({{ selectedIds.length }})
              </v-btn>
              <v-btn
                color="error"
                variant="tonal"
                size="small"
                prepend-icon="mdi-delete-sweep"
                @click="confirmBatchDelete"
              >
                删除选中 ({{ selectedIds.length }})
              </v-btn>
            </div>
          </v-col>
        </v-row>
      </v-card-text>

      <div class="scroll-area">
        <v-card-text class="pa-6 pt-0">
          <!-- Empty state -->
          <div v-if="filteredAudio.length === 0" class="empty-state text-center py-16">
            <v-icon size="80" color="secondary" class="mb-6">mdi-archive-music-outline</v-icon>
            <div class="text-display text-h5 mb-2">音频库为空</div>
            <div class="text-body text-secondary mb-6">
              这里是全局音频仓库。导入音频后，可在任意项目中引用。
            </div>
            <AudioImporter mode="global" @imported="onImported" />
          </div>

          <!-- Grid view -->
          <v-row v-else-if="viewMode === 'grid'">
            <v-col
              v-for="audio in filteredAudio"
              :key="audio.id"
              cols="12"
              sm="6"
              lg="4"
              xl="3"
            >
              <v-card
                class="audio-item"
                :class="{ selected: isSelected(audio.id), processing: audio.import_status === 'processing' }"
                variant="flat"
                rounded="lg"
                @click="toggleSelect(audio.id)"
              >
                <v-progress-linear
                  v-if="audio.import_status === 'processing'"
                  indeterminate
                  color="primary"
                  height="3"
                  class="audio-progress"
                />
                <div class="select-indicator">
                  <v-checkbox
                    :model-value="isSelected(audio.id)"
                    hide-details
                    density="compact"
                    @click.stop
                    @update:model-value="setSelected(audio.id, $event)"
                  />
                </div>
                <div
                  v-if="audio.import_status === 'processing'"
                  class="audio-wave processing-wave"
                  aria-hidden="true"
                >
                  <div
                    v-for="i in 12"
                    :key="i"
                    class="wave-bar"
                    :style="{ height: waveHeight(audio.id, i), animationDelay: `${i * 80}ms` }"
                  />
                </div>
                <div v-else class="audio-wave" aria-hidden="true">
                  <div
                    v-for="i in 12"
                    :key="i"
                    class="wave-bar"
                    :style="{ height: waveHeight(audio.id, i) }"
                  />
                </div>
                <v-card-text class="pa-4 audio-content">
                  <div class="d-flex justify-space-between align-start mb-3">
                    <v-icon
                      :color="statusColor(audio.import_status)"
                      size="32"
                    >
                      {{ statusIcon(audio.import_status) }}
                    </v-icon>
                    <div class="d-flex gap-1">
                      <v-btn
                        icon="mdi-pencil"
                        variant="text"
                        size="small"
                        color="primary"
                        :disabled="audio.import_status !== 'ready'"
                        @click.stop="openEdit(audio)"
                      />
                      <v-btn
                        icon="mdi-delete-outline"
                        variant="text"
                        size="small"
                        color="error"
                        @click.stop="confirmDelete(audio)"
                      />
                    </div>
                  </div>
                  <div class="text-body text-subtitle-1 font-weight-medium text-truncate mb-1">
                    {{ audio.title }}
                  </div>
                  <div class="text-mono text-caption text-secondary mb-3">
                    <span>{{ audio.artist || '未知艺术家' }}</span>
                    <span v-if="audio.import_status === 'ready'">
                      · {{ formatDuration(audio.duration_secs) }}
                    </span>
                    <span v-else-if="audio.import_status === 'pending'"> · 等待处理…</span>
                    <span v-else-if="audio.import_status === 'processing'"> · 正在导入…</span>
                    <span v-else-if="audio.import_status === 'error'"> · 导入失败</span>
                  </div>
                  <div class="d-flex justify-space-between align-center text-mono text-caption text-secondary">
                    <span v-if="audio.import_status === 'ready'">
                      {{ audio.sample_rate }} Hz · {{ audio.channels }} ch
                    </span>
                    <span v-else>—</span>
                    <v-chip
                      :color="statusColor(audio.import_status)"
                      size="x-small"
                      variant="tonal"
                      class="status-chip"
                    >
                      {{ statusLabel(audio.import_status) }}
                    </v-chip>
                  </div>
                </v-card-text>
              </v-card>
            </v-col>
          </v-row>

          <!-- List view -->
          <v-list v-else bg-color="transparent" class="audio-list">
            <v-list-item
              v-for="audio in filteredAudio"
              :key="audio.id"
              class="audio-list-item mb-2 position-relative"
              :class="{ selected: isSelected(audio.id) }"
              rounded="lg"
              @click="toggleSelect(audio.id)"
            >
              <v-progress-linear
                v-if="audio.import_status === 'pending' || audio.import_status === 'processing'"
                :indeterminate="audio.import_status === 'processing'"
                :model-value="audio.import_status === 'pending' ? 0 : undefined"
                :color="statusColor(audio.import_status)"
                height="2"
                class="list-progress"
              />
              <template #prepend>
                <v-checkbox
                  :model-value="isSelected(audio.id)"
                  hide-details
                  density="compact"
                  class="mr-3"
                  @click.stop
                  @update:model-value="setSelected(audio.id, $event)"
                />
                <v-icon :color="statusColor(audio.import_status)" class="mr-3">
                  {{ statusIcon(audio.import_status) }}
                </v-icon>
              </template>
              <v-list-item-title>
                {{ audio.title }}
                <v-chip
                  :color="statusColor(audio.import_status)"
                  size="x-small"
                  variant="tonal"
                  class="ml-2 status-chip"
                >
                  {{ statusLabel(audio.import_status) }}
                </v-chip>
              </v-list-item-title>
              <v-list-item-subtitle>
                <span>{{ audio.artist || '未知艺术家' }}</span>
                <span v-if="audio.import_status === 'ready'">
                  · {{ formatDuration(audio.duration_secs) }}
                  · {{ audio.sample_rate }} Hz · {{ audio.channels }} ch
                </span>
                <span v-else-if="audio.import_status === 'pending'"> · 等待处理…</span>
                <span v-else-if="audio.import_status === 'processing'"> · 正在导入…</span>
                <span v-else-if="audio.import_status === 'error'"> · 导入失败</span>
              </v-list-item-subtitle>
              <template #append>
                <v-btn
                  icon="mdi-pencil"
                  variant="text"
                  size="small"
                  color="primary"
                  class="mr-1"
                  :disabled="audio.import_status !== 'ready'"
                  @click.stop="openEdit(audio)"
                />
                <v-btn
                  icon="mdi-delete-outline"
                  variant="text"
                  size="small"
                  color="error"
                  @click.stop="confirmDelete(audio)"
                />
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
      </div>
    </v-card>

    <!-- Edit Dialog -->
    <AudioEditDialog
      v-model="showEditDialog"
      :audio="audioToEdit"
      @saved="onImported"
    />

    <!-- Batch Edit Dialog -->
    <BatchAudioEditDialog
      v-model="showBatchEdit"
      :ids="selectedIds"
      @saved="onImported"
    />

    <!-- Batch Delete Confirmation Dialog -->
    <v-dialog v-model="showBatchDeleteDialog" max-width="460" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent dialog-accent--danger" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="error" size="28">mdi-alert-circle</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">BATCH DELETION</div>
              <div class="text-display text-h5">批量删除音频</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          确定要从全局音频库中删除选中的 <strong class="text-primary">{{ selectedIds.length }}</strong> 条音频吗？
          <br><br>
          <span class="text-secondary">处理中的音频将被取消导入并删除。</span>
          <br>
          <span class="text-error">警告：</span>已被项目引用的就绪音频无法删除。
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showBatchDeleteDialog = false">取消</v-btn>
          <v-btn color="error" class="action-btn" prepend-icon="mdi-delete-outline" :loading="batchDeleting" @click="handleBatchDelete">
            删除
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="showDeleteDialog" max-width="420" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent dialog-accent--danger" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="error" size="28">mdi-alert-circle</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">CONFIRM DELETION</div>
              <div class="text-display text-h5">删除音频</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          确定要从全局音频库中删除 <strong class="text-primary">{{ audioToDelete?.title }}</strong> 吗？
          <br><br>
          <span v-if="audioToDelete?.import_status === 'processing'" class="text-secondary">该音频正在导入，删除后将取消导入。</span>
          <br>
          <span class="text-error">警告：</span>如果该音频仍被任何项目引用，将无法删除。
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showDeleteDialog = false">取消</v-btn>
          <v-btn color="error" class="action-btn" prepend-icon="mdi-delete-outline" @click="handleDelete">
            删除
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useAudioStore, type AudioFile } from '@/stores/audio'
import { useToastStore } from '@/stores/toast'
import AudioImporter from '@/components/AudioImporter.vue'
import AudioEditDialog from '@/components/AudioEditDialog.vue'
import BatchAudioEditDialog from '@/components/BatchAudioEditDialog.vue'

const audioStore = useAudioStore()
const toast = useToastStore()
const search = ref('')
const selectedTag = ref<string | null>(null)
const showDeleteDialog = ref(false)
const showBatchDeleteDialog = ref(false)
const showEditDialog = ref(false)
const showBatchEdit = ref(false)
const audioToDelete = ref<AudioFile | null>(null)
const audioToEdit = ref<AudioFile | null>(null)
const viewMode = ref<'grid' | 'list'>('grid')
const selectedIds = ref<string[]>([])
const batchDeleting = ref(false)

onMounted(() => {
  audioStore.loadAllAudio()
  audioStore.ensureListening()
})

const allTags = computed(() => {
  const tags = new Set<string>()
  audioStore.allAudioFiles.forEach((a) => a.tags.forEach((t) => tags.add(t)))
  return Array.from(tags).slice(0, 8)
})

const filteredAudio = computed(() => {
  let list = audioStore.allAudioFiles
  const q = search.value.trim().toLowerCase()
  if (q) {
    list = list.filter(
      (a) =>
        a.title.toLowerCase().includes(q) ||
        (a.artist?.toLowerCase().includes(q) ?? false) ||
        a.source_hash.toLowerCase().includes(q)
    )
  }
  if (selectedTag.value) {
    list = list.filter((a) => a.tags.includes(selectedTag.value!))
  }
  return list
})

function onImported() {
  audioStore.loadAllAudio()
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}

function statusLabel(status: string): string {
  switch (status) {
    case 'pending':
      return '等待中'
    case 'processing':
      return '导入中'
    case 'error':
      return '失败'
    case 'cancelled':
      return '已取消'
    case 'ready':
    default:
      return '就绪'
  }
}

function statusColor(status: string): string {
  switch (status) {
    case 'pending':
      return 'secondary'
    case 'processing':
      return 'primary'
    case 'error':
    case 'cancelled':
      return 'error'
    case 'ready':
    default:
      return 'primary'
  }
}

function statusIcon(status: string): string {
  switch (status) {
    case 'pending':
      return 'mdi-clock-outline'
    case 'processing':
      return 'mdi-progress-download'
    case 'error':
      return 'mdi-alert-circle-outline'
    case 'cancelled':
      return 'mdi-cancel'
    case 'ready':
    default:
      return 'mdi-music-note'
  }
}

function waveHeight(id: string, i: number): string {
  const char = id.charCodeAt(i % id.length)
  const h = 20 + ((char * i) % 70)
  return `${h}%`
}

function isSelected(id: string) {
  return selectedIds.value.includes(id)
}

function setSelected(id: string, value: boolean | null) {
  const selected = new Set(selectedIds.value)
  if (value) {
    selected.add(id)
  } else {
    selected.delete(id)
  }
  selectedIds.value = Array.from(selected)
}

function toggleSelect(id: string) {
  setSelected(id, !isSelected(id))
}

function selectAll() {
  const all = filteredAudio.value.map((a) => a.id)
  selectedIds.value = Array.from(new Set([...selectedIds.value, ...all]))
}

function invertSelection() {
  const all = filteredAudio.value.map((a) => a.id)
  selectedIds.value = all.filter((id) => !selectedIds.value.includes(id))
}

function openEdit(audio: AudioFile) {
  audioToEdit.value = audio
  showEditDialog.value = true
}

function confirmDelete(audio: AudioFile) {
  audioToDelete.value = audio
  showDeleteDialog.value = true
}

function confirmBatchDelete() {
  showBatchDeleteDialog.value = true
}

async function handleBatchDelete() {
  batchDeleting.value = true
  let succeeded = 0
  let failed = 0
  try {
    const ids = [...selectedIds.value]
    for (const id of ids) {
      try {
        await audioStore.deleteAudio(id)
        succeeded++
      } catch (err) {
        failed++
      }
    }
    selectedIds.value = []
    showBatchDeleteDialog.value = false
    if (failed > 0) {
      toast.display(
        `已删除 ${succeeded} 条音频，${failed} 条因被项目引用或出现错误未删除`,
        'error',
        5000
      )
    } else {
      toast.display(`成功删除 ${succeeded} 条音频`, 'success', 3000)
    }
  } finally {
    batchDeleting.value = false
  }
}

async function handleDelete() {
  if (!audioToDelete.value) return
  try {
    await audioStore.deleteAudio(audioToDelete.value.id)
    selectedIds.value = selectedIds.value.filter((id) => id !== audioToDelete.value!.id)
    showDeleteDialog.value = false
    audioToDelete.value = null
  } catch (err) {
    // Error already handled by api client
  }
}
</script>

<style scoped>
.archive-view {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.archive-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.archive-title {
  color: #ffb020;
}

.toolbar-area {
  flex: 0 0 auto;
}

.scroll-area {
  flex: 1 1 auto;
  overflow-y: auto;
  min-height: 0;
}

.search-field :deep(.v-field__outline) {
  color: rgba(74, 66, 56, 0.6);
}

.tag-filter :deep(.v-chip) {
  color: #c4b5a0;
  border-color: rgba(74, 66, 56, 0.6);
}

.tag-filter :deep(.v-chip--selected) {
  background: rgba(255, 176, 32, 0.14) !important;
  color: #ffb020;
  border-color: rgba(255, 176, 32, 0.4);
}

.audio-item {
  background: rgba(37, 33, 28, 0.5);
  border: 1px solid rgba(74, 66, 56, 0.3);
  transition: all 0.25s ease;
  overflow: hidden;
  position: relative;
  cursor: pointer;
}

.audio-progress {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 3;
}

.processing-wave .wave-bar {
  animation: pulse-bar 1s ease-in-out infinite;
}

@keyframes pulse-bar {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.status-chip {
  font-weight: 500;
}

.list-progress {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1;
}

.audio-item:hover,
.audio-item.selected {
  background: rgba(255, 176, 32, 0.06);
  border-color: rgba(255, 176, 32, 0.25);
  transform: translateY(-2px);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.25);
}

.select-indicator {
  position: absolute;
  top: 4px;
  left: 4px;
  z-index: 2;
}

.audio-wave {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  height: 48px;
  padding: 12px 16px 0;
  gap: 4px;
  opacity: 0.35;
}

.wave-bar {
  flex: 1;
  min-width: 3px;
  background: linear-gradient(180deg, #ffb020 0%, rgba(255, 176, 32, 0.2) 100%);
  border-radius: 2px 2px 0 0;
  transition: height 0.3s ease;
}

.audio-item:hover .wave-bar {
  background: linear-gradient(180deg, #ffb020 0%, rgba(255, 176, 32, 0.5) 100%);
}

.audio-content {
  position: relative;
  z-index: 1;
}

.hash {
  opacity: 0.6;
  font-size: 0.7rem;
}

.empty-state {
  border: 2px dashed rgba(74, 66, 56, 0.6);
  border-radius: 20px;
}

.audio-list {
  padding: 0;
}

.audio-list-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
  cursor: pointer;
}

.audio-list-item:hover,
.audio-list-item.selected {
  background: rgba(255, 176, 32, 0.06);
  border-color: rgba(255, 176, 32, 0.25);
}

.dialog-card {
  background: #1a1714;
  border: 1px solid rgba(74, 66, 56, 0.5);
  position: relative;
  overflow: hidden;
}

.dialog-accent {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #ff8a80 0%, rgba(255, 138, 128, 0.3) 100%);
}

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
