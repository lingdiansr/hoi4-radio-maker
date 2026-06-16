<template>
  <v-layout style="height: 100vh">
    <v-navigation-drawer permanent width="280">
      <ProjectList />
    </v-navigation-drawer>
    <v-main style="padding: 24px">
      <h1 class="text-h4 mb-4">HOI4 Radio Maker</h1>
      <p class="text-body-1">选择一个项目或创建新项目开始。</p>
    </v-main>
  </v-layout>
</template>

<script setup lang="ts">
import ProjectList from '@/components/ProjectList.vue'
</script>
