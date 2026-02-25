import { useState, useEffect } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { fetchBatches, fetchBatch, deleteBatch, bulkDeleteBatches } from '../utils/api'

const STATUS_STYLE = {
  completed: { dot: 'bg-green-500', text: 'text-green-400', label: 'Completed' },
  active: { dot: 'bg-yellow-500', text: 'text-yellow-400', label: 'Active' },
  partial: { dot: 'bg-red-500', text: 'text-red-400', label: 'Partial' },
}

const TICKET_STATUS_STYLE = {
  todo: 'text-blue-400',
  claimed: 'text-yellow-400',
  in_progress: 'text-orange-400',
  review: 'text-purple-400',
  done: 'text-green-400',
  failed: 'text-red-400',
}

function formatDuration(startedAt, completedAt) {
  if (!startedAt || !completedAt) return '-'
  const ms = new Date(completedAt) - new Date(startedAt)
  const secs = Math.floor(ms / 1000)
  if (secs < 60) return `${secs}s`
  const mins = Math.floor(secs / 60)
  const remSecs = secs % 60
  return `${mins}m ${remSecs}s`
}

function ProgressBar({ total, completed, failed }) {
  if (total === 0) return <div className="w-full h-2 bg-gray-700 rounded" />
  const doneWidth = ((completed - failed) / total) * 100
  const failWidth = (failed / total) * 100
  return (
    <div className="w-full h-2 bg-gray-700 rounded overflow-hidden flex">
      <div className="h-full bg-green-500 transition-all" style={{ width: `${doneWidth}%` }} />
      <div className="h-full bg-red-500 transition-all" style={{ width: `${failWidth}%` }} />
    </div>
  )
}

function ScoreBadge({ score }) {
  if (score == null) return <span className="text-gray-500">-</span>
  let color = 'text-red-400'
  let star = ''
  if (score >= 90) { color = 'text-yellow-300'; star = ' ★' }
  else if (score >= 70) color = 'text-green-400'
  else if (score >= 50) color = 'text-yellow-400'
  return <span className={`font-bold ${color}`}>{score}{star}</span>
}

function ConfirmDialog({ open, title, message, onConfirm, onCancel }) {
  if (!open) return null
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
      <div className="bg-[#1a2744] rounded-lg p-6 max-w-sm w-full mx-4">
        <h3 className="text-white font-semibold mb-2">{title}</h3>
        <p className="text-gray-400 text-sm mb-4">{message}</p>
        <div className="flex gap-3 justify-end">
          <button onClick={onCancel} className="px-3 py-1.5 text-sm text-gray-400 hover:text-white transition-colors">
            Cancel
          </button>
          <button onClick={onConfirm} className="px-3 py-1.5 text-sm rounded bg-red-600 text-white hover:bg-red-700 transition-colors">
            Delete
          </button>
        </div>
      </div>
    </div>
  )
}

