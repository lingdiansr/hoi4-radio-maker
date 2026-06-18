<template>
  <v-layout style="min-height: 100vh">
    <v-navigation-drawer permanent width="300" class="bureau-sider">
      <ProjectList />
    </v-navigation-drawer>
    <v-main>
      <div class="project-header pa-6 pb-4">
        <div class="d-flex justify-space-between align-start">
          <div>
            <div class="text-mono text-caption text-secondary mb-1">PROJECT</div>
            <h1 class="text-display text-h3 mb-2">
              {{ projectStore.currentProject?.name || '未选择项目' }}
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
              验证
            </v-btn>
            <v-btn
              color="primary"
              prepend-icon="mdi-cube-send"
              class="action-btn generate-btn"
              @click="generate"
            >
              生成 Mod
            </v-btn>
          </div>
        </div>
      </div>

      <v-divider opacity="0.2" />

      <v-tabs v-model="tab" class="bureau-tabs" bg-color="transparent">
        <v-tab value="audio" prepend-icon="mdi-music-box-multiple">音频库</v-tab>
        <v-tab value="stations" prepend-icon="mdi-antenna">电台编辑</v-tab>
        <v-tab value="settings" prepend-icon="mdi-file-cog">项目设置</v-tab>
      </v-tabs>

      <v-window v-model="tab" class="bureau-window">
        <v-window-item value="audio">
          <AudioLibraryView />
        </v-window-item>
        <v-window-item value="stations">
          <StationEditorView />
        </v-window-item>
        <v-window-item value="settings">
          <ProjectSettingsView />
        </v-window-item>
      </v-window>
    </v-main>
  </v-layout>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import ProjectList from '@/components/ProjectList.vue'
import AudioLibraryView from '@/views/AudioLibraryView.vue'
import StationEditorView from '@/views/StationEditorView.vue'
import ProjectSettingsView from '@/views/ProjectSettingsView.vue'
import { useProjectStore } from '@/stores/project'
import { useCommand } from '@/composables/useCommand'
import { useToastStore } from '@/stores/toast'

const tab = ref('audio')
const route = useRoute()
const projectStore = useProjectStore()
const { run } = useCommand()
const toast = useToastStore()

onMounted(() => {
  const id = route.params.id as string
  const found = projectStore.projects.find((p) => p.id === id)
  if (found) {
    projectStore.setCurrentProject(found)
  }
})

async function generate() {
  if (!projectStore.currentProject) return
  const out = await run<string>('generate_project_mod', {
    projectId: projectStore.currentProject.id,
  })
  if (out) {
    toast.display(`已生成到: ${out}`, 'success', 6000)
  }
}

async function validate() {
  if (!projectStore.currentProject) return
  const report = await run<any>('validate_project_mod', {
    projectId: projectStore.currentProject.id,
  })
  if (report) {
    const status = report.passed ? '通过' : '未通过'
    toast.display(
      `验证${status} · 错误: ${report.errors.length} · 警告: ${report.warnings.length}`,
      report.passed ? 'success' : 'error',
      6000
    )
  }
}
</script>

<style scoped>
.bureau-sider {
  background: rgba(26, 23, 20, 0.85) !important;
  backdrop-filter: blur(8px);
}

.project-header {
  background: linear-gradient(180deg, rgba(255, 176, 32, 0.04) 0%, transparent 100%);
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.generate-btn {
  box-shadow: 0 0 18px rgba(255, 176, 32, 0.2);
}

.bureau-tabs :deep(.v-tab) {
  text-transform: none;
  letter-spacing: 0.03em;
  font-weight: 600;
}

.bureau-window {
  background: rgba(18, 16, 14, 0.3);
  min-height: calc(100vh - 220px);
}
</style>
