<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'

interface PR {
  number: number
  title: string
  url: string
  repo: string
  role: 'author' | 'reviewer' | 'other'
  hasNewComments: boolean
  hasNewCommits: boolean
  actionRequired: boolean
  labels?: string[]
  isDraft: boolean
  reviewDecision?: string
  author?: string
  ciStatus?: string | null
}

const props = defineProps<{ pr: PR; showPin?: boolean }>()

const prefs = useDashboardPrefsStore()

const owner = computed(() => props.pr.repo.split('/')[0])
const repo = computed(() => props.pr.repo.split('/')[1])
const route = computed(() => `/github/${owner.value}/${repo.value}/${props.pr.number}`)
const pinned = computed(() => prefs.isPinned('github_pr', props.pr))

function togglePin(e: Event) {
  e.preventDefault()
  e.stopPropagation()
  prefs.togglePin('github_pr', props.pr)
}

const reviewIcon = computed(() => {
  switch (props.pr.reviewDecision) {
    case 'approved': return { icon: '✓', color: 'var(--color-success)' }
    case 'changes_requested': return { icon: '✗', color: 'var(--color-error)' }
    case 'review_required': return { icon: '○', color: 'var(--color-warning)' }
    default: return null
  }
})

const ciColor = computed(() => {
  switch (props.pr.ciStatus) {
    case 'success': return 'var(--color-success, #22c55e)'
    case 'failure': return 'var(--color-error, #ef4444)'
    case 'pending': return 'var(--color-text-muted, #9ca3af)'
    default: return null
  }
})

const ciTitle = computed(() => {
  switch (props.pr.ciStatus) {
    case 'success': return 'CI passed'
    case 'failure': return 'CI failed'
    case 'pending': return 'CI in progress'
    default: return ''
  }
})
</script>

<template>
  <RouterLink
    :to="route"
    class="group block p-4 rounded-lg border transition-all hover:shadow-sm relative"
    :style="{
      background: pr.actionRequired ? 'var(--color-action-required-bg, rgba(239, 68, 68, 0.04))' : 'var(--color-surface)',
      borderColor: pr.actionRequired ? 'var(--color-error)' : 'var(--color-border)'
    }"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = pr.actionRequired ? 'var(--color-action-required-bg, rgba(239, 68, 68, 0.04))' : 'var(--color-surface)'"
  >
    <!-- Header: PR number + CI status + indicators -->
    <div class="flex items-center gap-2 mb-2">
      <span class="text-xs font-mono" style="color: var(--color-text-muted)">#{{ pr.number }}</span>
      <span v-if="ciColor" class="w-2 h-2 rounded-full" :style="{ background: ciColor }" :title="ciTitle" />
      <span v-if="pr.role === 'other' && pr.author" class="text-xs" style="color: var(--color-text-muted)">by {{ pr.author }}</span>
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
        <span v-if="pr.hasNewComments" class="w-2 h-2 rounded-full" style="background: var(--color-primary)" title="New comments" />
        <span v-if="pr.hasNewCommits" class="w-2 h-2 rounded-full" style="background: var(--color-success)" title="New commits" />
        <span v-if="reviewIcon" class="text-sm font-bold" :style="{ color: reviewIcon.color }">{{ reviewIcon.icon }}</span>
        <a :href="pr.url" target="_blank" rel="noopener noreferrer" @click.stop title="Open on GitHub" class="ml-1 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--color-text-muted)">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
        </a>
      </div>
    </div>

    <!-- Title -->
    <div class="text-sm font-medium leading-snug mb-2" style="color: var(--color-text)">{{ pr.title }}</div>

    <!-- Footer: badges -->
    <div class="flex items-center gap-2 flex-wrap">
      <span
        v-if="pr.actionRequired"
        class="text-xs px-2 py-0.5 rounded-full font-semibold"
        style="background: var(--color-error); color: white"
      >action required</span>
      <span
        v-if="pr.role !== 'other'"
        class="text-xs px-2 py-0.5 rounded-full font-medium"
        :style="{
          background: pr.role === 'author' ? 'var(--color-primary)' : 'var(--color-warning)',
          color: 'white'
        }"
      >{{ pr.role }}</span>
      <span v-if="pr.isDraft" class="text-xs px-2 py-0.5 rounded-full" style="background: var(--color-surface-hover); color: var(--color-text-muted)">draft</span>
      <span
        v-for="label in (pr.labels || []).slice(0, 2)"
        :key="label"
        class="text-xs px-2 py-0.5 rounded-full"
        style="background: var(--color-surface-hover); color: var(--color-text-muted)"
      >{{ label }}</span>
    </div>
  </RouterLink>
</template>
