import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invokeCommand } from '@/api/client'
import { useProjectStore } from '@/stores/project'

export interface AudioFile {
  id: string
  title: string
  artist?: string
  duration_secs: number
  sample_rate: number
  channels: number
}

export const useAudioStore = defineStore('audio', () => {
  const audioFiles = ref<AudioFile[]>([])
  const projectStore = useProjectStore()

  async function loadAudio() {
    if (!projectStore.currentProject) return
    audioFiles.value = await invokeCommand<AudioFile[]>('list_audio_files', {
      projectId: projectStore.currentProject.id,
    })
  }

  async function importAudio(path: string) {
    if (!projectStore.currentProject) return
    await invokeCommand('import_audio', {
      projectId: projectStore.currentProject.id,
      path,
    })
    await loadAudio()
  }

  async function deleteAudio(id: string) {
    await invokeCommand('delete_audio_file', { id })
    await loadAudio()
  }

  return { audioFiles, loadAudio, importAudio, deleteAudio }
})
