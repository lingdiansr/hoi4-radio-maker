<template>
  <div class="pa-4">
    <div class="d-flex align-center mb-4">
      <h3 class="text-h6 mr-4">音频库</h3>
      <AudioImporter @import="onImport" />
    </div>
    <v-list v-if="audioStore.audioFiles.length > 0">
      <v-list-item
        v-for="audio in audioStore.audioFiles"
        :key="audio.id"
        :title="audio.title"
        :subtitle="`${audio.artist || '未知艺术家'} · ${audio.duration_secs}s · ${audio.sample_rate}Hz · ${audio.channels}ch`"
      >
        <template #append>
          <v-btn icon="mdi-delete" variant="text" color="error" @click="audioStore.deleteAudio(audio.id)" />
        </template>
      </v-list-item>
    </v-list>
    <v-alert v-else type="info" text="暂无音频文件，点击上方按钮导入。" />
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import AudioImporter from '@/components/AudioImporter.vue'
import { useAudioStore } from '@/stores/audio'

const audioStore = useAudioStore()

onMounted(() => {
  audioStore.loadAudio()
})

async function onImport(paths: string[]) {
  for (const path of paths) {
    await audioStore.importAudio(path)
  }
}
</script>
