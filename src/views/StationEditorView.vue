<template>
  <div class="pa-6">
    <v-card class="station-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-center pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">BROADCAST CHANNELS</div>
          <div class="text-display text-h5">电台编辑</div>
        </div>
        <v-btn
          color="primary"
          prepend-icon="mdi-plus"
          class="create-btn"
          @click="openCreateDialog"
        >
          新建电台
        </v-btn>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div v-if="stationStore.stations.length === 0" class="empty-state text-center py-12">
          <v-icon size="64" color="secondary" class="mb-4">mdi-antenna</v-icon>
          <div class="text-body text-secondary text-h6 mb-2">暂无电台</div>
          <div class="text-body text-secondary mb-4">创建一个电台，然后从音频库添加歌曲</div>
          <v-btn color="primary" prepend-icon="mdi-plus" @click="openCreateDialog">新建电台</v-btn>
        </div>

        <template v-else>
          <v-tabs v-model="activeTab" class="station-tabs" bg-color="transparent">
            <v-tab
              v-for="station in stationStore.stations"
              :key="station.id"
              :value="station.id"
              prepend-icon="mdi-radio"
            >
              {{ station.name }}
            </v-tab>
          </v-tabs>

          <v-window v-model="activeTab" class="mt-4">
            <v-window-item
              v-for="station in stationStore.stations"
              :key="station.id"
              :value="station.id"
            >
              <v-card class="entry-card" variant="flat" rounded="lg">
                <v-card-title class="d-flex justify-space-between align-center flex-wrap">
                  <span class="text-display text-h6">{{ station.name }}</span>
                  <div class="d-flex align-center gap-2">
                    <v-btn
                      icon="mdi-pencil"
                      variant="text"
                      size="small"
                      color="primary"
                      title="重命名"
                      @click.stop="openRename(station)"
                    />
                    <v-btn
                      icon="mdi-arrow-left"
                      variant="text"
                      size="small"
                      :disabled="isFirst(station.id)"
                      title="前移"
                      @click.stop="moveStation(station.id, -1)"
                    />
                    <v-btn
                      icon="mdi-arrow-right"
                      variant="text"
                      size="small"
                      :disabled="isLast(station.id)"
                      title="后移"
                      @click.stop="moveStation(station.id, 1)"
                    />
                    <v-chip size="small" color="primary" class="text-mono">
                      {{ station.entries.length }} tracks
                    </v-chip>
                    <v-btn
                      icon="mdi-delete-outline"
                      variant="text"
                      size="small"
                      color="error"
                      title="删除电台"
                      @click="confirmDelete(station)"
                    />
                  </div>
                </v-card-title>
                <v-card-text>
                  <v-list v-if="station.entries.length > 0" bg-color="transparent">
                    <v-list-item
                      v-for="(entry, idx) in station.entries"
                      :key="entry.audio_file_id"
                      class="entry-item mb-2"
                      rounded="lg"
                    >
                      <template #prepend>
                        <div class="d-flex flex-column align-center mr-2 reorder-controls">
                          <v-btn
                            icon="mdi-chevron-up"
                            variant="text"
                            density="compact"
                            size="x-small"
                            :disabled="idx === 0"
                            @click.stop="moveEntry(station.id, idx, -1)"
                          />
                          <v-btn
                            icon="mdi-chevron-down"
                            variant="text"
                            density="compact"
                            size="x-small"
                            :disabled="idx === station.entries.length - 1"
                            @click.stop="moveEntry(station.id, idx, 1)"
                          />
                        </div>
                        <v-icon color="primary" class="mr-4">mdi-music-note</v-icon>
                      </template>
                      <v-list-item-title>{{ audioTitle(entry.audio_file_id) }}</v-list-item-title>
                      <v-list-item-subtitle class="d-flex align-center py-2">
                        <v-text-field
                          v-model.number="entry.chance.factor"
                          type="number"
                          min="0"
                          step="0.1"
                          density="compact"
                          variant="outlined"
                          hide-details
                          label="factor"
                          class="factor-field"
                          @change="saveFactor(station.id, entry)"
                        />
                        <v-chip
                          size="x-small"
                          variant="tonal"
                          color="secondary"
                          class="ml-3"
                        >
                          {{ entry.chance.modifiers.length }} 个条件
                        </v-chip>
                      </v-list-item-subtitle>
                      <template #append>
                        <v-btn
                          icon="mdi-tune"
                          variant="text"
                          color="primary"
                          class="mr-1"
                          title="编辑播放条件"
                          @click.stop="openChanceEditor(station.id, entry)"
                        />
                        <v-btn
                          icon="mdi-delete-outline"
                          variant="text"
                          color="error"
                          @click.stop="removeEntry(station.id, entry.audio_file_id)"
                        />
                      </template>
                    </v-list-item>
                  </v-list>
                  <v-alert v-else color="secondary" variant="tonal" icon="mdi-information" text="该电台暂无歌曲，点击下方按钮从音频库添加" />

                  <v-divider class="my-4" opacity="0.2" />

                  <v-btn
                    color="primary"
                    variant="outlined"
                    prepend-icon="mdi-music-box-multiple"
                    class="action-btn"
                    @click="openPicker(station.id)"
                  >
                    从音频库添加歌曲
                  </v-btn>
                </v-card-text>
              </v-card>
            </v-window-item>
          </v-window>
        </template>
      </v-card-text>
    </v-card>

    <!-- Create Station Dialog -->
    <v-dialog v-model="showCreateDialog" max-width="460" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-radio</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">NEW CHANNEL</div>
              <div class="text-display text-h5">新建电台</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="newStationName"
            label="电台名称"
            placeholder="例如：前线战报"
            prepend-inner-icon="mdi-antenna"
            hide-details="auto"
            :rules="[required]"
            @keyup.enter="createStation"
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
            @click="createStation"
          >
            创建
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Rename Station Dialog -->
    <v-dialog v-model="showRenameDialog" max-width="460" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-pencil</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">RENAME CHANNEL</div>
              <div class="text-display text-h5">重命名电台</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="renameName"
            label="电台名称"
            placeholder="例如：前线战报"
            prepend-inner-icon="mdi-antenna"
            hide-details="auto"
            :rules="[required]"
            @keyup.enter="doRename"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showRenameDialog = false">取消</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="doRename"
          >
            保存
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Chance Config Editor -->
    <v-dialog v-model="showChanceDialog" max-width="640" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-tune</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">PLAYBACK CONDITIONS</div>
              <div class="text-display text-h5">编辑播放条件</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <div v-if="chanceEditorData">
            <v-text-field
              v-model.number="chanceEditorData.factor"
              type="number"
              min="0"
              step="0.1"
              label="基础权重 factor"
              variant="outlined"
              density="comfortable"
              hide-details="auto"
              class="mb-4"
            />

            <div class="d-flex justify-space-between align-center mb-3">
              <div class="text-body font-weight-medium">修饰器（modifier）</div>
              <v-btn
                color="primary"
                variant="text"
                size="small"
                prepend-icon="mdi-plus"
                @click="addModifier"
              >
                添加修饰器
              </v-btn>
            </div>

            <div v-if="chanceEditorData.modifiers.length === 0" class="text-body text-secondary text-center py-4">
              暂无修饰器，歌曲将只使用基础权重
            </div>

            <v-card
              v-for="(modifier, mIdx) in chanceEditorData.modifiers"
              :key="mIdx"
              class="modifier-card mb-4"
              variant="outlined"
              rounded="lg"
            >
              <v-card-text class="pa-4">
                <div class="d-flex justify-space-between align-center mb-3">
                  <div class="text-mono text-caption text-secondary">MODIFIER #{{ mIdx + 1 }}</div>
                  <v-btn
                    icon="mdi-delete-outline"
                    variant="text"
                    size="small"
                    color="error"
                    @click="removeModifier(mIdx)"
                  />
                </div>
                <div class="d-flex gap-3 mb-3">
                  <v-text-field
                    v-model.number="modifier.factor"
                    type="number"
                    step="0.1"
                    label="factor"
                    variant="outlined"
                    density="compact"
                    hide-details
                    clearable
                  />
                  <v-text-field
                    v-model.number="modifier.add"
                    type="number"
                    step="0.1"
                    label="add"
                    variant="outlined"
                    density="compact"
                    hide-details
                    clearable
                  />
                  <v-text-field
                    v-model.number="modifier.base"
                    type="number"
                    step="0.1"
                    label="base"
                    variant="outlined"
                    density="compact"
                    hide-details
                    clearable
                  />
                </div>

                <div class="d-flex justify-space-between align-center mb-2">
                  <div class="text-body text-caption">触发条件</div>
                  <v-btn
                    color="primary"
                    variant="text"
                    size="x-small"
                    prepend-icon="mdi-plus"
                    @click="addTrigger(modifier)"
                  >
                    添加条件
                  </v-btn>
                </div>

                <div
                  v-for="(trigger, tIdx) in modifier.triggers"
                  :key="tIdx"
                  class="d-flex align-center gap-2 mb-2"
                >
                  <v-select
                    v-model="trigger.type"
                    :items="triggerTypes"
                    item-title="label"
                    item-value="value"
                    label="条件类型"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-type"
                  />
                  <v-text-field
                    v-if="trigger.type === 'tag'"
                    v-model="trigger.value"
                    label="国家标签"
                    placeholder="CHI"
                    variant="outlined"
                    density="compact"
                    hide-details
                  />
                  <v-select
                    v-else-if="trigger.type === 'has_war'"
                    v-model="trigger.value"
                    :items="[{ label: '是', value: true }, { label: '否', value: false }]"
                    item-title="label"
                    item-value="value"
                    label="处于战争"
                    variant="outlined"
                    density="compact"
                    hide-details
                  />
                  <v-text-field
                    v-else-if="trigger.type === 'has_government'"
                    v-model="trigger.ideology"
                    label="意识形态"
                    placeholder="democratic"
                    variant="outlined"
                    density="compact"
                    hide-details
                  />
                  <v-text-field
                    v-else-if="trigger.type === 'is_in_faction'"
                    v-model="trigger.tag"
                    label="阵营国家"
                    placeholder="USA"
                    variant="outlined"
                    density="compact"
                    hide-details
                  />
                  <v-btn
                    icon="mdi-delete-outline"
                    variant="text"
                    size="small"
                    color="error"
                    @click="removeTrigger(modifier, tIdx)"
                  />
                </div>
              </v-card-text>
            </v-card>
          </div>
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showChanceDialog = false">取消</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="saveChance"
          >
            保存
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Audio Picker -->
    <AudioPickerDialog
      v-model="showPicker"
      @confirm="onAudioSelected"
    />

    <!-- Delete Station Dialog -->
    <v-dialog v-model="showDeleteDialog" max-width="420" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent dialog-accent--danger" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="error" size="28">mdi-alert-circle</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">CONFIRM DELETION</div>
              <div class="text-display text-h5">删除电台</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          确定要删除电台 <strong class="text-primary">{{ stationToDelete?.name }}</strong> 吗？
          <br><br>
          电台内的所有歌曲条目也将被删除，但音频文件仍会保留在全局音频库中。
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
import { ref, onMounted, watch, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useStationStore, type Station, type StationEntry, type ChanceConfig, type Modifier, type Trigger, type TriggerType } from '@/stores/station'
import { useAudioStore } from '@/stores/audio'
import { logger } from '@/utils/logger'
import { useToastStore } from '@/stores/toast'
import AudioPickerDialog from '@/components/AudioPickerDialog.vue'

