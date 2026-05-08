<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { VueDraggable } from 'vue-draggable-plus'
import { useDashboardStore } from '@/stores/dashboard'
import { useDashboardPrefsStore } from '@/stores/dashboardPrefs'
import { useAutoRefresh } from '@/composables/useAutoRefresh'
import {
  useFilteredGitHubPRs,
  useFilteredGitLabMRs,
  useFilteredGitLabPipelines,
  useFilteredJiraTickets,
  useFilteredCalendarEvents,
} from '@/composables/useFilteredItems'
import { useImportantItemsAll } from '@/composables/useImportantItems'
import type { SectionType } from '@/types/dashboard'
import DashboardSection from '@/components/DashboardSection.vue'
import MeetingCard from '@/components/MeetingCard.vue'
import PRCard from '@/components/PRCard.vue'
import TicketCard from '@/components/TicketCard.vue'
import MRCard from '@/components/MRCard.vue'
import PipelineCard from '@/components/PipelineCard.vue'
import ErrorBanner from '@/components/ErrorBanner.vue'
import GitHubPRFilterPopover from '@/components/GitHubPRFilterPopover.vue'
import GitLabMRFilterPopover from '@/components/GitLabMRFilterPopover.vue'
import GitLabPipelineFilterPopover from '@/components/GitLabPipelineFilterPopover.vue'
import JiraTicketFilterPopover from '@/components/JiraTicketFilterPopover.vue'
import CalendarEventFilterPopover from '@/components/CalendarEventFilterPopover.vue'
import ImportantRulesPopover from '@/components/ImportantRulesPopover.vue'
import SectionVisibilityPopover from '@/components/SectionVisibilityPopover.vue'

const store = useDashboardStore()
const prefs = useDashboardPrefsStore()

// Load prefs on mount
onMounted(() => prefs.load())

const { refresh, isRefreshing } = useAutoRefresh(() => store.fetchDashboard())

// Auto-cleanup stale pins after data refresh
watch(() => store.items, (items) => {
  if (items.length) prefs.cleanupStalePins(items)
})

// ---- Filtered data ----

const filteredGitHubPRs = useFilteredGitHubPRs(
  computed(() => store.githubPRs),
  computed(() => prefs.filters.github_pr)
)
const filteredGitLabMRs = useFilteredGitLabMRs(
  computed(() => store.gitlabMRs),
  computed(() => prefs.filters.gitlab_mr)
)
const filteredPipelines = useFilteredGitLabPipelines(
  computed(() => store.gitlabPipelines),
  computed(() => prefs.filters.gitlab_pipeline)
)
const filteredJiraTickets = useFilteredJiraTickets(
  computed(() => store.jiraTickets),
  computed(() => prefs.filters.jira_ticket)
)
const filteredCalendarEvents = useFilteredCalendarEvents(
  computed(() => store.calendarEvents),
  computed(() => prefs.filters.calendar_event)
)

// ---- Important items (across all types) ----

const importantSplit = useImportantItemsAll(
  computed(() => store.items),
  computed(() => prefs.importantRules),
  computed(() => prefs.pinnedItems)
)

const importantItems = computed(() => importantSplit.value.important)

// ---- Section order (draggable) ----
const sectionOrder = ref<SectionType[]>([...prefs.sectionOrder])

watch(() => prefs.sectionOrder, (newOrder) => {
  sectionOrder.value = [...newOrder]
}, { deep: true })

// Calendar layout toggle
function toggleCalendarLayout() {
  prefs.setCalendarLayout(prefs.calendarLayout === 'sidebar' ? 'inline' : 'sidebar')
}

function onDragEnd() {
  prefs.setSectionOrder(sectionOrder.value)
}

// Visibility: controlled by user toggle, NOT by whether there's data
function showSection(section: SectionType): boolean {
  if (!prefs.isSectionVisible(section)) return false
  if (section === 'calendar_event' && prefs.calendarLayout === 'sidebar') return false
  return true
}

