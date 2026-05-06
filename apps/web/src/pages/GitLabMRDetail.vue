<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/lib/api'
import MarkdownContent from '@/components/MarkdownContent.vue'

const route = useRoute()
const mr = ref<any>(null)
const loading = ref(true)
const error = ref('')
const showSystemNotes = ref(false)

const filteredNotes = computed(() => {
  if (!mr.value?.notes) return []
  return showSystemNotes.value ? mr.value.notes : mr.value.notes.filter((n: any) => !n.system)
})

onMounted(async () => {
  try {
    const { projectId, iid } = route.params as { projectId: string; iid: string }
    const { data } = await api.getGitLabMRDetail(Number(projectId), Number(iid))
    mr.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load MR details'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <router-link to="/gitlab" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] mb-4 inline-block">&larr; Back to GitLab</router-link>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else-if="mr">
      <div class="flex items-start justify-between mb-6">
        <h1 class="text-2xl font-bold text-[var(--color-text)]">{{ mr.title }}</h1>
        <a :href="mr.url" target="_blank" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] shrink-0 ml-4">Open in GitLab &#8599;</a>
      </div>

      <!-- Pipeline status -->
      <div v-if="mr.pipelines?.length" class="mb-4 flex items-center gap-2">
        <span class="text-sm text-[var(--color-text-muted)]">Pipeline:</span>
        <span
          class="text-sm px-2 py-0.5 rounded"
          :class="{
            'bg-green-500/10 text-green-600': mr.pipelines[0].status === 'success',
            'bg-red-500/10 text-red-600': mr.pipelines[0].status === 'failed',
            'bg-blue-500/10 text-blue-600': mr.pipelines[0].status === 'running',
            'bg-yellow-500/10 text-yellow-600': mr.pipelines[0].status === 'pending',
            'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]': !['success','failed','running','pending'].includes(mr.pipelines[0].status),
          }"
        >{{ mr.pipelines[0].status }}</span>
      </div>

      <!-- Description -->
      <div v-if="mr.description" class="mb-8 p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
        <MarkdownContent :content="mr.description" />
      </div>

      <!-- Notes -->
      <section v-if="mr.notes?.length">
        <div class="flex items-center gap-4 mb-3">
          <h2 class="text-lg font-semibold text-[var(--color-text)]">Notes</h2>
          <label class="flex items-center gap-2 text-sm text-[var(--color-text-muted)] cursor-pointer">
            <input type="checkbox" v-model="showSystemNotes" class="rounded" />
            Show system notes
          </label>
        </div>
        <div class="space-y-3">
          <div v-for="note in filteredNotes" :key="note.id" class="p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
            <div class="flex items-center gap-2 mb-2">
              <span class="font-medium text-[var(--color-text)]">{{ note.author }}</span>
              <span v-if="note.system" class="text-xs px-2 py-0.5 rounded bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]">system</span>
              <span class="text-xs text-[var(--color-text-muted)] ml-auto">{{ new Date(note.created_at).toLocaleString() }}</span>
            </div>
            <MarkdownContent :content="note.body" />
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
