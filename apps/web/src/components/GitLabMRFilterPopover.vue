<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import FilterPopover from './FilterPopover.vue'
import { hasActiveFilter } from '@/composables/useFilteredItems'

const prefs = useDashboardPrefsStore()
const filter = computed(() => prefs.filters.gitlab_mr)
const active = computed(() => hasActiveFilter('gitlab_mr', prefs.filters))

function toggleRole(role: string) {
  const roles = [...filter.value.roles]
  const idx = roles.indexOf(role)
  if (idx >= 0) roles.splice(idx, 1)
  else roles.push(role)
  prefs.updateGitLabMRFilter({ roles })
}

function toggleHideDrafts() {
  prefs.updateGitLabMRFilter({ hide_drafts: !filter.value.hide_drafts })
}

function clearAll() {
  prefs.updateGitLabMRFilter({ roles: [], hide_drafts: false })
}
</script>

<template>
  <FilterPopover :active="active">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Filter MRs</span>
        <button v-if="active" @click="clearAll" class="text-xs hover:underline" style="color: var(--color-primary)">Clear</button>
      </div>

      <!-- Roles -->
      <div>
        <span class="text-xs font-medium block mb-1.5" style="color: var(--color-text)">Role</span>
        <label v-for="role in ['author', 'reviewer', 'other']" :key="role" class="flex items-center gap-2 py-0.5 cursor-pointer">
          <input type="checkbox" :checked="filter.roles.includes(role)" @change="toggleRole(role)" class="rounded" />
          <span class="text-sm capitalize" style="color: var(--color-text)">{{ role }}</span>
        </label>
      </div>

      <!-- Toggles -->
      <div>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="filter.hide_drafts" @change="toggleHideDrafts" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">Hide drafts</span>
        </label>
      </div>
    </div>
  </FilterPopover>
</template>
