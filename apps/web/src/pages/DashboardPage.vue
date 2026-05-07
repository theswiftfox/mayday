<script setup lang="ts">
import { useDashboardStore } from '@/stores/dashboard'
import { useAutoRefresh } from '@/composables/useAutoRefresh'
import DashboardSection from '@/components/DashboardSection.vue'
import MeetingCard from '@/components/MeetingCard.vue'
import PRCard from '@/components/PRCard.vue'
import TicketCard from '@/components/TicketCard.vue'
import MRCard from '@/components/MRCard.vue'
import PipelineCard from '@/components/PipelineCard.vue'
import ErrorBanner from '@/components/ErrorBanner.vue'

const store = useDashboardStore()
const { refresh, isRefreshing } = useAutoRefresh(() => store.fetchDashboard())
</script>

<template>
  <div class="p-6 max-w-7xl mx-auto">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold">Good morning</h1>
        <p class="text-sm text-[var(--color-text-muted)]">
          Here's your day at a glance
        </p>
      </div>
      <div class="flex items-center gap-3">
        <span
          v-if="store.refreshing"
          class="text-xs text-[var(--color-text-muted)] animate-pulse"
        >
          Updating...
        </span>
        <button
          @click="refresh"
          :disabled="isRefreshing"
          class="px-4 py-2 text-sm rounded-md bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-hover)] disabled:opacity-50 transition-colors"
        >
          {{ isRefreshing ? 'Refreshing...' : 'Refresh' }}
        </button>
      </div>
    </div>

    <!-- Errors -->
    <ErrorBanner v-if="store.errors.length" :errors="store.errors" />

    <!-- Loading state -->
    <div v-if="store.loading && !store.items.length" class="text-center py-12 text-[var(--color-text-muted)]">
      Loading your day...
    </div>

    <!-- Content: two-column layout -->
    <div v-else class="flex gap-6">
      <!-- Main column: PRs, MRs, Pipelines, Tickets -->
      <div class="flex-1 min-w-0 space-y-8">
        <!-- GitHub PRs (card grid) -->
        <DashboardSection
          v-if="store.githubPRs.length"
          title="GitHub PRs"
          :count="store.githubPRs.length"
          link="/github"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <PRCard
              v-for="pr in store.githubPRs.slice(0, 6)"
              :key="pr.id"
              :pr="pr"
            />
          </div>
        </DashboardSection>

        <!-- GitLab MRs (card grid) -->
        <DashboardSection
          v-if="store.gitlabMRs.length"
          title="GitLab MRs"
          :count="store.gitlabMRs.length"
          link="/gitlab"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <MRCard
              v-for="mr in store.gitlabMRs.slice(0, 6)"
              :key="mr.id"
              :mr="mr"
            />
          </div>
        </DashboardSection>

        <!-- GitLab Pipelines (only failed/running) -->
        <DashboardSection
          v-if="store.failedPipelines.length"
          title="Pipelines"
          :count="store.failedPipelines.length"
          link="/gitlab"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <PipelineCard
              v-for="pipeline in store.failedPipelines.slice(0, 6)"
              :key="pipeline.id"
              :pipeline="pipeline"
            />
          </div>
        </DashboardSection>

        <!-- JIRA Tickets (card grid) -->
        <DashboardSection
          v-if="store.jiraTickets.length"
          title="JIRA Tickets"
          :count="store.jiraTickets.length"
          link="/jira"
        >
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <TicketCard
              v-for="ticket in store.jiraTickets.slice(0, 6)"
              :key="ticket.id"
              :ticket="ticket"
            />
          </div>
        </DashboardSection>

        <!-- Empty state -->
        <div v-if="!store.items.length && !store.loading" class="text-center py-12">
          <p class="text-[var(--color-text-muted)]">No items to show.</p>
          <p class="text-sm text-[var(--color-text-muted)] mt-1">
            Configure your integrations in
            <RouterLink to="/settings" class="text-[var(--color-primary)] hover:underline">Settings</RouterLink>
          </p>
        </div>
      </div>

      <!-- Right column: Calendar -->
      <aside
        v-if="store.calendarEvents.length"
        class="hidden lg:block w-80 shrink-0"
      >
        <DashboardSection
          title="Calendar"
          :count="store.calendarEvents.length"
          link="/calendar"
        >
          <div class="space-y-2">
            <MeetingCard
              v-for="event in store.calendarEvents.slice(0, 8)"
              :key="event.id"
              :meeting="event"
            />
          </div>
        </DashboardSection>
      </aside>
    </div>

    <!-- Calendar fallback for small screens (shown below content when sidebar hidden) -->
    <div
      v-if="store.calendarEvents.length"
      class="lg:hidden mt-8"
    >
      <DashboardSection
        title="Calendar"
        :count="store.calendarEvents.length"
        link="/calendar"
      >
        <div class="space-y-2">
          <MeetingCard
            v-for="event in store.calendarEvents.slice(0, 5)"
            :key="event.id"
            :meeting="event"
          />
        </div>
      </DashboardSection>
    </div>

    <!-- Last updated -->
    <p v-if="store.lastUpdated" class="text-xs text-[var(--color-text-muted)] mt-6 text-right">
      Last updated: {{ new Date(store.lastUpdated).toLocaleTimeString() }}
    </p>
  </div>
</template>
