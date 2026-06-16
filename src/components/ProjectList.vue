<template>
  <div class="pa-4">
    <v-btn color="primary" block @click="showDialog = true">
      新建项目
    </v-btn>
    <v-divider class="my-4" />
    <v-list>
      <v-list-item
        v-for="p in projectStore.projects"
        :key="p.id"
        :title="p.name"
        :subtitle="p.version"
        @click="selectProject(p)"
      />
    </v-list>

    <v-dialog v-model="showDialog" max-width="500">
      <v-card>
        <v-card-title>新建项目</v-card-title>
        <v-card-text>
          <v-text-field v-model="form.name" label="名称" />
          <v-text-field v-model="form.version" label="版本" />
          <v-text-field v-model="form.supported_version" label="支持的游戏版本" />
          <v-text-field v-model="form.output_dir" label="输出目录" />
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn text @click="showDialog = false">取消</v-btn>
          <v-btn color="primary" @click="handleCreate">创建</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore, type Project } from '@/stores/project'

const projectStore = useProjectStore()
const router = useRouter()
const showDialog = ref(false)
const form = reactive({
  name: 'My Radio Mod',
  version: '0.1.0',
  supported_version: '*',
  output_dir: '',
})

onMounted(() => {
  projectStore.loadProjects()
})

async function handleCreate() {
  const p = await projectStore.createProject({
    ...form,
    tags: ['Sound'],
    author: undefined,
  })
  if (p) {
    showDialog.value = false
    router.push({ name: 'project', params: { id: p.id } })
  }
}

function selectProject(p: Project) {
  projectStore.setCurrentProject(p)
  router.push({ name: 'project', params: { id: p.id } })
}
</script>
