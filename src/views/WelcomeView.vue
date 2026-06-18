<template>
  <v-main class="d-flex align-center justify-center bureau-hero">
    <div class="hero-content text-center pa-8">
      <div class="hero-badge mb-6">
        <span class="text-mono text-caption">BROADCAST BUREAU</span>
      </div>
      <h1 class="text-display hero-title mb-4">HOI4 Radio Maker</h1>
      <p class="text-body hero-subtitle mb-8">
        为你的钢铁雄心 IV 战役谱写声音。
        <br />
        创建项目、导入音频、编辑电台、一键生成 Mod。
      </p>
      <v-btn
        color="primary"
        size="large"
        prepend-icon="mdi-radio-tower"
        class="hero-cta"
        @click="emitCreate"
      >
        新建广播项目
      </v-btn>
    </div>
  </v-main>
</template>

<script setup lang="ts">
function emitCreate() {
  window.dispatchEvent(new CustomEvent('bureau:create-project'))
}
</script>

<style scoped>
.bureau-hero {
  position: relative;
  min-height: 100vh;
}

.bureau-hero::before {
  content: '';
  position: absolute;
  inset: 0;
  background:
    radial-gradient(circle at 50% 50%, rgba(255, 176, 32, 0.08) 0%, transparent 50%),
    repeating-linear-gradient(
      90deg,
      transparent,
      transparent 80px,
      rgba(255, 176, 32, 0.03) 80px,
      rgba(255, 176, 32, 0.03) 81px
    );
  pointer-events: none;
}

.hero-content {
  position: relative;
  max-width: 640px;
  animation: heroFadeIn 1s ease-out;
}

@keyframes heroFadeIn {
  from {
    opacity: 0;
    transform: translateY(24px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.hero-badge {
  display: inline-block;
  border: 1px solid rgba(255, 176, 32, 0.4);
  padding: 6px 16px;
  letter-spacing: 0.2em;
  color: #ffb020;
}

.hero-title {
  font-size: clamp(2.8rem, 6vw, 4.5rem);
  line-height: 1.05;
  color: #efebe3;
  text-shadow: 0 2px 24px rgba(255, 176, 32, 0.2);
}

.hero-subtitle {
  font-size: 1.15rem;
  line-height: 1.7;
  color: #c4b5a0;
  max-width: 520px;
  margin: 0 auto;
}

.hero-cta {
  font-size: 1.05rem;
  padding: 0 28px;
  box-shadow: 0 0 24px rgba(255, 176, 32, 0.25);
  transition: box-shadow 0.3s ease, transform 0.2s ease;
}

.hero-cta:hover {
  box-shadow: 0 0 36px rgba(255, 176, 32, 0.4);
  transform: translateY(-2px);
}
</style>
