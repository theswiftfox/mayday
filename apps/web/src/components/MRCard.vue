<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'

interface MR {
  iid: number
  title: string
  url: string
  project: string
  project_id: number
  role: 'author' | 'reviewer'
  has_new_comments: boolean
  has_new_commits: boolean
  labels?: string[]
  is_draft: boolean
  merge_status?: string
}

const props = defineProps<{ mr: MR; showPin?: boolean }>()

const prefs = useDashboardPrefsStore()

const route = computed(() => `/gitlab/${props.mr.project_id}/${props.mr.iid}`)
const pinned = computed(() => prefs.isPinned('gitlab_mr', props.mr))

function togglePin(e: Event) {
  e.preventDefault()
  e.stopPropagation()
  prefs.togglePin('gitlab_mr', props.mr)
}

const mergeIcon = computed(() => {
  if (!props.mr.merge_status) return null
  return props.mr.merge_status === 'can_be_merged'
    ? { icon: '✓', color: 'var(--color-success)', label: 'Ready to merge' }
    : { icon: '⚠', color: 'var(--color-warning)', label: props.mr.merge_status }
})
</script>

<template>
  <RouterLink
    :to="route"
    class="group block p-4 rounded-lg border transition-all hover:shadow-sm"
    style="background: var(--color-surface); border-color: var(--color-border)"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface)'"
  >
    <!-- Header: project ref + indicators -->
    <div class="flex items-center gap-2 mb-2">
      <span class="text-xs font-mono" style="color: var(--color-text-muted)">{{ mr.project }}!{{ mr.iid }}</span>
      <div class="ml-auto flex items-center gap-1.5">
        <button
          v-if="showPin"
          @click="togglePin"
          class="p-0.5 rounded transition-colors"
          :class="pinned ? '' : 'opacity-0 group-hover:opacity-100'"
          :style="{ color: pinned ? 'var(--color-warning)' : 'var(--color-text-muted)' }"
          :title="pinned ? 'Unpin from Important' : 'Pin to Important'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" :viewBox="pinned ? '0 0 20 20' : '0 0 24 24'" :fill="pinned ? 'currentColor' : 'none'" :stroke="pinned ? 'none' : 'currentColor'" :stroke-width="pinned ? undefined : '2'">
            <path v-if="pinned" d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            <path v-else stroke-linecap="round" stroke-linejoin="round" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
          </svg>
        </button>
        <span v-if="mr.has_new_comments" class="w-2 h-2 rounded-full" style="background: var(--color-primary)" title="New comments" />
        <span v-if="mr.has_new_commits" class="w-2 h-2 rounded-full" style="background: var(--color-success)" title="New commits" />
        <span v-if="mergeIcon" class="text-sm font-bold" :style="{ color: mergeIcon.color }" :title="mergeIcon.label">{{ mergeIcon.icon }}</span>
        <a :href="mr.url" target="_blank" @click.stop title="Open on GitLab" class="ml-1 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--color-text-muted)">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
        </a>
      </div>
    </div>

    <!-- Title -->
    <div class="text-sm font-medium leading-snug mb-2" style="color: var(--color-text)">{{ mr.title }}</div>

    <!-- Footer: badges -->
    <div class="flex items-center gap-2 flex-wrap">
      <span
        class="text-xs px-2 py-0.5 rounded-full font-medium text-white"
        :style="{
          background: mr.role === 'author' ? 'var(--color-primary)' : 'var(--color-warning)'
        }"
      >{{ mr.role }}</span>
      <span v-if="mr.is_draft" class="text-xs px-2 py-0.5 rounded-full" style="background: var(--color-surface-hover); color: var(--color-text-muted)">draft</span>
      <span
        v-for="label in (mr.labels || []).slice(0, 2)"
        :key="label"
        class="text-xs px-2 py-0.5 rounded-full"
        style="background: var(--color-surface-hover); color: var(--color-text-muted)"
      >{{ label }}</span>
    </div>
  </RouterLink>
</template>
