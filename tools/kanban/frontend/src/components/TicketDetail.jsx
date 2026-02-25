import { useState, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import { fetchTicket, fetchTicketLogs } from '../utils/api'
import LogViewer from './LogViewer'
import DiffViewer from './DiffViewer'

const STATUS_COLORS = {
  todo: '#3b82f6',
  claimed: '#f59e0b',
  in_progress: '#ef4444',
  review: '#8b5cf6',
  done: '#22c55e',
  failed: '#dc2626',
}

function formatDuration(start, end) {
  if (!start || !end) return '—'
  const ms = new Date(end) - new Date(start)
  const secs = Math.floor(ms / 1000)
  if (secs < 60) return `${secs}s`
  const mins = Math.floor(secs / 60)
  const remSecs = secs % 60
  if (mins < 60) return `${mins}m ${remSecs}s`
  const hours = Math.floor(mins / 60)
  return `${hours}h ${mins % 60}m`
}

function parseJsonSafe(str) {
  if (!str) return []
  if (Array.isArray(str)) return str
  try { return JSON.parse(str) } catch { return [] }
}

export default function TicketDetail({ ticket, onClose }) {
  const hasBody = Boolean(ticket.body)
  const [activeTab, setActiveTab] = useState(hasBody ? 'body' : 'logs')
  const [logs, setLogs] = useState([])
  const [events, setEvents] = useState([])
  const [detail, setDetail] = useState(null)

  useEffect(() => {
    fetchTicket(ticket.id)
      .then(data => {
        setDetail(data)
        setEvents(data.events || [])
      })
      .catch(() => {})

    fetchTicketLogs(ticket.id)
      .then(setLogs)
      .catch(() => {})
  }, [ticket.id, ticket._logRefresh])

  const t = detail || ticket
  const files = parseJsonSafe(t.files)
  const deps = parseJsonSafe(t.dependencies)
  const statusColor = STATUS_COLORS[t.status] || '#6b7280'

  const tabs = [
    { id: 'body', label: 'Body', disabled: !t.body },
    { id: 'logs', label: 'Logs' },
    { id: 'diff', label: 'Diff', disabled: !t.diff_summary && !t.diff_full },
    { id: 'events', label: 'Events' },
  ]

  return (
    <div className="fixed inset-0 z-40 flex justify-end" onClick={onClose}>
      <div className="absolute inset-0 bg-black/40" />
      <div
        className="relative w-[480px] h-full bg-[#16213e] border-l border-gray-700 overflow-y-auto shadow-2xl flex flex-col"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="sticky top-0 bg-[#16213e] border-b border-gray-700 px-5 py-4 flex items-start justify-between z-10">
          <div className="flex-1 mr-3">
            <h2 className="text-lg font-semibold text-white leading-snug">{t.title}</h2>
            <div className="flex items-center gap-2 mt-1">
              <span
                className="text-xs px-2 py-0.5 rounded-full font-medium"
                style={{
                  backgroundColor: statusColor + '20',
                  color: statusColor,
                }}
              >
                {t.status}
              </span>
              {t.id && (
                <span className="text-xs text-gray-500 font-mono">{t.id.slice(0, 8)}</span>
              )}
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white text-2xl leading-none mt-0.5 shrink-0"
            aria-label="Close"
          >
            &times;
          </button>
        </div>

        {/* Info section */}
        <div className="px-5 py-4 space-y-2 text-sm border-b border-gray-700/50">
          {t.system && (
            <div>
              <span className="text-gray-500 w-24 inline-block">System:</span>
              <span className="text-gray-300">{t.system}</span>
            </div>
          )}
          <div>
            <span className="text-gray-500 w-24 inline-block">Priority:</span>
            <span className="text-gray-300">{t.priority}</span>
          </div>
          {t.assignee && (
            <div>
              <span className="text-gray-500 w-24 inline-block">Assignee:</span>
              <span className="text-gray-300">{t.assignee}</span>
            </div>
          )}
          {t.branch && (
            <div>
              <span className="text-gray-500 w-24 inline-block">Branch:</span>
              <span className="text-blue-400 font-mono text-xs">{t.branch}</span>
            </div>
          )}
          {files.length > 0 && (
            <div>
              <span className="text-gray-500 w-24 inline-block">Files:</span>
              <span className="text-gray-300 font-mono text-xs">{files.join(', ')}</span>
            </div>
          )}
          {deps.length > 0 && (
            <div>
              <span className="text-gray-500 w-24 inline-block">Deps:</span>
              <span className="text-gray-400 text-xs">
                {deps.map(d => (typeof d === 'string' ? d.slice(0, 8) : d)).join(', ')}
              </span>
            </div>
          )}
          <div>
            <span className="text-gray-500 w-24 inline-block">Created:</span>
            <span className="text-gray-300">
              {t.created_at ? new Date(t.created_at).toLocaleString() : '—'}
            </span>
          </div>
          <div>
            <span className="text-gray-500 w-24 inline-block">Duration:</span>
            <span className="text-gray-300">{formatDuration(t.started_at, t.completed_at)}</span>
          </div>
          {t.error_message && (
            <div className="mt-2 p-2 bg-red-600/10 rounded border border-red-600/20">
              <span className="text-red-400 text-xs">{t.error_message}</span>
            </div>
          )}
        </div>

        {/* Tabs */}
        <div className="flex border-b border-gray-700/50 shrink-0">
          {tabs.map(tab => (
            <button
              key={tab.id}
              onClick={() => !tab.disabled && setActiveTab(tab.id)}
              disabled={tab.disabled}
              className={`flex-1 py-2 text-sm font-medium transition-colors ${
                tab.disabled
                  ? 'text-gray-600 cursor-not-allowed'
                  : activeTab === tab.id
                    ? 'text-blue-400 border-b-2 border-blue-400'
                    : 'text-gray-500 hover:text-gray-300'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Tab content */}
        <div className="p-4 flex-1 overflow-y-auto">
          {activeTab === 'body' && t.body && (
            <div className="prose prose-invert prose-sm max-w-none">
              <ReactMarkdown>{t.body}</ReactMarkdown>
            </div>
          )}
          {activeTab === 'logs' && <LogViewer logs={logs} />}
          {activeTab === 'diff' && (
            <DiffViewer diffSummary={t.diff_summary} diffFull={t.diff_full} />
          )}
          {activeTab === 'events' && (
            <div className="space-y-2">
              {events.length === 0 ? (
                <p className="text-gray-500 text-sm">No events yet</p>
              ) : (
                events.map((ev, i) => (
                  <div key={i} className="flex items-start gap-2 text-xs">
                    <span className="text-gray-500 shrink-0">
                      {new Date(ev.timestamp).toLocaleTimeString()}
                    </span>
                    <span className="text-gray-400">
                      {ev.event_type}:{' '}
                      <span className="text-gray-500">{ev.old_value}</span>
                      {' → '}
                      <span className="text-white">{ev.new_value}</span>
                    </span>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
