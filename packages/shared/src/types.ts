// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
// Integration source types
export type IntegrationSource = 'github' | 'jira' | 'gitlab' | 'outlook';

export interface IntegrationConfig {
  source: IntegrationSource;
  enabled: boolean;
  pollIntervalMs: number;
  lastSynced?: string; // ISO date
}

// GitHub types
export interface GitHubConfig extends IntegrationConfig {
  source: 'github';
  token: string;
  username: string;
  repos?: string[]; // optional filter: org/repo format
}

export interface GitHubPR {
  id: number;
  number: number;
  title: string;
  url: string;
  repo: string;
  author: string;
  state: 'open' | 'closed' | 'merged';
  isDraft: boolean;
  createdAt: string;
  updatedAt: string;
  // Attention tracking
  role: 'author' | 'reviewer';
  hasNewComments: boolean;
  hasNewCommits: boolean;
  commentCount: number;
  lastCommitAt?: string;
  labels: string[];
  reviewDecision?: 'approved' | 'changes_requested' | 'review_required';
}

// JIRA types
export interface JiraConfig extends IntegrationConfig {
  source: 'jira';
  host: string; // e.g., yourcompany.atlassian.net
  email: string;
  apiToken: string;
  projectKeys?: string[]; // optional filter
}

export interface JiraTicket {
  id: string;
  key: string; // e.g., PROJ-123
  title: string;
  url: string;
  status: string;
  statusCategory: 'todo' | 'in_progress' | 'done';
  priority: string;
  assignee?: string;
  type: string; // Bug, Story, Task, etc.
  updatedAt: string;
  createdAt: string;
  labels: string[];
  sprintName?: string;
}

// GitLab types
export interface GitLabConfig extends IntegrationConfig {
  source: 'gitlab';
  host: string; // e.g., gitlab.com or self-hosted
  token: string;
  username: string;
  projectIds?: number[]; // optional filter
}

export interface GitLabMR {
  id: number;
  iid: number;
  title: string;
  url: string;
  project: string;
  author: string;
  state: 'opened' | 'closed' | 'merged';
  isDraft: boolean;
  createdAt: string;
  updatedAt: string;
  role: 'author' | 'reviewer';
  hasNewComments: boolean;
  hasNewCommits: boolean;
  commentCount: number;
  labels: string[];
  mergeStatus?: string;
}

export interface GitLabPipeline {
  id: number;
  status: 'running' | 'pending' | 'success' | 'failed' | 'canceled' | 'skipped';
  ref: string;
  url: string;
  project: string;
  createdAt: string;
  updatedAt: string;
  duration?: number; // seconds
}

// Outlook types (Microsoft Graph, and EWS sources)
export interface OutlookConfig extends IntegrationConfig {
  source: 'outlook';
  clientId?: string; // Azure AD app client ID
  tenantId?: string; // Azure AD tenant, defaults to 'common'
  refreshToken?: string; // stored after OAuth flow
}

export interface OutlookMeeting {
  id: string;
  subject: string;
  startTime: string; // ISO
  endTime: string; // ISO
  isAllDay: boolean;
  location?: string;
  organizer: string;
  attendees: string[];
  isOnline: boolean;
  onlineUrl?: string;
  responseStatus: 'accepted' | 'tentative' | 'declined' | 'none';
  body?: string; // HTML body for detail view
}

// Unified dashboard types
export type DashboardItem =
  | { type: 'github_pr'; data: GitHubPR }
  | { type: 'jira_ticket'; data: JiraTicket }
  | { type: 'gitlab_mr'; data: GitLabMR }
  | { type: 'gitlab_pipeline'; data: GitLabPipeline }
  | { type: 'outlook_meeting'; data: OutlookMeeting };

export interface DashboardState {
  items: DashboardItem[];
  lastUpdated: string;
  errors: IntegrationError[];
}

export interface IntegrationError {
  source: IntegrationSource;
  message: string;
  timestamp: string;
}

// Settings
export type AppConfig = {
  integrations: {
    github?: GitHubConfig;
    jira?: JiraConfig;
    gitlab?: GitLabConfig;
    outlook?: OutlookConfig;
  };
  general: {
    refreshOnFocus: boolean;
    theme: 'light' | 'dark' | 'system';
  };
};

// API response types
export interface ApiResponse<T> {
  data: T;
  timestamp: string;
}

export interface ApiError {
  error: string;
  code: string;
  source?: IntegrationSource;
}
