<!-- SPDX-License-Identifier: Apache-2.0 -->
<!-- SPDX-FileCopyrightText: 2026 Elena Gantner -->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { api } from '@/lib/api'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { useFilteredGitLabMRs, useFilteredGitLabPipelines } from '@/composables/useFilteredItems'
import { useImportantItems } from '@/composables/useImportantItems'
import MRCard from '@/components/MRCard.vue'
import PipelineCard from '@/components/PipelineCard.vue'
import GitLabMRFilterPopover from '@/components/GitLabMRFilterPopover.vue'
import GitLabPipelineFilterPopover from '@/components/GitLabPipelineFilterPopover.vue'

const mrs = ref<any[]>([])
const pipelines = ref<any[]>([])
const loading = ref(true)
const error = ref('')
const prefs = useDashboardPrefsStore()

onMounted(async () => {
  await prefs.load()
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

// Apply shared filters
const filteredMRs = useFilteredGitLabMRs(mrs, computed(() => prefs.filters.gitlab_mr))
const filteredPipelines = useFilteredGitLabPipelines(pipelines, computed(() => prefs.filters.gitlab_pipeline))

// Split MRs into important + rest
const mrSplit = useImportantItems(
  filteredMRs,
  'gitlab_mr',
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems)
)

// Split Pipelines into important + rest
const pipelineSplit = useImportantItems(
  filteredPipelines,
  'gitlab_pipeline',
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems)
)

const importantMRs = computed(() => mrSplit.value.important)
const importantPipelines = computed(() => pipelineSplit.value.important)
const hasImportant = computed(() => importantMRs.value.length + importantPipelines.value.length > 0)

const restAuthored = computed(() => mrSplit.value.rest.filter(mr => mr.role === 'author'))
const restReviewing = computed(() => mrSplit.value.rest.filter(mr => mr.role === 'reviewer'))
const restOther = computed(() => mrSplit.value.rest.filter(mr => mr.role === 'other'))
const restPipelines = computed(() => pipelineSplit.value.rest)
</script>

<template>
  <div class="p-6 max-w-4xl mx-auto">
    <div class="flex items-center gap-4 mb-6">
      <router-link to="/" class="text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]">&larr; Back</router-link>
      <h1 class="text-2xl font-bold text-[var(--color-text)]">GitLab</h1>
    </div>

    <div v-if="loading" class="text-[var(--color-text-muted)]">Loading...</div>
    <div v-else-if="error" class="text-red-500 bg-red-500/10 p-4 rounded">{{ error }}</div>
    <div v-else>
      <!-- Important (combined MRs + Pipelines) -->
      <section v-if="hasImportant" class="mb-8">
        <h2 class="text-lg font-semibold text-[var(--color-text)] mb-3 flex items-center gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-warning)">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          Important
          <span class="text-xs px-1.5 py-0.5 rounded-full font-medium" style="background: var(--color-surface-hover); color: var(--color-text-muted)">{{ importantMRs.length + importantPipelines.length }}</span>
        </h2>
        <div class="space-y-3">
          <MRCard v-for="mr in importantMRs" :key="mr.id" :mr="mr" show-pin />
          <PipelineCard v-for="pipeline in importantPipelines" :key="pipeline.id" :pipeline="pipeline" show-pin />
        </div>
      </section>

      <!-- Merge Requests -->
      <div class="flex items-center gap-3 mb-4">
        <h2 class="text-xl font-semibold text-[var(--color-text)]">Merge Requests</h2>
        <GitLabMRFilterPopover />
      </div>

      <section v-if="restAuthored.length" class="mb-6">
        <h3 class="text-base font-medium text-[var(--color-text-muted)] mb-3">Authored</h3>
        <div class="space-y-3">
          <MRCard v-for="mr in restAuthored" :key="mr.id" :mr="mr" show-pin />
        </div>
      </section>

      <section v-if="restReviewing.length" class="mb-6">
        <h3 class="text-base font-medium text-[var(--color-text-muted)] mb-3">Reviewing</h3>
        <div class="space-y-3">
          <MRCard v-for="mr in restReviewing" :key="mr.id" :mr="mr" show-pin />
        </div>
      </section>

      <section v-if="restOther.length" class="mb-8">
        <h3 class="text-base font-medium text-[var(--color-text-muted)] mb-3">Other</h3>
        <div class="space-y-3">
          <MRCard v-for="mr in restOther" :key="mr.id" :mr="mr" show-pin />
        </div>
      </section>

      <p v-if="!filteredMRs.length" class="text-[var(--color-text-muted)] mb-8">No merge requests found.</p>

      <!-- Pipelines -->
      <div class="flex items-center gap-3 mb-4">
        <h2 class="text-xl font-semibold text-[var(--color-text)]">Pipelines</h2>
        <GitLabPipelineFilterPopover />
      </div>
      <div v-if="restPipelines.length" class="space-y-3">
        <PipelineCard v-for="pipeline in restPipelines" :key="pipeline.id" :pipeline="pipeline" show-pin />
      </div>
      <p v-else class="text-[var(--color-text-muted)]">No pipelines found.</p>
    </div>
  </div>
</template>
