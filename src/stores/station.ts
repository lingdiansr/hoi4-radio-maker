import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '@/api/client'
import { useProjectStore } from '@/stores/project'
import { logger } from '@/utils/logger'

export type TriggerType = 'has_war' | 'tag' | 'has_government' | 'is_in_faction' | 'generic'

export interface Trigger {
  type: TriggerType
  value?: boolean | string
  ideology?: string
  tag?: string
  /** Trigger name for `type: 'generic'`, taken from the loaded vocabulary. */
  name?: string
}

export interface Modifier {
  factor?: number
  add?: number
  base?: number
  triggers: Trigger[]
}

export interface ChanceConfig {
  factor: number
  modifiers: Modifier[]
}

export interface StationEntry {
  audio_file_id: string
  chance: ChanceConfig
}

/** A trigger documented by the loaded game/mod vocabulary. */
export interface TriggerDef {
  name: string
  scopes: string[]
  /** Documented targets; scope keywords here are valid values. */
  targets: string[]
}

/** Value kind inferred from how the game/mod scripts use a trigger. */
export type ValueKind = 'boolean' | 'number' | 'text'

/** Trigger/tag/ideology vocabulary a project loads from the game and its mods. */
export interface ScriptVocabulary {
  triggers: Record<string, TriggerDef>
  country_tags: string[]
  ideologies: string[]
}

export interface Station {
  id: string
  name: string
  subdir?: string | null
  entries: StationEntry[]
}

export const useStationStore = defineStore('station', () => {
  const stations = ref<Station[]>([])
  const vocabulary = ref<ScriptVocabulary | null>(null)
  // Value kinds are looked up on demand (the backend scans scripts until it has
  // enough examples) and kept for the session.
  const valueKinds = ref<Record<string, ValueKind | null>>({})
  const projectStore = useProjectStore()

  /** Resolve a trigger's value kind once, then serve it from the cache. */
  async function ensureValueKind(name: string) {
    if (!name || name in valueKinds.value) return valueKinds.value[name]
    if (!projectStore.currentProject) return null
    const kind = await invokeCommand<ValueKind | null>('trigger_value_kind', {
      projectId: projectStore.currentProject.id,
      name,
    })
    valueKinds.value[name] = kind ?? null
    return valueKinds.value[name]
  }

  /** Load the project's trigger vocabulary; empty when nothing is configured. */
  async function loadVocabulary() {
    if (!projectStore.currentProject) return
    vocabulary.value = await invokeCommand<ScriptVocabulary>('project_script_vocabulary', {
      projectId: projectStore.currentProject.id,
    })
    // Cached kinds belong to the previous source set.
    valueKinds.value = {}
  }

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

  async function renameStation(stationId: string, name: string) {
    if (!projectStore.currentProject) return
    const updated = await invokeCommand<Station>('rename_station', {
      projectId: projectStore.currentProject.id,
      stationId,
      name,
    })
    const idx = stations.value.findIndex((s) => s.id === stationId)
    if (idx !== -1) {
      stations.value[idx] = updated
    }
    return updated
  }

  async function setStationSubdir(stationId: string, subdir: string | null) {
    const updated = await invokeCommand<Station>('set_station_subdir', {
      stationId,
      subdir,
    })
    const idx = stations.value.findIndex((s) => s.id === stationId)
    if (idx !== -1) {
      stations.value[idx] = updated
    }
    return updated
  }

  async function reorderStations(orderedIds: string[]) {
    if (!projectStore.currentProject) return
    await invokeCommand('reorder_stations', {
      projectId: projectStore.currentProject.id,
      stationIds: orderedIds,
    })
    await loadStations()
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

  async function updateEntry(stationId: string, audioFileId: string, chance: ChanceConfig) {
    await invokeCommand('update_station_entry', {
      stationId,
      audioFileId,
      chance,
    })
    await loadStations()
  }

  async function removeEntry(stationId: string, audioFileId: string) {
    await invokeCommand('remove_station_entry', { stationId, audioFileId })
    await loadStations()
  }

  async function reorderEntries(stationId: string, orderedIds: string[]) {
    await invokeCommand('reorder_station_entries', {
      stationId,
      audioIds: orderedIds,
    })
    await loadStations()
  }

  async function deleteStation(stationId: string) {
    await invokeCommand('delete_station', { stationId })
    stations.value = stations.value.filter((s) => s.id !== stationId)
  }

  return {
    stations,
    vocabulary,
    valueKinds,
    loadVocabulary,
    ensureValueKind,
    loadStations,
    createStation,
    renameStation,
    setStationSubdir,
    reorderStations,
    addEntry,
    updateEntry,
    removeEntry,
    reorderEntries,
    deleteStation,
  }
})
