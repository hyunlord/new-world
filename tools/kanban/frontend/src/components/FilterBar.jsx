import { useState } from 'react'

const STATUSES = ['todo', 'claimed', 'in_progress', 'review', 'done', 'failed']
const STATUS_COLORS = {
  todo: '#3b82f6', claimed: '#f59e0b', in_progress: '#ef4444',
  review: '#8b5cf6', done: '#22c55e', failed: '#dc2626',
}

export default function FilterBar({ filters, onFiltersChange }) {
  const toggleStatus = (s) => {
    const current = filters.status ? filters.status.split(',') : []
    const updated = current.includes(s)
      ? current.filter(x => x !== s)
      : [...current, s]
    onFiltersChange({ ...filters, status: updated.join(',') || undefined })
  }

  const activeStatuses = filters.status ? filters.status.split(',') : []

  return (
    <div className="bg-[#16213e] rounded-lg p-4 mb-4 space-y-3">
      <div className="flex items-center gap-2 flex-wrap">
        <span className="text-xs text-gray-500 mr-2">Status:</span>
        {STATUSES.map(s => (
          <button
            key={s}
            onClick={() => toggleStatus(s)}
            className={`text-xs px-2 py-1 rounded-full border transition-colors ${
              activeStatuses.includes(s)
                ? 'border-transparent text-white'
                : 'border-gray-600 text-gray-400 hover:text-gray-300'
            }`}
            style={activeStatuses.includes(s) ? { backgroundColor: STATUS_COLORS[s] + '40', color: STATUS_COLORS[s] } : {}}
          >
            {s.replace('_', ' ')}
          </button>
        ))}
      </div>
      <div className="flex gap-3 items-center">
        <input
          type="text"
          placeholder="Search tickets..."
          value={filters.search || ''}
          onChange={e => onFiltersChange({ ...filters, search: e.target.value || undefined })}
          className="flex-1 bg-[#0f0f0f] border border-gray-700 rounded px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
        />
        <input
          type="text"
          placeholder="System..."
          value={filters.system || ''}
          onChange={e => onFiltersChange({ ...filters, system: e.target.value || undefined })}
          className="w-40 bg-[#0f0f0f] border border-gray-700 rounded px-3 py-1.5 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500"
        />
        <input
          type="date"
          value={filters.dateFrom || ''}
          onChange={e => onFiltersChange({ ...filters, dateFrom: e.target.value || undefined })}
          className="bg-[#0f0f0f] border border-gray-700 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500"
        />
        <span className="text-gray-500">—</span>
        <input
          type="date"
          value={filters.dateTo || ''}
          onChange={e => onFiltersChange({ ...filters, dateTo: e.target.value || undefined })}
          className="bg-[#0f0f0f] border border-gray-700 rounded px-3 py-1.5 text-sm text-white focus:outline-none focus:border-blue-500"
        />
      </div>
    </div>
  )
}
