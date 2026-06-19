import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invokeCommand } from '@/api/client'
import { logger } from '@/utils/logger'

export interface Settings {
  ffmpeg_path?: string
  ffprobe_path?: string
  hoi4_game_dir?: string
  theme: string
  import_concurrency: number
  default_project_dir?: string
  default_author?: string
  default_version?: string
  default_supported_version?: string
  default_tags: string[]
}

export interface SettingsResponse extends Settings {
  detected_supported_version?: string
}

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<Settings | null>(null)
  const detectedSupportedVersion = ref<string | undefined>(undefined)
  const loading = ref(false)

  const effectiveSupportedVersion = computed(() => {
    return settings.value?.default_supported_version || detectedSupportedVersion.value || ''
  })

  async function loadSettings() {
    loading.value = true
    try {
      const resp = await invokeCommand<SettingsResponse>('get_settings')
      logger.info('settings store: loaded settings', resp)
      const { detected_supported_version, ...rest } = resp
      settings.value = rest
      detectedSupportedVersion.value = detected_supported_version
      return resp
    } finally {
      loading.value = false
    }
  }

  async function saveSettings(patch: Partial<Settings>) {
    const current = settings.value
    if (!current) {
      throw new Error('settings not loaded')
    }
    const next: Settings = { ...current, ...patch }
    await invokeCommand('save_settings', { settings: next })
    settings.value = next
    logger.info('settings store: saved settings')
  }

  async function getDefaultProjectDir() {
    return invokeCommand<string>('get_default_project_dir')
  }

  return {
    settings,
    detectedSupportedVersion,
    loading,
    effectiveSupportedVersion,
    loadSettings,
    saveSettings,
    getDefaultProjectDir,
  }
})
