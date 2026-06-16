<template>
  <v-container>
    <h2 class="text-h5 mb-4">设置</h2>
    <v-text-field v-model="settings.ffmpeg_path" label="ffmpeg 路径" placeholder="ffmpeg" />
    <v-text-field v-model="settings.ffprobe_path" label="ffprobe 路径" placeholder="ffprobe" />
    <v-text-field v-model="settings.hoi4_game_dir" label="HOI4 游戏目录" placeholder="..." />
    <v-select
      v-model="settings.theme"
      label="主题"
      :items="themeOptions"
      item-title="label"
      item-value="value"
    />
    <v-btn color="primary" @click="save">保存</v-btn>
  </v-container>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue'
import { invokeCommand } from '@/api/client'
import { useCommand } from '@/composables/useCommand'

interface Settings {
  ffmpeg_path: string | null
  ffprobe_path: string | null
  hoi4_game_dir: string | null
  theme: string
}

const { run } = useCommand()

const settings = reactive<Settings>({
  ffmpeg_path: null,
  ffprobe_path: null,
  hoi4_game_dir: null,
  theme: 'dark',
})

const themeOptions = [
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
]

onMounted(async () => {
  const saved = await invokeCommand<Settings>('get_settings')
  Object.assign(settings, saved)
})

async function save() {
  await run('save_settings', { settings }, { successMsg: '设置已保存' })
}
</script>
