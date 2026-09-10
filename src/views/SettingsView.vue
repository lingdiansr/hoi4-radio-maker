<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-start pa-6 pb-2">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">GLOBAL CONFIGURATION</div>
          <div class="text-display text-h5">{{ $t('settings.title') }}</div>
        </div>
        <v-btn
          variant="text"
          prepend-icon="mdi-arrow-left"
          class="back-btn"
          @click="router.back()"
        >
          {{ $t('common.back') }}
        </v-btn>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-row v-if="settingsStore.settings">
          <v-col cols="12" md="8">
            <PathField
              v-model="ffmpegPath"
              :label="$t('settings.ffmpegPath')"
              :placeholder="$t('settings.ffmpegPlaceholder')"
              prepend-inner-icon="mdi-movie-play"
              picker-mode="file"
              class="mb-4"
            />
            <PathField
              v-model="ffprobePath"
              :label="$t('settings.ffprobePath')"
              :placeholder="$t('settings.ffprobePlaceholder')"
              prepend-inner-icon="mdi-magnify-scan"
              picker-mode="file"
              class="mb-4"
            />
            <PathField
              v-model="hoi4Path"
              :label="$t('settings.hoi4Dir')"
              :placeholder="$t('settings.hoi4Placeholder')"
              prepend-inner-icon="mdi-folder-open"
              picker-mode="directory"
              class="mb-4"
            />
            <PathField
              v-model="defaultProjectDir"
              :label="$t('settings.defaultLibrary')"
              :placeholder="$t('settings.defaultLibraryPlaceholder')"
              prepend-inner-icon="mdi-folder-cog"
              picker-mode="directory"
              class="mb-4"
            />
            <v-text-field
              v-model="settings.default_author"
              :label="$t('settings.defaultAuthor')"
              :placeholder="$t('settings.defaultAuthorPlaceholder')"
              prepend-inner-icon="mdi-account"
              class="mb-4"
              hide-details="auto"
              clearable
            />
            <v-text-field
              v-model="settings.default_version"
              :label="$t('settings.defaultVersion')"
              placeholder="0.1.0"
              prepend-inner-icon="mdi-tag-outline"
              class="mb-4"
              hide-details="auto"
            />
            <v-text-field
              v-model="settings.default_supported_version"
              :label="$t('settings.defaultSupportedVersion')"
              :placeholder="$t('settings.defaultSupportedPlaceholder')"
              prepend-inner-icon="mdi-gamepad-variant"
              class="mb-4"
              hide-details="auto"
              clearable
            />
            <v-combobox
              v-model="settings.default_tags"
              :label="$t('settings.defaultTags')"
              :placeholder="$t('project.tagsPlaceholder')"
              prepend-inner-icon="mdi-tag-multiple"
              multiple
              chips
              clearable
              class="mb-4"
              hide-details="auto"
            />
            <v-slider
              v-model="settings.import_concurrency"
              :label="$t('settings.concurrency')"
              min="1"
              max="16"
              step="1"
              thumb-label
              prepend-icon="mdi-swap-horizontal"
              class="mb-4"
              hide-details="auto"
            />
            <v-select
              v-model="settings.theme"
              :label="$t('settings.theme')"
              :items="themeOptions"
              item-title="label"
              item-value="value"
              prepend-inner-icon="mdi-palette"
              class="mb-4"
              hide-details="auto"
            />
            <v-select
              :model-value="settingsStore.settings.language || currentLocale"
              :label="$t('settings.language')"
              :items="languageOptions"
              item-title="label"
              item-value="value"
              prepend-inner-icon="mdi-translate"
              class="mb-4"
              hide-details="auto"
              @update:model-value="settingsStore.setLanguage"
            />
            <v-alert
              v-if="!settingsStore.ffmpegAvailable"
              type="warning"
              variant="tonal"
              class="mb-6"
              :text="$t('settings.ffmpegMissing')"
            />
            <v-btn
              color="primary"
              size="large"
              prepend-icon="mdi-content-save"
              class="save-btn"
              :loading="saving"
              @click="save"
            >
              {{ $t('settings.save') }}
            </v-btn>
          </v-col>

          <v-col cols="12" md="4">
            <v-card class="hint-card" variant="flat" rounded="lg">
              <v-card-text>
                <v-icon color="primary" size="32" class="mb-2">mdi-information-outline</v-icon>
                <div class="text-body text-secondary text-body-2">
                  {{ $t('settings.hintIntro') }}
                  <br><br>
                  <i18n-t keypath="settings.hintPaths" tag="span">
                    <template #libraryDir>
                      <strong>{{ $t('settings.hintLibraryDir') }}</strong>
                    </template>
                    <template #supportedVersion>
                      <strong>{{ $t('settings.hintSupportedVersion') }}</strong>
                    </template>
                  </i18n-t>
                  <br><br>
                  {{ $t('settings.hintConcurrency') }}
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
        <v-row v-else>
          <v-col cols="12" class="text-center py-8">
            <v-progress-circular indeterminate color="primary" />
            <div class="text-secondary mt-2">{{ $t('common.loading') }}</div>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <v-card class="settings-card mt-6" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">DIAGNOSTICS</div>
        <div class="text-display text-h5">{{ $t('settings.logsTitle') }}</div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div class="text-body text-secondary mb-4">
          {{ $t('settings.logsBody') }}
        </div>
        <div class="d-flex gap-3 flex-wrap">
          <v-btn
            color="primary"
            prepend-icon="mdi-folder-open-outline"
            @click="openLogFolder"
          >
            {{ $t('settings.openLogFolder') }}
          </v-btn>
          <v-btn
            variant="outlined"
            prepend-icon="mdi-content-copy"
            @click="copyLogPath"
          >
            {{ $t('settings.copyLogPath') }}
          </v-btn>
        </div>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { appLogDir } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { LOCALE_LABELS, SUPPORTED_LOCALES } from '@/i18n'
