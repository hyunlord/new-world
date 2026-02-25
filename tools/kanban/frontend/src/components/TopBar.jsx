import { NavLink } from 'react-router-dom'

const navItems = [
  { to: '/board', label: 'Board' },
  { to: '/batches', label: 'Batches' },
  { to: '/history', label: 'History' },
  { to: '/stats', label: 'Stats' },
]

export default function TopBar({ stats }) {
  return (
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
    </div>
  )
}
