import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '@/api/client'
import { logger } from '@/utils/logger'

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

  async function loadAudio(projectId: string) {
    audioFiles.value = await invokeCommand<AudioFile[]>('list_audio_files', { projectId })
  }

  async function loadAllAudio() {
    logger.info('audio store: loading all audio files')
    allAudioFiles.value = await invokeCommand<AudioFile[]>('list_all_audio_files')
    logger.info(`audio store: loaded ${allAudioFiles.value.length} audio file(s)`)
  }

  async function importGlobalBatch(paths: string[]): Promise<BatchImportResult> {
    importing.value = true
    logger.info(`audio store: starting global import of ${paths.length} file(s)`)
    try {
      const result = await invokeCommand<BatchImportResult>('import_audio_batch', {
        paths,
      })
      logger.info(
        `audio store: import returned created=${result.created.length} existing=${result.existing.length}`
      )
      await loadAllAudio()
      return result
    } catch (err) {
      logger.error(`audio store: global import failed: ${err}`)
      throw err
    } finally {
      importing.value = false
    }
  }

  async function importBatch(projectId: string, paths: string[]): Promise<BatchImportResult> {
    importing.value = true
    try {
      const result = await invokeCommand<BatchImportResult>('import_audio_batch', {
        projectId,
        paths,
      })
      await loadAudio(projectId)
      return result
    } finally {
      importing.value = false
    }
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

  return {
    audioFiles,
    allAudioFiles,
    importing,
    loadAudio,
    loadAllAudio,
    importGlobalBatch,
    importBatch,
    addToProject,
    removeFromProject,
    deleteAudio,
  }
})
