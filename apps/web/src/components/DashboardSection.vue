<script setup lang="ts">
defineProps<{
  title: string
  count: number
  link?: string
}>()

const collapsed = ref(false)

import { ref } from 'vue'
import { RouterLink } from 'vue-router'
</script>

<template>
  <section class="mb-6">
    <div
      class="flex items-center justify-between cursor-pointer py-2"
      @click="collapsed = !collapsed"
    >
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium" style="color: var(--color-text)">{{ title }}</span>
        <span
          class="text-xs px-1.5 py-0.5 rounded-full font-medium"
          style="background: var(--color-surface-hover); color: var(--color-text-muted)"
        >{{ count }}</span>
        <svg
          class="w-4 h-4 transition-transform"
          :class="{ '-rotate-90': collapsed }"
          style="color: var(--color-text-muted)"
          fill="none" viewBox="0 0 24 24" stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </div>
      <RouterLink
        v-if="link"
        :to="link"
        class="text-xs hover:underline"
        style="color: var(--color-primary)"
        @click.stop
      >View all</RouterLink>
    </div>
    <div v-show="!collapsed" class="space-y-2">
      <slot />
    </div>
  </section>
</template>
