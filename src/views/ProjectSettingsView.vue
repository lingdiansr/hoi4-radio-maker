<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">PROJECT DOSSIER</div>
        <div class="text-display text-h5">项目设置</div>
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
              保存项目设置
            </v-btn>

            <v-divider class="my-6" opacity="0.2" />

            <div class="text-mono text-caption text-secondary mb-3">PROJECT AUDIO REFERENCES</div>
            <v-card class="ref-card" variant="flat" rounded="lg">
              <v-card-title class="d-flex justify-space-between align-center pa-4">
                <span class="text-body text-subtitle-1 font-weight-medium">项目引用的音频</span>
                <v-btn
                  color="primary"
                  variant="text"
                  size="small"
                  prepend-icon="mdi-music-box-multiple"
                  class="action-btn"
                  @click="showPicker = true"
                >
                  从音频库添加
                </v-btn>
              </v-card-title>
              <v-card-text class="pa-4 pt-0">
                <v-list v-if="audioStore.audioFiles.length > 0" bg-color="transparent">
                  <v-list-item
                    v-for="audio in audioStore.audioFiles"
                    :key="audio.id"
                    class="ref-item mb-2"
                    rounded="lg"
                  >
                    <template #prepend>
                      <v-icon color="primary" class="mr-3">mdi-music-note</v-icon>
                    </template>
                    <v-list-item-title>{{ audio.title }}</v-list-item-title>
                    <v-list-item-subtitle>
                      {{ formatDuration(audio.duration_secs) }} · {{ audio.sample_rate }} Hz
                    </v-list-item-subtitle>
                    <template #append>
                      <v-btn
                        icon="mdi-link-off"
                        variant="text"
                        size="small"
                        color="error"
                        title="移除引用"
                        @click="removeRef(audio.id)"
                      />
                    </template>
                  </v-list-item>
                </v-list>
                <div v-else class="text-body text-secondary text-center py-4">
                  暂无引用音频，请从音频库添加
                </div>
              </v-card-text>
            </v-card>
          </v-col>

          <v-col cols="12" lg="4">
            <v-card class="hint-card" variant="flat" rounded="lg">
              <v-card-text>
                <v-icon color="primary" size="32" class="mb-2">mdi-information-outline</v-icon>
                <div class="text-body text-secondary text-body-2">
                  项目名称、版本与输出目录会写入生成的 Mod 描述文件。修改输出目录不会影响已导入的音频文件。
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <AudioPickerDialog
      v-model="showPicker"
      @confirm="onAudioSelected"
    />
  </div>
</template>

<script setup lang="ts">
import { reactive, watch, ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useProjectStore, type UpdateProjectRequest } from '@/stores/project'
import { useAudioStore } from '@/stores/audio'
import { useCommand } from '@/composables/useCommand'
import PathField from '@/components/PathField.vue'
import AudioPickerDialog from '@/components/AudioPickerDialog.vue'

const route = useRoute()
const projectStore = useProjectStore()
const audioStore = useAudioStore()
const { run } = useCommand()

const saving = ref(false)
const showPicker = ref(false)

const projectId = computed(() => route.params.id as string)

onMounted(() => {
  if (projectId.value) {
    audioStore.loadAudio(projectId.value)
  }
})

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
  form.name = p.name
  form.version = p.version
  form.supported_version = p.supported_version
  form.tags = [...p.tags]
  form.author = p.author ?? undefined
  form.output_dir = p.output_dir
}

watch(
  () => projectStore.currentProject?.id,
  () => syncFromProject(),
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
      { successMsg: '项目设置已保存' }
    )
  } finally {
    saving.value = false
  }
}

async function onAudioSelected(audioIds: string[]) {
  if (!projectId.value) return
  await audioStore.addToProject(projectId.value, audioIds)
}

async function removeRef(audioId: string) {
  if (!projectId.value) return
  await audioStore.removeFromProject(projectId.value, audioId)
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  return `${m}:${String(s).padStart(2, '0')}`
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
  height: 100%;
}

.ref-card {
  background: rgba(37, 33, 28, 0.5);
  border: 1px solid rgba(74, 66, 56, 0.3);
}

.ref-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.ref-item:hover {
  background: rgba(255, 176, 32, 0.06) !important;
  border-color: rgba(255, 176, 32, 0.2);
}
</style>
