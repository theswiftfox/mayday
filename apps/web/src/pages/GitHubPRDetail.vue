<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/lib/api'
import MarkdownContent from '@/components/MarkdownContent.vue'
import CommentThread from '@/components/CommentThread.vue'
import type { ThreadComment } from '@/components/CommentThread.vue'

const route = useRoute()
const pr = ref<any>(null)
const loading = ref(true)
const error = ref('')
const showResolved = ref(false)

onMounted(async () => {
  try {
    const { owner, repo, number } = route.params as { owner: string; repo: string; number: string }
    const { data } = await api.getGitHubPRDetail(owner, repo, Number(number))
    pr.value = data
  } catch (e: any) {
    error.value = e.message || 'Failed to load PR details'
  } finally {
    loading.value = false
  }
})

// Build a unified timeline: reviews + issue comments + review threads, sorted by date
const timeline = computed(() => {
  if (!pr.value) return []

  const items: Array<{
    type: 'review' | 'issue_comment' | 'review_thread'
    date: string
    data: any
  }> = []

  // Reviews (approval / changes requested / commented)
  for (const review of pr.value.reviews || []) {
    // Skip empty COMMENTED reviews (they're just the container for review threads)
    if (review.state === 'COMMENTED' && !review.body) continue
    items.push({
      type: 'review',
      date: review.submitted_at,
      data: review,
    })
  }

  // Issue comments (top-level conversation)
  for (const comment of pr.value.issue_comments || []) {
    items.push({
      type: 'issue_comment',
      date: comment.created_at,
      data: comment,
    })
  }

  // Review threads (code review discussions)
  for (const thread of pr.value.review_threads || []) {
    const firstComment = thread.comments?.[0]
    items.push({
      type: 'review_thread',
      date: firstComment?.created_at || '',
      data: thread,
    })
  }

  items.sort((a, b) => a.date.localeCompare(b.date))
  return items
})

const visibleTimeline = computed(() => {
  if (showResolved.value) return timeline.value
  return timeline.value.filter(
    (item) => item.type !== 'review_thread' || !item.data.is_resolved
  )
})

const resolvedCount = computed(() =>
  (pr.value?.review_threads || []).filter((t: any) => t.is_resolved).length
)

function toThreadComments(thread: any): ThreadComment[] {
  return (thread.comments || []).map((c: any) => ({
    id: c.id,
    author: c.author,
    body: c.body,
    created_at: c.created_at,
  }))
}

function reviewStateClass(state: string): string {
  switch (state) {
    case 'APPROVED':
      return 'bg-green-500/10 text-green-500'
    case 'CHANGES_REQUESTED':
      return 'bg-red-500/10 text-red-500'
    default:
      return 'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]'
  }
}

function formatDate(iso: string): string {
  if (!iso) return ''
  return new Date(iso).toLocaleString()
}
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <router-link to="/github" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] mb-4 inline-block">&larr; Back to GitHub PRs</router-link>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else-if="pr">
      <div class="flex items-start justify-between mb-6">
        <h1 class="text-2xl font-bold text-[var(--color-text)]">{{ pr.title }}</h1>
        <a :href="pr.url" target="_blank" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)] shrink-0 ml-4">Open in GitHub &#8599;</a>
      </div>

      <!-- PR Body -->
      <div v-if="pr.body" class="mb-8 p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]">
        <MarkdownContent :content="pr.body" />
      </div>

      <!-- Activity header -->
      <div class="flex items-center gap-4 mb-4">
        <h2 class="text-lg font-semibold text-[var(--color-text)]">Activity</h2>
        <label v-if="resolvedCount > 0" class="flex items-center gap-2 text-sm cursor-pointer" style="color: var(--color-text-muted)">
          <input type="checkbox" v-model="showResolved" class="rounded" />
          Show resolved ({{ resolvedCount }})
        </label>
      </div>

      <!-- Unified timeline -->
      <div v-if="visibleTimeline.length" class="space-y-3">
        <template v-for="item in visibleTimeline" :key="`${item.type}-${item.data.id || item.date}`">
          <!-- Review event (approval / changes requested) -->
          <div
            v-if="item.type === 'review'"
            class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-sm font-medium" style="color: var(--color-text)">{{ item.data.author }}</span>
              <span class="text-xs px-2 py-0.5 rounded" :class="reviewStateClass(item.data.state)">
                {{ item.data.state.replace('_', ' ') }}
              </span>
              <span class="text-xs ml-auto" style="color: var(--color-text-muted)">{{ formatDate(item.data.submitted_at) }}</span>
            </div>
            <MarkdownContent v-if="item.data.body" :content="item.data.body" />
          </div>

          <!-- Issue comment (top-level conversation) -->
          <div
            v-else-if="item.type === 'issue_comment'"
            class="p-4 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]"
          >
            <div class="flex items-center gap-2 mb-2">
              <span class="text-sm font-medium" style="color: var(--color-text)">{{ item.data.author }}</span>
              <span class="text-xs ml-auto" style="color: var(--color-text-muted)">{{ formatDate(item.data.created_at) }}</span>
            </div>
            <MarkdownContent :content="item.data.body" />
          </div>

          <!-- Review thread (code review discussion) -->
          <CommentThread
            v-else-if="item.type === 'review_thread'"
            :comments="toThreadComments(item.data)"
            :resolved="item.data.is_resolved"
            :outdated="item.data.is_outdated"
            :path="item.data.path"
            :line="item.data.line"
          />
        </template>
      </div>
      <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
        No activity yet.
      </p>
    </div>
  </div>
</template>
