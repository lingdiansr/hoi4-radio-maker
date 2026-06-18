<template>
  <div class="pa-6">
    <v-card class="audio-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-center pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">AUDIO ARCHIVE</div>
          <div class="text-display text-h5">音频库</div>
        </div>
        <AudioImporter :project-id="projectId" />
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-alert
          v-if="!projectId"
          type="warning"
          variant="tonal"
          text="未选择项目"
          class="mb-4"
        />

        <div v-else-if="audioStore.audioFiles.length === 0" class="empty-state text-center py-12">
          <v-icon size="64" color="secondary" class="mb-4">mdi-music-box-outline</v-icon>
          <div class="text-body text-secondary text-h6 mb-2">暂无音频</div>
          <div class="text-body text-secondary mb-4">点击右上角导入音频到当前项目</div>
        </div>

        <v-row v-else>
          <v-col
            v-for="audio in audioStore.audioFiles"
            :key="audio.id"
            cols="12"
            sm="6"
            lg="4"
          >
            <v-card class="audio-item" variant="flat" rounded="lg">
              <v-card-text class="pa-4">
                <div class="d-flex align-start gap-3">
                  <v-icon color="primary" size="32">mdi-music-note</v-icon>
                  <div class="flex-grow-1">
                    <div class="text-body text-subtitle-1 font-weight-medium">{{ audio.title }}</div>
                    <div class="text-mono text-caption text-secondary mt-1">
                      {{ formatDuration(audio.duration_secs) }} · {{ audio.sample_rate }} Hz · {{ audio.channels }} ch
                    </div>
                    <div class="text-mono text-caption text-secondary mt-1 hash">
                      {{ audio.source_hash.slice(0, 12) }}…
                    </div>
                  </div>
                  <v-btn
                    icon="mdi-delete-outline"
                    variant="text"
                    size="small"
                    color="error"
                    @click="audioStore.deleteAudio(audio.id)"
                  />
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useAudioStore } from '@/stores/audio'
import AudioImporter from '@/components/AudioImporter.vue'

const route = useRoute()
const audioStore = useAudioStore()

const projectId = computed(() => route.params.id as string)

onMounted(() => {
  if (projectId.value) {
    audioStore.loadAudio(projectId.value)
  }
})

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}
</script>

<style scoped>
.audio-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
}

.audio-item {
  background: rgba(37, 33, 28, 0.5);
  border: 1px solid rgba(74, 66, 56, 0.3);
  transition: all 0.2s ease;
}

.audio-item:hover {
  background: rgba(255, 176, 32, 0.06);
  border-color: rgba(255, 176, 32, 0.2);
}

.hash {
  opacity: 0.7;
}

.empty-state {
  border: 2px dashed rgba(74, 66, 56, 0.6);
  border-radius: 16px;
}
</style>