export function BatchList() {
  const [batches, setBatches] = useState([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState(new Set())
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const navigate = useNavigate()

  const loadBatches = () => {
    fetchBatches({ sort: 'created_at', order: 'desc', limit: 50 })
      .then(data => setBatches(data.batches))
      .catch(console.error)
      .finally(() => setLoading(false))
  }

  useEffect(() => { loadBatches() }, [])

  const toggleSelect = (e, id) => {
    e.stopPropagation()
    setSelected(prev => {
      const next = new Set(prev)
      next.has(id) ? next.delete(id) : next.add(id)
      return next
    })
  }

  const handleBulkDelete = async () => {
    try {
      await bulkDeleteBatches([...selected])
      setSelected(new Set())
      setShowDeleteConfirm(false)
      loadBatches()
    } catch (err) {
      console.error('Failed to delete batches:', err)
    }
  }

  const handleSingleDelete = async (e, id) => {
    e.stopPropagation()
    if (!confirm('Delete this batch and all its tickets?')) return
    try {
      await deleteBatch(id)
      loadBatches()
    } catch (err) {
      console.error('Failed to delete batch:', err)
    }
  }

  if (loading) return <div className="p-6 text-gray-500">Loading batches...</div>

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-white">Batches</h2>
        {selected.size > 0 && (
          <button
            onClick={() => setShowDeleteConfirm(true)}
            className="px-3 py-1.5 text-sm rounded bg-red-600/20 text-red-400 hover:bg-red-600/40 transition-colors"
          >
            Delete Selected ({selected.size})
          </button>
        )}
      </div>

      <ConfirmDialog
        open={showDeleteConfirm}
        title="Delete Batches"
        message={`Delete ${selected.size} batch(es) and all their tickets? This cannot be undone.`}
        onConfirm={handleBulkDelete}
        onCancel={() => setShowDeleteConfirm(false)}
      />

      {batches.length === 0 ? (
        <p className="text-gray-500">No batches yet</p>
      ) : (
        <div className="space-y-2">
          {batches.map(b => {
            const st = STATUS_STYLE[b.status] || STATUS_STYLE.active
            return (
              <div
                key={b.id}
                onClick={() => navigate(`/batches/${b.id}`)}
                className="bg-[#16213e] rounded-lg p-4 cursor-pointer hover:bg-[#1a2744] transition-colors flex items-center gap-4 group"
              >
                <input
                  type="checkbox"
                  checked={selected.has(b.id)}
                  onChange={(e) => toggleSelect(e, b.id)}
                  onClick={(e) => e.stopPropagation()}
                  className="w-4 h-4 rounded border-gray-600 bg-gray-800 text-blue-500 focus:ring-0 flex-shrink-0"
                />
                <div className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${st.dot}`} />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-white truncate">{b.title}</p>
                  {b.source_prompt && (
                    <p className="text-xs text-gray-500 truncate mt-0.5">{b.source_prompt}</p>
                  )}
                </div>
                <div className="w-16 text-right flex-shrink-0">
                  <ScoreBadge score={b.quality_score} />
                </div>
                <div className="flex items-center gap-4 flex-shrink-0">
                  <span className="text-xs text-gray-400 w-16 text-right">
                    {b.completed_tickets}/{b.total_tickets}
                  </span>
                  <div className="w-24">
                    <ProgressBar total={b.total_tickets} completed={b.completed_tickets} failed={0} />
                  </div>
                  <span className="text-xs text-gray-500 w-20 text-right">
                    {b.created_at ? new Date(b.created_at).toLocaleDateString() : ''}
                  </span>
                </div>
                <button
                  onClick={(e) => handleSingleDelete(e, b.id)}
                  className="text-gray-600 hover:text-red-400 transition-colors opacity-0 group-hover:opacity-100 flex-shrink-0"
                  title="Delete batch"
                >
                  &#x1F5D1;
                </button>
              </div>
            )
          })}
        </div>
      )}
    </div>
  )
}

export function BatchDetail() {
  const { batchId } = useParams()
  const navigate = useNavigate()
  const [batch, setBatch] = useState(null)
  const [loading, setLoading] = useState(true)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)

  useEffect(() => {
    fetchBatch(batchId)
      .then(setBatch)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [batchId])

  const handleDelete = async () => {
    try {
      await deleteBatch(batchId)
      navigate('/batches')
    } catch (err) {
      console.error('Failed to delete batch:', err)
    }
  }

  if (loading) return <div className="p-6 text-gray-500">Loading batch...</div>
  if (!batch) return <div className="p-6 text-red-400">Batch not found</div>

  const st = STATUS_STYLE[batch.status] || STATUS_STYLE.active
  const tickets = batch.tickets || []
  const ratio = batch.dispatch_ratio || { codex: 0, direct: 0, percentage: 0 }

  const doneCount = tickets.filter(t => t.status === 'done').length
  const failedCount = tickets.filter(t => t.status === 'failed').length
  const resolvedCount = doneCount + failedCount
  const qualityScore = typeof batch.quality_score === 'number' ? batch.quality_score : null
  const successPoints = resolvedCount > 0 ? Math.round((doneCount / resolvedCount) * 40) : 0
  const noRetryPoints = resolvedCount > 0 ? Math.round((doneCount / resolvedCount) * 20) : 0
  const dispatchPoints = Math.max(0, Math.min(10, Math.round(((ratio.percentage || 0) / 100) * 10)))
  const speedPoints = qualityScore == null
    ? null
    : Math.max(0, Math.min(30, qualityScore - successPoints - noRetryPoints - dispatchPoints))

  return (
    <div className="p-6">
      <Link to="/batches" className="text-sm text-blue-400 hover:text-blue-300 mb-4 inline-block">
        &larr; Back to Batches
      </Link>

      <ConfirmDialog
        open={showDeleteConfirm}
        title="Delete Batch"
        message={`Delete batch "${batch.title}" and all ${tickets.length} tickets? This cannot be undone.`}
        onConfirm={handleDelete}
        onCancel={() => setShowDeleteConfirm(false)}
      />

      {/* Header */}
      <div className="bg-[#16213e] rounded-lg p-5 mb-4">
        <div className="flex items-center gap-3 mb-3">
          <div className={`w-3 h-3 rounded-full ${st.dot}`} />
          <h2 className="text-lg font-semibold text-white">{batch.title}</h2>
          <span className={`text-xs ${st.text}`}>{st.label}</span>
          <div className="flex-1" />
          <button
            onClick={() => setShowDeleteConfirm(true)}
            className="px-3 py-1 text-xs rounded bg-red-600/20 text-red-400 hover:bg-red-600/40 transition-colors"
          >
            Delete Batch
          </button>
        </div>
        {batch.description && (
          <p className="text-sm text-gray-400 mb-2">{batch.description}</p>
        )}
        <div className="flex gap-6 text-xs text-gray-500 mt-3">
          {batch.source_prompt && <span>Source: {batch.source_prompt}</span>}
          <span>Created: {batch.created_at ? new Date(batch.created_at).toLocaleString() : '-'}</span>
        </div>
        <div className="flex gap-6 mt-3 text-sm">
          <span className="text-gray-400">
            Progress: <span className="text-white font-medium">{batch.completed_tickets}/{batch.total_tickets}</span>
          </span>
          <span className="text-gray-400">
            Dispatch: <span className="text-blue-400 font-medium">{ratio.codex} codex</span>
            {' / '}
            <span className="text-orange-400 font-medium">{ratio.direct} direct</span>
            {' = '}
            <span className="text-white font-medium">{ratio.percentage}%</span>
          </span>
        </div>
        <div className="mt-3 w-64">
          <ProgressBar total={batch.total_tickets} completed={batch.completed_tickets} failed={0} />
        </div>
      </div>

      {/* Quality Score */}
      {qualityScore != null && (
        <div className="bg-[#16213e] rounded-lg p-4 mb-4">
          <p className="text-sm text-gray-300">
            Quality Score: <ScoreBadge score={qualityScore} />
          </p>
          <div className="mt-2 text-xs text-gray-400 space-y-1">
            <p>├ Success Rate: {successPoints}/40 ({doneCount}/{resolvedCount || 0})</p>
            <p>├ Speed: {speedPoints == null ? '-' : speedPoints}/30</p>
            <p>├ No Retries: {noRetryPoints}/20</p>
            <p>└ Dispatch: {dispatchPoints}/10</p>
          </div>
        </div>
      )}

      {/* Ticket Table */}
      <div className="bg-[#16213e] rounded-lg overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700">
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-3 w-12">#</th>
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-3">Title</th>
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-3 w-24">Method</th>
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-3 w-28">Status</th>
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-3 w-24">Duration</th>
            </tr>
          </thead>
          <tbody>
            {tickets.map((t, i) => (
              <tr key={t.id} className="border-b border-gray-700/50 hover:bg-[#1a2744]">
                <td className="px-4 py-2.5 text-gray-500">{t.ticket_number ?? i + 1}</td>
                <td className="px-4 py-2.5 text-white">{t.title}</td>
                <td className="px-4 py-2.5">
                  <span className={`text-xs font-medium px-1.5 py-0.5 rounded ${
                    t.dispatch_method === 'codex'
                      ? 'bg-green-500/10 text-green-400'
                      : 'bg-orange-500/10 text-orange-400'
                  }`}>
                    {(t.dispatch_method || 'codex').toUpperCase()}
                  </span>
                </td>
                <td className={`px-4 py-2.5 text-xs font-medium ${TICKET_STATUS_STYLE[t.status] || 'text-gray-400'}`}>
                  {t.status === 'done' ? 'Done' : t.status === 'failed' ? 'Failed' : t.status?.replace('_', ' ')}
                </td>
                <td className="px-4 py-2.5 text-gray-500 text-xs">
                  {formatDuration(t.started_at, t.completed_at)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {tickets.length === 0 && (
          <p className="text-center text-gray-500 py-6 text-sm">No tickets in this batch</p>
        )}
      </div>
    </div>
  )
}
