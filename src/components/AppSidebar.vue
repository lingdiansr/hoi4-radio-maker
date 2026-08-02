<template>
  <v-navigation-drawer permanent width="300" class="bureau-sider" :rail="false">
    <div class="sidebar-inner d-flex flex-column h-100">
      <div class="brand pa-6 pb-4">
        <div class="text-display text-h5 brand-title">Radio Bureau</div>
        <div class="text-mono text-caption text-secondary">HOI4 RADIO MAKER</div>
      </div>

      <v-divider opacity="0.2" class="mx-4" />

      <div class="pa-4">
        <v-btn
          color="primary"
          prepend-icon="mdi-plus-circle"
          class="create-btn w-100"
          height="44"
          @click="showCreateDialog = true"
        >
          新建项目
        </v-btn>
      </div>

      <div class="nav-section px-4 pb-2">
        <div class="text-mono text-caption text-secondary nav-label">WORKSPACE</div>
      </div>

      <v-list class="project-list-items" bg-color="transparent" density="compact">
        <v-list-item
          v-for="(p, index) in projectStore.projects"
          :key="p.id"
          :title="p.name"
          :subtitle="`${p.version} · ${formatDate(p.output_dir)}`"
          class="project-item mb-2"
          rounded="lg"
          :active="isProjectActive(p.id)"
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

      <v-divider opacity="0.2" class="mx-4" />

      <div class="global-nav pa-4">
        <v-btn
          variant="text"
          prepend-icon="mdi-music-box-multiple"
          class="nav-link w-100 justify-start"
          :class="{ active: route.path === '/audio' }"
          @click="router.push('/audio')"
        >
          音频库
        </v-btn>
        <v-btn
          variant="text"
          prepend-icon="mdi-cog"
          class="nav-link w-100 justify-start"
          :class="{ active: route.path === '/settings' }"
          @click="router.push('/settings')"
        >
          全局设置
        </v-btn>
      </div>
    </div>

    <!-- Create Project Dialog -->
    <v-dialog v-model="showCreateDialog" max-width="560" class="bureau-dialog" persistent>
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
            v-model="form.library_dir"
            label="项目库目录"
            placeholder="选择项目库目录"
            prepend-inner-icon="mdi-folder-open"
            picker-mode="directory"
            class="mb-4"
            :rules="[required]"
          />
          <v-text-field
            :model-value="projectDir"
            label="项目目录"
            placeholder="自动根据项目库目录与名称生成"
            prepend-inner-icon="mdi-folder-cog"
            class="mb-4"
            hide-details="auto"
            readonly
          />
          <v-text-field
            v-model="form.author"
            label="作者"
            placeholder="可选"
            prepend-inner-icon="mdi-account"
            hide-details="auto"
            class="mb-4"
          />
          <v-combobox
            v-model="form.tags"
            label="标签"
            placeholder="输入后按回车添加"
            prepend-inner-icon="mdi-tag-multiple"
            multiple
            chips
            clearable
            hide-details="auto"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showCreateDialog = false">取消</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="handleCreate"
          >
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
          确定要删除项目 <strong class="text-primary">{{ projectToDelete?.name }}</strong> 吗？此操作不会删除全局音频库中的音频。
          <v-checkbox
            v-model="deleteFiles"
            label="同时删除项目目录及 .mod 文件（不可恢复）"
            color="error"
            hide-details
            density="compact"
            class="mt-4"
          />
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
  </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectStore, type Project } from '@/stores/project'
import { useSettingsStore } from '@/stores/settings'
import PathField from '@/components/PathField.vue'
import { sanitizeFolderName } from '@/utils/sanitize'
import { logger } from '@/utils/logger'

const projectStore = useProjectStore()
const settingsStore = useSettingsStore()
const router = useRouter()
const route = useRoute()

const showCreateDialog = ref(false)
const showDeleteDialog = ref(false)
const projectToDelete = ref<Project | null>(null)
const deleteFiles = ref(false)
const form = reactive({
  name: 'My Radio Mod',
  version: '0.1.0',
  supported_version: '*',
  library_dir: '',
  author: '',
  tags: ['Sound'],
})