// Check if a section has filtered-out items (raw data exists but filters hide everything)
function isFilteredEmpty(section: SectionType): boolean {
  switch (section) {
    case 'github_pr':
      return store.githubPRs.length > 0 && filteredGitHubPRs.value.length === 0
    case 'gitlab':
      return (store.gitlabMRs.length > 0 || store.gitlabPipelines.length > 0) &&
        filteredGitLabMRs.value.length === 0 && filteredPipelines.value.length === 0
    case 'jira_ticket':
      return store.jiraTickets.length > 0 && filteredJiraTickets.value.length === 0
    case 'calendar_event':
      return store.calendarEvents.length > 0 && filteredCalendarEvents.value.length === 0
    default:
      return false
  }
}


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
      <div class="flex items-center gap-2">
        <SectionVisibilityPopover />
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
      <!-- Main column: dynamically ordered sections -->
      <div class="flex-1 min-w-0">
        <VueDraggable
          v-model="sectionOrder"
          handle=".drag-handle"
          :animation="200"
          ghost-class="opacity-30"
          @end="onDragEnd"
          class="space-y-8"
        >
          <div v-for="section in sectionOrder" :key="section" v-show="showSection(section)">
            <!-- Important -->
            <template v-if="section === 'important'">
              <DashboardSection
                title="Important"
                :count="importantItems.length"
                draggable
              >
                <template #actions>
                  <ImportantRulesPopover />
                </template>

                <template v-if="importantItems.length">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <template v-for="item in importantItems.slice(0, 8)" :key="`${item.type}-${item.data.id || item.data.key}`">
                      <PRCard v-if="item.type === 'github_pr'" :pr="item.data" show-pin />
                      <MRCard v-else-if="item.type === 'gitlab_mr'" :mr="item.data" show-pin />
                      <PipelineCard v-else-if="item.type === 'gitlab_pipeline'" :pipeline="item.data" show-pin />
                      <TicketCard v-else-if="item.type === 'jira_ticket'" :ticket="item.data" show-pin />
                      <MeetingCard v-else-if="item.type === 'calendar_event'" :meeting="item.data" show-pin />
                    </template>
                  </div>
                </template>
                <template v-else>
                  <p class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                    No important items. Configure rules via the gear icon or pin items manually.
                  </p>
                </template>
              </DashboardSection>
            </template>

            <!-- GitHub PRs -->
            <template v-else-if="section === 'github_pr'">
              <DashboardSection
                title="GitHub PRs"
                :count="filteredGitHubPRs.length"
                link="/github"
                draggable
              >
                <template #actions>
                  <GitHubPRFilterPopover />
                </template>
                <template v-if="filteredGitHubPRs.length">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <PRCard
                      v-for="pr in filteredGitHubPRs.slice(0, 6)"
                      :key="pr.id"
                      :pr="pr"
                      show-pin
                    />
                  </div>
                </template>
                <p v-else-if="isFilteredEmpty('github_pr')" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  All items hidden by filters. Adjust filters to see PRs.
                </p>
                <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  No pull requests.
                </p>
              </DashboardSection>
            </template>

            <!-- GitLab (MRs + Pipelines) -->
            <template v-else-if="section === 'gitlab'">
              <DashboardSection
                title="GitLab"
                :count="filteredGitLabMRs.length + filteredPipelines.length"
                link="/gitlab"
                draggable
              >
                <template #actions>
                  <GitLabMRFilterPopover />
                  <GitLabPipelineFilterPopover />
                </template>

                <!-- MRs sub-group -->
                <template v-if="filteredGitLabMRs.length">
                  <h4 v-if="filteredPipelines.length || store.gitlabPipelines.length" class="text-xs font-semibold uppercase tracking-wide mb-2" style="color: var(--color-text-muted)">Merge Requests</h4>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <MRCard
                      v-for="mr in filteredGitLabMRs.slice(0, 6)"
                      :key="mr.id"
                      :mr="mr"
                      show-pin
                    />
                  </div>
                </template>

                <!-- Pipelines sub-group -->
                <template v-if="filteredPipelines.length">
                  <h4 v-if="filteredGitLabMRs.length || store.gitlabMRs.length" class="text-xs font-semibold uppercase tracking-wide mb-2" :class="{ 'mt-4': filteredGitLabMRs.length }" style="color: var(--color-text-muted)">Pipelines</h4>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <PipelineCard
                      v-for="pipeline in filteredPipelines.slice(0, 6)"
                      :key="pipeline.id"
                      :pipeline="pipeline"
                      show-pin
                    />
                  </div>
                </template>

                <!-- Empty states -->
                <p v-if="isFilteredEmpty('gitlab')" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  All items hidden by filters. Adjust filters to see GitLab items.
                </p>
                <p v-else-if="!filteredGitLabMRs.length && !filteredPipelines.length" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  No merge requests or pipelines.
                </p>
              </DashboardSection>
            </template>

            <!-- JIRA Tickets -->
            <template v-else-if="section === 'jira_ticket'">
              <DashboardSection
                title="JIRA Tickets"
                :count="filteredJiraTickets.length"
                link="/jira"
                draggable
              >
                <template #actions>
                  <JiraTicketFilterPopover />
                </template>
                <template v-if="filteredJiraTickets.length">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <TicketCard
                      v-for="ticket in filteredJiraTickets.slice(0, 6)"
                      :key="ticket.id"
                      :ticket="ticket"
                      show-pin
                    />
                  </div>
                </template>
                <p v-else-if="isFilteredEmpty('jira_ticket')" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  All items hidden by filters. Adjust filters to see tickets.
                </p>
                <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  No tickets.
                </p>
              </DashboardSection>
            </template>

            <!-- Calendar (inline mode) -->
            <template v-else-if="section === 'calendar_event'">
              <DashboardSection
                title="Calendar"
                :count="filteredCalendarEvents.length"
                link="/calendar"
                draggable
              >
                <template #actions>
                  <button
                    @click.stop="toggleCalendarLayout"
                    class="p-1 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
                    title="Move to sidebar"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-text-muted)">
                      <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v12a1 1 0 01-1 1H4a1 1 0 01-1-1V4zm1 0v12h8V4H4zm10 0v12h2V4h-2z" clip-rule="evenodd" />
                    </svg>
                  </button>
                  <CalendarEventFilterPopover />
                </template>
                <template v-if="filteredCalendarEvents.length">
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <MeetingCard
                      v-for="event in filteredCalendarEvents.slice(0, 8)"
                      :key="event.id"
                      :meeting="event"
                      show-pin
                    />
                  </div>
                </template>
                <p v-else-if="isFilteredEmpty('calendar_event')" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  All events hidden by filters. Adjust filters to see events.
                </p>
                <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
                  No events.
                </p>
              </DashboardSection>
            </template>
          </div>
        </VueDraggable>

        <!-- Empty state (only when truly no integrations configured) -->
        <div v-if="!store.items.length && !store.loading" class="text-center py-12">
          <p class="text-[var(--color-text-muted)]">No items to show.</p>
          <p class="text-sm text-[var(--color-text-muted)] mt-1">
            Configure your integrations in
            <RouterLink to="/settings" class="text-[var(--color-primary)] hover:underline">Settings</RouterLink>
          </p>
        </div>
      </div>

      <!-- Right column: Calendar (sidebar mode) -->
      <aside
        v-if="prefs.calendarLayout === 'sidebar' && prefs.isSectionVisible('calendar_event')"
        class="hidden lg:block w-80 shrink-0"
      >
        <DashboardSection
          title="Calendar"
          :count="filteredCalendarEvents.length"
          link="/calendar"
        >
          <template #actions>
            <button
              @click.stop="toggleCalendarLayout"
              class="p-1 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
              title="Move to main content"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor" style="color: var(--color-text-muted)">
                <path fill-rule="evenodd" d="M3 4a1 1 0 011-1h12a1 1 0 011 1v12a1 1 0 01-1 1H4a1 1 0 01-1-1V4zm1 0v12h12V4H4z" clip-rule="evenodd" />
              </svg>
            </button>
            <CalendarEventFilterPopover />
          </template>
          <template v-if="filteredCalendarEvents.length">
            <div class="space-y-2">
              <MeetingCard
                v-for="event in filteredCalendarEvents.slice(0, 8)"
                :key="event.id"
                :meeting="event"
                show-pin
              />
            </div>
          </template>
          <p v-else-if="isFilteredEmpty('calendar_event')" class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
            All events hidden by filters.
          </p>
          <p v-else class="text-sm py-4 text-center" style="color: var(--color-text-muted)">
            No events.
          </p>
        </DashboardSection>
      </aside>
    </div>

    <!-- Calendar fallback for small screens (sidebar mode only) -->
    <div
      v-if="prefs.calendarLayout === 'sidebar' && prefs.isSectionVisible('calendar_event') && filteredCalendarEvents.length"
      class="lg:hidden mt-8"
    >
      <DashboardSection
        title="Calendar"
        :count="filteredCalendarEvents.length"
        link="/calendar"
      >
        <template #actions>
          <CalendarEventFilterPopover />
        </template>
        <div class="space-y-2">
          <MeetingCard
            v-for="event in filteredCalendarEvents.slice(0, 5)"
            :key="event.id"
            :meeting="event"
            show-pin
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
