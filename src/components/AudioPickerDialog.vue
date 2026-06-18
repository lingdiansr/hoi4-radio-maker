<template>
  <v-dialog v-model="dialog" max-width="720" class="bureau-dialog">
    <v-card class="dialog-card picker-card">
      <div class="dialog-accent" />
      <v-card-title class="dialog-title pa-6 pb-2">
        <div class="d-flex align-center gap-3">
          <v-icon color="primary" size="28">mdi-music-box-multiple</v-icon>
          <div>
            <div class="text-mono text-caption text-secondary">AUDIO ARCHIVE</div>
            <div class="text-display text-h5">从音频库选择</div>
          </div>
        </div>
      </v-card-title>

      <v-card-text class="pa-6 pt-4">
        <v-text-field
          v-model="search"
          label="搜索音频"
          placeholder="标题、艺术家、哈希…"
          prepend-inner-icon="mdi-magnify"
          variant="outlined"
          density="comfortable"
          hide-details
          class="mb-4"
        />

        <div class="audio-list">
          <v-list v-if="filteredAudio.length > 0" bg-color="transparent">
            <v-list-item
              v-for="audio in filteredAudio"
              :key="audio.id"
              class="audio-list-item mb-2"
              rounded="lg"
              @click="toggle(audio.id)"
            >
              <template #prepend>
                <v-checkbox-btn
                  :model-value="selected.has(audio.id)"
                  color="primary"
                  class="mr-2"
                  @click.stop="toggle(audio.id)"
                />
              </template>
              <v-list-item-title>{{ audio.title }}</v-list-item-title>
              <v-list-item-subtitle>
                {{ audio.artist || '未知艺术家' }} · {{ formatDuration(audio.duration_secs) }}
                · {{ audio.sample_rate }} Hz · {{ audio.source_hash.slice(0, 8) }}
              </v-list-item-subtitle>
            </v-list-item>
          </v-list>
          <div v-else class="empty-state text-center py-8">
            <v-icon size="48" color="secondary" class="mb-2">mdi-music-note-off</v-icon>
            <div class="text-body text-secondary">未找到匹配音频</div>
          </div>
        </div>
      </v-card-text>

      <v-divider opacity="0.2" />

      <v-card-actions class="pa-6">
        <div class="text-body text-secondary">
          已选择 {{ selected.size }} 首
        </div>
        <v-spacer />
        <AudioImporter
          mode="global"
          class="mr-3"
          @imported="onImported"
        />
        <v-btn variant="text" class="action-btn" @click="dialog = false">取消</v-btn>
        <v-btn color="primary" class="action-btn" prepend-icon="mdi-plus" @click="confirm">
          添加
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useAudioStore } from '@/stores/audio'
import AudioImporter from '@/components/AudioImporter.vue'

const dialog = defineModel<boolean>({ required: true })

const emit = defineEmits<{
  (e: 'confirm', audioIds: string[]): void
}>()

const audioStore = useAudioStore()
const search = ref('')
const selected = ref<Set<string>>(new Set())

const filteredAudio = computed(() => {
  const q = search.value.trim().toLowerCase()
  let list = audioStore.allAudioFiles
  if (q) {
    list = list.filter(
      (a) =>
        a.title.toLowerCase().includes(q) ||
        (a.artist?.toLowerCase().includes(q) ?? false) ||
        a.source_hash.toLowerCase().includes(q)
    )
  }
  return list
})

watch(dialog, (open) => {
  if (open) {
    audioStore.loadAllAudio()
    selected.value.clear()
  }
})

function toggle(id: string) {
  if (selected.value.has(id)) {
    selected.value.delete(id)
  } else {
    selected.value.add(id)
  }
}

function onImported() {
  audioStore.loadAllAudio()
}

function confirm() {
  if (selected.value.size === 0) return
  emit('confirm', Array.from(selected.value))
  dialog.value = false
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}
</script>

<style scoped>
.picker-card {
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
  background: linear-gradient(90deg, #ffb020 0%, rgba(255, 176, 32, 0.3) 100%);
}

.dialog-title {
  padding-top: 28px;
}

.audio-list {
  max-height: 420px;
  overflow-y: auto;
}

.audio-list-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.audio-list-item:hover {
  background: rgba(255, 176, 32, 0.06) !important;
  border-color: rgba(255, 176, 32, 0.2);
}

.empty-state {
  border: 2px dashed rgba(74, 66, 56, 0.5);
  border-radius: 12px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
