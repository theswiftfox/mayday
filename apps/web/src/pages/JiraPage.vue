<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { useFilteredJiraTickets } from '@/composables/useFilteredItems'
import { useImportantItems } from '@/composables/useImportantItems'
import { useNow } from '@/composables/useNow'
import TicketCard from '@/components/TicketCard.vue'
import JiraTicketFilterPopover from '@/components/JiraTicketFilterPopover.vue'

const tickets = ref<any[]>([])
const loading = ref(true)
const error = ref('')
const prefs = useDashboardPrefsStore()

onMounted(async () => {
  await prefs.load()
  try {
    const { data } = await api.getJiraTickets()
    tickets.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load JIRA tickets'
  } finally {
    loading.value = false
  }
})

// Apply shared filters
const filtered = useFilteredJiraTickets(tickets, computed(() => prefs.filters.jiraTicket))

// Split into important + rest
const now = useNow()
const split = useImportantItems(
  filtered,
  'jira_ticket',
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems),
  now
)

const importantTickets = computed(() => split.value.important)
const restTickets = computed(() => split.value.rest)

const inProgress = computed(() => restTickets.value.filter(t => t.statusCategory === 'in_progress'))
const todo = computed(() => restTickets.value.filter(t => t.statusCategory === 'todo'))
const other = computed(() => restTickets.value.filter(t => !['in_progress', 'todo'].includes(t.statusCategory)))
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">JIRA Tickets</h1>
      <div class="ml-auto">
        <JiraTicketFilterPopover />
      </div>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <!-- Important -->
      <section v-if="importantTickets.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-warning)">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          Important
          <span class="text-xs px-1.5 py-0.5 rounded-full font-medium" style="background: var(--color-surface-hover); color: var(--color-text-muted)">{{ importantTickets.length }}</span>
        </h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in importantTickets" :key="ticket.key" :ticket="ticket" show-pin />
        </div>
      </section>

      <section v-if="inProgress.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">In Progress</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in inProgress" :key="ticket.key" :ticket="ticket" show-pin />
        </div>
      </section>

      <section v-if="todo.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">To Do</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in todo" :key="ticket.key" :ticket="ticket" show-pin />
        </div>
      </section>

      <section v-if="other.length">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Other</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in other" :key="ticket.key" :ticket="ticket" show-pin />
        </div>
      </section>

      <p v-if="!filtered.length" class="text-[var(--color-text-muted)]">No tickets found.</p>
    </div>
  </div>
</template>
