<template>
  <div class="pa-6">
    <v-card class="station-card" variant="elevated" rounded="xl">
      <v-card-title class="d-flex justify-space-between align-center pa-6">
        <div>
          <div class="text-mono text-caption text-secondary mb-1">BROADCAST CHANNELS</div>
          <div class="text-display text-h5">{{ $t('station.title') }}</div>
        </div>
        <v-btn
          color="primary"
          prepend-icon="mdi-plus"
          class="create-btn"
          @click="openCreateDialog"
        >
          {{ $t('station.newStation') }}
        </v-btn>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <div v-if="stationStore.stations.length === 0" class="empty-state text-center py-12">
          <v-icon size="64" color="secondary" class="mb-4">mdi-antenna</v-icon>
          <div class="text-body text-secondary text-h6 mb-2">{{ $t('station.emptyTitle') }}</div>
          <div class="text-body text-secondary mb-4">{{ $t('station.emptyBody') }}</div>
          <v-btn color="primary" prepend-icon="mdi-plus" @click="openCreateDialog">{{ $t('station.newStation') }}</v-btn>
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
                  <span class="text-display text-h6">
                    {{ station.name }}
                    <span class="text-mono text-caption text-secondary ml-2">music/{{
                      station.subdir || slug(station.name)
                    }}/</span>
                  </span>
                  <div class="d-flex align-center gap-2">
                    <v-btn
                      icon="mdi-pencil"
                      variant="text"
                      size="small"
                      color="primary"
                      :title="$t('station.rename')"
                      @click.stop="openRename(station)"
                    />
                    <v-btn
                      icon="mdi-folder-outline"
                      variant="text"
                      size="small"
                      :color="station.subdir ? 'primary' : undefined"
                      :title="$t('station.outputSubdir')"
                      @click.stop="openSubdir(station)"
                    />
                    <v-btn
                      icon="mdi-arrow-left"
                      variant="text"
                      size="small"
                      :disabled="isFirst(station.id)"
                      :title="$t('station.moveUp')"
                      @click.stop="moveStation(station.id, -1)"
                    />
                    <v-btn
                      icon="mdi-arrow-right"
                      variant="text"
                      size="small"
                      :disabled="isLast(station.id)"
                      :title="$t('station.moveDown')"
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
                      :title="$t('station.deleteLabel')"
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
                          {{ $t('station.modifierCount', { count: entry.chance.modifiers.length }) }}
                        </v-chip>
                      </v-list-item-subtitle>
                      <template #append>
                        <v-btn
                          icon="mdi-tune"
                          variant="text"
                          color="primary"
                          class="mr-1"
                          :title="$t('station.editChance')"
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
                  <v-alert v-else color="secondary" variant="tonal" icon="mdi-information" :text="$t('station.noSongs')" />

                  <v-divider class="my-4" opacity="0.2" />

                  <v-btn
                    color="primary"
                    variant="outlined"
                    prepend-icon="mdi-music-box-multiple"
                    class="action-btn"
                    @click="openPicker(station.id)"
                  >
                    {{ $t('station.addSongs') }}
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
              <div class="text-display text-h5">{{ $t('station.createTitle') }}</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="newStationName"
            :label="$t('station.nameLabel')"
            :placeholder="$t('station.namePlaceholder')"
            prepend-inner-icon="mdi-antenna"
            hide-details="auto"
            :rules="[required]"
            @keyup.enter="createStation"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showCreateDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="createStation"
          >
            {{ $t('common.create') }}
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
              <div class="text-display text-h5">{{ $t('station.renameTitle') }}</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="renameName"
            :label="$t('station.nameLabel')"
            :placeholder="$t('station.namePlaceholder')"
            prepend-inner-icon="mdi-antenna"
            hide-details="auto"
            :rules="[required]"
            @keyup.enter="doRename"
          />
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showRenameDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="doRename"
          >
            {{ $t('common.save') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!-- Output Subdirectory Dialog -->
    <v-dialog v-model="showSubdirDialog" max-width="460" class="bureau-dialog">
      <v-card class="dialog-card">
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-folder-outline</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">OUTPUT SUBDIRECTORY</div>
              <div class="text-display text-h5">{{ $t('station.subdirTitle') }}</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4">
          <v-text-field
            v-model="subdirName"
            :label="$t('station.subdirLabel')"
            :placeholder="$t('station.subdirPlaceholder', { name: subdirStationName })"
            prepend-inner-icon="mdi-folder-music-outline"
            hide-details="auto"
            @keyup.enter="doSetSubdir"
          />
          <div class="text-caption text-secondary mt-2">
            {{ $t('station.subdirHint') }}
          </div>
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showSubdirDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="doSetSubdir"
          >
            {{ $t('common.save') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <!--
      Chance Config Editor.

      The width follows the triggers it holds (see `triggerRowWidths`): a radio
      using short vanilla conditions stays a compact panel, while one using a
      30-character mod trigger — or the 82-character longest shipped name —
      grows just enough to show the text in full. Vuetify still clamps the
      dialog to the window, where the rows wrap instead of overflowing.
    -->
    <v-dialog
      v-model="showChanceDialog"
      :max-width="triggerRowWidths.dialog"
      class="bureau-dialog"
    >
      <v-card
        class="dialog-card"
        :style="{
          '--trigger-name-w': triggerRowWidths.name + 'px',
          '--trigger-value-w': triggerRowWidths.value + 'px',
        }"
      >
        <div class="dialog-accent" />
        <v-card-title class="dialog-title pa-6 pb-2">
          <div class="d-flex align-center gap-3">
            <v-icon color="primary" size="28">mdi-tune</v-icon>
            <div>
              <div class="text-mono text-caption text-secondary">PLAYBACK CONDITIONS</div>
              <div class="text-display text-h5">{{ $t('station.chanceTitle') }}</div>
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
              :label="$t('station.baseFactor')"
              variant="outlined"
              density="comfortable"
              hide-details="auto"
              class="mb-4"
            />

            <div class="d-flex justify-space-between align-center mb-3">
              <div class="text-body font-weight-medium">{{ $t('station.modifiersTitle') }}</div>
              <v-btn
                color="primary"
                variant="text"
                size="small"
                prepend-icon="mdi-plus"
                @click="addModifier"
              >
                {{ $t('station.addModifier') }}
              </v-btn>
            </div>

            <div v-if="chanceEditorData.modifiers.length === 0" class="text-body text-secondary text-center py-4">
              {{ $t('station.noModifiers') }}
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
                  <div class="text-body text-caption">{{ $t('station.triggers') }}</div>
                  <v-btn
                    color="primary"
                    variant="text"
                    size="x-small"
                    prepend-icon="mdi-plus"
                    @click="addTrigger(modifier)"
                  >
                    {{ $t('station.addTrigger') }}
                  </v-btn>
                </div>

                <div
                  v-for="(trigger, tIdx) in modifier.triggers"
                  :key="tIdx"
                  class="trigger-row mb-2"
                >
                  <v-select
                    :model-value="trigger.type"
                    :items="triggerTypes"
                    item-title="label"
                    item-value="value"
                    :label="$t('station.triggerType')"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-type"
                    @update:model-value="(v) => onTriggerTypeChange(trigger, v as TriggerType)"
                  />
                  <v-combobox
                    v-if="trigger.type === 'tag'"
                    :model-value="asString(trigger.value)"
                    :items="countryTagOptions"
                    :label="$t('station.countryTag')"
                    :placeholder="$t('station.triggerValuePlaceholder')"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-value"
                    @update:model-value="(v) => (trigger.value = asString(v))"
                  />
                  <v-select
                    v-else-if="trigger.type === 'has_war'"
                    v-model="trigger.value"
                    :items="[{ label: $t('station.yes'), value: true }, { label: $t('station.no'), value: false }]"
                    item-title="label"
                    item-value="value"
                    :label="$t('station.atWar')"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-value"
                  />
                  <v-combobox
                    v-else-if="trigger.type === 'has_government'"
                    v-model="trigger.ideology"
                    :items="ideologyOptions"
                    :label="$t('station.ideology')"
                    :placeholder="$t('station.ideologyPlaceholder')"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-value"
                  />
                  <v-combobox
                    v-else-if="trigger.type === 'is_in_faction'"
                    v-model="trigger.tag"
                    :items="countryTagOptions"
                    :label="$t('station.factionCountry')"
                    :placeholder="$t('station.triggerValuePlaceholder')"
                    variant="outlined"
                    density="compact"
                    hide-details
                    class="trigger-value"
                  />
                  <template v-else-if="trigger.type === 'generic'">
                    <v-combobox
                      v-model="trigger.name"
                      :items="vocabularyTriggerOptions"
                      :label="$t('station.triggerName')"
                      :placeholder="$t('station.triggerNamePlaceholder')"
                      variant="outlined"
                      density="compact"
                      hide-details
                      class="trigger-name"
                      @update:model-value="(v) => onTriggerNameChange(trigger, asString(v))"
                    />
                    <v-select
                      v-if="valueKindFor(trigger.name) === 'boolean'"
                      :model-value="asString(trigger.value)"
                      :items="booleanValueOptions"
                      item-title="title"
                      item-value="value"
                      :label="$t('station.triggerValue')"
                      variant="outlined"
                      density="compact"
                      hide-details
                      class="trigger-value"
                      @update:model-value="(v) => (trigger.value = asString(v))"
                    />
                    <v-text-field
                      v-else-if="valueKindFor(trigger.name) === 'number'"
                      :model-value="asString(trigger.value)"
                      type="number"
                      :label="$t('station.triggerValue')"
                      :placeholder="$t('station.triggerNumberPlaceholder')"
                      variant="outlined"
                      density="compact"
                      hide-details
                      class="trigger-value"
                      @update:model-value="(v) => (trigger.value = asString(v))"
                    />
                    <v-combobox
                      v-else
                      :model-value="asString(trigger.value)"
                      :items="valueOptionsFor(trigger.name)"
                      :label="$t('station.triggerValue')"
                      :placeholder="$t('station.triggerValuePlaceholder')"
                      variant="outlined"
                      density="compact"
                      hide-details
                      class="trigger-value"
                      @update:model-value="(v) => (trigger.value = asString(v))"
                    />
                  </template>
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
          <v-btn variant="text" class="action-btn" @click="showChanceDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn
            color="primary"
            class="action-btn"
            prepend-icon="mdi-check-circle"
            @click="saveChance"
          >
            {{ $t('common.save') }}
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
              <div class="text-display text-h5">{{ $t('station.deleteTitle') }}</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          <i18n-t keypath="station.deleteConfirm" tag="span">
            <template #name>
              <strong class="text-primary">{{ stationToDelete?.name }}</strong>
            </template>
          </i18n-t>
          <br><br>
          {{ $t('station.deleteBody') }}
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="showDeleteDialog = false">{{ $t('common.cancel') }}</v-btn>
          <v-btn color="error" class="action-btn" prepend-icon="mdi-delete-outline" @click="handleDelete">
            {{ $t('common.delete') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { useStationStore, type Station, type StationEntry, type ChanceConfig, type Modifier, type Trigger, type TriggerType, type ValueKind } from '@/stores/station'
import { useProjectStore } from '@/stores/project'
import { useAudioStore } from '@/stores/audio'
import { errorMessage } from '@/utils/errors'
import { logger } from '@/utils/logger'
import { useToastStore } from '@/stores/toast'
import AudioPickerDialog from '@/components/AudioPickerDialog.vue'

const route = useRoute()
const { t } = useI18n()
const stationStore = useStationStore()
const projectStore = useProjectStore()
const audioStore = useAudioStore()
const toast = useToastStore()
const activeTab = ref<string>('')
const showCreateDialog = ref(false)
const showRenameDialog = ref(false)
const showSubdirDialog = ref(false)
const subdirName = ref('')
const subdirStationId = ref('')
const subdirStationName = ref('')
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

const triggerTypes = computed(() => [
  { label: t('station.triggerWar'), value: 'has_war' as TriggerType },
  { label: t('station.triggerTag'), value: 'tag' as TriggerType },
  { label: t('station.triggerIdeology'), value: 'has_government' as TriggerType },
  { label: t('station.triggerFaction'), value: 'is_in_faction' as TriggerType },
  { label: t('station.triggerGeneric'), value: 'generic' as TriggerType },
])

/**
 * Trigger names from the project's loaded vocabulary, restricted to
 * country-scoped triggers since music chances are evaluated for a country.
 */
/** Country tags from the loaded vocabulary, offered as suggestions. */
const countryTagOptions = computed(() => stationStore.vocabulary?.country_tags ?? [])

/** Ideology ids from the loaded vocabulary, offered as suggestions. */
const ideologyOptions = computed(() => stationStore.vocabulary?.ideologies ?? [])

/*
 * Trigger-row metrics, in CSS pixels, read off the rendered dialog:
 *
 *  - ROW_FURNITURE is the type select, the delete button and the three gaps.
 *  - DIALOG_CHROME is what sits between the dialog edge and a row (24px card
 *    padding, the modifier card's 16px padding and its border).
 *  - FIELD_INSET is what a field spends on its own padding and trailing icon,
 *    so text of width N needs a field of N + FIELD_INSET.
 */
const TRIGGER_TYPE_W = 163
const TRIGGER_ROW_FURNITURE = TRIGGER_TYPE_W + 40 + 3 * 8
const TRIGGER_DIALOG_CHROME = 88
const TRIGGER_FIELD_INSET = 48
const TRIGGER_NAME_MIN_W = 180
const TRIGGER_VALUE_MIN_W = 150
// Canvas text measurement differs slightly from how an <input> lays the same
// string out, so the row gets a few pixels of headroom before it would wrap.
const TRIGGER_DIALOG_SLACK = 16
const TRIGGER_DIALOG_MIN_W = 640
// High enough that even the widest real content (the 82-character name beside
// a 25-character ideology) lays out on one line. Only content that wide ever
// reaches the cap, and Vuetify clamps to the window regardless.
const TRIGGER_DIALOG_MAX_W = 1300

/** Reused canvas context for text measurement. */
let measureCtx: CanvasRenderingContext2D | null | undefined

/**
 * Pixel width of `text` in the app's UI font. Fields are sized from their
 * content, and an `<input>` cannot be asked how wide its value is once the
 * value fits, so the width is measured instead.
 */
function measureText(text: string): number {
  if (measureCtx === undefined) {
    measureCtx = document.createElement('canvas').getContext('2d')
    const host = document.querySelector('.v-application') ?? document.body
    if (measureCtx && host) {
      const style = getComputedStyle(host)
      measureCtx.font = `${style.fontWeight} ${style.fontSize} ${style.fontFamily}`
    }
  }
  // Without canvas, fall back to a rough average advance width.
  return measureCtx ? measureCtx.measureText(text).width : text.length * 8
}

/** The text a row displays, whichever field its trigger kind uses. */
function triggerValueText(trigger: Trigger): string {
  if (trigger.type === 'has_government') return asString(trigger.ideology)
  if (trigger.type === 'is_in_faction') return asString(trigger.tag)
  if (trigger.type === 'has_war') return t('station.no')
  return asString(trigger.value)
}

/**
 * Column widths and dialog width derived from the triggers being edited.
 *
 * Sizing from content keeps the dialog as narrow as the radio actually needs
 * while still showing every name and value in full: a project with short
 * vanilla conditions gets a compact panel, and one with long mod triggers grows
 * only by the difference. Recomputed as triggers are added, picked or typed.
 */
const triggerRowWidths = computed(() => {
  let nameText = 0
  let valueText = 0
  // Only `generic` triggers carry a name, so rows of the dedicated kinds (war,
  // country tag, ideology, faction) need no name column at all.
  let hasName = false

  for (const modifier of chanceEditorData.value?.modifiers ?? []) {
    for (const trigger of modifier.triggers) {
      if (trigger.type === 'generic') {
        hasName = true
        nameText = Math.max(nameText, measureText(asString(trigger.name)))
        // A boolean row renders a yes/no select, whose label is never the
        // widest thing in the row.
        if (valueKindFor(trigger.name) !== 'boolean') {
          valueText = Math.max(valueText, measureText(asString(trigger.value)))
        }
      } else {
        valueText = Math.max(valueText, measureText(triggerValueText(trigger)))
      }
    }
  }

  const name = hasName ? Math.max(TRIGGER_NAME_MIN_W, nameText + TRIGGER_FIELD_INSET) : 0
  const value = Math.max(TRIGGER_VALUE_MIN_W, valueText + TRIGGER_FIELD_INSET)
  const dialog = Math.min(
    TRIGGER_DIALOG_MAX_W,
    Math.max(
      TRIGGER_DIALOG_MIN_W,
      TRIGGER_ROW_FURNITURE + name + value + TRIGGER_DIALOG_CHROME + TRIGGER_DIALOG_SLACK,
    ),
  )

  return { name, value, dialog }
})

/** Cached value kind for a trigger name; undefined until it has been looked up. */
function valueKindFor(name?: string): ValueKind | null | undefined {
  return name ? stationStore.valueKinds[name] : undefined
}

/** Boolean triggers are written as `yes` / `no`, so offer exactly those. */
const booleanValueOptions = computed(() => [
  { title: t('station.yes'), value: 'yes' },
  { title: t('station.no'), value: 'no' },
])

/**
 * Candidate values for a free-text trigger: the scope keywords documented as
 * its targets (`THIS`, `ROOT`, `PREV`, …) are valid values themselves.
 */
function valueOptionsFor(name?: string): string[] {
  const def = name ? stationStore.vocabulary?.triggers[name] : undefined
  return (def?.targets ?? []).filter(
    (target) => target !== 'none' && target !== 'any'
  )
}

/**
 * Look up how the scripts use the chosen trigger and seed a value that is valid
 * for that kind. A boolean trigger starts at `yes`; every other kind starts
 * empty, so no stale `yes` is carried into a tag or a numeric trigger.
 */
async function onTriggerNameChange(trigger: Trigger, name: string) {
  const kind = await stationStore.ensureValueKind(name)
  trigger.value = kind === 'boolean' ? 'yes' : ''
}

const vocabularyTriggerOptions = computed(() => {
  const triggers = stationStore.vocabulary?.triggers ?? {}
  return Object.values(triggers)
    .filter((d) => d.scopes.length === 0 || d.scopes.includes('COUNTRY'))
    .map((d) => d.name)
    .sort((a, b) => a.localeCompare(b))
})

function required(v: string) {
  return !!v || t('common.required')
}

/** Mirrors the backend `slugify_id`: the folder the station will actually use. */
function slug(name: string) {
  const collapsed = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
  return collapsed || 'station'
}

onMounted(() => {
  if (projectId.value) {
    audioStore.loadAudio(projectId.value)
  }
})

// Stations belong to the current project, which the parent view resolves
// asynchronously; wait for it instead of firing once on mount.
watch(
  () => projectStore.currentProject?.id,
  (id) => {
    if (!id) return
    stationStore.loadStations()
    stationStore.loadVocabulary()
  },
  { immediate: true }
)

// The trigger vocabulary depends on the project's selected sources, so reload
// it whenever those change (e.g. mods picked in the project info tab).
watch(
  () => {
    const p = projectStore.currentProject
    return p ? `${p.load_vanilla_triggers}|${p.trigger_mod_dirs.join(',')}` : ''
  },
  (key, prev) => {
    if (key && prev !== undefined && key !== prev) {
      stationStore.loadVocabulary()
    }
  }
)

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
  } catch (err) {
    toast.display(errorMessage(err), 'error', 4000)
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
  } catch (err) {
    toast.display(errorMessage(err), 'error', 4000)
  }
}

function openSubdir(station: Station) {
  subdirStationId.value = station.id
  subdirStationName.value = station.name
  subdirName.value = station.subdir ?? ''
  showSubdirDialog.value = true
}

async function doSetSubdir() {
  if (!subdirStationId.value) return
  const raw = subdirName.value.trim()
  try {
    await stationStore.setStationSubdir(subdirStationId.value, raw === '' ? null : raw)
    showSubdirDialog.value = false
    subdirName.value = ''
    subdirStationId.value = ''
  } catch (err) {
    toast.display(errorMessage(err), 'error', 4000)
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

  // Resolve the value kind of every generic trigger already on this entry, so
  // each row renders the control matching its type (yes/no, number, or text)
  // instead of falling back to the text field until the user re-picks a name.
  for (const modifier of chanceEditorData.value.modifiers) {
    for (const trigger of modifier.triggers) {
      if (trigger.type === 'generic' && trigger.name) {
        stationStore.ensureValueKind(trigger.name)
      }
    }
  }
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

/**
 * Switch a trigger's kind, resetting the fields it uses.
 *
 * Each kind reads a different field (`value` / `ideology` / `tag` / `name`), so
 * a value left over from the previous kind would otherwise be submitted with
 * the wrong type — a boolean `has_war` value becoming a country tag, say.
 */
/** Normalise a combobox's value, which may be null, into a plain string. */
function asString(v: unknown): string {
  return v == null ? '' : String(v)
}

function onTriggerTypeChange(trigger: Trigger, type: TriggerType) {
  trigger.type = type
  trigger.value = type === 'has_war' ? true : ''
  trigger.ideology = ''
  trigger.tag = ''
  trigger.name = ''
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
      // Coerce each kind to the type the backend expects: `has_war` is a
      // boolean, while the string-valued kinds must never carry a boolean.
      triggers: m.triggers.map((t) => {
        const base: Trigger = { type: t.type }
        if (t.type === 'has_war') {
          base.value = t.value === true || t.value === 'true'
        } else if (t.type === 'tag') {
          base.value = t.value == null ? '' : String(t.value)
        } else if (t.type === 'has_government') {
          base.ideology = t.ideology == null ? '' : String(t.ideology)
        } else if (t.type === 'is_in_faction') {
          base.tag = t.tag == null ? '' : String(t.tag)
        } else if (t.type === 'generic') {
          base.name = t.name == null ? '' : String(t.name)
          base.value = t.value == null ? '' : String(t.value)
        }
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
  background: rgba(var(--v-theme-surface), 0.7);
  border: 1px solid rgba(var(--v-theme-outline), 0.4);
}

.create-btn {
  text-transform: none;
}

.station-tabs :deep(.v-tab) {
  text-transform: none;
  letter-spacing: 0.02em;
}

.entry-card {
  background: rgba(var(--v-theme-surface-variant), 0.5);
  border: 1px solid rgba(var(--v-theme-outline), 0.3);
}

.entry-item {
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.entry-item:hover {
  background: rgba(var(--v-theme-primary), 0.06) !important;
  border-color: rgba(var(--v-theme-primary), 0.2);
}

.reorder-controls {
  width: 28px;
}

.factor-field {
  max-width: 120px;
}

.modifier-card {
  background: rgba(var(--v-theme-surface-variant), 0.5);
  border-color: rgba(var(--v-theme-outline), 0.4);
}

/*
 * A trigger row packs a type select, an optional name, a value control, and a
 * delete button.
 *
 * Two mechanisms used to squash the value control: flex items default to
 * `min-width: auto` (they refuse to shrink below content width, so a long mod
 * trigger name stole the space), and once shrink was allowed, a narrow window
 * let the value control shrink without limit — the dialog is only as wide as
 * `min(100% - 48px, max-width)`, so a small window left it a ~40px sliver.
 *
 * Both column widths now come from the measured text they hold, applied as
 * `--trigger-name-w` / `--trigger-value-w` on the dialog, so every row shares
 * one set of columns (the type / name / value / delete columns stay aligned)
 * while the panel itself is only as wide as its content needs.
 *
 * flex-basis carries that content width, which also makes it the wrap
 * threshold: flex-wrap breaks on the hypothetical size, so when the dialog is
 * too narrow to show a name beside its value the row wraps and the name takes a
 * line of its own rather than truncating. The small min-widths are only the
 * last resort before the dialog would overflow a very small window.
 */
.trigger-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.trigger-row > * {
  min-width: 0;
}

.trigger-type {
  flex: 0 1 auto;
  /* Never narrower than the selected type's own label, whatever the width. */
  min-width: fit-content;
}

.trigger-name {
  flex: 1 1 var(--trigger-name-w, 220px);
  min-width: 140px;
}

.trigger-value {
  flex: 1 1 var(--trigger-value-w, 170px);
  min-width: 120px;
}

.empty-state {
  border: 2px dashed rgba(var(--v-theme-outline), 0.6);
  border-radius: 16px;
}

.dialog-card {
  background: rgb(var(--v-theme-surface));
  border: 1px solid rgba(var(--v-theme-outline), 0.5);
  position: relative;
  overflow: hidden;
}

.dialog-accent {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, rgb(var(--v-theme-primary)) 0%, rgba(var(--v-theme-primary), 0.3) 100%);
}

.dialog-accent--danger {
  background: linear-gradient(90deg, rgb(var(--v-theme-error)) 0%, rgba(var(--v-theme-error), 0.3) 100%);
}

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
