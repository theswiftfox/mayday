// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
//
// Demo fixture data for screenshots and demos.
// Themed around World of Warcraft for fun easter eggs.

import type {
  DashboardItem,
  GitHubPR,
  JiraTicket,
  GitLabMR,
  GitLabPipeline,
  CalendarEvent,
} from '@/types/api'

// --- Helpers ---

/** Returns an ISO date string relative to now */
function hoursAgo(h: number): string {
  return new Date(Date.now() - h * 3600_000).toISOString()
}

function minutesAgo(m: number): string {
  return new Date(Date.now() - m * 60_000).toISOString()
}

function todayAt(hour: number, minute = 0): string {
  const d = new Date()
  d.setHours(hour, minute, 0, 0)
  return d.toISOString()
}

// --- GitHub PRs ---

const githubPRs: GitHubPR[] = [
  {
    id: 1001,
    number: 1337,
    title: 'fix(frost-mage): prevent Ice Lance from proccing itself infinitely',
    url: 'https://github.com/azeroth-dev/wow-core/pull/1337',
    repo: 'azeroth-dev/wow-core',
    author: 'Jaina',
    state: 'open',
    isDraft: false,
    createdAt: hoursAgo(6),
    updatedAt: hoursAgo(1),
    role: 'author',
    hasNewComments: true,
    hasNewCommits: false,
    actionRequired: false,
    commentCount: 4,
    lastCommitAt: hoursAgo(3),
    labels: ['bug', 'frost-spec'],
    reviewDecision: 'review_required',
    ciStatus: 'success',
  },
  {
    id: 1002,
    number: 42,
    title: 'feat(druid): add fourth spec "Boomchicken"',
    url: 'https://github.com/azeroth-dev/wow-core/pull/42',
    repo: 'azeroth-dev/wow-core',
    author: 'Malfurion',
    state: 'open',
    isDraft: false,
    createdAt: hoursAgo(48),
    updatedAt: hoursAgo(2),
    role: 'reviewer',
    hasNewComments: false,
    hasNewCommits: true,
    actionRequired: true,
    commentCount: 12,
    lastCommitAt: hoursAgo(2),
    labels: ['feature', 'balance'],
    reviewDecision: 'changes_requested',
    ciStatus: 'failure',
  },
  {
    id: 1003,
    number: 404,
    title: 'chore(loot-tables): rebalance Ashes of Al\'ar drop rate',
    url: 'https://github.com/azeroth-dev/loot-system/pull/404',
    repo: 'azeroth-dev/loot-system',
    author: 'Kael\'thas',
    state: 'open',
    isDraft: true,
    createdAt: hoursAgo(24),
    updatedAt: hoursAgo(8),
    role: 'author',
    hasNewComments: false,
    hasNewCommits: false,
    actionRequired: false,
    commentCount: 2,
    lastCommitAt: hoursAgo(12),
    labels: ['loot', 'classic'],
    reviewDecision: null,
    ciStatus: 'pending',
  },
  {
    id: 1004,
    number: 256,
    title: 'fix(warrior): Charge should not work while rooted',
    url: 'https://github.com/azeroth-dev/wow-core/pull/256',
    repo: 'azeroth-dev/wow-core',
    author: 'Saurfang',
    state: 'open',
    isDraft: false,
    createdAt: hoursAgo(72),
    updatedAt: hoursAgo(4),
    role: 'reviewer',
    hasNewComments: true,
    hasNewCommits: false,
    actionRequired: false,
    commentCount: 7,
    lastCommitAt: hoursAgo(24),
    labels: ['pvp', 'arms-spec'],
    reviewDecision: 'approved',
    ciStatus: 'success',
  },
]

// --- JIRA Tickets ---

const jiraTickets: JiraTicket[] = [
  {
    id: '10001',
    key: 'RAID-451',
    title: 'Ragnaros despawns if tank stands in fire too long',
    url: 'https://jira.azeroth.dev/browse/RAID-451',
    status: 'In Progress',
    statusCategory: 'in_progress',
    priority: 'Critical',
    assignee: 'Thrall',
    issueType: 'Bug',
    updatedAt: hoursAgo(1),
    createdAt: hoursAgo(72),
    labels: ['molten-core', 'boss-encounter'],
    sprintName: 'Sprint 40 - Firelands Cleanup',
  },
  {
    id: '10002',
    key: 'RAID-452',
    title: 'Add enrage timer to Patchwerk encounter',
    url: 'https://jira.azeroth.dev/browse/RAID-452',
    status: 'To Do',
    statusCategory: 'todo',
    priority: 'High',
    assignee: 'Kel\'Thuzad',
    issueType: 'Task',
    updatedAt: hoursAgo(5),
    createdAt: hoursAgo(120),
    labels: ['naxxramas'],
    sprintName: 'Sprint 40 - Firelands Cleanup',
  },
  {
    id: '10003',
    key: 'PVP-89',
    title: 'Warsong Gulch flag carrier speed cap implementation',
    url: 'https://jira.azeroth.dev/browse/PVP-89',
    status: 'In Review',
    statusCategory: 'in_progress',
    priority: 'Medium',
    assignee: null,
    issueType: 'Story',
    updatedAt: hoursAgo(3),
    createdAt: hoursAgo(48),
    labels: ['battlegrounds', 'pvp'],
    sprintName: 'Sprint 40 - Firelands Cleanup',
  },
  {
    id: '10004',
    key: 'RAID-460',
    title: 'Onyxia Deep Breath timing is inconsistent across resets',
    url: 'https://jira.azeroth.dev/browse/RAID-460',
    status: 'Done',
    statusCategory: 'done',
    priority: 'Low',
    assignee: 'Varian',
    issueType: 'Bug',
    updatedAt: hoursAgo(2),
    createdAt: hoursAgo(200),
    labels: ['onyxias-lair'],
    sprintName: null,
  },
]

