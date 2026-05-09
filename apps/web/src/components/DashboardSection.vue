<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref } from 'vue'
import { RouterLink } from 'vue-router'

defineProps<{
  title: string
  count: number
  link?: string
  /** Show drag handle for reordering */
  draggable?: boolean
}>()

const collapsed = ref(false)
</script>

<template>
  <section class="mb-6">
    <div
      class="flex items-center justify-between cursor-pointer py-2"
      @click="collapsed = !collapsed"
    >
      <div class="flex items-center gap-2">
        <!-- Drag handle -->
        <span
          v-if="draggable"
          class="drag-handle cursor-grab active:cursor-grabbing p-0.5 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
          @click.stop
          title="Drag to reorder"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-text-muted)">
            <path d="M7 2a2 2 0 10.001 4.001A2 2 0 007 2zm0 6a2 2 0 10.001 4.001A2 2 0 007 8zm0 6a2 2 0 10.001 4.001A2 2 0 007 14zm6-8a2 2 0 10-.001-4.001A2 2 0 0013 6zm0 2a2 2 0 10.001 4.001A2 2 0 0013 8zm0 6a2 2 0 10.001 4.001A2 2 0 0013 14z" />
          </svg>
        </span>

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
      <div class="flex items-center gap-1" @click.stop>
        <!-- Slot for filter/action buttons -->
        <slot name="actions" />
        <RouterLink
          v-if="link"
          :to="link"
          class="text-xs hover:underline ml-2"
          style="color: var(--color-primary)"
        >View all</RouterLink>
      </div>
    </div>
    <div v-show="!collapsed" class="space-y-2">
      <slot />
    </div>
  </section>
</template>
