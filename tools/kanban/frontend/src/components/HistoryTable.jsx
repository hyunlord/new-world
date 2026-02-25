import { useState, useEffect, useCallback } from 'react'
import { fetchTickets } from '../utils/api'
import FilterBar from './FilterBar'

const STATUS_COLORS = {
  todo: '#3b82f6', claimed: '#f59e0b', in_progress: '#ef4444',
  review: '#8b5cf6', done: '#22c55e', failed: '#dc2626',
}

function formatDuration(start, end) {
  if (!start || !end) return '—'
  const ms = new Date(end) - new Date(start)
  const secs = Math.floor(ms / 1000)
  if (secs < 60) return `${secs}s`
  const mins = Math.floor(secs / 60)
  if (mins < 60) return `${mins}m ${secs % 60}s`
  const hours = Math.floor(mins / 60)
  return `${hours}h ${mins % 60}m`
}

export default function HistoryTable({ onSelectTicket }) {
  const [tickets, setTickets] = useState([])
  const [total, setTotal] = useState(0)
  const [offset, setOffset] = useState(0)
  const [filters, setFilters] = useState({})
  const limit = 50

  const load = useCallback(async () => {
    try {
      const params = { limit, offset, sort: 'created_at', order: 'desc' }
      if (filters.status) params.status = filters.status
      if (filters.system) params.system = filters.system
      if (filters.search) params.search = filters.search
      if (filters.dateFrom) params.date_from = filters.dateFrom
      if (filters.dateTo) params.date_to = filters.dateTo
      params.include_dismissed = 'true'
      const data = await fetchTickets(params)
      setTickets(data.tickets)
      setTotal(data.total)
    } catch (err) {
      console.error('Failed to load:', err)
    }
  }, [offset, filters])

  useEffect(() => { load() }, [load])

  const handleFilters = (f) => {
    setFilters(f)
    setOffset(0)
  }

  return (
    <div className="p-6">
      <FilterBar filters={filters} onFiltersChange={handleFilters} />
      <div className="bg-[#16213e] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="text-gray-400 text-xs uppercase border-b border-gray-700">
              <th className="text-left px-4 py-3">Title</th>
              <th className="text-left px-4 py-3">Status</th>
              <th className="text-left px-4 py-3">System</th>
              <th className="text-left px-4 py-3">Assignee</th>
              <th className="text-left px-4 py-3">Created</th>
              <th className="text-left px-4 py-3">Completed</th>
              <th className="text-left px-4 py-3">Duration</th>
              <th className="text-left px-4 py-3">Commit</th>
            </tr>
          </thead>
          <tbody>
            {tickets.map(t => (
              <tr
                key={t.id}
                onClick={() => onSelectTicket(t)}
                className="border-b border-gray-700/50 hover:bg-[#1a1a2e] cursor-pointer transition-colors"
              >
                <td className="px-4 py-3 text-white font-medium max-w-xs truncate">{t.title}</td>
                <td className="px-4 py-3">
                  <span
                    className="text-xs px-2 py-0.5 rounded-full"
                    style={{ backgroundColor: (STATUS_COLORS[t.status] || '#6b7280') + '20', color: STATUS_COLORS[t.status] || '#6b7280' }}
                  >
                    {t.status}
                  </span>
                </td>
                <td className="px-4 py-3 text-gray-400">{t.system || '—'}</td>
                <td className="px-4 py-3 text-gray-400">{t.assignee || '—'}</td>
                <td className="px-4 py-3 text-gray-500 text-xs">{new Date(t.created_at).toLocaleString()}</td>
                <td className="px-4 py-3 text-gray-500 text-xs">{t.completed_at ? new Date(t.completed_at).toLocaleString() : '—'}</td>
                <td className="px-4 py-3 text-gray-400 text-xs">{formatDuration(t.started_at, t.completed_at)}</td>
                <td className="px-4 py-3">
                  {t.commit_url ? (
                    <a href={t.commit_url} target="_blank" rel="noopener noreferrer"
                       className="text-blue-400 hover:text-blue-300 font-mono text-xs">
                      {t.commit_hash?.substring(0, 7)}
                    </a>
                  ) : (
                    <span className="text-gray-600">-</span>
                  )}
                </td>
              </tr>
            ))}
            {tickets.length === 0 && (
              <tr><td colSpan={8} className="px-4 py-8 text-center text-gray-500">No tickets found</td></tr>
            )}
          </tbody>
        </table>
      </div>
      {/* Pagination */}
      <div className="flex items-center justify-between mt-4 text-sm text-gray-400">
        <span>Showing {offset + 1}–{Math.min(offset + limit, total)} of {total}</span>
        <div className="flex gap-2">
          <button
            onClick={() => setOffset(Math.max(0, offset - limit))}
            disabled={offset === 0}
            className="px-3 py-1 rounded bg-[#16213e] border border-gray-700 disabled:opacity-30 hover:bg-[#1a1a2e]"
          >
            Prev
          </button>
          <button
            onClick={() => setOffset(offset + limit)}
            disabled={offset + limit >= total}
            className="px-3 py-1 rounded bg-[#16213e] border border-gray-700 disabled:opacity-30 hover:bg-[#1a1a2e]"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  )
}
