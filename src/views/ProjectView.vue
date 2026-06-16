<template>
  <v-layout style="height: 100vh">
    <v-navigation-drawer permanent width="280">
      <ProjectList />
    </v-navigation-drawer>
    <v-main style="padding: 24px">
      <div class="d-flex justify-space-between align-center mb-4">
        <h2 class="text-h5">{{ projectStore.currentProject?.name }}</h2>
        <v-btn color="primary" @click="generate">生成 Mod</v-btn>
      </div>
      <v-tabs v-model="tab">
        <v-tab value="audio">音频库</v-tab>
        <v-tab value="stations">电台编辑</v-tab>
      </v-tabs>
      <v-window v-model="tab">
        <v-window-item value="audio">
          <AudioLibraryView />
        </v-window-item>
        <v-window-item value="stations">
          <StationEditorView />
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
import { useProjectStore } from '@/stores/project'
import { useCommand } from '@/composables/useCommand'

const tab = ref('audio')
const route = useRoute()
const projectStore = useProjectStore()
const { run } = useCommand()

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
    alert(`已生成到: ${out}`)
  }
}
</script>
