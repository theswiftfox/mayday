<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/lib/api'
import MarkdownContent from '@/components/MarkdownContent.vue'
import CommentThread from '@/components/CommentThread.vue'
import type { ThreadComment } from '@/components/CommentThread.vue'

const route = useRoute()
const mr = ref<any>(null)
const loading = ref(true)
const error = ref('')
const showSystemNotes = ref(false)
const showResolved = ref(false)

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

// Transform discussions into a list for display
const discussions = computed(() => {
  if (!mr.value?.discussions) return []
  return mr.value.discussions as Array<{
    id: string
    individual_note: boolean
    notes: Array<{
      id: number
      author: string
      body: string
      created_at: string
      system: boolean
      resolvable: boolean
      resolved: boolean
    }>
  }>
})

// Filter logic: hide system notes and optionally hide resolved threads
const filteredDiscussions = computed(() => {
  return discussions.value
    .map((d) => {
      // Filter system notes unless toggled on
      const notes = showSystemNotes.value
        ? d.notes
        : d.notes.filter((n) => !n.system)
      return { ...d, notes }
    })
    .filter((d) => {
      // Remove empty discussions (all notes were system and hidden)
      if (!d.notes.length) return false
      // Hide resolved threads unless toggled on
      if (!showResolved.value && isDiscussionResolved(d)) return false
      return true
    })
})

const resolvedCount = computed(() =>
  discussions.value.filter((d) => isDiscussionResolved(d)).length
)

function isDiscussionResolved(d: { notes: Array<{ resolvable: boolean; resolved: boolean }> }): boolean {
  // A discussion is resolved if all resolvable notes are resolved
  const resolvable = d.notes.filter((n) => n.resolvable)
  return resolvable.length > 0 && resolvable.every((n) => n.resolved)
}

function toThreadComments(notes: any[]): ThreadComment[] {
  return notes.map((n) => ({
    id: n.id,
    author: n.author,
    body: n.body,
    created_at: n.created_at,
    tag: n.system ? 'system' : undefined,
  }))
}

function formatDate(iso: string): string {
  if (!iso) return ''
  return new Date(iso).toLocaleString()
}
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
      <div v-if="mr.description" class="mb-8 p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <MarkdownContent :content="mr.description" />
      </div>

      <!-- Activity header -->
      <div class="flex items-center gap-4 mb-4">
        <h2 class="text-lg font-semibold text-[var(--color-text)]">Activity</h2>
        <label class="flex items-center gap-2 text-sm cursor-pointer" style="color: var(--color-text-muted)">
          <input type="checkbox" v-model="showSystemNotes" class="rounded" />
          System notes
        </label>
        <label v-if="resolvedCount > 0" class="flex items-center gap-2 text-sm cursor-pointer" style="color: var(--color-text-muted)">
          <input type="checkbox" v-model="showResolved" class="rounded" />
          Show resolved ({{ resolvedCount }})
        </label>
      </div>

      <!-- Discussions -->
      <div v-if="filteredDiscussions.length" class="space-y-3">
        <template v-for="discussion in filteredDiscussions" :key="discussion.id">
          <!-- Single note (individual_note = true): render as a simple comment -->
          <div
            v-if="discussion.individual_note && discussion.notes.length === 1"
            class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-sm font-medium" style="color: var(--color-text)">{{ discussion.notes[0].author }}</span>
              <span
                v-if="discussion.notes[0].system"
                class="text-xs px-2 py-0.5 rounded bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
              >system</span>
              <span class="text-xs ml-auto" style="color: var(--color-text-muted)">{{ formatDate(discussion.notes[0].created_at) }}</span>
            </div>
            <MarkdownContent :content="discussion.notes[0].body" />
          </div>

          <!-- Threaded discussion (multiple notes or resolvable) -->
          <CommentThread
            v-else
            :comments="toThreadComments(discussion.notes)"
            :resolved="isDiscussionResolved(discussion)"
          />
        </template>
      </div>
      <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
        No activity yet.
      </p>
    </div>
  </div>
</template>