const route = useRoute()
const stationStore = useStationStore()
const audioStore = useAudioStore()
const toast = useToastStore()
const activeTab = ref<string>('')
const showCreateDialog = ref(false)
const showRenameDialog = ref(false)
const showDeleteDialog = ref(false)
const showPicker = ref(false)
const showChanceDialog = ref(false)
const newStationName = ref('')
const renameName = ref('')
const renameStationId = ref('')
const currentStationId = ref<string>('')
const stationToDelete = ref<Station | null>(null)
const chanceEditorStationId = ref('')
const chanceEditorAudioId = ref('')
const chanceEditorData = ref<ChanceConfig | null>(null)

const projectId = computed(() => route.params.id as string)

const triggerTypes = [
  { label: '战争状态', value: 'has_war' as TriggerType },
  { label: '国家标签', value: 'tag' as TriggerType },
  { label: '意识形态', value: 'has_government' as TriggerType },
  { label: '同阵营国家', value: 'is_in_faction' as TriggerType },
]

function required(v: string) {
  return !!v || '此项为必填'
}

onMounted(() => {
  stationStore.loadStations()
  if (projectId.value) {
    audioStore.loadAudio(projectId.value)
  }
})

watch(
  () => stationStore.stations,
  (stations) => {
    if (stations.length > 0 && !activeTab.value) {
      activeTab.value = stations[0].id
    }
  },
  { immediate: true }
)

