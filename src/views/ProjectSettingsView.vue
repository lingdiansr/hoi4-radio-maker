<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">PROJECT DOSSIER</div>
        <div class="text-display text-h5">项目信息</div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-row>
          <v-col cols="12" lg="8">
            <v-text-field
              v-model="form.name"
              label="项目名称"
              placeholder="例如：东方之声"
              prepend-inner-icon="mdi-radio-tower"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              v-model="form.version"
              label="Mod 版本"
              placeholder="0.1.0"
              prepend-inner-icon="mdi-tag-outline"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              v-model="form.supported_version"
              label="支持的游戏版本"
              placeholder="*"
              prepend-inner-icon="mdi-gamepad-variant-outline"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              :model-value="form.output_dir"
              label="项目目录"
              prepend-inner-icon="mdi-folder-open"
              class="mb-4"
              hide-details="auto"
              readonly
            />
            <v-text-field
              v-model="authorInput"
              label="作者"
              placeholder="可选"
              prepend-inner-icon="mdi-account-edit"
              class="mb-4"
              hide-details="auto"
            />
            <v-combobox
              v-model="form.tags"
              label="标签"
              placeholder="输入后按回车添加"
              prepend-inner-icon="mdi-tag-multiple"
              multiple
              chips
              class="mb-6"
              hide-details="auto"
            />

            <v-btn
              color="primary"
              size="large"
              prepend-icon="mdi-content-save"
              class="save-btn"
              :loading="saving"
              @click="save"
            >
              保存项目信息
            </v-btn>

          </v-col>

          <v-col cols="12" lg="4" align-self="start">
            <v-card class="hint-card" variant="flat" rounded="lg">
              <v-card-text>
                <v-icon color="primary" size="32" class="mb-2">mdi-information-outline</v-icon>
                <div class="text-body text-secondary text-body-2">
                  项目名称、版本与项目目录会写入生成的 Mod 描述文件。项目目录在项目创建时确定，此处仅作查看；如需更改，请新建项目。
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <!-- Dirty change guard -->
    <v-dialog v-model="showDiscardDialog" max-width="460" class="bureau-dialog" persistent>
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-alert-circle-outline</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">UNSAVED CHANGES</div>
              <div class="text-display text-h5">放弃未保存的修改？</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          项目信息已被修改但尚未保存。切换项目将放弃这些更改。
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="cancelDiscard">取消</v-btn>
          <v-btn color="primary" class="action-btn" prepend-icon="mdi-check-circle" @click="confirmDiscard">
            放弃修改
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch, ref, computed, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore, type UpdateProjectRequest } from '@/stores/project'
import { useCommand } from '@/composables/useCommand'

const router = useRouter()
const projectStore = useProjectStore()
const { run } = useCommand()

const saving = ref(false)
const isDirty = ref(false)
const isSyncing = ref(false)
const isReverting = ref(false)
const showDiscardDialog = ref(false)
const pendingProjectId = ref<string | null>(null)
const previousProjectId = ref<string | null>(null)

const form = reactive<UpdateProjectRequest>({
  name: '',
  version: '',
  supported_version: '',
  tags: [],
  author: undefined,
  output_dir: '',
})

const authorInput = computed({
  get: () => form.author ?? '',
  set: (v: string) => {
    form.author = v.trim() || undefined
  },
})

function required(v: string) {
  return !!v || '此项为必填'
}

function syncFromProject() {
  const p = projectStore.currentProject
  if (!p) return
  isSyncing.value = true
  form.name = p.name
  form.version = p.version
  form.supported_version = p.supported_version
  form.tags = [...p.tags]
  form.author = p.author ?? undefined
  form.output_dir = p.output_dir
  isDirty.value = false
  nextTick(() => {
    isSyncing.value = false
  })
}

watch(
  form,
  () => {
    if (isSyncing.value) return
    isDirty.value = true
  },
  { deep: true }
)

watch(
  () => projectStore.currentProject?.id,
  (newId, oldId) => {
    if (isReverting.value) return
    if (!newId) {
      syncFromProject()
      return
    }
    if (isDirty.value) {
      pendingProjectId.value = newId
      previousProjectId.value = oldId ?? null
      showDiscardDialog.value = true
      return
    }
    syncFromProject()
  },
  { immediate: true }
)

async function save() {
  const p = projectStore.currentProject
  if (!p) return
  saving.value = true
  try {
    await run(
      'update_project',
      { id: p.id, req: { ...form } },
      { successMsg: '项目信息已保存' }
    )
    isDirty.value = false
  } finally {
    saving.value = false
  }
}

function confirmDiscard() {
  showDiscardDialog.value = false
  isDirty.value = false
  syncFromProject()
  pendingProjectId.value = null
}

function cancelDiscard() {
  showDiscardDialog.value = false
  const prevId = previousProjectId.value
  pendingProjectId.value = null
  previousProjectId.value = null
  if (!prevId) return
  const prev = projectStore.projects.find((p) => p.id === prevId) ?? null
  isReverting.value = true
  projectStore.setCurrentProject(prev)
  router.replace({ name: 'project', params: { id: prevId } })
  nextTick(() => {
    isReverting.value = false
  })
}

</script>

<style scoped>
.settings-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
}

.save-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.hint-card {
  background: rgba(255, 176, 32, 0.06);
  border: 1px solid rgba(255, 176, 32, 0.2);
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

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
