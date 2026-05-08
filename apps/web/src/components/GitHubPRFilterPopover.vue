<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import FilterPopover from './FilterPopover.vue'
import { hasActiveFilter } from '@/composables/useFilteredItems'

const prefs = useDashboardPrefsStore()
const filter = computed(() => prefs.filters.github_pr)
const active = computed(() => hasActiveFilter('github_pr', prefs.filters))

function toggleRole(role: string) {
  const roles = [...filter.value.roles]
  const idx = roles.indexOf(role)
  if (idx >= 0) roles.splice(idx, 1)
  else roles.push(role)
  prefs.updateGitHubPRFilter({ roles })
}

function toggleHideDrafts() {
  prefs.updateGitHubPRFilter({ hide_drafts: !filter.value.hide_drafts })
}

function toggleActionRequired() {
  prefs.updateGitHubPRFilter({ action_required_only: !filter.value.action_required_only })
}

function clearAll() {
  prefs.updateGitHubPRFilter({ roles: [], hide_drafts: false, action_required_only: false })
}
</script>

<template>
  <FilterPopover :active="active">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Filter PRs</span>
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
      <div class="space-y-1.5">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="filter.hide_drafts" @change="toggleHideDrafts" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">Hide drafts</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="filter.action_required_only" @change="toggleActionRequired" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">Action required only</span>
        </label>
      </div>
    </div>
  </FilterPopover>
</template>