function audioTitle(id: string): string {
  const audio = audioStore.audioFiles.find((a) => a.id === id)
  return audio?.title || id
}

function isFirst(stationId: string): boolean {
  return stationStore.stations[0]?.id === stationId
}

function isLast(stationId: string): boolean {
  return stationStore.stations[stationStore.stations.length - 1]?.id === stationId
}

function openCreateDialog() {
  newStationName.value = ''
  showCreateDialog.value = true
}

async function createStation() {
  const name = newStationName.value.trim()
  if (!name) return
  try {
    await stationStore.createStation(name)
    showCreateDialog.value = false
    newStationName.value = ''
  } catch (err: any) {
    if (err?.type === 'station_name_exists') {
      toast.display(`电台名称 "${name}" 已存在`, 'error', 4000)
    } else {
      throw err
    }
  }
}

function openRename(station: Station) {
  renameStationId.value = station.id
  renameName.value = station.name
  showRenameDialog.value = true
}

async function doRename() {
  const name = renameName.value.trim()
  if (!name || !renameStationId.value) return
  try {
    await stationStore.renameStation(renameStationId.value, name)
    showRenameDialog.value = false
    renameName.value = ''
    renameStationId.value = ''
  } catch (err: any) {
    if (err?.type === 'station_name_exists') {
      toast.display(`电台名称 "${name}" 已存在`, 'error', 4000)
    } else {
      throw err
    }
  }
}

