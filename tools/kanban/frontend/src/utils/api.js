const API_BASE = '/api'

async function request(path, options = {}) {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { 'Content-Type': 'application/json', ...options.headers },
    ...options,
  })
  if (!res.ok) {
    const err = await res.text()
    throw new Error(`API ${res.status}: ${err}`)
  }
  return res.json()
}

export async function fetchTickets(params = {}) {
  const qs = new URLSearchParams()
  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== '') qs.append(k, v)
  })
  return request(`/tickets?${qs}`)
}

export async function fetchTicket(id) {
  return request(`/tickets/${id}`)
}

export async function createTicket(data) {
  return request('/tickets', { method: 'POST', body: JSON.stringify(data) })
}

export async function updateTicket(id, data) {
  return request(`/tickets/${id}`, { method: 'PATCH', body: JSON.stringify(data) })
}

export async function deleteTicket(id) {
  return request(`/tickets/${id}`, { method: 'DELETE' })
}

export async function clearAllTickets() {
  return request('/tickets/clear', { method: 'POST' })
}

export async function fetchTicketLogs(id) {
  return request(`/tickets/${id}/logs`)
}

export async function addTicketLog(id, data) {
  return request(`/tickets/${id}/logs`, { method: 'POST', body: JSON.stringify(data) })
}

export async function fetchStats() {
  return request('/stats')
}

export async function fetchDailyStats(days = 30) {
  return request(`/stats/daily?days=${days}`)
}

export async function fetchBatches(params = {}) {
  const qs = new URLSearchParams()
  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== '') qs.append(k, v)
  })
  return request(`/batches?${qs}`)
}

export async function fetchBatch(id) {
  return request(`/batches/${id}`)
}

export async function createBatch(data) {
  return request('/batches', { method: 'POST', body: JSON.stringify(data) })
}

export async function updateBatch(id, data) {
  return request(`/batches/${id}`, { method: 'PATCH', body: JSON.stringify(data) })
}
