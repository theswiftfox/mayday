<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import type { ImportantRules } from '@/types/dashboard'

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

interface RuleGroup {
  label: string
  rules: Array<{ key: keyof ImportantRules; label: string }>
}

const ruleGroups: RuleGroup[] = [
  {
    label: 'GitHub PRs',
    rules: [
      { key: 'github_action_required', label: 'Action required' },
      { key: 'github_new_comments', label: 'New comments' },
      { key: 'github_new_commits', label: 'New commits' },
      { key: 'github_changes_requested', label: 'Changes requested' },
    ],
  },
  {
    label: 'GitLab MRs',
    rules: [
      { key: 'gitlab_mr_new_comments', label: 'New comments' },
      { key: 'gitlab_mr_new_commits', label: 'New commits' },
    ],
  },
  {
    label: 'GitLab Pipelines',
    rules: [
      { key: 'gitlab_pipeline_failed', label: 'Failed pipelines' },
    ],
  },
  {
    label: 'JIRA Tickets',
    rules: [
      { key: 'jira_high_priority', label: 'High/Critical priority' },
    ],
  },
  {
    label: 'Calendar',
    rules: [
      { key: 'calendar_starting_soon', label: 'Starting within 15 min' },
    ],
  },
]

function toggleRule(key: keyof ImportantRules) {
  prefs.updateImportantRule(key, !prefs.importantRules[key])
}

function hasAnyRule(): boolean {
  return Object.values(prefs.importantRules).some(Boolean)
}
</script>

<template>
  <div ref="popoverRef" class="relative">
    <button
      @click.stop="toggle"
      class="p-1 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
      title="Configure important rules"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-text-muted)">
        <path fill-rule="evenodd" d="M11.49 3.17c-.38-1.56-2.6-1.56-2.98 0a1.532 1.532 0 01-2.286.948c-1.372-.836-2.942.734-2.106 2.106.54.886.061 2.042-.947 2.287-1.561.379-1.561 2.6 0 2.978a1.532 1.532 0 01.947 2.287c-.836 1.372.734 2.942 2.106 2.106a1.532 1.532 0 012.287.947c.379 1.561 2.6 1.561 2.978 0a1.533 1.533 0 012.287-.947c1.372.836 2.942-.734 2.106-2.106a1.533 1.533 0 01.947-2.287c1.561-.379 1.561-2.6 0-2.978a1.532 1.532 0 01-.947-2.287c.836-1.372-.734-2.942-2.106-2.106a1.532 1.532 0 01-2.287-.947zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
      </svg>
    </button>

    <Transition name="popover">
      <div
        v-if="open"
        class="absolute right-0 top-full mt-2 z-50 min-w-64 rounded-lg border shadow-lg p-4"
        style="background: var(--color-surface); border-color: var(--color-border)"
      >
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Important Rules</span>
          </div>

          <p class="text-xs" style="color: var(--color-text-muted)">
            Items matching these rules automatically appear in the Important section.
          </p>

          <div v-for="group in ruleGroups" :key="group.label" class="space-y-1">
            <span class="text-xs font-medium block" style="color: var(--color-text-muted)">{{ group.label }}</span>
            <label
              v-for="rule in group.rules"
              :key="rule.key"
              class="flex items-center gap-2 py-0.5 cursor-pointer"
            >
              <input
                type="checkbox"
                :checked="prefs.importantRules[rule.key]"
                @change="toggleRule(rule.key)"
                class="rounded"
              />
              <span class="text-sm" style="color: var(--color-text)">{{ rule.label }}</span>
            </label>
          </div>

          <p v-if="!hasAnyRule()" class="text-xs italic" style="color: var(--color-text-muted)">
            No rules active. You can still pin items manually.
          </p>
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
