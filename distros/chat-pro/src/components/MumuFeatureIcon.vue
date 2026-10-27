<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import helpSmall from '../assets/mumu-ui/help-96.png'
import helpLarge from '../assets/mumu-ui/help-256.png'
import librarySmall from '../assets/mumu-ui/library-96.png'
import libraryLarge from '../assets/mumu-ui/library-256.png'
import modelSmall from '../assets/mumu-ui/model-96.png'
import modelLarge from '../assets/mumu-ui/model-256.png'
import settingsSmall from '../assets/mumu-ui/settings-96.png'
import settingsLarge from '../assets/mumu-ui/settings-256.png'

const props = withDefaults(defineProps<{
  name: 'library' | 'model' | 'settings' | 'help'
  size?: 'nav' | 'heading' | 'empty'
}>(), { size: 'nav' })

const assets = {
  library: { small: librarySmall, large: libraryLarge, fallback: '▦' },
  model: { small: modelSmall, large: modelLarge, fallback: '◇' },
  settings: { small: settingsSmall, large: settingsLarge, fallback: '⚙' },
  help: { small: helpSmall, large: helpLarge, fallback: '?' },
}
const asset = computed(() => assets[props.name])
const failed = ref(false)
watch(() => props.name, () => {
  failed.value = false
})
const displaySize = computed(() => ({ nav: '40px', heading: '56px', empty: '112px' })[props.size])
</script>

<template>
  <span class="mumu-feature-icon" :class="`mumu-feature-icon--${size}`" :data-feature-icon="name" aria-hidden="true">
    <img
      v-if="!failed"
      :src="asset.small"
      :srcset="`${asset.small} 96w, ${asset.large} 256w`"
      :sizes="displaySize"
      width="96"
      height="96"
      alt=""
      draggable="false"
      decoding="async"
      @error="failed = true"
    >
    <span v-else class="mumu-feature-icon__fallback">{{ asset.fallback }}</span>
  </span>
</template>

<style scoped>
.mumu-feature-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  width: 40px;
  height: 40px;
  pointer-events: none;
  user-select: none;
  vertical-align: middle;
}
.mumu-feature-icon img { display: block; width: 100%; height: 100%; object-fit: contain; }
.mumu-feature-icon--heading { width: 56px; height: 56px; }
.mumu-feature-icon--empty { width: 112px; height: 112px; }
.mumu-feature-icon__fallback { font-size: 24px; line-height: 1; color: inherit; }
/* Native-looking title bars stay compact in the classic skin. */
:global(html[data-skin="win98"] .mumu-feature-icon--heading) { width: 28px; height: 28px; }
</style>