async function moveStation(stationId: string, delta: number) {
  const idx = stationStore.stations.findIndex((s) => s.id === stationId)
  if (idx === -1) return
  const newIdx = idx + delta
  if (newIdx < 0 || newIdx >= stationStore.stations.length) return
  const ordered = [...stationStore.stations.map((s) => s.id)]
  const [moved] = ordered.splice(idx, 1)
  ordered.splice(newIdx, 0, moved)
  await stationStore.reorderStations(ordered)
}

function openPicker(stationId: string) {
  currentStationId.value = stationId
  showPicker.value = true
}

async function onAudioSelected(audioIds: string[]) {
  if (!currentStationId.value || !projectId.value) {
    logger.warn('station editor: missing station or project id when adding audio')
    return
  }
  logger.info(
    `station editor: adding ${audioIds.length} audio file(s) to project ${projectId.value} and station ${currentStationId.value}`
  )
  try {
    await audioStore.addToProject(projectId.value, audioIds)
    for (const audioId of audioIds) {
      await stationStore.addEntry(currentStationId.value, audioId, { factor: 1, modifiers: [] })
    }
    logger.info('station editor: audio added successfully')
  } catch (err) {
    logger.error(`station editor: failed to add audio: ${JSON.stringify(err)}`)
    throw err
  }
}

