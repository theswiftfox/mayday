import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('./pages/DashboardPage.vue'),
  },
  {
    path: '/github',
    name: 'github',
    component: () => import('./pages/GitHubPage.vue'),
  },
  {
    path: '/github/:owner/:repo/:number',
    name: 'github-pr-detail',
    component: () => import('./pages/GitHubPRDetail.vue'),
  },
  {
    path: '/jira',
    name: 'jira',
    component: () => import('./pages/JiraPage.vue'),
  },
  {
    path: '/jira/:key',
    name: 'jira-ticket-detail',
    component: () => import('./pages/JiraTicketDetail.vue'),
  },
  {
    path: '/gitlab',
    name: 'gitlab',
    component: () => import('./pages/GitLabPage.vue'),
  },
  {
    path: '/gitlab/:projectId/:iid',
    name: 'gitlab-mr-detail',
    component: () => import('./pages/GitLabMRDetail.vue'),
  },
  {
    path: '/calendar',
    name: 'calendar',
    component: () => import('./pages/CalendarPage.vue'),
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('./pages/SettingsPage.vue'),
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
