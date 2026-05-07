<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

interface PR {
  number: number
  title: string
  url: string
  repo: string
  role: 'author' | 'reviewer' | 'other'
  has_new_comments: boolean
  has_new_commits: boolean
  action_required: boolean
  labels?: string[]
  is_draft: boolean
  review_decision?: string
  author?: string
  ci_status?: string | null
}

const props = defineProps<{ pr: PR }>()

const owner = computed(() => props.pr.repo.split('/')[0])
const repo = computed(() => props.pr.repo.split('/')[1])
const route = computed(() => `/github/${owner.value}/${repo.value}/${props.pr.number}`)

const reviewIcon = computed(() => {
  switch (props.pr.review_decision) {
    case 'approved': return { icon: '✓', color: 'var(--color-success)' }
    case 'changes_requested': return { icon: '✗', color: 'var(--color-error)' }
    case 'review_required': return { icon: '○', color: 'var(--color-warning)' }
    default: return null
  }
})

const ciColor = computed(() => {
  switch (props.pr.ci_status) {
    case 'success': return 'var(--color-success, #22c55e)'
    case 'failure': return 'var(--color-error, #ef4444)'
    case 'pending': return 'var(--color-text-muted, #9ca3af)'
    default: return null
  }
})

const ciTitle = computed(() => {
  switch (props.pr.ci_status) {
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
      background: pr.action_required ? 'var(--color-action-required-bg, rgba(239, 68, 68, 0.04))' : 'var(--color-surface)',
      borderColor: pr.action_required ? 'var(--color-error)' : 'var(--color-border)'
    }"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = pr.action_required ? 'var(--color-action-required-bg, rgba(239, 68, 68, 0.04))' : 'var(--color-surface)'"
  >
    <!-- Header: PR number + CI status + indicators -->
    <div class="flex items-center gap-2 mb-2">
      <span class="text-xs font-mono" style="color: var(--color-text-muted)">#{{ pr.number }}</span>
      <span v-if="ciColor" class="w-2 h-2 rounded-full" :style="{ background: ciColor }" :title="ciTitle" />
      <span v-if="pr.role === 'other' && pr.author" class="text-xs" style="color: var(--color-text-muted)">by {{ pr.author }}</span>
      <div class="ml-auto flex items-center gap-1.5">
        <span v-if="pr.has_new_comments" class="w-2 h-2 rounded-full" style="background: var(--color-primary)" title="New comments" />
        <span v-if="pr.has_new_commits" class="w-2 h-2 rounded-full" style="background: var(--color-success)" title="New commits" />
        <span v-if="reviewIcon" class="text-sm font-bold" :style="{ color: reviewIcon.color }">{{ reviewIcon.icon }}</span>
        <a :href="pr.url" target="_blank" @click.stop title="Open on GitHub" class="ml-1 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--color-text-muted)">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
        </a>
      </div>
    </div>

    <!-- Title -->
    <div class="text-sm font-medium leading-snug mb-2" style="color: var(--color-text)">{{ pr.title }}</div>

    <!-- Footer: badges -->
    <div class="flex items-center gap-2 flex-wrap">
      <span
        v-if="pr.action_required"
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
      <span v-if="pr.is_draft" class="text-xs px-2 py-0.5 rounded-full" style="background: var(--color-surface-hover); color: var(--color-text-muted)">draft</span>
      <span
        v-for="label in (pr.labels || []).slice(0, 2)"
        :key="label"
        class="text-xs px-2 py-0.5 rounded-full"
        style="background: var(--color-surface-hover); color: var(--color-text-muted)"
      >{{ label }}</span>
    </div>
  </RouterLink>
</template>
