import { useState } from 'react'
import { NavLink } from 'react-router-dom'

const navItems = [
  { to: '/board', label: 'Board' },
  { to: '/batches', label: 'Batches' },
  { to: '/history', label: 'History' },
  { to: '/stats', label: 'Stats' },
]

export default function TopBar({ stats, onClearAll }) {
  const [showConfirm, setShowConfirm] = useState(false)

  return (
    <>
      <div className="sticky top-0 z-50 bg-[#16213e] border-b border-gray-700 px-6 py-3 flex items-center justify-between">
        <h1 className="text-lg font-bold text-white">WorldSim Kanban</h1>
        <nav className="flex gap-4">
          {navItems.map(item => (
            <NavLink
              key={item.to}
              to={item.to}
              className={({ isActive }) =>
                `px-3 py-1 rounded text-sm font-medium transition-colors ${
                  isActive ? 'bg-blue-600 text-white' : 'text-gray-400 hover:text-white'
                }`
              }
            >
              {item.label}
            </NavLink>
          ))}
        </nav>
        <div className="flex items-center gap-4">
          {stats && (
            <div className="flex gap-4 text-sm">
              <span className="text-gray-400">
                Total: <span className="text-white font-medium">{stats.total}</span>
              </span>
              <span className="text-gray-400">
                Active: <span className="text-yellow-400 font-medium">{stats.active}</span>
              </span>
              <span className="text-gray-400">
                Done: <span className="text-green-400 font-medium">{stats.done}</span>
              </span>
              <span className="text-gray-400">
                Failed: <span className="text-red-400 font-medium">{stats.failed}</span>
              </span>
              <span className="text-gray-400">
                Rate: <span className="text-white font-medium">{stats.rate}%</span>
              </span>
            </div>
          )}
          {onClearAll && (
            <button
              onClick={() => setShowConfirm(true)}
              className="px-3 py-1 text-xs font-medium rounded bg-red-600/20 text-red-400 hover:bg-red-600/40 transition-colors"
            >
              Clear All
            </button>
          )}
        </div>
      </div>
      {showConfirm && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60">
          <div className="bg-[#1a1a2e] rounded-lg p-6 max-w-sm border border-gray-700">
            <p className="text-white text-sm mb-4">Delete all tickets? This cannot be undone.</p>
            <div className="flex justify-end gap-3">
              <button
                onClick={() => setShowConfirm(false)}
                className="px-3 py-1.5 text-sm rounded text-gray-400 hover:text-white transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => { setShowConfirm(false); onClearAll() }}
                className="px-3 py-1.5 text-sm rounded bg-red-600 text-white hover:bg-red-700 transition-colors"
              >
                Delete All
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
