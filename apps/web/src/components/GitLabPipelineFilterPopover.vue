<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import FilterPopover from './FilterPopover.vue'
import { hasActiveFilter } from '@/composables/useFilteredItems'

const prefs = useDashboardPrefsStore()
const filter = computed(() => prefs.filters.gitlabPipeline)
const active = computed(() => hasActiveFilter('gitlab_pipeline', prefs.filters))

const allStatuses = ['failed', 'running', 'pending', 'canceled']

function toggleStatus(status: string) {
  const statuses = [...filter.value.statuses]
  const idx = statuses.indexOf(status)
  if (idx >= 0) statuses.splice(idx, 1)
  else statuses.push(status)
  prefs.updateGitLabPipelineFilter({ statuses })
}

function clearAll() {
  prefs.updateGitLabPipelineFilter({ statuses: [] })
}
</script>

<template>
  <FilterPopover :active="active">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Filter Pipelines</span>
        <button v-if="active" @click="clearAll" class="text-xs hover:underline" style="color: var(--color-primary)">Clear</button>
      </div>

      <div>
        <span class="text-xs font-medium block mb-1.5" style="color: var(--color-text)">Status</span>
        <label v-for="status in allStatuses" :key="status" class="flex items-center gap-2 py-0.5 cursor-pointer">
          <input type="checkbox" :checked="filter.statuses.includes(status)" @change="toggleStatus(status)" class="rounded" />
          <span class="text-sm capitalize" style="color: var(--color-text)">{{ status }}</span>
        </label>
      </div>

      <p class="text-xs" style="color: var(--color-text-muted)">
        Default: shows failed, running, and pending pipelines.
      </p>
    </div>
  </FilterPopover>
</template>
