import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '@/api/client'

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
  created_at: string
  updated_at: string
}

export interface BatchImportResult {
  created: AudioFile[]
  existing: AudioFile[]
}

export const useAudioStore = defineStore('audio', () => {
  const audioFiles = ref<AudioFile[]>([])
  const allAudioFiles = ref<AudioFile[]>([])
  const importing = ref(false)
  const importProgress = ref<{ total: number; processed: number } | null>(null)

  async function loadAudio(projectId: string) {
    audioFiles.value = await invokeCommand<AudioFile[]>('list_audio_files', { projectId })
  }

  async function loadAllAudio() {
    allAudioFiles.value = await invokeCommand<AudioFile[]>('list_all_audio_files')
  }

  async function importBatch(projectId: string, paths: string[]) {
    importing.value = true
    importProgress.value = { total: paths.length, processed: 0 }
    try {
      const result = await invokeCommand<BatchImportResult>('import_audio_batch', {
        projectId,
        paths,
      })
      await loadAudio(projectId)
      return result
    } finally {
      importing.value = false
      importProgress.value = null
    }
  }

  async function deleteAudio(id: string) {
    await invokeCommand('delete_audio_file', { id })
    audioFiles.value = audioFiles.value.filter((a) => a.id !== id)
    allAudioFiles.value = allAudioFiles.value.filter((a) => a.id !== id)
  }

  return {
    audioFiles,
    allAudioFiles,
    importing,
    importProgress,
    loadAudio,
    loadAllAudio,
    importBatch,
    deleteAudio,
  }
})
