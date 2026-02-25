import { useState, useEffect } from 'react'
import { fetchBatches } from '../utils/api'
import TicketCard from './TicketCard'

const COLUMNS = [
  { id: 'todo', label: 'To Do', color: '#3b82f6' },
  { id: 'claimed', label: 'Claimed', color: '#f59e0b' },
  { id: 'in_progress', label: 'In Progress', color: '#ef4444' },
  { id: 'review', label: 'Review', color: '#8b5cf6' },
  { id: 'done', label: 'Done', color: '#22c55e' },
]

const PRIORITY_ORDER = { critical: 0, high: 1, medium: 2, low: 3 }

function sortTickets(tickets) {
  return [...tickets].sort((a, b) => {
    const pa = PRIORITY_ORDER[a.priority] ?? 2
    const pb = PRIORITY_ORDER[b.priority] ?? 2
    if (pa !== pb) return pa - pb
    return new Date(b.created_at) - new Date(a.created_at)
  })
}

export default function KanbanBoard({ tickets, onSelectTicket, onDeleteTicket, onDismissTicket }) {
  const [batches, setBatches] = useState([])
  const [batchFilter, setBatchFilter] = useState('')

  useEffect(() => {
    fetchBatches({ status: 'active', limit: 20, sort: 'created_at', order: 'desc' })
      .then(data => setBatches(data.batches || []))
      .catch(console.error)
  }, [])

  const filteredTickets = batchFilter
    ? tickets.filter(t => t.batch_id === batchFilter)
    : tickets

  const handleDismissAllFailed = async () => {
    if (!onDismissTicket) return
    const failedIds = failed.map(t => t.id)
    for (const id of failedIds) {
      await onDismissTicket(id)
    }
  }

  const failed = filteredTickets.filter(t => t.status === 'failed')

  return (
    <div>
      {/* Batch filter bar */}
      <div className="px-6 pt-4 pb-2">
        <select
          value={batchFilter}
          onChange={e => setBatchFilter(e.target.value)}
          className="bg-[#16213e] text-gray-300 text-sm rounded px-3 py-1.5 border border-gray-700 focus:border-blue-500 focus:outline-none"
        >
          <option value="">All Batches</option>
          {batches.map(b => (
            <option key={b.id} value={b.id}>
              {b.title} ({b.completed_tickets}/{b.total_tickets})
            </option>
          ))}
        </select>
      </div>

      <div className="w-full flex gap-4 px-6 pb-6 min-h-[calc(100vh-110px)]">
        {COLUMNS.map(col => {
          const colTickets = sortTickets(filteredTickets.filter(t => t.status === col.id))
          return (
            <div key={col.id} className="flex-1 min-w-0">
              <div
                className="rounded-t-lg px-3 py-2 flex items-center justify-between bg-[#16213e]"
                style={{ borderTop: `3px solid ${col.color}` }}
              >
                <h2 className="text-sm font-semibold text-gray-300">{col.label}</h2>
                <span
                  className="text-xs font-medium px-2 py-0.5 rounded-full"
                  style={{ backgroundColor: col.color + '20', color: col.color }}
                >
                  {colTickets.length}
                </span>
              </div>
              <div className="space-y-2 mt-2 max-h-[calc(100vh-200px)] overflow-y-auto pr-1">
                {colTickets.map(ticket => (
                  <TicketCard
                    key={ticket.id}
                    ticket={ticket}
                    allTickets={filteredTickets}
                    onClick={() => onSelectTicket(ticket)}
                    onDelete={onDeleteTicket}
                  />
                ))}
              </div>
              {col.id === 'in_progress' && failed.length > 0 && (
                <div className="mt-4">
                  <div className="rounded-t-lg px-3 py-2 border-t-2 border-red-600 bg-[#16213e] flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <h2 className="text-sm font-semibold text-red-400">Failed</h2>
                      <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-red-600/20 text-red-400">
                        {failed.length}
                      </span>
                    </div>
                    {onDismissTicket && (
                      <button
                        onClick={handleDismissAllFailed}
                        className="text-xs text-gray-500 hover:text-gray-300 transition-colors"
                      >
                        Dismiss All
                      </button>
                    )}
                  </div>
                  <div className="space-y-2 mt-2 max-h-60 overflow-y-auto">
                    {sortTickets(failed).map(ticket => (
                      <TicketCard
                        key={ticket.id}
                        ticket={ticket}
                        allTickets={filteredTickets}
                        onClick={() => onSelectTicket(ticket)}
                        onDelete={onDeleteTicket}
                        onDismiss={onDismissTicket}
                        failed
                      />
                    ))}
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
}
