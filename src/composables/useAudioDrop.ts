import { ref, onMounted, onBeforeUnmount } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useAudioStore, type BatchImportResult } from '@/stores/audio'
import { invokeCommand } from '@/api/client'

const AUDIO_EXT = ['mp3', 'flac', 'wav', 'ogg', 'm4a', 'aac', 'wma']

export function useAudioDrop(onImported?: () => void) {
  const audioStore = useAudioStore()
  const dragActive = ref(false)
  let unlisten: (() => void) | undefined

  async function importPaths(paths: string[]) {
    const audio = paths.filter((p) => {
      const ext = p.split('.').pop()?.toLowerCase() ?? ''
      return AUDIO_EXT.includes(ext)
    })
    if (audio.length === 0) return
    audioStore.importing = true
    try {
      await invokeCommand<BatchImportResult>('import_audio_batch', { paths: audio })
      await audioStore.loadAllAudio()
      onImported?.()
    } finally {
      audioStore.importing = false
    }
  }

  onMounted(async () => {
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over' || event.payload.type === 'enter') {
        dragActive.value = true
      } else if (event.payload.type === 'leave') {
        dragActive.value = false
      } else if (event.payload.type === 'drop') {
        dragActive.value = false
        importPaths(event.payload.paths)
      }
    })
  })

  onBeforeUnmount(() => { unlisten?.() })

  return { dragActive }
}
