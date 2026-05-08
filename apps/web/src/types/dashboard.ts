// Dashboard customization types — mirrors Rust DashboardConfig in config.rs

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
  section_order: SectionType[]
  visible_sections: SectionType[]
  calendar_layout: 'sidebar' | 'inline'
  important_rules: ImportantRules
  pinned_items: PinnedItem[]
  filters: DashboardFilters
}

export interface ImportantRules {
  github_action_required: boolean
  github_new_comments: boolean
  github_new_commits: boolean
  github_changes_requested: boolean
  gitlab_mr_new_comments: boolean
  gitlab_mr_new_commits: boolean
  gitlab_pipeline_failed: boolean
  jira_high_priority: boolean
  calendar_starting_soon: boolean
}

export interface PinnedItem {
  item_type: string
  item_id: string
}

export interface DashboardFilters {
  github_pr: GitHubPRFilter
  gitlab_mr: GitLabMRFilter
  gitlab_pipeline: GitLabPipelineFilter
  jira_ticket: JiraTicketFilter
  calendar_event: CalendarEventFilter
}

export interface GitHubPRFilter {
  roles: string[]
  hide_drafts: boolean
  action_required_only: boolean
}

export interface GitLabMRFilter {
  roles: string[]
  hide_drafts: boolean
}

export interface GitLabPipelineFilter {
  statuses: string[]
}

export interface JiraTicketFilter {
  status_categories: string[]
  priorities: string[]
  issue_types: string[]
}

export interface CalendarEventFilter {
  hide_all_day: boolean
  online_only: boolean
}

// Item ID helpers — used for pinning

export function getItemId(itemType: string, item: any): string {
  switch (itemType) {
    case 'github_pr':
      return `${item.repo}#${item.number}`
    case 'gitlab_mr':
      return `${item.project_id}!${item.iid}`
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
    section_order: [...ALL_SECTIONS],
    visible_sections: [...ALL_SECTIONS],
    calendar_layout: 'sidebar',
    important_rules: {
      github_action_required: false,
      github_new_comments: false,
      github_new_commits: false,
      github_changes_requested: false,
      gitlab_mr_new_comments: false,
      gitlab_mr_new_commits: false,
      gitlab_pipeline_failed: false,
      jira_high_priority: false,
      calendar_starting_soon: false,
    },
    pinned_items: [],
    filters: {
      github_pr: { roles: [], hide_drafts: false, action_required_only: false },
      gitlab_mr: { roles: [], hide_drafts: false },
      gitlab_pipeline: { statuses: [] },
      jira_ticket: { status_categories: [], priorities: [], issue_types: [] },
      calendar_event: { hide_all_day: false, online_only: false },
    },
  }
}
