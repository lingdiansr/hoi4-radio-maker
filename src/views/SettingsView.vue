<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-start pa-6 pb-2">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">GLOBAL CONFIGURATION</div>
          <div class="text-display text-h5">全局设置</div>
        </div>
        <v-btn
          variant="text"
          prepend-icon="mdi-arrow-left"
          class="back-btn"
          @click="router.back()"
        >
          返回
        </v-btn>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-row>
          <v-col cols="12" md="8">
            <PathField
              v-model="ffmpegPath"
              label="ffmpeg 路径"
              placeholder="选择 ffmpeg 可执行文件"
              prepend-inner-icon="mdi-movie-play"
              picker-mode="file"
              class="mb-4"
            />
            <PathField
              v-model="ffprobePath"
              label="ffprobe 路径"
              placeholder="选择 ffprobe 可执行文件"
              prepend-inner-icon="mdi-magnify-scan"
              picker-mode="file"
              class="mb-4"
            />
            <PathField
              v-model="hoi4Path"
              label="HOI4 游戏目录"
              placeholder="选择 Hearts of Iron IV 安装目录"
              prepend-inner-icon="mdi-folder-open"
              picker-mode="directory"
              class="mb-4"
            />
            <v-slider
              v-model="settings.import_concurrency"
              label="导入并发数"
              min="1"
              max="16"
              step="1"
              thumb-label
              prepend-icon="mdi-swap-horizontal"
              class="mb-4"
              hide-details="auto"
            />
            <v-select
              v-model="settings.theme"
              label="主题"
              :items="themeOptions"
              item-title="label"
              item-value="value"
              prepend-inner-icon="mdi-palette"
              class="mb-6"
              hide-details="auto"
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
                  全局设置会保存在应用数据目录中，对所有项目生效。启动时会自动探测 ffmpeg 与 ffprobe；若未找到且未手动指定，将提示错误。导入并发数控制同时计算文件哈希的并行度（ffmpeg 转码会在此基础上减半运行）。
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>

    <v-card class="settings-card mt-6" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">DIAGNOSTICS</div>
        <div class="text-display text-h5">日志与诊断</div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div class="text-body text-secondary mb-4">
          日志文件保存在应用日志目录中，遇到问题时可用于排查。当前日志级别：Info。
        </div>
        <div class="d-flex gap-3 flex-wrap">
          <v-btn
            color="primary"
            prepend-icon="mdi-folder-open-outline"
            @click="openLogFolder"
          >
            打开日志文件夹
          </v-btn>
          <v-btn
            variant="outlined"
            prepend-icon="mdi-content-copy"
            @click="copyLogPath"
          >
            复制日志路径
          </v-btn>
        </div>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { appLogDir } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { invokeCommand } from '@/api/client'
import { useCommand } from '@/composables/useCommand'
import { logger } from '@/utils/logger'
import PathField from '@/components/PathField.vue'

const router = useRouter()

interface Settings {
  ffmpeg_path: string | null
  ffprobe_path: string | null
  hoi4_game_dir: string | null
  theme: string
  import_concurrency: number
}

const { run } = useCommand()

const settings = reactive<Settings>({
  ffmpeg_path: null,
  ffprobe_path: null,
  hoi4_game_dir: null,
  theme: 'dark',
  import_concurrency: 8,
})

const ffmpegPath = computed({
  get: () => settings.ffmpeg_path ?? '',
  set: (v: string) => {
    settings.ffmpeg_path = v.trim() || null
  },
})

const ffprobePath = computed({
  get: () => settings.ffprobe_path ?? '',
  set: (v: string) => {
    settings.ffprobe_path = v.trim() || null
  },
})

const hoi4Path = computed({
  get: () => settings.hoi4_game_dir ?? '',
  set: (v: string) => {
    settings.hoi4_game_dir = v.trim() || null
  },
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

async function openLogFolder() {
  try {
    const dir = await appLogDir()
    await openPath(dir)
    logger.info(`Opened log folder: ${dir}`)
  } catch (err) {
    logger.error(`Failed to open log folder: ${err}`)
  }
}

async function copyLogPath() {
  try {
    const dir = await appLogDir()
    await navigator.clipboard.writeText(dir)
    logger.info(`Copied log path: ${dir}`)
  } catch (err) {
    logger.error(`Failed to copy log path: ${err}`)
  }
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

.back-btn {
  text-transform: none;
  letter-spacing: 0.02em;
  color: #c4b5a0;
}

.back-btn:hover {
  color: #ffb020;
}

.hint-card {
  background: rgba(255, 176, 32, 0.06);
  border: 1px solid rgba(255, 176, 32, 0.2);
  height: 100%;
}
</style>
