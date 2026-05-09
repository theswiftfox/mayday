// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
import { computed, type Ref } from 'vue'
import type {
  GitHubPRFilter,
  GitLabMRFilter,
  GitLabPipelineFilter,
  JiraTicketFilter,
  CalendarEventFilter,
} from '@/types/dashboard'

/** Apply GitHub PR filters to a list of PRs */
export function useFilteredGitHubPRs(prs: Ref<any[]>, filter: Ref<GitHubPRFilter>) {
  return computed(() => {
    return prs.value.filter((pr) => {
      const f = filter.value
      if (f.roles.length && !f.roles.includes(pr.role)) return false
      if (f.hideDrafts && pr.isDraft) return false
      if (f.actionRequiredOnly && !pr.actionRequired) return false
      return true
    })
  })
}

/** Apply GitLab MR filters to a list of MRs */
export function useFilteredGitLabMRs(mrs: Ref<any[]>, filter: Ref<GitLabMRFilter>) {
  return computed(() => {
    return mrs.value.filter((mr) => {
      const f = filter.value
      if (f.roles.length && !f.roles.includes(mr.role)) return false
      if (f.hideDrafts && mr.isDraft) return false
      return true
    })
  })
}

/** Apply GitLab Pipeline filters */
export function useFilteredGitLabPipelines(pipelines: Ref<any[]>, filter: Ref<GitLabPipelineFilter>) {
  return computed(() => {
    return pipelines.value.filter((p) => {
      const f = filter.value
      if (f.statuses.length) {
        return f.statuses.includes(p.status)
      }
      // No filter active: show all pipelines
      return true
    })
  })
}

/** Apply JIRA Ticket filters */
export function useFilteredJiraTickets(tickets: Ref<any[]>, filter: Ref<JiraTicketFilter>) {
  return computed(() => {
    return tickets.value.filter((t) => {
      const f = filter.value
      if (f.statusCategories.length && !f.statusCategories.includes(t.statusCategory)) return false
      if (f.priorities.length && !f.priorities.includes(t.priority?.toLowerCase())) return false
      if (f.issueTypes.length && !f.issueTypes.includes(t.issueType?.toLowerCase())) return false
      return true
    })
  })
}

/** Apply Calendar Event filters */
export function useFilteredCalendarEvents(events: Ref<any[]>, filter: Ref<CalendarEventFilter>) {
  return computed(() => {
    return events.value.filter((e) => {
      const f = filter.value
      if (f.hideAllDay && e.isAllDay) return false
      if (f.onlineOnly && !e.isOnline) return false
      return true
    })
  })
}

/** Check if any filter is active for a given section type */
export function hasActiveFilter(sectionType: string, filters: any): boolean {
  switch (sectionType) {
    case 'github_pr': {
      const f = filters.githubPr as GitHubPRFilter
      return f.roles.length > 0 || f.hideDrafts || f.actionRequiredOnly
    }
    case 'gitlab_mr': {
      const f = filters.gitlabMr as GitLabMRFilter
      return f.roles.length > 0 || f.hideDrafts
    }
    case 'gitlab_pipeline': {
      const f = filters.gitlabPipeline as GitLabPipelineFilter
      return f.statuses.length > 0
    }
    case 'jira_ticket': {
      const f = filters.jiraTicket as JiraTicketFilter
      return f.statusCategories.length > 0 || f.priorities.length > 0 || f.issueTypes.length > 0
    }
    case 'calendar_event': {
      const f = filters.calendarEvent as CalendarEventFilter
      return f.hideAllDay || f.onlineOnly
    }
    default:
      return false
  }
}
