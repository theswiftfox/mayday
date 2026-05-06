<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/lib/api'

const events = ref<any[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const { data } = await api.getCalendarEvents()
    events.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load calendar events'
  } finally {
    loading.value = false
  }
})

function formatTime(dateStr: string) {
  return new Date(dateStr).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">Today's Calendar</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else-if="events.length" class="relative">
      <div class="absolute left-4 top-0 bottom-0 w-px bg-[var(--color-border)]"></div>
      <div v-for="event in events" :key="event.id" class="relative pl-10 pb-6">
        <div class="absolute left-3 top-2 w-3 h-3 rounded-full bg-[var(--color-primary)] border-2 border-[var(--color-background)]"></div>
        <div class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
          <div class="flex items-center gap-3 mb-2">
            <span v-if="event.is_all_day" class="text-sm font-mono text-[var(--color-primary)]">All day</span>
            <span v-else class="text-sm font-mono text-[var(--color-primary)]">{{ formatTime(event.start_time) }} – {{ formatTime(event.end_time) }}</span>
          </div>
          <h3 class="text-lg font-medium text-[var(--color-text)] mb-1">{{ event.subject }}</h3>
          <p v-if="event.organizer" class="text-sm text-[var(--color-text-muted)] mb-2">Organizer: {{ event.organizer }}</p>
          <p v-if="event.location" class="text-sm text-[var(--color-text-muted)] mb-2">{{ event.location }}</p>
          <a v-if="event.online_url" :href="event.online_url" target="_blank" class="inline-block text-sm text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">Join Meeting &nearr;</a>
        </div>
      </div>
    </div>
    <p v-else class="text-[var(--color-text-muted)]">No events today.</p>
  </div>
</template>