function required(v: string) {
  return !!v.trim() || '此项为必填'
}

const projectDir = computed(() => {
  if (!form.library_dir) return ''
  const base = form.library_dir.replace(/[\\/]+$/, '')
  const folder = sanitizeFolderName(form.name)
  return `${base}/${folder}/${folder}`
})

onMounted(() => {
  projectStore.loadProjects()
  settingsStore.loadSettings()
})

watch(showCreateDialog, async (visible) => {
  if (!visible) return
  await settingsStore.loadSettings()
  const s = settingsStore.settings
  if (!s) return
  form.name = 'My Radio Mod'
  form.version = s.default_version || '0.1.0'
  form.supported_version = settingsStore.effectiveSupportedVersion || '*'
  form.author = s.default_author || ''
  form.tags = s.default_tags.length ? [...s.default_tags] : ['Sound']
  form.library_dir = await settingsStore.getDefaultLibraryDir()
})

function isProjectActive(id: string) {
  return route.name === 'project' && route.params.id === id
}

async function handleCreate() {
  if (!form.name || !form.version || !form.supported_version || !form.library_dir) return
  const p = await projectStore.createProject({
    name: form.name,
    version: form.version,
    supported_version: form.supported_version,
    output_dir: form.library_dir,
    tags: form.tags.length ? form.tags : ['Sound'],
    author: form.author.trim() || undefined,
  })
  if (p) {
    showCreateDialog.value = false
    router.push({ name: 'project', params: { id: p.id } })
  }
}

function selectProject(p: Project) {
  projectStore.setCurrentProject(p)
  router.push({ name: 'project', params: { id: p.id } })
}

function confirmDelete(p: Project) {
  projectToDelete.value = p
  deleteFiles.value = false
  showDeleteDialog.value = true
}

async function handleDelete() {
  if (!projectToDelete.value) return
  const deletedId = projectToDelete.value.id
  try {
    await projectStore.deleteProject(deletedId, deleteFiles.value)
    showDeleteDialog.value = false
    projectToDelete.value = null
    deleteFiles.value = false
    if (projectStore.projects.length === 0) {
      router.push({ name: 'welcome' })
    } else if (route.name === 'project' && route.params.id === deletedId) {
      router.push({ name: 'welcome' })
    }
  } catch (err) {
    logger.error(`Failed to delete project: ${err}`)
  }
}

function formatDate(path: string) {
  return path.split('/').pop() || path
}
</script>

<style scoped>
.bureau-sider {
  background: rgba(26, 23, 20, 0.92) !important;
  backdrop-filter: blur(10px);
  border-right: 1px solid rgba(74, 66, 56, 0.4);
}

.brand {
  border-bottom: 1px solid rgba(74, 66, 56, 0.3);
}

.brand-title {
  letter-spacing: 0.04em;
  color: #ffb020;
}

.create-btn {
  font-size: 0.95rem;
  text-transform: none;
  letter-spacing: 0.02em;
}

.nav-section {
  margin-top: 8px;
}

.nav-label {
  opacity: 0.7;
  letter-spacing: 0.08em;
  font-size: 0.65rem;
}

.project-list-items {
  flex: 1 1 auto;
  overflow-y: auto;
  min-height: 0;
  padding: 0 12px;
}

.project-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
  color: #c4b5a0;
}

.project-item:hover {
  background: rgba(255, 176, 32, 0.08) !important;
  border-color: rgba(255, 176, 32, 0.25);
}

.project-item.v-list-item--active {
  background: rgba(255, 176, 32, 0.14) !important;
  border-color: rgba(255, 176, 32, 0.4);
  color: #ffb020;
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

.global-nav {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-link {
  color: #c4b5a0;
  text-transform: none;
  letter-spacing: 0.02em;
  justify-content: flex-start;
  padding-left: 16px;
  transition: color 0.2s ease, background 0.2s ease;
}

.nav-link:hover,
.nav-link.active {
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
