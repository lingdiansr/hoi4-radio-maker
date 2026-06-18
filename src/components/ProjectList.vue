<template>
  <div class="project-list pa-4">
    <div class="list-header mb-6">
      <div class="text-display text-h5 mb-1">Projects</div>
      <div class="text-mono text-caption text-secondary">ARCHIVE</div>
    </div>

    <v-btn
      color="primary"
      prepend-icon="mdi-plus-circle"
      class="create-btn mb-6 w-100"
      height="44"
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

    <v-spacer />

    <div class="sidebar-footer pa-4">
      <v-btn
        variant="text"
        prepend-icon="mdi-cog"
        class="settings-link w-100 justify-start"
        @click="router.push('/settings')"
      >
        全局设置
      </v-btn>
    </div>

    <!-- Create Project Dialog -->
    <v-dialog v-model="showDialog" max-width="560" class="bureau-dialog" persistent>
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-radio-tower</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">NEW BROADCAST</div>
              <div class="text-display text-h5">新建广播项目</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="form.name"
            label="项目名称"
            placeholder="例如：东方之声"
            prepend-inner-icon="mdi-form-textbox"
            class="mb-4"
            hide-details="auto"
            :rules="[required]"
          />
          <v-row class="mb-2">
            <v-col cols="6">
              <v-text-field
                v-model="form.version"
                label="Mod 版本"
                placeholder="0.1.0"
                prepend-inner-icon="mdi-tag-outline"
                hide-details="auto"
                :rules="[required]"
              />
            </v-col>
            <v-col cols="6">
              <v-text-field
                v-model="form.supported_version"
                label="支持的游戏版本"
                placeholder="*"
                prepend-inner-icon="mdi-gamepad-variant-outline"
                hide-details="auto"
                :rules="[required]"
              />
            </v-col>
          </v-row>
          <PathField
            v-model="form.output_dir"
            label="输出目录"
            placeholder="选择 Mod 输出目录"
            prepend-inner-icon="mdi-folder-open"
            picker-mode="directory"
            class="mb-4"
            :rules="[required]"
          />
          <v-text-field
            v-model="form.author"
            label="作者"
            placeholder="可选"
            prepend-inner-icon="mdi-account"
            hide-details="auto"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showDialog = false">取消</v-btn>
          <v-btn color="primary" class="action-btn" prepend-icon="mdi-check-circle" @click="handleCreate">
            创建
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Delete Confirmation Dialog -->
    <v-dialog v-model="showDeleteDialog" max-width="420" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent dialog-accent--danger" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="error" size="28">mdi-alert-circle</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">CONFIRM DELETION</div>
              <div class="text-display text-h5">确认删除</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          确定要删除项目 <strong class="text-primary">{{ projectToDelete?.name }}</strong> 吗？此操作不会删除输出目录中的文件。
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showDeleteDialog = false">取消</v-btn>
          <v-btn color="error" class="action-btn" prepend-icon="mdi-delete-outline" @click="handleDelete">
            删除
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore, type Project } from '@/stores/project'
import PathField from '@/components/PathField.vue'

const router = useRouter()

const projectStore = useProjectStore()
const showDialog = ref(false)
const showDeleteDialog = ref(false)
const projectToDelete = ref<Project | null>(null)
const form = reactive({
  name: 'My Radio Mod',
  version: '0.1.0',
  supported_version: '*',
  output_dir: '',
  author: '',
})

function required(v: string) {
  return !!v || '此项为必填'
}

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
  if (!form.name || !form.version || !form.supported_version || !form.output_dir) return
  const p = await projectStore.createProject({
    ...form,
    tags: ['Sound'],
    author: form.author.trim() || undefined,
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
  flex: 1 1 auto;
  overflow-y: auto;
  min-height: 0;
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

.sidebar-footer {
  border-top: 1px solid rgba(74, 66, 56, 0.4);
  margin-top: auto;
}

.settings-link {
  color: #c4b5a0;
  text-transform: none;
  letter-spacing: 0.02em;
  justify-content: flex-start;
  transition: color 0.2s ease, background 0.2s ease;
}

.settings-link:hover {
  color: #ffb020;
  background: rgba(255, 176, 32, 0.08);
}

.dialog-card {
  background: #1a1714;
  border: 1px solid rgba(74, 66, 56, 0.5);
  position: relative;
  overflow: hidden;
}

.dialog-accent {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, #ffb020 0%, rgba(255, 176, 32, 0.3) 100%);
}

.dialog-accent--danger {
  background: linear-gradient(90deg, #ff8a80 0%, rgba(255, 138, 128, 0.3) 100%);
}

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
