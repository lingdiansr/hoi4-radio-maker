<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">CONFIGURATION</div>
        <div class="text-display text-h5">设置</div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-row>
          <v-col cols="12" md="8">
            <v-text-field
              v-model="settings.ffmpeg_path"
              label="ffmpeg 路径"
              placeholder="ffmpeg"
              prepend-inner-icon="mdi-movie-play"
              class="mb-4"
            />
            <v-text-field
              v-model="settings.ffprobe_path"
              label="ffprobe 路径"
              placeholder="ffprobe"
              prepend-inner-icon="mdi-magnify-scan"
              class="mb-4"
            />
            <v-text-field
              v-model="settings.hoi4_game_dir"
              label="HOI4 游戏目录"
              placeholder="..."
              prepend-inner-icon="mdi-folder-open"
              class="mb-4"
            />
            <v-select
              v-model="settings.theme"
              label="主题"
              :items="themeOptions"
              item-title="label"
              item-value="value"
              prepend-inner-icon="mdi-palette"
              class="mb-6"
            />
            <v-btn
              color="primary"
              size="large"
              prepend-icon="mdi-content-save"
              class="save-btn"
              @click="save"
            >
              保存设置
            </v-btn>
          </v-col>

          <v-col cols="12" md="4">
            <v-card class="hint-card" variant="flat" rounded="lg">
              <v-card-text>
                <v-icon color="primary" size="32" class="mb-2">mdi-information-outline</v-icon>
                <div class="text-body text-secondary text-body-2">
                  如果 ffmpeg 和 ffprobe 不在系统 PATH 中，请在此指定完整路径。主题更改将在下次启动时完全生效。
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>
  </div>
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
</style>
