import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '@/api/client'
import { useProjectStore } from '@/stores/project'
import { logger } from '@/utils/logger'

export interface ChanceConfig {
  factor: number
  modifiers?: any[]
}

export interface StationEntry {
  audio_file_id: string
  chance: ChanceConfig
}

export interface Station {
  id: string
  name: string
  entries: StationEntry[]
}

export const useStationStore = defineStore('station', () => {
  const stations = ref<Station[]>([])
  const projectStore = useProjectStore()

  async function loadStations() {
    if (!projectStore.currentProject) return
    stations.value = await invokeCommand<Station[]>('list_stations', {
      projectId: projectStore.currentProject.id,
    })
  }

  async function createStation(name: string) {
    if (!projectStore.currentProject) return
    const station = await invokeCommand<Station>('create_station', {
      projectId: projectStore.currentProject.id,
      name,
    })
    stations.value.push(station)
    return station
  }

  async function addEntry(stationId: string, audioFileId: string, chance: ChanceConfig) {
    logger.info(`station store: adding entry ${audioFileId} to station ${stationId}`)
    try {
      await invokeCommand('add_station_entry', {
        stationId,
        audioFileId,
        chance,
      })
      await loadStations()
      logger.info(`station store: entry ${audioFileId} added to station ${stationId}`)
    } catch (err) {
      logger.error(`station store: failed to add entry: ${JSON.stringify(err)}`)
      throw err
    }
  }

  async function removeEntry(stationId: string, audioFileId: string) {
    await invokeCommand('remove_station_entry', { stationId, audioFileId })
    await loadStations()
  }

  async function deleteStation(stationId: string) {
    await invokeCommand('delete_station', { stationId })
    stations.value = stations.value.filter((s) => s.id !== stationId)
  }

  return { stations, loadStations, createStation, addEntry, removeEntry, deleteStation }
})
