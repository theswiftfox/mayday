// In Tauri, the frontend is served from a custom protocol so we need the full server URL.
// In dev/web mode, Vite proxies /api to the server.
const isTauri = !!(window as any).__TAURI_INTERNALS__
const API_BASE = isTauri ? 'http://localhost:3001/api' : '/api'

class ApiClient {
  private baseUrl: string

  constructor(baseUrl: string = API_BASE) {
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

  // Dashboard
  async getDashboard() {
    return this.request<{
      items: Array<{ type: string; data: any }>
      errors: Array<{ source: string; message: string }>
      last_updated: string
    }>('/dashboard')
  }

  // GitHub
  async getGitHubPRs() {
    return this.request<{ data: any[] }>('/github/prs')
  }

  async getGitHubPRDetail(owner: string, repo: string, number: number) {
    return this.request<{ data: any }>(`/github/prs/${owner}/${repo}/${number}`)
  }

  async detectGhCli() {
    return this.request<{ success: boolean; username?: string; message?: string; source?: string }>(
      '/github/auth/detect-gh-cli',
      { method: 'POST' }
    )
  }

  async startGitHubDeviceCode(clientId: string) {
    return this.request<{
      device_code: string
      user_code: string
      verification_uri: string
      expires_in: number
      interval: number
    }>('/github/auth/device-code/start', {
      method: 'POST',
      body: JSON.stringify({ client_id: clientId }),
    })
  }

  async pollGitHubDeviceCode(clientId: string, deviceCode: string) {
    return this.request<{ status: string; username?: string }>(
      '/github/auth/device-code/poll',
      {
        method: 'POST',
        body: JSON.stringify({ client_id: clientId, device_code: deviceCode }),
      }
    )
  }

  // JIRA
  async getJiraTickets() {
    return this.request<{ data: any[] }>('/jira/tickets')
  }

  async getJiraTicketDetail(key: string) {
    return this.request<{ data: any }>(`/jira/tickets/${key}`)
  }

  // GitLab
  async getGitLabMRs() {
    return this.request<{ data: any[] }>('/gitlab/mrs')
  }

  async getGitLabMRDetail(projectId: number, iid: number) {
    return this.request<{ data: any }>(`/gitlab/mrs/${projectId}/${iid}`)
  }

  async getGitLabPipelines() {
    return this.request<{ data: any[] }>('/gitlab/pipelines')
  }

  // Calendar
  async getCalendarEvents() {
    return this.request<{ data: any[] }>('/calendar/events')
  }

  async startCalendarAuth(source?: string, flow?: string) {
    return this.request<{ auth_url: string; source: string; flow: string }>('/calendar/auth/start', {
      method: 'POST',
      body: JSON.stringify({ source: source || 'ews', flow: flow || 'redirect' }),
    })
  }

  async getCalendarAuthStatus() {
    return this.request<{ connected: boolean }>('/calendar/auth/status')
  }

  async startCalendarDeviceCode(source?: string) {
    return this.request<{
      user_code: string
      verification_uri: string
      expires_in: number
      interval: number
    }>('/calendar/auth/device-code/start', {
      method: 'POST',
      body: JSON.stringify({ source: source || 'ews' }),
    })
  }

  async pollCalendarDeviceCode() {
    return this.request<{ status: string; error?: string }>('/calendar/auth/device-code/poll', {
      method: 'POST',
    })
  }

  async exchangeCalendarCode(code: string, redirectUri?: string) {
    return this.request<{ status: string }>('/calendar/auth/exchange-code', {
      method: 'POST',
      body: JSON.stringify({ code, redirect_uri: redirectUri }),
    })
  }

  // Config
  async getConfig() {
    return this.request<any>('/config')
  }

  async updateConfig(config: any) {
    return this.request<any>('/config', {
      method: 'PUT',
      body: JSON.stringify(config),
    })
  }
}

export const api = new ApiClient()
