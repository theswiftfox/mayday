<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'

interface Ticket {
  key: string
  title: string
  url: string
  status: string
  status_category: 'todo' | 'in_progress' | 'done'
  priority?: string
  issue_type?: string
  sprint_name?: string
}

const props = defineProps<{ ticket: Ticket }>()

const route = computed(() => `/jira/${props.ticket.key}`)

const statusStyle = computed(() => {
  switch (props.ticket.status_category) {
    case 'done': return { bg: 'var(--color-success)', color: 'white' }
    case 'in_progress': return { bg: 'var(--color-primary)', color: 'white' }
    default: return { bg: 'var(--color-surface-hover)', color: 'var(--color-text-muted)' }
  }
})

const priorityIcon = computed(() => {
  switch (props.ticket.priority?.toLowerCase()) {
    case 'highest': case 'critical': return '⬆⬆'
    case 'high': return '⬆'
    case 'medium': return '—'
    case 'low': return '⬇'
    case 'lowest': return '⬇⬇'
    default: return null
  }
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
    <!-- Header: key + priority -->
    <div class="flex items-center gap-2 mb-2">
      <span class="text-xs font-mono font-medium" style="color: var(--color-text-muted)">{{ ticket.key }}</span>
      <span v-if="ticket.issue_type" class="text-xs" style="color: var(--color-text-muted)">{{ ticket.issue_type }}</span>
      <div class="ml-auto flex items-center gap-1.5">
        <span v-if="priorityIcon" class="text-xs" :title="ticket.priority">{{ priorityIcon }}</span>
        <a :href="ticket.url" target="_blank" @click.stop title="Open in JIRA" class="ml-1 opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--color-text-muted)">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
        </a>
      </div>
    </div>

    <!-- Title -->
    <div class="text-sm font-medium leading-snug mb-2" style="color: var(--color-text)">{{ ticket.title }}</div>

    <!-- Footer: status + sprint -->
    <div class="flex items-center gap-2 flex-wrap">
      <span
        class="text-xs px-2 py-0.5 rounded-full font-medium"
        :style="{ background: statusStyle.bg, color: statusStyle.color }"
      >{{ ticket.status }}</span>
      <span
        v-if="ticket.sprint_name"
        class="text-xs px-2 py-0.5 rounded-full"
        style="background: var(--color-surface-hover); color: var(--color-text-muted)"
      >{{ ticket.sprint_name }}</span>
    </div>
  </RouterLink>
</template>
