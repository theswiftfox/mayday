<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { useFilteredCalendarEvents } from '@/composables/useFilteredItems'
import { useImportantItems } from '@/composables/useImportantItems'
import CalendarEventFilterPopover from '@/components/CalendarEventFilterPopover.vue'

const events = ref<any[]>([])
const loading = ref(true)
const error = ref('')
const prefs = useDashboardPrefsStore()

onMounted(async () => {
  await prefs.load()
  try {
    const { data } = await api.getCalendarEvents()
    events.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load calendar events'
  } finally {
    loading.value = false
  }
})

// Apply shared filters
const filtered = useFilteredCalendarEvents(events, computed(() => prefs.filters.calendar_event))

// Split into important + rest
const split = useImportantItems(
  filtered,
  'calendar_event',
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems)
)

const importantEvents = computed(() => split.value.important)
const restEvents = computed(() => split.value.rest)

function formatTime(dateStr: string) {
  return new Date(dateStr).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const pinned = (event: any) => prefs.isPinned('calendar_event', event)

function togglePin(event: any) {
  prefs.togglePin('calendar_event', event)
}
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">Today's Calendar</h1>
      <div class="ml-auto">
        <CalendarEventFilterPopover />
      </div>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <template v-else>
      <!-- Important events -->
      <section v-if="importantEvents.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-warning)">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          Important
          <span class="text-xs px-1.5 py-0.5 rounded-full font-medium" style="background: var(--color-surface-hover); color: var(--color-text-muted)">{{ importantEvents.length }}</span>
        </h2>
        <div class="relative">
          <div class="absolute left-4 top-0 bottom-0 w-px" style="background: var(--color-warning)"></div>
          <div v-for="event in importantEvents" :key="event.id" class="relative pl-10 pb-6">
            <div class="absolute left-3 top-2 w-3 h-3 rounded-full border-2" style="background: var(--color-warning); border-color: var(--color-background)"></div>
            <div class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
              <div class="flex items-center gap-3 mb-2">
                <span v-if="event.is_all_day" class="text-sm font-mono text-[var(--color-primary)]">All day</span>
                <span v-else class="text-sm font-mono text-[var(--color-primary)]">{{ formatTime(event.start_time) }} &ndash; {{ formatTime(event.end_time) }}</span>
                <button
                  @click="togglePin(event)"
                  class="ml-auto p-0.5 rounded transition-colors"
                  :style="{ color: pinned(event) ? 'var(--color-warning)' : 'var(--color-text-muted)' }"
                  :title="pinned(event) ? 'Unpin from Important' : 'Pin to Important'"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                  </svg>
                </button>
              </div>
              <h3 class="text-lg font-medium text-[var(--color-text)] mb-1">{{ event.subject }}</h3>
              <p v-if="event.organizer" class="text-sm text-[var(--color-text-muted)] mb-2">Organizer: {{ event.organizer }}</p>
              <p v-if="event.location" class="text-sm text-[var(--color-text-muted)] mb-2">{{ event.location }}</p>
              <a v-if="event.online_url" :href="event.online_url" target="_blank" class="inline-block text-sm text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">Join Meeting &nearr;</a>
            </div>
          </div>
        </div>
      </section>

      <!-- Rest of events -->
      <div v-if="restEvents.length" class="relative">
        <div class="absolute left-4 top-0 bottom-0 w-px bg-[var(--color-border)]"></div>
        <div v-for="event in restEvents" :key="event.id" class="relative pl-10 pb-6">
          <div class="absolute left-3 top-2 w-3 h-3 rounded-full bg-[var(--color-primary)] border-2 border-[var(--color-background)]"></div>
          <div class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] group">
            <div class="flex items-center gap-3 mb-2">
              <span v-if="event.is_all_day" class="text-sm font-mono text-[var(--color-primary)]">All day</span>
              <span v-else class="text-sm font-mono text-[var(--color-primary)]">{{ formatTime(event.start_time) }} &ndash; {{ formatTime(event.end_time) }}</span>
              <button
                @click="togglePin(event)"
                class="ml-auto p-0.5 rounded transition-colors opacity-0 group-hover:opacity-100"
                :style="{ color: 'var(--color-text-muted)' }"
                title="Pin to Important"
              >
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
              </button>
            </div>
            <h3 class="text-lg font-medium text-[var(--color-text)] mb-1">{{ event.subject }}</h3>
            <p v-if="event.organizer" class="text-sm text-[var(--color-text-muted)] mb-2">Organizer: {{ event.organizer }}</p>
            <p v-if="event.location" class="text-sm text-[var(--color-text-muted)] mb-2">{{ event.location }}</p>
            <a v-if="event.online_url" :href="event.online_url" target="_blank" class="inline-block text-sm text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">Join Meeting &nearr;</a>
          </div>
        </div>
      </div>
      <p v-if="!filtered.length" class="text-[var(--color-text-muted)]">No events today.</p>
    </template>
  </div>
</template>
