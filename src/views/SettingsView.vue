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
        <v-row v-if="settingsStore.settings">
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
            <PathField
              v-model="defaultProjectDir"
              label="默认项目根目录"
              placeholder="选择默认存放 mods 的目录"
              prepend-inner-icon="mdi-folder-cog"
              picker-mode="directory"
              class="mb-4"
            />
            <v-text-field
              v-model="settings.default_author"
              label="默认作者"
              placeholder="未设置时使用系统用户名"
              prepend-inner-icon="mdi-account"
              class="mb-4"
              hide-details="auto"
              clearable
            />
            <v-text-field
              v-model="settings.default_version"
              label="默认项目版本"
              placeholder="0.1.0"
              prepend-inner-icon="mdi-tag-outline"
              class="mb-4"
              hide-details="auto"
            />
            <v-text-field
              v-model="settings.default_supported_version"
              label="默认兼容游戏版本"
              placeholder="留空时自动探测 HOI4 版本"
              prepend-inner-icon="mdi-gamepad-variant"
              class="mb-4"
              hide-details="auto"
              clearable
            />
            <v-combobox
              v-model="settings.default_tags"
              label="默认标签"
              placeholder="输入后按回车添加"
              prepend-inner-icon="mdi-tag-multiple"
              multiple
              chips
              clearable
              class="mb-4"
              hide-details="auto"
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
              class="mb-4"
              hide-details="auto"
            />
            <v-alert
              v-if="!settingsStore.ffmpegAvailable"
              type="warning"
              variant="tonal"
              class="mb-6"
              text="未检测到 ffmpeg / ffprobe。请安装 ffmpeg 或在上方手动指定路径，否则无法导入音频。"
            />
            <v-btn
              color="primary"
              size="large"
              prepend-icon="mdi-content-save"
              class="save-btn"
              :loading="saving"
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
                  全局设置会保存在应用数据目录中，对所有项目生效。启动时会自动探测 ffmpeg 与 ffprobe；若未找到且未手动指定，将提示错误。
                  <br><br>
                  <strong>默认项目根目录</strong>用于新建项目时自动生成文件夹与 .mod 文件。<strong>默认兼容游戏版本</strong>留空时，会尝试从 HOI4 游戏目录读取 launcher-settings.json 自动填充。
                  <br><br>
                  导入并发数控制同时计算文件哈希的并行度（ffmpeg 转码会在此基础上减半运行）。
                </div>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
        <v-row v-else>
          <v-col cols="12" class="text-center py-8">
            <v-progress-circular indeterminate color="primary" />
            <div class="text-secondary mt-2">加载设置中...</div>
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
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { appLogDir } from '@tauri-apps/api/path'
import { openPath } from '@tauri-apps/plugin-opener'
import { useSettingsStore } from '@/stores/settings'
import { logger } from '@/utils/logger'
import PathField from '@/components/PathField.vue'

const router = useRouter()
const settingsStore = useSettingsStore()

const saving = ref(false)

const settings = computed({
  get: () => settingsStore.settings!,
  set: (v) => {
    settingsStore.settings = v
  },
})

const ffmpegPath = computed({
  get: () => settingsStore.settings?.ffmpeg_path ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.ffmpeg_path = v.trim() || undefined
    }
  },
})

const ffprobePath = computed({
  get: () => settingsStore.settings?.ffprobe_path ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.ffprobe_path = v.trim() || undefined
    }
  },
})

const hoi4Path = computed({
  get: () => settingsStore.settings?.hoi4_game_dir ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.hoi4_game_dir = v.trim() || undefined
    }
  },
})

const defaultProjectDir = computed({
  get: () => settingsStore.settings?.default_project_dir ?? '',
  set: (v: string) => {
    if (settingsStore.settings) {
      settingsStore.settings.default_project_dir = v.trim() || undefined
    }
  },
})

const themeOptions = [
  { label: '浅色', value: 'light' },
  { label: '深色', value: 'dark' },
]

onMounted(async () => {
  await settingsStore.loadSettings()
})

async function save() {
  saving.value = true
  try {
    await settingsStore.saveSettings(settings.value)
    logger.info('Settings saved')
  } finally {
    saving.value = false
  }
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
