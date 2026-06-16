<template>
  <v-app>
    <router-view />
    <v-snackbar v-model="toast.show" :color="toast.color" :timeout="toast.timeout">
      {{ toast.message }}
      <template #actions>
        <v-btn variant="text" @click="toast.show = false">关闭</v-btn>
      </template>
    </v-snackbar>
  </v-app>
</template>

<script setup lang="ts">
import { useToastStore } from '@/stores/toast'

const toast = useToastStore()
</script>
