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
      if (f.hide_drafts && pr.is_draft) return false
      if (f.action_required_only && !pr.action_required) return false
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
      if (f.hide_drafts && mr.is_draft) return false
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
      if (f.status_categories.length && !f.status_categories.includes(t.status_category)) return false
      if (f.priorities.length && !f.priorities.includes(t.priority?.toLowerCase())) return false
      if (f.issue_types.length && !f.issue_types.includes(t.issue_type?.toLowerCase())) return false
      return true
    })
  })
}

/** Apply Calendar Event filters */
export function useFilteredCalendarEvents(events: Ref<any[]>, filter: Ref<CalendarEventFilter>) {
  return computed(() => {
    return events.value.filter((e) => {
      const f = filter.value
      if (f.hide_all_day && e.is_all_day) return false
      if (f.online_only && !e.is_online) return false
      return true
    })
  })
}

/** Check if any filter is active for a given section type */
export function hasActiveFilter(sectionType: string, filters: any): boolean {
  switch (sectionType) {
    case 'github_pr': {
      const f = filters.github_pr as GitHubPRFilter
      return f.roles.length > 0 || f.hide_drafts || f.action_required_only
    }
    case 'gitlab_mr': {
      const f = filters.gitlab_mr as GitLabMRFilter
      return f.roles.length > 0 || f.hide_drafts
    }
    case 'gitlab_pipeline': {
      const f = filters.gitlab_pipeline as GitLabPipelineFilter
      return f.statuses.length > 0
    }
    case 'jira_ticket': {
      const f = filters.jira_ticket as JiraTicketFilter
      return f.status_categories.length > 0 || f.priorities.length > 0 || f.issue_types.length > 0
    }
    case 'calendar_event': {
      const f = filters.calendar_event as CalendarEventFilter
      return f.hide_all_day || f.online_only
    }
    default:
      return false
  }
}
