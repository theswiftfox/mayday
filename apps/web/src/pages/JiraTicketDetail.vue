<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/lib/api'
import MarkdownContent from '@/components/MarkdownContent.vue'

const route = useRoute()
const ticket = ref<any>(null)
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const { key } = route.params as { key: string }
    const { data } = await api.getJiraTicketDetail(key)
    ticket.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load ticket details'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <router-link to="/jira" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] mb-4 inline-block">&larr; Back to JIRA Tickets</router-link>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else-if="ticket">
      <div class="flex items-start justify-between mb-6">
        <div>
          <span class="text-sm text-[var(--color-text-muted)]">{{ ticket.key }}</span>
          <h1 class="text-2xl font-bold text-[var(--color-text)]">{{ ticket.title }}</h1>
        </div>
        <a :href="ticket.url" target="_blank" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] shrink-0 ml-4">Open in JIRA &#8599;</a>
      </div>

      <!-- Description (JIRA returns rendered HTML) -->
      <div v-if="ticket.description" class="mb-8 p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
        <div class="markdown-content" v-html="ticket.description"></div>
      </div>

      <!-- Subtasks -->
      <section v-if="ticket.subtasks?.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Subtasks</h2>
        <div class="space-y-2">
          <div v-for="subtask in ticket.subtasks" :key="subtask.key" class="flex items-center gap-3 p-3 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
            <span class="text-xs text-[var(--color-text-muted)]">{{ subtask.key }}</span>
            <span class="text-[var(--color-text)]">{{ subtask.title }}</span>
            <span class="text-xs px-2 py-0.5 rounded bg-[var(--color-surface-hover)] text-[var(--color-text-muted)] ml-auto">{{ subtask.status }}</span>
          </div>
        </div>
      </section>

      <!-- Comments -->
      <section v-if="ticket.comments?.length">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Comments</h2>
        <div class="space-y-3">
          <div v-for="comment in ticket.comments" :key="comment.id" class="p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
            <div class="flex items-center gap-2 mb-2">
              <span class="font-medium text-[var(--color-text)]">{{ comment.author }}</span>
              <span class="text-xs text-[var(--color-text-muted)] ml-auto">{{ new Date(comment.created_at).toLocaleString() }}</span>
            </div>
            <MarkdownContent :content="comment.body" />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