// --- GitLab MRs ---

const gitlabMRs: GitLabMR[] = [
  {
    id: 2001,
    iid: 77,
    title: 'Update dungeon finder matchmaking algorithm for heroic queues',
    url: 'https://gitlab.azeroth.dev/dungeon-team/instance-server/-/merge_requests/77',
    projectPath: 'dungeon-team/instance-server',
    projectId: 501,
    author: 'Sylvanas',
    state: 'opened',
    isDraft: false,
    createdAt: hoursAgo(10),
    updatedAt: hoursAgo(1),
    role: 'author',
    hasNewComments: true,
    hasNewCommits: false,
    commentCount: 3,
    labels: ['matchmaking', 'performance'],
    mergeStatus: 'can_be_merged',
  },
  {
    id: 2002,
    iid: 78,
    title: 'Implement cross-realm auction house sync',
    url: 'https://gitlab.azeroth.dev/economy-team/auction-service/-/merge_requests/78',
    projectPath: 'economy-team/auction-service',
    projectId: 502,
    author: 'Gazlowe',
    state: 'opened',
    isDraft: true,
    createdAt: hoursAgo(36),
    updatedAt: hoursAgo(6),
    role: 'reviewer',
    hasNewComments: false,
    hasNewCommits: true,
    commentCount: 8,
    labels: ['economy', 'cross-realm'],
    mergeStatus: 'unchecked',
  },
]

// --- GitLab Pipelines ---

const gitlabPipelines: GitLabPipeline[] = [
  {
    id: 9001,
    status: 'failed',
    refName: 'fix/hearthstone-disconnect',
    url: 'https://gitlab.azeroth.dev/dungeon-team/instance-server/-/pipelines/9001',
    projectPath: 'dungeon-team/instance-server',
    projectId: 501,
    createdAt: minutesAgo(45),
    updatedAt: minutesAgo(30),
    duration: 342,
  },
  {
    id: 9002,
    status: 'running',
    refName: 'feat/transmog-preview',
    url: 'https://gitlab.azeroth.dev/ui-team/character-screen/-/pipelines/9002',
    projectPath: 'ui-team/character-screen',
    projectId: 503,
    createdAt: minutesAgo(12),
    updatedAt: minutesAgo(1),
    duration: null,
  },
  {
    id: 9003,
    status: 'success',
    refName: 'main',
    url: 'https://gitlab.azeroth.dev/economy-team/auction-service/-/pipelines/9003',
    projectPath: 'economy-team/auction-service',
    projectId: 502,
    createdAt: hoursAgo(2),
    updatedAt: hoursAgo(1),
    duration: 187,
  },
  {
    id: 9004,
    status: 'canceled',
    refName: 'chore/update-talent-trees',
    url: 'https://gitlab.azeroth.dev/dungeon-team/instance-server/-/pipelines/9004',
    projectPath: 'dungeon-team/instance-server',
    projectId: 501,
    createdAt: hoursAgo(3),
    updatedAt: hoursAgo(2),
    duration: 58,
  },
]

// --- Calendar Events ---

const calendarEvents: CalendarEvent[] = [
  {
    id: 'cal-001',
    subject: 'Raid Night: Molten Core',
    startTime: todayAt(20, 0),
    endTime: todayAt(23, 0),
    isAllDay: false,
    location: 'Blackrock Mountain',
    organizer: 'Ragnaros',
    isOnline: true,
    onlineUrl: 'https://discord.gg/azeroth-raiders',
    responseStatus: 'accepted',
  },
  {
    id: 'cal-002',
    subject: 'Guild Officers Meeting',
    startTime: todayAt(18, 30),
    endTime: todayAt(19, 0),
    isAllDay: false,
    location: null,
    organizer: 'Thrall',
    isOnline: true,
    onlineUrl: 'https://discord.gg/azeroth-officers',
    responseStatus: 'accepted',
  },
  {
    id: 'cal-003',
    subject: 'Darkmoon Faire',
    startTime: todayAt(0, 0),
    endTime: todayAt(23, 59),
    isAllDay: true,
    location: 'Darkmoon Island',
    organizer: 'Silas Darkmoon',
    isOnline: false,
    onlineUrl: null,
    responseStatus: 'tentative',
  },
  {
    id: 'cal-004',
    subject: 'Arena 2v2 Practice',
    startTime: todayAt(16, 0),
    endTime: todayAt(17, 30),
    isAllDay: false,
    location: 'Nagrand Arena',
    organizer: 'Valeera',
    isOnline: false,
    onlineUrl: null,
    responseStatus: 'accepted',
  },
]

// --- Assemble Dashboard ---

export const demoDashboardItems: DashboardItem[] = [
  ...githubPRs.map((data) => ({ type: 'github_pr' as const, data })),
  ...jiraTickets.map((data) => ({ type: 'jira_ticket' as const, data })),
  ...gitlabMRs.map((data) => ({ type: 'gitlab_mr' as const, data })),
  ...gitlabPipelines.map((data) => ({ type: 'gitlab_pipeline' as const, data })),
  ...calendarEvents.map((data) => ({ type: 'calendar_event' as const, data })),
]

export const demoLastUpdated = new Date().toISOString()