import { useSettingsStore } from '@/stores/settings'
import { logger } from '@/utils/logger'
import PathField from '@/components/PathField.vue'

const router = useRouter()
const settingsStore = useSettingsStore()
const { t, locale: currentLocale } = useI18n()

const saving = ref(false)

const settings = computed({
  get: () => settingsStore.settings!,
  set: (v) => {
    settingsStore.settings = v
  },
})

const ffmpegPath = computed({
  get: () => settingsStore.settings?.ffmpeg_path ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.ffmpeg_path = v.trim() || undefined
    }
  },
})

const ffprobePath = computed({
  get: () => settingsStore.settings?.ffprobe_path ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.ffprobe_path = v.trim() || undefined
    }
  },
})

const hoi4Path = computed({
  get: () => settingsStore.settings?.hoi4_game_dir ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.hoi4_game_dir = v.trim() || undefined
    }
  },
})

const defaultProjectDir = computed({
  get: () => settingsStore.settings?.default_project_dir ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.default_project_dir = v.trim() || undefined
    }
  },
})

const themeOptions = computed(() => [
  { label: t('settings.themeSystem'), value: 'system' },
  { label: t('settings.themeLight'), value: 'light' },
  { label: t('settings.themeDark'), value: 'dark' },
])

const languageOptions = computed(() =>
  SUPPORTED_LOCALES.map((value) => ({ label: LOCALE_LABELS[value], value }))
)

onMounted(async () => {
  await settingsStore.loadSettings()
})

async function save() {
  saving.value = true
  try {
    await settingsStore.saveSettings(settings.value)
    logger.info('Settings saved')
  } finally {
    saving.value = false
  }
}

async function openLogFolder() {
  try {
    const dir = await appLogDir()
    await openPath(dir)
    logger.info(`Opened log folder: ${dir}`)
  } catch (err) {
    logger.error(`Failed to open log folder: ${err}`)
  }
}

async function copyLogPath() {
  try {
    const dir = await appLogDir()
    await navigator.clipboard.writeText(dir)
    logger.info(`Copied log path: ${dir}`)
  } catch (err) {
    logger.error(`Failed to copy log path: ${err}`)
  }
}
</script>

<style scoped>
.settings-card {
  background: rgba(var(--v-theme-surface), 0.7);
  border: 1px solid rgba(var(--v-theme-outline), 0.4);
}

.save-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.back-btn {
  text-transform: none;
  letter-spacing: 0.02em;
  color: rgb(var(--v-theme-on-surface-variant));
}

.back-btn:hover {
  color: rgb(var(--v-theme-primary));
}

.hint-card {
  background: rgba(var(--v-theme-primary), 0.06);
  border: 1px solid rgba(var(--v-theme-primary), 0.2);
  height: 100%;
}
</style>
