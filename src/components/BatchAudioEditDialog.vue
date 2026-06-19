<template>
  <v-dialog :model-value="modelValue" max-width="520" class="bureau-dialog" persistent @update:model-value="$emit('update:modelValue', $event)">
    <v-card class="dialog-card">
      <div class="dialog-accent" />
      <v-card-title class="dialog-title pa-6 pb-2">
        <div class="d-flex align-center gap-3">
          <v-icon color="primary" size="28">mdi-pencil-box-multiple</v-icon>
          <div>
            <div class="text-mono text-caption text-secondary">BATCH EDIT</div>
            <div class="text-display text-h5">批量编辑音频</div>
            <div class="text-secondary text-body-2 mt-1">已选择 {{ ids.length }} 项</div>
          </div>
        </div>
      </v-card-title>

      <v-card-text class="pa-6 pt-4">
        <v-checkbox
          v-model="apply.artist"
          label="修改艺术家"
          hide-details
          density="comfortable"
          class="mb-2"
        />
        <v-text-field
          v-model="form.artist"
          placeholder="留空表示清空"
          prepend-inner-icon="mdi-account-music"
          class="mb-4"
          hide-details="auto"
          :disabled="!apply.artist"
        />

        <v-checkbox
          v-model="apply.volume"
          label="修改音量"
          hide-details
          density="comfortable"
          class="mb-2"
        />
        <v-slider
          v-model="form.volume"
          min="0"
          max="1"
          step="0.05"
          thumb-label
          class="mb-4"
          hide-details="auto"
          :disabled="!apply.volume"
        />

        <v-checkbox
          v-model="apply.tags"
          label="替换标签"
          hide-details
          density="comfortable"
          class="mb-2"
        />
        <v-combobox
          v-model="form.tags"
          placeholder="输入后按回车添加"
          prepend-inner-icon="mdi-tag-multiple"
          multiple
          chips
          clearable
          class="mb-4"
          hide-details="auto"
          :disabled="!apply.tags"
        />
      </v-card-text>

      <v-divider opacity="0.2" />

      <v-card-actions class="pa-6">
        <v-spacer />
        <v-btn variant="text" class="action-btn" @click="close">取消</v-btn>
        <v-btn
          color="primary"
          class="action-btn"
          prepend-icon="mdi-content-save"
          :loading="saving"
          @click="save"
        >
          应用
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import { useAudioStore, type AudioFile } from '@/stores/audio'

const props = defineProps<{
  modelValue: boolean
  ids: string[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved', audio: AudioFile[]): void
}>()

const audioStore = useAudioStore()
const saving = ref(false)

const form = reactive({
  artist: '',
  volume: 0.75,
  tags: [] as string[],
})

const apply = reactive({
  artist: false,
  volume: false,
  tags: false,
})

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) {
      form.artist = ''
      form.volume = 0.75
      form.tags = []
      apply.artist = false
      apply.volume = false
      apply.tags = false
    }
  }
)

function close() {
  emit('update:modelValue', false)
}

async function save() {
  if (props.ids.length === 0) {
    close()
    return
  }
  saving.value = true
  try {
    const req: { artist?: string | null; volume?: number; tags?: string[] } = {}
    if (apply.artist) {
      req.artist = form.artist.trim() || null
    }
    if (apply.volume) {
      req.volume = form.volume
    }
    if (apply.tags) {
      req.tags = form.tags
    }
    const updated = await audioStore.batchUpdateAudio(props.ids, req)
    emit('saved', updated)
    close()
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
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
