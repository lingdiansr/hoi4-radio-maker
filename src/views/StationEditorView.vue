<template>
  <div class="pa-4">
    <div class="d-flex align-center mb-4">
      <h3 class="text-h6 mr-4">电台编辑</h3>
      <v-btn prepend-icon="mdi-plus" @click="createStation">新建电台</v-btn>
    </div>

    <v-alert v-if="stationStore.stations.length === 0" type="info" text="暂无电台，请先创建一个电台。" />

    <v-tabs v-else v-model="activeTab">
      <v-tab v-for="station in stationStore.stations" :key="station.id" :value="station.id">
        {{ station.name }}
      </v-tab>
    </v-tabs>

    <v-window v-if="stationStore.stations.length > 0" v-model="activeTab">
      <v-window-item v-for="station in stationStore.stations" :key="station.id" :value="station.id">
        <v-card class="mt-4">
          <v-card-title>歌曲列表</v-card-title>
          <v-card-text>
            <v-list>
              <v-list-item
                v-for="entry in station.entries"
                :key="entry.audio_file_id"
                :title="entry.audio_file_id"
                :subtitle="`factor: ${entry.chance.factor}`"
              >
                <template #append>
                  <v-btn icon="mdi-delete" variant="text" color="error" @click="removeEntry(station.id, entry.audio_file_id)" />
                </template>
              </v-list-item>
            </v-list>
            <v-divider class="my-4" />
            <div class="d-flex align-center">
              <v-select
                v-model="selectedAudio"
                label="选择音频"
                :items="audioStore.audioFiles"
                item-title="title"
                item-value="id"
                class="mr-4"
              />
              <v-btn color="primary" @click="addEntry(station.id)">添加</v-btn>
            </div>
          </v-card-text>
        </v-card>
      </v-window-item>
    </v-window>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useStationStore } from '@/stores/station'
import { useAudioStore } from '@/stores/audio'

const stationStore = useStationStore()
const audioStore = useAudioStore()
const activeTab = ref<string>('')
const selectedAudio = ref<string>('')

onMounted(() => {
  stationStore.loadStations()
  audioStore.loadAudio()
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

async function createStation() {
  const name = prompt('电台名称') || 'New Station'
  await stationStore.createStation(name)
}

async function addEntry(stationId: string) {
  if (!selectedAudio.value) return
  await stationStore.addEntry(stationId, selectedAudio.value, { factor: 1 })
  selectedAudio.value = ''
}

async function removeEntry(stationId: string, audioFileId: string) {
  await stationStore.removeEntry(stationId, audioFileId)
}
</script>
