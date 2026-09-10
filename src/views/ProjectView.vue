<template>
  <div class="project-view">
    <div class="project-header pa-6 pb-4">
      <div class="d-flex justify-space-between align-start">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">PROJECT</div>
          <h1 class="text-display text-h3 mb-2">
            {{ projectStore.currentProject?.name || $t('nav.noProjectSelected') }}
          </h1>
          <div v-if="projectStore.currentProject" class="text-mono text-secondary">
            {{ projectStore.currentProject.version }} · {{ projectStore.currentProject.supported_version }}
          </div>
        </div>
        <div class="d-flex gap-3">
          <v-btn
            variant="outlined"
            prepend-icon="mdi-check-circle"
            class="action-btn"
            @click="validate"
          >
            {{ $t('project.validate') }}
          </v-btn>
          <v-btn
            color="primary"
            prepend-icon="mdi-cube-send"
            class="action-btn generate-btn"
            @click="generate"
          >
            {{ $t('project.generateMod') }}
          </v-btn>
        </div>
      </div>
    </div>

    <v-divider opacity="0.2" />

    <v-tabs v-model="tab" class="bureau-tabs" bg-color="transparent">
      <v-tab value="stations" prepend-icon="mdi-antenna">{{ $t('nav.stationEditor') }}</v-tab>
      <v-tab value="settings" prepend-icon="mdi-file-cog">{{ $t('nav.projectInfo') }}</v-tab>
    </v-tabs>

    <v-window v-model="tab" class="bureau-window">
      <v-window-item value="stations">
        <StationEditorView />
      </v-window-item>
      <v-window-item value="settings">
        <ProjectSettingsView />
      </v-window-item>
    </v-window>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import StationEditorView from '@/views/StationEditorView.vue'
import ProjectSettingsView from '@/views/ProjectSettingsView.vue'
import { useProjectStore } from '@/stores/project'
import { useAudioStore } from '@/stores/audio'
import { useCommand } from '@/composables/useCommand'
import { useToastStore } from '@/stores/toast'

interface ValidationReport {
  passed: boolean
  errors: string[]
  warnings: string[]
}

const tab = ref('stations')
const route = useRoute()
const router = useRouter()
const projectStore = useProjectStore()
const audioStore = useAudioStore()
const { run } = useCommand()
const toast = useToastStore()
const { t } = useI18n()

async function loadProject(id: string) {
  if (!id) return
  await projectStore.loadProjects()
  const found = projectStore.projects.find((p) => p.id === id)
  if (found) {
    projectStore.setCurrentProject(found)
    await audioStore.loadAudio(id)
  } else {
    projectStore.setCurrentProject(null)
    toast.display(t('project.notFound'), 'error', 4000)
    router.replace('/')
  }
}

watch(
  () => route.params.id as string,
  (id) => loadProject(id),
  { immediate: true }
)

async function generate() {
  if (!projectStore.currentProject) return
  const out = await run<string>('generate_project_mod', {
    projectId: projectStore.currentProject.id,
  })
  if (out) {
    toast.display(t('project.generatedTo', { path: out }), 'success', 6000)
  }
}

async function validate() {
  if (!projectStore.currentProject) return
  const report = await run<ValidationReport>('validate_project_mod', {
    projectId: projectStore.currentProject.id,
  })
  if (report) {
    const status = report.passed ? t('project.validatePassed') : t('project.validateFailed')
    toast.display(
      t('project.validateSummary', {
        status,
        errors: report.errors.length,
        warnings: report.warnings.length,
      }),
      report.passed ? 'success' : 'error',
      6000
    )
  }
}
</script>

<style scoped>
.project-view {
  min-height: 100vh;
}

.project-header {
  background: linear-gradient(180deg, rgba(var(--v-theme-primary), 0.04) 0%, transparent 100%);
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.generate-btn {
  box-shadow: 0 0 18px rgba(var(--v-theme-primary), 0.2);
}

.bureau-tabs :deep(.v-tab) {
  text-transform: none;
  letter-spacing: 0.03em;
  font-weight: 600;
}

.bureau-window {
  background: rgba(var(--v-theme-background), 0.3);
  min-height: calc(100vh - 220px);
}
</style>
