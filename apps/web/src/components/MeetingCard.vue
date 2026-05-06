<script setup lang="ts">
interface Meeting {
  subject: string
  start_time: string
  end_time: string
  is_online: boolean
  online_url?: string
  location?: string
  organizer?: string
  is_all_day?: boolean
}

defineProps<{ meeting: Meeting }>()

function formatTime(iso: string): string {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div
    class="flex items-center gap-4 px-4 py-3 rounded-lg border transition-colors"
    style="background: var(--color-surface); border-color: var(--color-border)"
    @mouseenter="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface-hover)'"
    @mouseleave="($event.currentTarget as HTMLElement).style.background = 'var(--color-surface)'"
  >
    <!-- Time column -->
    <div class="shrink-0 w-24 text-right">
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
  </div>
</template>
