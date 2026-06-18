<template>
  <v-app class="radio-bureau-app">
    <div class="grain-overlay" aria-hidden="true" />
    <v-layout style="min-height: 100vh">
      <AppSidebar />
      <v-main>
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
import { onMounted } from 'vue'
import { invokeCommand, isAppError } from '@/api/client'
import { useToastStore } from '@/stores/toast'
import AppSidebar from '@/components/AppSidebar.vue'

const toast = useToastStore()

onMounted(async () => {
  try {
    await invokeCommand('get_settings')
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

.radio-bureau-app {
  background:
    radial-gradient(ellipse at 20% 0%, rgba(255, 176, 32, 0.06) 0%, transparent 45%),
    radial-gradient(ellipse at 80% 100%, rgba(143, 158, 138, 0.05) 0%, transparent 40%),
    #12100e !important;
  color: #efebe3;
  font-family: var(--font-body);
}

.grain-overlay {
  pointer-events: none;
  position: fixed;
  inset: 0;
  z-index: 9999;
  opacity: 0.04;
  mix-blend-mode: overlay;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noiseFilter'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noiseFilter)'/%3E%3C/svg%3E");
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
  background: rgba(18, 16, 14, 0.85);
  backdrop-filter: blur(2px);
}

.bureau-dialog .v-overlay__content {
  box-shadow:
    0 24px 48px rgba(0, 0, 0, 0.4),
    0 0 0 1px rgba(255, 176, 32, 0.08);
}

/* Custom scrollbar */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: #1a1714;
}

::-webkit-scrollbar-thumb {
  background: #4a4238;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #ffb020;
}
</style>
