<template>
  <div class="pa-6">
    <v-card class="settings-card" variant="elevated" rounded="xl">
      <v-card-title class="pa-6 pb-2">
        <div class="text-mono text-caption text-secondary mb-1">PROJECT DOSSIER</div>
        <div class="text-display text-h5">{{ $t('project.infoTitle') }}</div>
      </v-card-title>

      <v-divider opacity="0.2" />

      <v-card-text class="pa-6">
        <v-row>
          <v-col cols="12" lg="8">
            <v-text-field
              v-model="form.name"
              :label="$t('project.name')"
              :placeholder="$t('project.namePlaceholder')"
              prepend-inner-icon="mdi-radio-tower"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              v-model="form.version"
              :label="$t('project.version')"
              placeholder="0.1.0"
              prepend-inner-icon="mdi-tag-outline"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              v-model="form.supported_version"
              :label="$t('project.supportedVersion')"
              placeholder="*"
              prepend-inner-icon="mdi-gamepad-variant-outline"
              class="mb-4"
              hide-details="auto"
              :rules="[required]"
            />
            <v-text-field
              :model-value="form.output_dir"
              :label="$t('project.outputDir')"
              prepend-inner-icon="mdi-folder-open"
              class="mb-4"
              hide-details="auto"
              readonly
            />
            <v-text-field
              v-model="authorInput"
              :label="$t('project.author')"
              :placeholder="$t('common.optional')"
              prepend-inner-icon="mdi-account-edit"
              class="mb-4"
              hide-details="auto"
            />
            <v-combobox
              v-model="form.tags"
              :label="$t('project.tags')"
              :placeholder="$t('project.tagsPlaceholder')"
              prepend-inner-icon="mdi-tag-multiple"
              multiple
              chips
              class="mb-4"
              hide-details="auto"
            />

            <v-divider opacity="0.2" class="mb-6" />

            <!-- Trigger vocabulary sources: part of this same form, so the one
                 save button below applies to them too. -->
            <div class="text-mono text-caption text-secondary mb-1">TRIGGER SOURCES</div>
            <div class="text-body-1 mb-4">{{ $t('triggers.sourcesTitle') }}</div>

            <v-checkbox
              :model-value="form.load_vanilla_triggers"
              :label="$t('triggers.loadVanilla')"
              hide-details
              density="comfortable"
              class="mb-2"
              @update:model-value="(v) => (form.load_vanilla_triggers = !!v)"
            />
            <div class="text-caption text-secondary mb-4">
              {{ $t('triggers.loadVanillaHint') }}
            </div>

            <v-select
              v-model="form.trigger_mod_dirs"
              :items="modOptions"
              :label="$t('triggers.loadMods')"
              :placeholder="$t('triggers.selectMods')"
              multiple
              chips
              closable-chips
              item-title="title"
              item-value="value"
              prepend-inner-icon="mdi-puzzle-outline"
            />
            <div class="text-caption text-secondary mt-2 mb-6">
              <template v-if="modOptions.length">{{ $t('triggers.loadModsHint') }}</template>
              <template v-else-if="form.trigger_mod_dirs.length">
                {{ $t('triggers.modsUnavailable', { count: form.trigger_mod_dirs.length }) }}
              </template>
              <template v-else>{{ $t('triggers.noMods') }}</template>
            </div>

            <v-btn
              color="primary"
              size="large"
              prepend-icon="mdi-content-save"
              class="save-btn"
              :loading="saving"
              @click="save"
            >
              {{ $t('project.saveInfo') }}
            </v-btn>

          </v-col>

          <v-col cols="12" lg="4" align-self="start">
            <v-card class="hint-card" variant="flat" rounded="lg">
              <v-card-text>
                <v-icon color="primary" size="32" class="mb-2">mdi-information-outline</v-icon>
                <div class="text-body text-secondary text-body-2">
                  {{ $t('project.infoHint') }}
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
              <div class="text-display text-h5">{{ $t('project.discardTitle') }}</div>
            </div>
          </div>
        </v-card-title>

        <v-card-text class="pa-6 pt-4 text-body-1">
          {{ $t('project.discardBody') }}
        </v-card-text>

        <v-divider opacity="0.2" />

        <v-card-actions class="pa-6">
          <v-spacer />
          <v-btn variant="text" class="action-btn" @click="cancelDiscard">{{ $t('common.cancel') }}</v-btn>
          <v-btn color="primary" class="action-btn" prepend-icon="mdi-check-circle" @click="confirmDiscard">
            {{ $t('project.discardConfirm') }}
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch, ref, computed, nextTick, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useProjectStore, type UpdateProjectRequest, type WorkshopMod } from '@/stores/project'
import { invokeCommand } from '@/api/client'
import { errorMessage } from '@/utils/errors'
import { logger } from '@/utils/logger'
import { useToastStore } from '@/stores/toast'

const router = useRouter()
const projectStore = useProjectStore()
const toast = useToastStore()
const { t } = useI18n()

// Trigger vocabulary sources. The mod list comes from the installed Steam
// Workshop; the selection lives on `form.trigger_mod_dirs` (bound with v-model).
const workshopMods = ref<WorkshopMod[]>([])

const modOptions = computed(() =>
  workshopMods.value.map((m) => ({ title: m.name, value: m.path }))
)

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
  load_vanilla_triggers: true,
  trigger_mod_dirs: [],
})

async function loadWorkshopMods() {
  try {
    workshopMods.value = await invokeCommand<WorkshopMod[]>('list_workshop_mods')
  } catch (err) {
    // A missing/custom Steam layout simply yields no mods to pick from.
    logger.warn(`project settings: workshop mods unavailable: ${JSON.stringify(err)}`)
    workshopMods.value = []
  }
}

const authorInput = computed({
  get: () => form.author ?? '',
  set: (v: string) => {
    form.author = v.trim() || undefined
  },
})

function required(v: string) {
  return !!v || t('common.required')
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
  form.load_vanilla_triggers = p.load_vanilla_triggers
  form.trigger_mod_dirs = [...p.trigger_mod_dirs]
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
    // Go through the store: it refreshes `currentProject`, which the station
    // editor watches to reload the trigger vocabulary.
    const updated = await projectStore.updateProject(p.id, { ...form })
    logger.info(`project settings: saved project ${updated.id}`)
    toast.display(t('project.infoSaved'), 'success')
    isDirty.value = false
  } catch (err) {
    toast.display(errorMessage(err), 'error', 4000)
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

onMounted(loadWorkshopMods)

</script>

<style scoped>
.settings-card {
  background: rgba(var(--v-theme-surface), 0.7);
  border: 1px solid rgba(var(--v-theme-outline), 0.4);
}

.save-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}

.hint-card {
  background: rgba(var(--v-theme-primary), 0.06);
  border: 1px solid rgba(var(--v-theme-primary), 0.2);
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

.dialog-title {
  padding-top: 28px;
}

.action-btn {
  text-transform: none;
  letter-spacing: 0.02em;
}
</style>