async function removeEntry(stationId: string, audioFileId: string) {
  await stationStore.removeEntry(stationId, audioFileId)
}

async function saveFactor(stationId: string, entry: StationEntry) {
  await stationStore.updateEntry(stationId, entry.audio_file_id, entry.chance)
}

function openChanceEditor(stationId: string, entry: StationEntry) {
  chanceEditorStationId.value = stationId
  chanceEditorAudioId.value = entry.audio_file_id
  chanceEditorData.value = JSON.parse(JSON.stringify(entry.chance)) as ChanceConfig
  showChanceDialog.value = true
}

function addModifier() {
  if (!chanceEditorData.value) return
  chanceEditorData.value.modifiers.push({ triggers: [] })
}

function removeModifier(index: number) {
  if (!chanceEditorData.value) return
  chanceEditorData.value.modifiers.splice(index, 1)
}

function addTrigger(modifier: Modifier) {
  modifier.triggers.push({ type: 'has_war', value: true })
}

function removeTrigger(modifier: Modifier, index: number) {
  modifier.triggers.splice(index, 1)
}

async function saveChance() {
  if (!chanceEditorData.value || !chanceEditorStationId.value || !chanceEditorAudioId.value) return
  await stationStore.updateEntry(
    chanceEditorStationId.value,
    chanceEditorAudioId.value,
    cleanChance(chanceEditorData.value)
  )
  showChanceDialog.value = false
}

function cleanChance(chance: ChanceConfig): ChanceConfig {
  return {
    factor: chance.factor,
    modifiers: chance.modifiers.map((m) => ({
      factor: m.factor,
      add: m.add,
      base: m.base,
      triggers: m.triggers.map((t) => {
        const base: Trigger = { type: t.type }
        if (t.type === 'has_war') base.value = t.value
        if (t.type === 'tag') base.value = t.value
        if (t.type === 'has_government') base.ideology = t.ideology
        if (t.type === 'is_in_faction') base.tag = t.tag
        return base
      }),
    })),
  }
}

async function moveEntry(stationId: string, index: number, delta: number) {
  const station = stationStore.stations.find((s) => s.id === stationId)
  if (!station) return
  const newIndex = index + delta
  if (newIndex < 0 || newIndex >= station.entries.length) return
  const ordered = [...station.entries.map((e) => e.audio_file_id)]
  const [moved] = ordered.splice(index, 1)
  ordered.splice(newIndex, 0, moved)
  await stationStore.reorderEntries(stationId, ordered)
}

function confirmDelete(station: Station) {
  stationToDelete.value = station
  showDeleteDialog.value = true
}

async function handleDelete() {
  if (!stationToDelete.value) return
  await stationStore.deleteStation(stationToDelete.value.id)
  showDeleteDialog.value = false
  stationToDelete.value = null
}
</script>

<style scoped>
.station-card {
  background: rgba(26, 23, 20, 0.7);
  border: 1px solid rgba(74, 66, 56, 0.4);
}

.create-btn {
  text-transform: none;
}

.station-tabs :deep(.v-tab) {
  text-transform: none;
  letter-spacing: 0.02em;
}

.entry-card {
  background: rgba(37, 33, 28, 0.5);
  border: 1px solid rgba(74, 66, 56, 0.3);
}

.entry-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.entry-item:hover {
  background: rgba(255, 176, 32, 0.06) !important;
  border-color: rgba(255, 176, 32, 0.2);
}

.reorder-controls {
  width: 28px;
}

.factor-field {
  max-width: 120px;
}

.modifier-card {
  background: rgba(26, 23, 20, 0.5);
  border-color: rgba(74, 66, 56, 0.4);
}

.trigger-type {
  max-width: 140px;
}

.empty-state {
  border: 2px dashed rgba(74, 66, 56, 0.6);
  border-radius: 16px;
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
