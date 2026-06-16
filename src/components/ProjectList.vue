<template>
  <div class="project-list pa-4">
    <div class="list-header mb-6">
      <div class="text-display text-h5 mb-1">Projects</div>
      <div class="text-mono text-caption text-secondary">ARCHIVE</div>
    </div>

    <v-btn
      color="primary"
      block
      size="large"
      prepend-icon="mdi-plus-circle"
      class="create-btn mb-6"
      @click="showDialog = true"
    >
      新建项目
    </v-btn>

    <v-divider class="mb-4" opacity="0.2" />

    <div v-if="projectStore.projects.length === 0" class="empty-state text-center py-8">
      <v-icon size="48" color="secondary" class="mb-2">mdi-archive-outline</v-icon>
      <div class="text-body text-secondary">暂无项目</div>
    </div>

    <v-list v-else class="project-list-items" bg-color="transparent">
      <v-list-item
        v-for="(p, index) in projectStore.projects"
        :key="p.id"
        :title="p.name"
        :subtitle="`${p.version} · ${formatDate(p.output_dir)}`"
        class="project-item mb-2"
        rounded="lg"
        @click="selectProject(p)"
      >
        <template #prepend>
          <div class="project-index text-mono">{{ String(index + 1).padStart(2, '0') }}</div>
        </template>
        <template #append>
          <v-btn
            icon="mdi-delete-outline"
            variant="text"
            size="small"
            color="error"
            class="delete-btn"
            @click.stop="confirmDelete(p)"
          />
        </template>
      </v-list-item>
    </v-list>

    <v-dialog v-model="showDialog" max-width="520" class="bureau-dialog">
      <v-card>
        <v-card-title class="text-display text-h5 pa-6 pb-2">新建广播项目</v-card-title>
        <v-card-text class="pa-6 pt-4">
          <v-text-field v-model="form.name" label="项目名称" placeholder="例如：东方之声" />
          <v-text-field v-model="form.version" label="Mod 版本" />
          <v-text-field v-model="form.supported_version" label="支持的游戏版本" />
          <v-text-field v-model="form.output_dir" label="输出目录" placeholder="/path/to/mod/output" />
        </v-card-text>
        <v-card-actions class="pa-6 pt-0">
          <v-spacer />
          <v-btn variant="text" @click="showDialog = false">取消</v-btn>
          <v-btn color="primary" @click="handleCreate">创建</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <v-dialog v-model="showDeleteDialog" max-width="400">
      <v-card>
        <v-card-title class="text-display text-h6 pa-6 pb-2">确认删除</v-card-title>
        <v-card-text class="pa-6 pt-4">
          确定要删除项目 <strong>{{ projectToDelete?.name }}</strong> 吗？此操作不会删除输出目录中的文件。
        </v-card-text>
        <v-card-actions class="pa-6 pt-0">
          <v-spacer />
          <v-btn variant="text" @click="showDeleteDialog = false">取消</v-btn>
          <v-btn color="error" @click="handleDelete">删除</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore, type Project } from '@/stores/project'

const projectStore = useProjectStore()
const router = useRouter()
const showDialog = ref(false)
const showDeleteDialog = ref(false)
const projectToDelete = ref<Project | null>(null)
const form = reactive({
  name: 'My Radio Mod',
  version: '0.1.0',
  supported_version: '*',
  output_dir: '',
})

onMounted(() => {
  projectStore.loadProjects()
  window.addEventListener('bureau:create-project', openCreateDialog)
})

onUnmounted(() => {
  window.removeEventListener('bureau:create-project', openCreateDialog)
})

function openCreateDialog() {
  showDialog.value = true
}

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

function confirmDelete(p: Project) {
  projectToDelete.value = p
  showDeleteDialog.value = true
}

async function handleDelete() {
  if (!projectToDelete.value) return
  await projectStore.deleteProject(projectToDelete.value.id)
  showDeleteDialog.value = false
  projectToDelete.value = null
}

function formatDate(path: string) {
  return path.split('/').pop() || path
}
</script>

<style scoped>
.project-list {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.list-header {
  border-bottom: 1px solid rgba(74, 66, 56, 0.4);
  padding-bottom: 12px;
}

.create-btn {
  font-size: 0.95rem;
}

.project-list-items {
  overflow-y: auto;
}

.project-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.project-item:hover {
  background: rgba(255, 176, 32, 0.08) !important;
  border-color: rgba(255, 176, 32, 0.25);
}

.project-index {
  width: 28px;
  color: #ffb020;
  font-size: 0.75rem;
  opacity: 0.7;
}

.delete-btn {
  opacity: 0;
  transition: opacity 0.2s ease;
}

.project-item:hover .delete-btn {
  opacity: 1;
}

.empty-state {
  opacity: 0.6;
}

.bureau-dialog :deep(.v-overlay__scrim) {
  background: rgba(18, 16, 14, 0.8);
}
</style>
