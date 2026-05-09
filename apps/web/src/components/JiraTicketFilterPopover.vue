<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import FilterPopover from './FilterPopover.vue'
import { hasActiveFilter } from '@/composables/useFilteredItems'

const prefs = useDashboardPrefsStore()
const filter = computed(() => prefs.filters.jira_ticket)
const active = computed(() => hasActiveFilter('jira_ticket', prefs.filters))

const allCategories = ['todo', 'in_progress', 'done']
const allPriorities = ['highest', 'high', 'medium', 'low', 'lowest']

function toggleCategory(cat: string) {
  const cats = [...filter.value.status_categories]
  const idx = cats.indexOf(cat)
  if (idx >= 0) cats.splice(idx, 1)
  else cats.push(cat)
  prefs.updateJiraTicketFilter({ status_categories: cats })
}

function togglePriority(priority: string) {
  const priorities = [...filter.value.priorities]
  const idx = priorities.indexOf(priority)
  if (idx >= 0) priorities.splice(idx, 1)
  else priorities.push(priority)
  prefs.updateJiraTicketFilter({ priorities })
}

function clearAll() {
  prefs.updateJiraTicketFilter({ status_categories: [], priorities: [], issue_types: [] })
}

const categoryLabels: Record<string, string> = {
  todo: 'To Do',
  in_progress: 'In Progress',
  done: 'Done',
}
</script>

<template>
  <FilterPopover :active="active">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Filter Tickets</span>
        <button v-if="active" @click="clearAll" class="text-xs hover:underline" style="color: var(--color-primary)">Clear</button>
      </div>

      <!-- Status Category -->
      <div>
        <span class="text-xs font-medium block mb-1.5" style="color: var(--color-text)">Status</span>
        <label v-for="cat in allCategories" :key="cat" class="flex items-center gap-2 py-0.5 cursor-pointer">
          <input type="checkbox" :checked="filter.status_categories.includes(cat)" @change="toggleCategory(cat)" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">{{ categoryLabels[cat] || cat }}</span>
        </label>
      </div>

      <!-- Priority -->
      <div>
        <span class="text-xs font-medium block mb-1.5" style="color: var(--color-text)">Priority</span>
        <label v-for="p in allPriorities" :key="p" class="flex items-center gap-2 py-0.5 cursor-pointer">
          <input type="checkbox" :checked="filter.priorities.includes(p)" @change="togglePriority(p)" class="rounded" />
          <span class="text-sm capitalize" style="color: var(--color-text)">{{ p }}</span>
        </label>
      </div>
    </div>
  </FilterPopover>
</template>
