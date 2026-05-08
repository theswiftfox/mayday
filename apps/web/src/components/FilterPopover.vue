<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

defineProps<{
  /** Show a dot indicator when filters are active */
  active?: boolean
}>()

const open = ref(false)
const popoverRef = ref<HTMLElement>()

function toggle() {
  open.value = !open.value
}

function handleClickOutside(e: MouseEvent) {
  if (popoverRef.value && !popoverRef.value.contains(e.target as Node)) {
    open.value = false
  }
}

onMounted(() => document.addEventListener('mousedown', handleClickOutside))
onUnmounted(() => document.removeEventListener('mousedown', handleClickOutside))
</script>

<template>
  <div ref="popoverRef" class="relative">
    <button
      @click.stop="toggle"
      class="p-1 rounded hover:bg-[var(--color-surface-hover)] transition-colors relative"
      :title="active ? 'Filters active' : 'Filter'"
    >
      <!-- Funnel icon -->
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" :style="{ color: active ? 'var(--color-primary)' : 'var(--color-text-muted)' }">
        <path fill-rule="evenodd" d="M3 3a1 1 0 011-1h12a1 1 0 011 1v3a1 1 0 01-.293.707L12 11.414V15a1 1 0 01-.293.707l-2 2A1 1 0 018 17v-5.586L3.293 6.707A1 1 0 013 6V3z" clip-rule="evenodd" />
      </svg>
      <!-- Active dot -->
      <span
        v-if="active"
        class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full"
        style="background: var(--color-primary)"
      />
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        class="absolute right-0 top-full mt-2 z-50 min-w-56 rounded-lg border shadow-lg p-4"
        style="background: var(--color-surface); border-color: var(--color-border)"
      >
        <slot />
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.popover-enter-active,
.popover-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.popover-enter-from,
.popover-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
