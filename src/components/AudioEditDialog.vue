<template>
  <v-dialog :model-value="modelValue" max-width="520" class="bureau-dialog" persistent @update:model-value="$emit('update:modelValue', $event)">
    <v-card class="dialog-card">
      <div class="dialog-accent" />
      <v-card-title class="dialog-title pa-6 pb-2">
        <div class="d-flex align-center gap-3">
          <v-icon color="primary" size="28">mdi-pencil-circle</v-icon>
          <div>
            <div class="text-mono text-caption text-secondary">EDIT AUDIO</div>
            <div class="text-display text-h5">编辑音频信息</div>
          </div>
        </div>
      </v-card-title>

      <v-card-text class="pa-6 pt-4">
        <v-text-field
          v-model="form.title"
          label="标题"
          prepend-inner-icon="mdi-music-note"
          class="mb-4"
          hide-details="auto"
        />
        <v-text-field
          v-model="form.artist"
          label="艺术家"
          placeholder="可选"
          prepend-inner-icon="mdi-account-music"
          class="mb-4"
          hide-details="auto"
          clearable
        />
        <v-slider
          v-model="form.volume"
          label="音量"
          min="0"
          max="1"
          step="0.05"
          thumb-label
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
          clearable
          class="mb-4"
          hide-details="auto"
        />
        <v-textarea
          v-model="form.notes"
          label="备注"
          placeholder="可选"
          prepend-inner-icon="mdi-note-text"
          rows="3"
          hide-details="auto"
          clearable
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
          保存
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
  audio: AudioFile | null
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'saved', audio: AudioFile): void
}>()

const audioStore = useAudioStore()
const saving = ref(false)

const form = reactive({
  title: '',
  artist: '',
  volume: 0.75,
  tags: [] as string[],
  notes: '',
})

function resetForm() {
  const a = props.audio
  if (!a) return
  form.title = a.title
  form.artist = a.artist ?? ''
  form.volume = a.volume
  form.tags = a.tags.length ? [...a.tags] : []
  form.notes = a.notes ?? ''
}

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) resetForm()
  },
  { immediate: true }
)

watch(
  () => props.audio?.id,
  () => {
    if (props.modelValue) resetForm()
  }
)

function close() {
  emit('update:modelValue', false)
}

async function save() {
  if (!props.audio) return
  saving.value = true
  try {
    const updated = await audioStore.updateAudio(props.audio.id, {
      title: form.title,
      artist: form.artist.trim() || null,
      volume: form.volume,
      tags: form.tags,
      notes: form.notes.trim() || null,
    })
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
