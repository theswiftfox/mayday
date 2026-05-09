// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
// Dashboard customization types — mirrors Rust DashboardConfig in config.rs
// Field names use camelCase to match `#[serde(rename_all = "camelCase")]`.
// String VALUES (section type identifiers, discriminators) stay snake_case.

export type SectionType =
  | 'important'
  | 'github_pr'
  | 'gitlab'
  | 'jira_ticket'
  | 'calendar_event'

export const ALL_SECTIONS: SectionType[] = [
  'important',
  'github_pr',
  'gitlab',
  'jira_ticket',
  'calendar_event',
]

export const SECTION_LABELS: Record<SectionType, string> = {
  important: 'Important',
  github_pr: 'GitHub PRs',
  gitlab: 'GitLab',
  jira_ticket: 'JIRA Tickets',
  calendar_event: 'Calendar',
}

export interface DashboardConfig {
  sectionOrder: SectionType[]
  visibleSections: SectionType[]
  calendarLayout: 'sidebar' | 'inline'
  importantRules: ImportantRules
  pinnedItems: PinnedItem[]
  filters: DashboardFilters
}

export interface ImportantRules {
  githubActionRequired: boolean
  githubNewComments: boolean
  githubNewCommits: boolean
  githubChangesRequested: boolean
  gitlabMrNewComments: boolean
  gitlabMrNewCommits: boolean
  gitlabPipelineFailed: boolean
  jiraHighPriority: boolean
  calendarStartingSoon: boolean
}

export interface PinnedItem {
  itemType: string
  itemId: string
}

export interface DashboardFilters {
  githubPr: GitHubPRFilter
  gitlabMr: GitLabMRFilter
  gitlabPipeline: GitLabPipelineFilter
  jiraTicket: JiraTicketFilter
  calendarEvent: CalendarEventFilter
}

export interface GitHubPRFilter {
  roles: string[]
  hideDrafts: boolean
  actionRequiredOnly: boolean
}

export interface GitLabMRFilter {
  roles: string[]
  hideDrafts: boolean
}

export interface GitLabPipelineFilter {
  statuses: string[]
}

export interface JiraTicketFilter {
  statusCategories: string[]
  priorities: string[]
  issueTypes: string[]
}

export interface CalendarEventFilter {
  hideAllDay: boolean
  onlineOnly: boolean
}

// Item ID helpers — used for pinning

export function getItemId(itemType: string, item: any): string {
  switch (itemType) {
    case 'github_pr':
      return `${item.repo}#${item.number}`
    case 'gitlab_mr':
      return `${item.projectId}!${item.iid}`
    case 'gitlab_pipeline':
      return String(item.id)
    case 'jira_ticket':
      return item.key
    case 'calendar_event':
      return item.id
    default:
      return ''
  }
}

export function defaultDashboardConfig(): DashboardConfig {
  return {
    sectionOrder: [...ALL_SECTIONS],
    visibleSections: [...ALL_SECTIONS],
    calendarLayout: 'sidebar',
    importantRules: {
      githubActionRequired: false,
      githubNewComments: false,
      githubNewCommits: false,
      githubChangesRequested: false,
      gitlabMrNewComments: false,
      gitlabMrNewCommits: false,
      gitlabPipelineFailed: false,
      jiraHighPriority: false,
      calendarStartingSoon: false,
    },
    pinnedItems: [],
    filters: {
      githubPr: { roles: [], hideDrafts: false, actionRequiredOnly: false },
      gitlabMr: { roles: [], hideDrafts: false },
      gitlabPipeline: { statuses: [] },
      jiraTicket: { statusCategories: [], priorities: [], issueTypes: [] },
      calendarEvent: { hideAllDay: false, onlineOnly: false },
    },
  }
}
