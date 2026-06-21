import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { invokeCommand } from '@/api/client'
import { logger } from '@/utils/logger'

export type ImportStatus = 'processing' | 'ready' | 'error' | 'cancelled'

export interface AudioFile {
  id: string
  source_hash: string
  title: string
  artist?: string
  source_path: string
  ogg_filename: string
  duration_secs: number
  sample_rate: number
  channels: number
  volume: number
  tags: string[]
  notes?: string
  import_status: ImportStatus
  created_at: string
  updated_at: string
}

export interface BatchImportFailedFile {
  path: string
  message: string
}

export interface BatchImportResult {
  created: AudioFile[]
  existing: AudioFile[]
  failed: BatchImportFailedFile[]
}

export interface UpdateAudioFileRequest {
  title?: string
  artist?: string | null
  volume?: number
  tags?: string[]
  notes?: string | null
}

export interface BatchUpdateAudioFileRequest {
  artist?: string | null
  volume?: number
  tags?: string[]
}

export interface ImportStartedPayload {
  session_id: string
  total: number
}

export interface ImportFilePayload {
  session_id: string
  audio: AudioFile
}

export interface ImportResultPayload {
  session_id: string
  created: AudioFile[]
  existing: AudioFile[]
  failed: BatchImportFailedFile[]
}

export const useAudioStore = defineStore('audio', () => {
  const audioFiles = ref<AudioFile[]>([])
  const allAudioFiles = ref<AudioFile[]>([])
  const importing = ref(false)
  const importTotal = ref(0)
  const importCompleted = ref(0)
  const importFailed = ref<BatchImportFailedFile[]>([])

  let unlisteners: UnlistenFn[] = []
  let listening = false

  async function loadAudio(projectId: string) {
    audioFiles.value = await invokeCommand<AudioFile[]>('list_audio_files', { projectId })
  }

  async function loadAllAudio() {
    logger.info('audio store: loading all audio files')
    allAudioFiles.value = await invokeCommand<AudioFile[]>('list_all_audio_files')
    logger.info(`audio store: loaded ${allAudioFiles.value.length} audio file(s)`)
  }

  async function addToProject(projectId: string, audioIds: string[]) {
    await invokeCommand('add_audio_to_project', {
      projectId,
      audioIds,
    })
    await loadAudio(projectId)
  }

  async function removeFromProject(projectId: string, audioId: string) {
    await invokeCommand('remove_audio_from_project', {
      projectId,
      audioId,
    })
    await loadAudio(projectId)
  }

  async function deleteAudio(id: string) {
    await invokeCommand('delete_audio_file', { id })
    allAudioFiles.value = allAudioFiles.value.filter((a) => a.id !== id)
  }

  async function updateAudio(id: string, req: UpdateAudioFileRequest) {
    const updated = await invokeCommand<AudioFile>('update_audio_file', { id, req })
    const idx = allAudioFiles.value.findIndex((a) => a.id === id)
    if (idx !== -1) {
      allAudioFiles.value[idx] = updated
    }
    return updated
  }

  async function batchUpdateAudio(ids: string[], req: BatchUpdateAudioFileRequest) {
    const updated = await invokeCommand<AudioFile[]>('batch_update_audio_files', {
      ids,
      req,
    })
    for (const audio of updated) {
      const idx = allAudioFiles.value.findIndex((a) => a.id === audio.id)
      if (idx !== -1) {
        allAudioFiles.value[idx] = audio
      }
    }
    return updated
  }

  function upsertAudioFile(audio: AudioFile) {
    if (audio.import_status === 'cancelled') {
      allAudioFiles.value = allAudioFiles.value.filter((a) => a.id !== audio.id)
      return
    }
    const idx = allAudioFiles.value.findIndex((a) => a.id === audio.id)
    if (idx !== -1) {
      allAudioFiles.value[idx] = audio
    } else {
      allAudioFiles.value.push(audio)
    }
  }

  async function ensureListening() {
    if (listening) return
    listening = true

    const started = await listen<ImportStartedPayload>('import:started', (event) => {
      importing.value = true
      importTotal.value = event.payload.total
      importCompleted.value = 0
      importFailed.value = []
    })

    const file = await listen<ImportFilePayload>('import:file', (event) => {
      const audio = event.payload.audio
      upsertAudioFile(audio)
      if (audio.import_status === 'ready' || audio.import_status === 'error') {
        importCompleted.value++
      }
    })

    const completed = await listen<ImportResultPayload>('import:completed', (event) => {
      importing.value = false
      importFailed.value = event.payload.failed
    })

    const cancelled = await listen<ImportResultPayload>('import:cancelled', (event) => {
      importing.value = false
      importFailed.value = event.payload.failed
    })

    unlisteners = [started, file, completed, cancelled]
  }

  function stopListening() {
    for (const unlisten of unlisteners) {
      unlisten()
    }
    unlisteners = []
    listening = false
  }

  return {
    audioFiles,
    allAudioFiles,
    importing,
    importTotal,
    importCompleted,
    importFailed,
    loadAudio,
    loadAllAudio,
    addToProject,
    removeFromProject,
    deleteAudio,
    updateAudio,
    batchUpdateAudio,
    ensureListening,
    stopListening,
  }
})
