// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//
// API response types — mirrors the Rust Serialize structs from myday-core.
// Field names use camelCase to match the Rust `#[serde(rename_all = "camelCase")]`.

export type ItemType = 'github_pr' | 'jira_ticket' | 'gitlab_mr' | 'gitlab_pipeline' | 'calendar_event'

// --- GitHub ---

export interface GitHubPR {
  id: number
  number: number
  title: string
  url: string
  repo: string
  author: string
  state: string
  isDraft: boolean
  createdAt: string
  updatedAt: string
  role: 'author' | 'reviewer' | 'other'
  hasNewComments: boolean
  hasNewCommits: boolean
  actionRequired: boolean
  commentCount: number
  lastCommitAt: string | null
  labels: string[]
  reviewDecision: string | null
  ciStatus: string | null
}

export interface GitHubReviewThread {
  id: string
  path: string
  isResolved: boolean
  comments: GitHubReviewComment[]
}

export interface GitHubReviewComment {
  id: number
  author: string
  body: string
  createdAt: string
}

export interface GitHubIssueComment {
  id: number
  author: string
  body: string
  createdAt: string
}

export interface GitHubReview {
  id: number
  author: string
  state: string
  body: string | null
  submittedAt: string
}

export interface GitHubPRDetail extends GitHubPR {
  body: string | null
  reviewThreads: GitHubReviewThread[]
  issueComments: GitHubIssueComment[]
  reviews: GitHubReview[]
}

// --- JIRA ---

export interface JiraTicket {
  id: string
  key: string
  title: string
  url: string
  status: string
  statusCategory: 'todo' | 'in_progress' | 'done'
  priority: string
  assignee: string | null
  issueType: string
  updatedAt: string
  createdAt: string
  labels: string[]
  sprintName: string | null
}

// --- GitLab ---

export interface GitLabMR {
  id: number
  iid: number
  title: string
  url: string
  projectPath: string
  projectId: number
  author: string
  state: string
  isDraft: boolean
  createdAt: string
  updatedAt: string
  role: 'author' | 'reviewer' | 'other'
  hasNewComments: boolean
  hasNewCommits: boolean
  commentCount: number
  labels: string[]
  mergeStatus: string | null
}

export interface GitLabPipeline {
  id: number
  status: string
  refName: string
  url: string
  projectPath: string
  projectId: number
  createdAt: string
  updatedAt: string
  duration: number | null
}

// --- Calendar ---

export interface CalendarEvent {
  id: string
  subject: string
  startTime: string
  endTime: string
  isAllDay: boolean
  location: string | null
  organizer: string | null
  isOnline: boolean
  onlineUrl: string | null
  responseStatus: string | null
}

// --- Dashboard response ---

export interface DashboardItem {
  type: ItemType
  data: GitHubPR | JiraTicket | GitLabMR | GitLabPipeline | CalendarEvent
}

export interface IntegrationError {
  source: string
  message: string
}

export interface DashboardResponse {
  items: DashboardItem[]
  errors: IntegrationError[]
  lastUpdated: string
}
