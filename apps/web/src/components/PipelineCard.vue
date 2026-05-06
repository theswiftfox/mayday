<script setup lang="ts">
import { computed } from 'vue'

interface Pipeline {
  id: number
  status: string
  ref_name: string
  url: string
  project: string
  duration?: number
  created_at: string
}

const props = defineProps<{ pipeline: Pipeline }>()

const statusColor = computed(() => {
  switch (props.pipeline.status) {
    case 'success': return 'var(--color-success)'
    case 'failed': return 'var(--color-error)'
    case 'running': return 'var(--color-primary)'
    case 'pending': return 'var(--color-warning)'
    default: return 'var(--color-text-muted)'
  }
})

const statusLabel = computed(() => {
  switch (props.pipeline.status) {
    case 'failed': return 'Failed'
    case 'running': return 'Running'
    case 'pending': return 'Pending'
    case 'canceled': return 'Canceled'
    default: return props.pipeline.status
  }
})

const formattedDuration = computed(() => {
  if (!props.pipeline.duration) return null
  const m = Math.floor(props.pipeline.duration / 60)
  const s = props.pipeline.duration % 60
  return m > 0 ? `${m}m ${s}s` : `${s}s`
})

const timeAgo = computed(() => {
  const diff = Date.now() - new Date(props.pipeline.created_at).getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24) return `${hrs}h ago`
  return `${Math.floor(hrs / 24)}d ago`
})
</script>

<template>
  <a
    :href="pipeline.url"
    target="_blank"
    class="group block p-4 rounded-lg border transition-all hover:shadow-sm"
    style="background: var(--color-surface); border-color: var(--color-border)"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface)'"
  >
    <!-- Header: status + time -->
    <div class="flex items-center gap-2 mb-2">
      <span class="w-2.5 h-2.5 rounded-full shrink-0" :style="{ background: statusColor }" />
      <span class="text-xs font-semibold uppercase" :style="{ color: statusColor }">{{ statusLabel }}</span>
      <span class="ml-auto text-xs" style="color: var(--color-text-muted)">{{ timeAgo }}</span>
      <span class="opacity-0 group-hover:opacity-100 transition-opacity" style="color: var(--color-text-muted)" title="Open pipeline">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M11 3a1 1 0 100 2h2.586l-6.293 6.293a1 1 0 101.414 1.414L15 6.414V9a1 1 0 102 0V4a1 1 0 00-1-1h-5z" /><path d="M5 5a2 2 0 00-2 2v8a2 2 0 002 2h8a2 2 0 002-2v-3a1 1 0 10-2 0v3H5V7h3a1 1 0 000-2H5z" /></svg>
      </span>
    </div>

    <!-- Branch + project -->
    <div class="text-sm font-medium font-mono mb-2" style="color: var(--color-text)">{{ pipeline.ref_name }}</div>
    <div class="text-xs" style="color: var(--color-text-muted)">{{ pipeline.project }}</div>

    <!-- Duration -->
    <div v-if="formattedDuration" class="mt-2 text-xs" style="color: var(--color-text-muted)">
      Duration: {{ formattedDuration }}
    </div>
  </a>
</template>
