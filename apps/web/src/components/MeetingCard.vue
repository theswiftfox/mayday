<script setup lang="ts">
import { computed } from 'vue'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'

interface Meeting {
  id?: string
  subject: string
  start_time: string
  end_time: string
  is_online: boolean
  online_url?: string
  location?: string
  organizer?: string
  is_all_day?: boolean
}

const props = defineProps<{ meeting: Meeting; showPin?: boolean }>()

const prefs = useDashboardPrefsStore()
const pinned = computed(() => prefs.isPinned('calendar_event', props.meeting))

function togglePin(e: Event) {
  e.preventDefault()
  e.stopPropagation()
  prefs.togglePin('calendar_event', props.meeting)
}

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div
    class="group relative flex items-center gap-3 px-4 py-3 rounded-lg border transition-colors"
    style="background: var(--color-surface); border-color: var(--color-border)"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface)'"
  >
    <!-- Time column -->
    <div class="shrink-0 w-14 text-right">
      <div v-if="meeting.is_all_day" class="text-xs font-medium" style="color: var(--color-text-muted)">All day</div>
      <template v-else>
        <div class="text-sm font-medium font-mono" style="color: var(--color-text)">{{ formatTime(meeting.start_time) }}</div>
        <div class="text-xs font-mono" style="color: var(--color-text-muted)">{{ formatTime(meeting.end_time) }}</div>
      </template>
    </div>

    <!-- Divider -->
    <div class="w-px h-8 shrink-0" style="background: var(--color-border)"></div>

    <!-- Content -->
    <div class="flex-1 min-w-0">
      <div class="text-sm font-medium truncate" style="color: var(--color-text)">{{ meeting.subject }}</div>
      <div v-if="meeting.organizer || meeting.location" class="text-xs mt-0.5 truncate" style="color: var(--color-text-muted)">
        <span v-if="meeting.organizer">{{ meeting.organizer }}</span>
        <span v-if="meeting.organizer && meeting.location"> · </span>
        <span v-if="meeting.location">{{ meeting.location }}</span>
      </div>
    </div>

    <!-- Join button -->
    <a
      v-if="meeting.is_online && meeting.online_url"
      :href="meeting.online_url"
      target="_blank"
      class="shrink-0 text-xs px-3 py-1.5 rounded-md font-medium transition-colors"
      style="background: var(--color-primary); color: white"
      @click.stop
    >Join</a>

    <!-- Pin button (absolute, top-right on hover) -->
    <button
      v-if="showPin"
      @click="togglePin"
      class="absolute top-1 right-1 p-1 rounded transition-colors"
      :class="pinned ? '' : 'opacity-0 group-hover:opacity-100'"
      :style="{ color: pinned ? 'var(--color-warning)' : 'var(--color-text-muted)' }"
      :title="pinned ? 'Unpin from Important' : 'Pin to Important'"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3" :viewBox="pinned ? '0 0 20 20' : '0 0 24 24'" :fill="pinned ? 'currentColor' : 'none'" :stroke="pinned ? 'none' : 'currentColor'" :stroke-width="pinned ? undefined : '2'">
        <path v-if="pinned" d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
        <path v-else stroke-linecap="round" stroke-linejoin="round" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
      </svg>
    </button>
  </div>
</template>
