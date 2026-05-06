<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import TicketCard from '@/components/TicketCard.vue'

const tickets = ref<any[]>([])
const loading = ref(true)
const error = ref('')

const inProgress = computed(() => tickets.value.filter(t => t.status_category === 'in_progress'))
const todo = computed(() => tickets.value.filter(t => t.status_category === 'todo'))
const other = computed(() => tickets.value.filter(t => !['in_progress', 'todo'].includes(t.status_category)))

onMounted(async () => {
  try {
    const { data } = await api.getJiraTickets()
    tickets.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load JIRA tickets'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">← Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">JIRA Tickets</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <section v-if="inProgress.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">In Progress</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in inProgress" :key="ticket.key" :ticket="ticket" />
        </div>
      </section>

      <section v-if="todo.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">To Do</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in todo" :key="ticket.key" :ticket="ticket" />
        </div>
      </section>

      <section v-if="other.length">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Other</h2>
        <div class="space-y-3">
          <TicketCard v-for="ticket in other" :key="ticket.key" :ticket="ticket" />
        </div>
      </section>

      <p v-if="!tickets.length" class="text-[var(--color-text-muted)]">No tickets found.</p>
    </div>
  </div>
</template>
