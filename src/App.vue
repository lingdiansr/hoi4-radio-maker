<template>
  <v-app class="radio-bureau-app">
    <div class="grain-overlay" aria-hidden="true" />
    <v-layout class="app-layout">
      <AppSidebar />
      <v-main class="app-main">
        <router-view />
      </v-main>
    </v-layout>
    <v-snackbar
      v-model="toast.show"
      :color="toast.color"
      :timeout="toast.timeout"
      location="top right"
      variant="elevated"
      class="bureau-snackbar"
    >
      {{ toast.message }}
      <template #actions>
        <v-btn variant="text" size="small" @click="toast.show = false">关闭</v-btn>
      </template>
    </v-snackbar>
  </v-app>
</template>

<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { isAppError } from '@/api/client'
import { useAudioStore } from '@/stores/audio'
import { useSettingsStore } from '@/stores/settings'
import { useToastStore } from '@/stores/toast'
import { useTheme } from '@/plugins/vuetify'
import { applyTheme, watchSystemTheme } from '@/plugins/theme'
import AppSidebar from '@/components/AppSidebar.vue'

const toast = useToastStore()
const audioStore = useAudioStore()
const settingsStore = useSettingsStore()
const theme = useTheme()

// Keep the Vuetify theme in sync with the persisted settings value. The watch
// fires once settings load (startup) and again whenever the user saves.
watch(
  () => settingsStore.settings?.theme,
  (value) => applyTheme(theme, value),
  { immediate: true }
)

// While the setting is 'system', react to OS color-scheme changes live.
watchSystemTheme(theme, () => settingsStore.settings?.theme === 'system')

onMounted(async () => {
  // Listen for import progress/result events for the whole app lifetime, so
  // drag-drop imports work from any view and progress stays live.
  audioStore.ensureListening()
  try {
    await settingsStore.loadSettings()
  } catch (err) {
    if (isAppError(err) && err.type === 'ffmpeg_not_found') {
      toast.display(err.message, 'error', 8000)
    }
    // Other settings errors are not critical on startup.
  }
})
</script>

<style>
:root {
  --font-display: 'Oranienbaum', 'Times New Roman', serif;
  --font-body: 'Source Serif 4', Georgia, serif;
  --font-mono: 'JetBrains Mono', 'Fira Code', monospace;
}

.app-layout {
  height: 100vh;
  overflow: hidden;
}

.app-main {
  flex: 1 1 auto;
  overflow-y: auto;
  min-height: 0;
}

.radio-bureau-app {
  background:
    radial-gradient(ellipse at 20% 0%, rgba(var(--v-theme-primary), 0.06) 0%, transparent 45%),
    radial-gradient(ellipse at 80% 100%, rgba(var(--v-theme-tertiary), 0.05) 0%, transparent 40%),
    rgb(var(--v-theme-background)) !important;
  background-repeat: no-repeat;
  color: rgb(var(--v-theme-on-background));
  font-family: var(--font-body);
}

.grain-overlay {
  pointer-events: none;
  position: fixed;
  inset: 0;
  z-index: 9999;
  opacity: 0.04;
  mix-blend-mode: overlay;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 512 512' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
  background-size: 512px 512px;
  background-repeat: repeat;
  will-change: transform;
  transform: translateZ(0);
  backface-visibility: hidden;
}

.text-display {
  font-family: var(--font-display) !important;
}

.text-body {
  font-family: var(--font-body) !important;
}

.text-mono {
  font-family: var(--font-mono) !important;
}

.v-application {
  font-family: var(--font-body) !important;
}

.v-btn {
  font-family: var(--font-body);
  font-weight: 600;
}

.v-card-title,
.v-card-subtitle {
  font-family: var(--font-body);
}

.v-list-item-title {
  font-family: var(--font-body);
  font-weight: 600;
}

.v-list-item-subtitle {
  font-family: var(--font-mono);
  font-size: 0.8rem;
  opacity: 0.72;
}

.bureau-snackbar .v-snackbar__content {
  font-family: var(--font-body);
}

.bureau-dialog .v-overlay__scrim {
  background: rgba(var(--v-theme-background), 0.85);
  backdrop-filter: blur(2px);
}

.bureau-dialog .v-overlay__content {
  box-shadow:
    0 24px 48px rgba(var(--v-theme-on-background), 0.4),
    0 0 0 1px rgba(var(--v-theme-primary), 0.08);
}

/* Flex gap utilities (Vuetify 3 does not enable gap-* by default) */
.gap-1 { gap: 4px !important; }
.gap-2 { gap: 8px !important; }
.gap-3 { gap: 12px !important; }
.gap-4 { gap: 16px !important; }
.gap-5 { gap: 20px !important; }
.gap-6 { gap: 24px !important; }

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: rgb(var(--v-theme-surface));
}

::-webkit-scrollbar-thumb {
  background: rgb(var(--v-theme-outline));
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgb(var(--v-theme-primary));
}
</style>
