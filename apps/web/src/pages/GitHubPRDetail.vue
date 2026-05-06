<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { api } from '@/lib/api'
import MarkdownContent from '@/components/MarkdownContent.vue'

const route = useRoute()
const pr = ref<any>(null)
const loading = ref(true)
const error = ref('')

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
      <div v-if="pr.body" class="mb-8 p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
        <MarkdownContent :content="pr.body" />
      </div>

      <!-- Reviews -->
      <section v-if="pr.reviews?.length" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Reviews</h2>
        <div class="space-y-3">
          <div v-for="review in pr.reviews" :key="review.id" class="p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
            <div class="flex items-center gap-2 mb-2">
              <span class="font-medium text-[var(--color-text)]">{{ review.author }}</span>
              <span
                class="text-xs px-2 py-0.5 rounded"
                :class="{
                  'bg-green-500/10 text-green-600': review.state === 'APPROVED',
                  'bg-red-500/10 text-red-600': review.state === 'CHANGES_REQUESTED',
                  'bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]': review.state !== 'APPROVED' && review.state !== 'CHANGES_REQUESTED',
                }"
              >{{ review.state }}</span>
              <span class="text-xs text-[var(--color-text-muted)] ml-auto">{{ new Date(review.submitted_at).toLocaleString() }}</span>
            </div>
            <MarkdownContent v-if="review.body" :content="review.body" />
          </div>
        </div>
      </section>

      <!-- Comments -->
      <section v-if="pr.comments?.length">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3">Comments</h2>
        <div class="space-y-3">
          <div v-for="comment in pr.comments" :key="comment.id" class="p-4 rounded bg-[var(--color-surface)] border border-[var(--color-border)]">
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
