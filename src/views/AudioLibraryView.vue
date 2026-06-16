<template>
  <div class="pa-6">
    <v-card class="audio-library-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-center pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">AUDIO ARCHIVE</div>
          <div class="text-display text-h5">音频库</div>
        </div>
        <AudioImporter @import="onImport" />
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div v-if="audioStore.audioFiles.length === 0" class="empty-drop text-center py-12">
          <v-icon size="64" color="secondary" class="mb-4 drop-icon">mdi-music-box-outline</v-icon>
          <div class="text-body text-secondary text-h6 mb-2">音频库为空</div>
          <div class="text-body text-secondary mb-6">导入 MP3 / WAV / FLAC / OGG 文件开始构建你的电台</div>
          <AudioImporter @import="onImport" />
        </div>

        <v-list v-else class="audio-list" bg-color="transparent">
          <v-list-item
            v-for="(audio, index) in audioStore.audioFiles"
            :key="audio.id"
            class="audio-item mb-3"
            rounded="lg"
          >
            <template #prepend>
              <div class="audio-number text-mono">{{ String(index + 1).padStart(2, '0') }}</div>
              <v-avatar color="surface-variant" size="48" class="mr-4">
                <v-icon color="primary">mdi-music-note</v-icon>
              </v-avatar>
            </template>

            <v-list-item-title>{{ audio.title }}</v-list-item-title>
            <v-list-item-subtitle>
              {{ audio.artist || '未知艺术家' }} · {{ formatDuration(audio.duration_secs) }} · {{ audio.sample_rate }}Hz · {{ audio.channels }}ch
            </v-list-item-subtitle>

            <template #append>
              <v-btn
                icon="mdi-delete-outline"
                variant="text"
                color="error"
                class="delete-btn"
                @click="audioStore.deleteAudio(audio.id)"
              />
            </template>
          </v-list-item>
        </v-list>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import AudioImporter from '@/components/AudioImporter.vue'
import { useAudioStore } from '@/stores/audio'

const audioStore = useAudioStore()

onMounted(() => {
  audioStore.loadAudio()
})

async function onImport(paths: string[]) {
  for (const path of paths) {
    await audioStore.importAudio(path)
  }
}

function formatDuration(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}
</script>

<style scoped>
.audio-library-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
}

.empty-drop {
  border: 2px dashed rgba(74, 66, 56, 0.6);
  border-radius: 16px;
  transition: all 0.3s ease;
}

.empty-drop:hover {
  border-color: rgba(255, 176, 32, 0.5);
  background: rgba(255, 176, 32, 0.04);
}

.drop-icon {
  animation: float 4s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-8px); }
}

.audio-list {
  overflow-y: auto;
}

.audio-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.audio-item:hover {
  background: rgba(255, 176, 32, 0.06) !important;
  border-color: rgba(255, 176, 32, 0.2);
}

.audio-number {
  width: 36px;
  color: #ffb020;
  font-size: 0.8rem;
  opacity: 0.6;
}

.delete-btn {
  opacity: 0.5;
  transition: opacity 0.2s ease;
}

.audio-item:hover .delete-btn {
  opacity: 1;
}
</style>
