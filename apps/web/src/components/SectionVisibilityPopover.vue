<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { ALL_SECTIONS, SECTION_LABELS } from '@/types/dashboard'
import type { SectionType } from '@/types/dashboard'

const prefs = useDashboardPrefsStore()
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

function toggleSection(section: SectionType) {
  prefs.toggleSectionVisibility(section)
}
</script>

<template>
  <div ref="popoverRef" class="relative">
    <button
      @click.stop="toggle"
      class="p-1.5 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
      title="Choose visible sections"
    >
      <!-- Eye icon -->
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-text-muted)">
        <path d="M10 12a2 2 0 100-4 2 2 0 000 4z" />
        <path fill-rule="evenodd" d="M.458 10C1.732 5.943 5.522 3 10 3s8.268 2.943 9.542 7c-1.274 4.057-5.064 7-9.542 7S1.732 14.057.458 10zM14 10a4 4 0 11-8 0 4 4 0 018 0z" clip-rule="evenodd" />
      </svg>
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        class="absolute right-0 top-full mt-2 z-50 min-w-48 rounded-lg border shadow-lg p-4"
        style="background: var(--color-surface); border-color: var(--color-border)"
      >
        <div class="space-y-3">
          <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Visible Sections</span>
          <div class="space-y-1">
            <label
              v-for="section in ALL_SECTIONS"
              :key="section"
              class="flex items-center gap-2 py-0.5 cursor-pointer"
            >
              <input
                type="checkbox"
                :checked="prefs.isSectionVisible(section)"
                @change="toggleSection(section)"
                class="rounded"
              />
              <span class="text-sm" style="color: var(--color-text)">{{ SECTION_LABELS[section] }}</span>
            </label>
          </div>
        </div>
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
