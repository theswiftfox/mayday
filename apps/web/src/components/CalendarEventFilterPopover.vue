<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import FilterPopover from './FilterPopover.vue'
import { hasActiveFilter } from '@/composables/useFilteredItems'

const prefs = useDashboardPrefsStore()
const filter = computed(() => prefs.filters.calendar_event)
const active = computed(() => hasActiveFilter('calendar_event', prefs.filters))

function toggleHideAllDay() {
  prefs.updateCalendarEventFilter({ hide_all_day: !filter.value.hide_all_day })
}

function toggleOnlineOnly() {
  prefs.updateCalendarEventFilter({ online_only: !filter.value.online_only })
}

function clearAll() {
  prefs.updateCalendarEventFilter({ hide_all_day: false, online_only: false })
}
</script>

<template>
  <FilterPopover :active="active">
    <div class="space-y-3">
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--color-text-muted)">Filter Events</span>
        <button v-if="active" @click="clearAll" class="text-xs hover:underline" style="color: var(--color-primary)">Clear</button>
      </div>

      <div class="space-y-1.5">
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="filter.hide_all_day" @change="toggleHideAllDay" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">Hide all-day events</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input type="checkbox" :checked="filter.online_only" @change="toggleOnlineOnly" class="rounded" />
          <span class="text-sm" style="color: var(--color-text)">Online meetings only</span>
        </label>
      </div>
    </div>
  </FilterPopover>
</template>
