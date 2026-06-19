<template>
  <div class="pa-6">
    <v-card class="station-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-center pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">BROADCAST CHANNELS</div>
          <div class="text-display text-h5">电台编辑</div>
        </div>
        <v-btn
          color="primary"
          prepend-icon="mdi-plus"
          class="create-btn"
          @click="openCreateDialog"
        >
          新建电台
        </v-btn>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div v-if="stationStore.stations.length === 0" class="empty-state text-center py-12">
          <v-icon size="64" color="secondary" class="mb-4">mdi-antenna</v-icon>
          <div class="text-body text-secondary text-h6 mb-2">暂无电台</div>
          <div class="text-body text-secondary mb-4">创建一个电台，然后从音频库添加歌曲</div>
          <v-btn color="primary" prepend-icon="mdi-plus" @click="openCreateDialog">新建电台</v-btn>
        </div>

        <template v-else>
          <v-tabs v-model="activeTab" class="station-tabs" bg-color="transparent">
            <v-tab
              v-for="station in stationStore.stations"
              :key="station.id"
              :value="station.id"
              prepend-icon="mdi-radio"
            >
              {{ station.name }}
            </v-tab>
          </v-tabs>

          <v-window v-model="activeTab" class="mt-4">
            <v-window-item
              v-for="station in stationStore.stations"
              :key="station.id"
              :value="station.id"
            >
              <v-card class="entry-card" variant="flat" rounded="lg">
                <v-card-title class="d-flex justify-space-between align-center">
                  <span class="text-display text-h6">歌曲列表</span>
                  <v-chip size="small" color="primary" class="text-mono">
                    {{ station.entries.length }} tracks
                  </v-chip>
                </v-card-title>
                <v-card-text>
                  <v-list v-if="station.entries.length > 0" bg-color="transparent">
                    <v-list-item
                      v-for="entry in station.entries"
                      :key="entry.audio_file_id"
                      class="entry-item mb-2"
                      rounded="lg"
                    >
                      <template #prepend>
                        <v-icon color="primary" class="mr-4">mdi-music-note</v-icon>
                      </template>
                      <v-list-item-title>{{ audioTitle(entry.audio_file_id) }}</v-list-item-title>
                      <v-list-item-subtitle>factor: {{ entry.chance.factor }}</v-list-item-subtitle>
                      <template #append>
                        <v-btn
                          icon="mdi-delete-outline"
                          variant="text"
                          color="error"
                          @click="removeEntry(station.id, entry.audio_file_id)"
                        />
                      </template>
                    </v-list-item>
                  </v-list>
                  <v-alert v-else type="info" variant="tonal" text="该电台暂无歌曲，点击下方按钮从音频库添加" />

                  <v-divider class="my-4" opacity="0.2" />

                  <v-btn
                    color="primary"
                    variant="outlined"
                    prepend-icon="mdi-music-box-multiple"
                    class="action-btn"
                    @click="openPicker(station.id)"
                  >
                    从音频库添加歌曲
                  </v-btn>
                </v-card-text>
              </v-card>
            </v-window-item>
          </v-window>
        </template>
      </v-card-text>
    </v-card>

    <!-- Create Station Dialog -->
    <v-dialog v-model="showCreateDialog" max-width="460" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-radio</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">NEW CHANNEL</div>
              <div class="text-display text-h5">新建电台</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="newStationName"
            label="电台名称"
            placeholder="例如：前线战报"
            prepend-inner-icon="mdi-antenna"
            hide-details="auto"
            :rules="[required]"
            @keyup.enter="createStation"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showCreateDialog = false">取消</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="createStation"
          >
            创建
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Audio Picker -->
    <AudioPickerDialog
      v-model="showPicker"
      @confirm="onAudioSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useStationStore } from '@/stores/station'
import { useAudioStore } from '@/stores/audio'
import { logger } from '@/utils/logger'
import AudioPickerDialog from '@/components/AudioPickerDialog.vue'

const route = useRoute()
const stationStore = useStationStore()
const audioStore = useAudioStore()
const activeTab = ref<string>('')
const showCreateDialog = ref(false)
const showPicker = ref(false)
const newStationName = ref('')
const currentStationId = ref<string>('')

const projectId = computed(() => route.params.id as string)

function required(v: string) {
  return !!v || '此项为必填'
}

onMounted(() => {
  stationStore.loadStations()
  if (projectId.value) {
    audioStore.loadAudio(projectId.value)
  }
})

watch(
  () => stationStore.stations,
  (stations) => {
    if (stations.length > 0 && !activeTab.value) {
      activeTab.value = stations[0].id
    }
  },
  { immediate: true }
)

function audioTitle(id: string): string {
  const audio = audioStore.audioFiles.find((a) => a.id === id)
  return audio?.title || id
}

function openCreateDialog() {
  newStationName.value = ''
  showCreateDialog.value = true
}

async function createStation() {
  const name = newStationName.value.trim()
  if (!name) return
  await stationStore.createStation(name)
  showCreateDialog.value = false
  newStationName.value = ''
}

function openPicker(stationId: string) {
  currentStationId.value = stationId
  showPicker.value = true
}

async function onAudioSelected(audioIds: string[]) {
  if (!currentStationId.value || !projectId.value) {
    logger.warn('station editor: missing station or project id when adding audio')
    return
  }
  logger.info(
    `station editor: adding ${audioIds.length} audio file(s) to project ${projectId.value} and station ${currentStationId.value}`
  )
  try {
    await audioStore.addToProject(projectId.value, audioIds)
    for (const audioId of audioIds) {
      await stationStore.addEntry(currentStationId.value, audioId, { factor: 1 })
    }
    logger.info('station editor: audio added successfully')
  } catch (err) {
    logger.error(`station editor: failed to add audio: ${JSON.stringify(err)}`)
    throw err
  }
}

async function removeEntry(stationId: string, audioFileId: string) {
  await stationStore.removeEntry(stationId, audioFileId)
}
</script>

<style scoped>
.station-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
}

.create-btn {
  text-transform: none;
}

.station-tabs :deep(.v-tab) {
  text-transform: none;
  letter-spacing: 0.02em;
}

.entry-card {
  background: rgba(37, 33, 28, 0.5);
  border: 1px solid rgba(74, 66, 56, 0.3);
}

.entry-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.entry-item:hover {
  background: rgba(255, 176, 32, 0.06) !important;
  border-color: rgba(255, 176, 32, 0.2);
}

.empty-state {
  border: 2px dashed rgba(74, 66, 56, 0.6);
  border-radius: 16px;
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
  background: linear-gradient(90deg, #ffb020 0%, rgba(255, 176, 32, 0.3) 100%);
}

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
