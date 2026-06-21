import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invokeCommand } from '@/api/client'
import type { AudioFile } from '@/stores/audio'

export type ImportPhase = 'hashing' | 'checking' | 'transcoding' | 'persisting'

export type ImportFileStatusType =
  | 'pending'
  | 'hashing'
  | 'existing'
  | 'transcoding'
  | 'created'
  | 'error'

export interface ImportFileStatus {
  path: string
  name: string
  status: ImportFileStatusType
  message?: string
}

export interface ImportStartedPayload {
  sessionId: string
  total: number
  files: ImportFileStatus[]
}

export interface ImportPhasePayload {
  sessionId: string
  phase: ImportPhase
  current: number
  total: number
}

export interface ImportFilePayload {
  sessionId: string
  path: string
  status: ImportFileStatusType
  message?: string
}

export interface ImportFailedFile {
  path: string
  message: string
}

export interface ImportResultPayload {
  sessionId: string
  created: AudioFile[]
  existing: AudioFile[]
  failed: ImportFailedFile[]
}

function basename(path: string): string {
  return path.replace(/\\/g, '/').split('/').pop() || path
}

export const useImportProgressStore = defineStore('importProgress', () => {
  const activeSessionId = ref<string | null>(null)
  const phase = ref<ImportPhase | null>(null)
  const phaseCurrent = ref(0)
  const phaseTotal = ref(0)
  const files = ref<ImportFileStatus[]>([])
  const result = ref<ImportResultPayload | null>(null)
  const cancelled = ref(false)

  const isActive = computed(() => activeSessionId.value !== null)
  const visible = computed(() => isActive.value || result.value !== null || cancelled.value)

  let listening = false

  function reset() {
    activeSessionId.value = null
    phase.value = null
    phaseCurrent.value = 0
    phaseTotal.value = 0
    files.value = []
    result.value = null
    cancelled.value = false
  }

  function startSession(paths: string[]): string {
    if (activeSessionId.value) {
      reset()
    }
    const sessionId = `import_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
    activeSessionId.value = sessionId
    phase.value = 'hashing'
    phaseCurrent.value = 0
    phaseTotal.value = paths.length
    files.value = paths.map((p) => ({
      path: p,
      name: basename(p),
      status: 'pending' as ImportFileStatusType,
    }))
    result.value = null
    cancelled.value = false
    return sessionId
  }

  function setPhase(p: ImportPhase, current: number, total: number) {
    phase.value = p
    phaseCurrent.value = current
    phaseTotal.value = total
  }

  function updateFile(path: string, status: ImportFileStatusType, message?: string) {
    const file = files.value.find((f) => f.path === path)
    if (file) {
      file.status = status
      file.message = message
    }
  }

  function complete(res: ImportResultPayload) {
    result.value = res
    activeSessionId.value = null
    phase.value = null
  }

  function setCancelled(res: ImportResultPayload) {
    result.value = res
    cancelled.value = true
    activeSessionId.value = null
    phase.value = null
  }

  function dismiss() {
    reset()
  }

  async function cancel() {
    if (!activeSessionId.value) return
    await invokeCommand('cancel_import', {})
  }

  async function ensureListening() {
    if (listening) return
    listening = true

    await listen<ImportStartedPayload>('import:started', (event) => {
      activeSessionId.value = event.payload.sessionId
      phase.value = 'hashing'
      phaseCurrent.value = 0
      phaseTotal.value = event.payload.total
      files.value = event.payload.files.map((f) => ({
        ...f,
        name: basename(f.path),
      }))
      result.value = null
      cancelled.value = false
    })

    await listen<ImportPhasePayload>('import:phase', (event) => {
      if (event.payload.sessionId !== activeSessionId.value) return
      setPhase(event.payload.phase, event.payload.current, event.payload.total)
    })

    await listen<ImportFilePayload>('import:file', (event) => {
      if (event.payload.sessionId !== activeSessionId.value) return
      updateFile(event.payload.path, event.payload.status, event.payload.message)
    })

    await listen<ImportResultPayload>('import:completed', (event) => {
      if (event.payload.sessionId !== activeSessionId.value) return
      complete(event.payload)
    })

    await listen<ImportResultPayload>('import:cancelled', (event) => {
      if (event.payload.sessionId !== activeSessionId.value) return
      setCancelled(event.payload)
    })
  }

  return {
    activeSessionId,
    phase,
    phaseCurrent,
    phaseTotal,
    files,
    result,
    cancelled,
    isActive,
    visible,
    startSession,
    setPhase,
    updateFile,
    complete,
    setCancelled,
    dismiss,
    cancel,
    ensureListening,
  }
})
