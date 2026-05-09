// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Elena Gantner
const isTauri = !!(window as any).__TAURI_INTERNALS__

// Lazy-load invoke only in Tauri mode
let _invoke: ((cmd: string, args?: any) => Promise<any>) | null = null

async function getInvoke() {
  if (!_invoke) {
    const { invoke } = await import('@tauri-apps/api/core')
    _invoke = invoke
  }
  return _invoke
}

class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string = '/api') {
    this.baseUrl = baseUrl
  }

  private async request<T>(path: string, options?: RequestInit): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
      ...options,
    })

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Unknown error' }))
      throw new Error(error.error || `HTTP ${response.status}`)
    }

    return response.json()
  }

  private async cmd<T>(command: string, args?: Record<string, any>): Promise<T> {
    const invoke = await getInvoke()
    return invoke(command, args)
  }

  // Dashboard
  async getDashboard() {
    type R = { items: Array<{ type: string; data: any }>; errors: Array<{ source: string; message: string }>; last_updated: string }
    if (isTauri) return this.cmd<R>('get_dashboard')
    return this.request<R>('/dashboard')
  }

  // GitHub
  async getGitHubPRs() {
    if (isTauri) return this.cmd<{ data: any[] }>('get_github_prs')
    return this.request<{ data: any[] }>('/github/prs')
  }

  async getGitHubPRDetail(owner: string, repo: string, number: number) {
    if (isTauri) return this.cmd<{ data: any }>('get_github_pr_detail', { owner, repo, number })
    return this.request<{ data: any }>(`/github/prs/${owner}/${repo}/${number}`)
  }

  async detectGhCli() {
    type R = { success: boolean; username?: string; message?: string; source?: string }
    if (isTauri) return this.cmd<R>('detect_gh_cli')
    return this.request<R>('/github/auth/detect-gh-cli', { method: 'POST' })
  }

  async useManualGitHubToken(token: string) {
    type R = { success: boolean; username?: string; message?: string; source?: string }
    if (isTauri) return this.cmd<R>('use_manual_github_token', { request: { token } })
    return this.request<R>('/github/auth/token', { method: 'POST', body: JSON.stringify({ token }) })
  }

  async startGitHubDeviceCode(clientId: string) {
    type R = { device_code: string; user_code: string; verification_uri: string; expires_in: number; interval: number }
    if (isTauri) return this.cmd<R>('start_github_device_code', { request: { client_id: clientId } })
    return this.request<R>('/github/auth/device-code/start', {
      method: 'POST',
      body: JSON.stringify({ client_id: clientId }),
    })
  }

  async pollGitHubDeviceCode(clientId: string, deviceCode: string) {
    type R = { status: string; username?: string }
    if (isTauri) return this.cmd<R>('poll_github_device_code', { request: { client_id: clientId, device_code: deviceCode } })
    return this.request<R>('/github/auth/device-code/poll', {
      method: 'POST',
      body: JSON.stringify({ client_id: clientId, device_code: deviceCode }),
    })
  }

  // JIRA
  async getJiraTickets() {
    if (isTauri) return this.cmd<{ data: any[] }>('get_jira_tickets')
    return this.request<{ data: any[] }>('/jira/tickets')
  }

  async getJiraTicketDetail(key: string) {
    if (isTauri) return this.cmd<{ data: any }>('get_jira_ticket_detail', { key })
    return this.request<{ data: any }>(`/jira/tickets/${key}`)
  }

  // GitLab
  async getGitLabMRs() {
    if (isTauri) return this.cmd<{ data: any[] }>('get_gitlab_mrs')
    return this.request<{ data: any[] }>('/gitlab/mrs')
  }

  async getGitLabMRDetail(projectId: number, iid: number) {
    if (isTauri) return this.cmd<{ data: any }>('get_gitlab_mr_detail', { projectId, iid })
    return this.request<{ data: any }>(`/gitlab/mrs/${projectId}/${iid}`)
  }

  async getGitLabPipelines() {
    if (isTauri) return this.cmd<{ data: any[] }>('get_gitlab_pipelines')
    return this.request<{ data: any[] }>('/gitlab/pipelines')
  }

  async resolveGitLabProject(host: string, path: string) {
    if (isTauri) return this.cmd<{ id: number; path: string }>('resolve_gitlab_project', { request: { host, path } })
    return this.request<{ id: number; path: string }>('/gitlab/resolve-project', {
      method: 'POST',
      body: JSON.stringify({ host, path }),
    })
  }

  // Calendar
  async getCalendarEvents() {
    if (isTauri) return this.cmd<{ data: any[] }>('get_calendar_events')
    return this.request<{ data: any[] }>('/calendar/events')
  }

  async startCalendarAuth(source?: string, flow?: string) {
    type R = { auth_url: string; source: string; flow: string }
    if (isTauri) return this.cmd<R>('start_calendar_auth', { request: { source: source || 'ews', flow: flow || 'manual' } })
    return this.request<R>('/calendar/auth/start', {
      method: 'POST',
      body: JSON.stringify({ source: source || 'ews', flow: flow || 'redirect' }),
    })
  }

  async getCalendarAuthStatus() {
    if (isTauri) return this.cmd<{ connected: boolean }>('get_calendar_auth_status')
    return this.request<{ connected: boolean }>('/calendar/auth/status')
  }

  async startCalendarDeviceCode(source?: string) {
    type R = { user_code: string; verification_uri: string; expires_in: number; interval: number }
    if (isTauri) return this.cmd<R>('start_calendar_device_code', { request: { source: source || 'ews' } })
    return this.request<R>('/calendar/auth/device-code/start', {
      method: 'POST',
      body: JSON.stringify({ source: source || 'ews' }),
    })
  }

  async pollCalendarDeviceCode() {
    type R = { status: string; error?: string }
    if (isTauri) return this.cmd<R>('poll_calendar_device_code')
    return this.request<R>('/calendar/auth/device-code/poll', { method: 'POST' })
  }

  async exchangeCalendarCode(code: string, redirectUri?: string) {
    type R = { status: string }
    if (isTauri) return this.cmd<R>('exchange_calendar_code', { request: { code, redirect_uri: redirectUri } })
    return this.request<R>('/calendar/auth/exchange-code', {
      method: 'POST',
      body: JSON.stringify({ code, redirect_uri: redirectUri }),
    })
  }

  // Config
  async getConfig() {
    if (isTauri) return this.cmd<any>('get_config')
    return this.request<any>('/config')
  }

  async updateConfig(config: any) {
    if (isTauri) return this.cmd<any>('update_config', { request: config })
    return this.request<any>('/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    })
  }

  async getDashboardConfig() {
    if (isTauri) return this.cmd<any>('get_dashboard_config')
    return this.request<any>('/config/dashboard')
  }

  async updateDashboardConfig(dashboard: any) {
    if (isTauri) return this.cmd<any>('update_dashboard_config', { dashboard })
    return this.request<any>('/config/dashboard', {
      method: 'PUT',
      body: JSON.stringify(dashboard),
    })
  }
}

export const api = new ApiClient()
