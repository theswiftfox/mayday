<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import MRCard from '@/components/MRCard.vue'
import PipelineCard from '@/components/PipelineCard.vue'

const mrs = ref<any[]>([])
const pipelines = ref<any[]>([])
const loading = ref(true)
const error = ref('')

const authored = computed(() => mrs.value.filter(mr => mr.role === 'author'))
const reviewing = computed(() => mrs.value.filter(mr => mr.role === 'reviewer'))

onMounted(async () => {
  try {
    const [mrsRes, pipelinesRes] = await Promise.all([
      api.getGitLabMRs(),
      api.getGitLabPipelines(),
    ])
    mrs.value = mrsRes.data
    pipelines.value = pipelinesRes.data
  } catch (e: any) {
    error.value = e.message || 'Failed to load GitLab data'
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">← Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">GitLab</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <h2 class="text-xl font-semibold text-[var(--color-text)] mb-4">Merge Requests</h2>

      <section v-if="authored.length" class="mb-6">
        <h3 class="text-base font-medium text-[var(--color-text-muted)] mb-3">Authored</h3>
        <div class="space-y-3">
          <MRCard v-for="mr in authored" :key="mr.id" :mr="mr" />
        </div>
      </section>

      <section v-if="reviewing.length" class="mb-8">
        <h3 class="text-base font-medium text-[var(--color-text-muted)] mb-3">Reviewing</h3>
        <div class="space-y-3">
          <MRCard v-for="mr in reviewing" :key="mr.id" :mr="mr" />
        </div>
      </section>

      <p v-if="!mrs.length" class="text-[var(--color-text-muted)] mb-8">No merge requests found.</p>

      <h2 class="text-xl font-semibold text-[var(--color-text)] mb-4">Pipelines</h2>
      <div v-if="pipelines.length" class="space-y-3">
        <PipelineCard v-for="pipeline in pipelines" :key="pipeline.id" :pipeline="pipeline" />
      </div>
      <p v-else class="text-[var(--color-text-muted)]">No pipelines found.</p>
    </div>
  </div>
</template>
