import { useState, useEffect } from 'react'
import { fetchStats, fetchDailyStats } from '../utils/api'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, PieChart, Pie, Cell, Legend } from 'recharts'

const PIE_COLORS = ['#3b82f6', '#22c55e', '#ef4444', '#f59e0b', '#8b5cf6', '#06b6d4', '#f97316', '#ec4899']

export default function StatsView() {
  const [stats, setStats] = useState(null)
  const [daily, setDaily] = useState(null)

  useEffect(() => {
    fetchStats().then(setStats).catch(console.error)
    fetchDailyStats(30).then(setDaily).catch(console.error)
  }, [])

  if (!stats) return <div className="p-6 text-gray-500">Loading stats...</div>

  const dailyData = daily ? Object.entries(daily.daily)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([date, val]) => ({ date: date.slice(5), done: val.done, failed: val.failed }))
    : []

  const systemData = stats.systems ? Object.entries(stats.systems)
    .filter(([k]) => k)
    .map(([name, value]) => ({ name: name || 'unknown', value }))
    : []

  const avgMin = stats.avg_duration_seconds ? Math.round(stats.avg_duration_seconds / 60) : null

  return (
    <div className="p-6 space-y-6">
      {/* Summary cards */}
      <div className="grid grid-cols-5 gap-4">
        <div className="bg-[#16213e] rounded-lg p-4">
          <p className="text-gray-500 text-xs uppercase">Total Tickets</p>
          <p className="text-3xl font-bold text-white mt-1">{stats.total}</p>
        </div>
        <div className="bg-[#16213e] rounded-lg p-4">
          <p className="text-gray-500 text-xs uppercase">Active</p>
          <p className="text-3xl font-bold text-yellow-400 mt-1">{stats.active}</p>
        </div>
        <div className="bg-[#16213e] rounded-lg p-4">
          <p className="text-gray-500 text-xs uppercase">Completed</p>
          <p className="text-3xl font-bold text-green-400 mt-1">{stats.done}</p>
        </div>
        <div className="bg-[#16213e] rounded-lg p-4">
          <p className="text-gray-500 text-xs uppercase">Success Rate</p>
          <p className="text-3xl font-bold text-white mt-1">{stats.rate}%</p>
          {avgMin !== null && <p className="text-xs text-gray-500 mt-1">Avg: {avgMin}m</p>}
        </div>
        <div className="bg-[#16213e] rounded-lg p-4">
          <p className="text-gray-500 text-xs uppercase">Dispatch Ratio</p>
          <p className="text-3xl font-bold text-blue-400 mt-1">{stats.dispatch_ratio ?? 0}%</p>
          <p className="text-xs text-gray-500 mt-1">
            {stats.active_batches ?? 0} active / {stats.total_batches ?? 0} batches
          </p>
        </div>
      </div>

      {/* Charts row */}
      <div className="grid grid-cols-2 gap-6">
        {/* Daily completions */}
        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Daily Completions (30d)</h3>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={dailyData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="date" stroke="#6b7280" fontSize={10} />
              <YAxis stroke="#6b7280" fontSize={10} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Bar dataKey="done" fill="#22c55e" radius={[2, 2, 0, 0]} />
              <Bar dataKey="failed" fill="#dc2626" radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* System distribution */}
        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">System Distribution</h3>
          {systemData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie data={systemData} cx="50%" cy="50%" outerRadius={80} dataKey="value" label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}>
                  {systemData.map((_, i) => (
                    <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
                <Legend />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <p className="text-gray-500 text-sm text-center py-8">No system data yet</p>
          )}
        </div>
      </div>

      {/* Dispatch method breakdown */}
      {stats.by_dispatch_method && Object.keys(stats.by_dispatch_method).length > 0 && (
        <div className="grid grid-cols-2 gap-6">
          {/* By dispatch method */}
          <div className="bg-[#16213e] rounded-lg p-4">
            <h3 className="text-sm font-semibold text-gray-300 mb-4">By Dispatch Method</h3>
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={Object.entries(stats.by_dispatch_method).map(([method, data]) => ({
                method: method.toUpperCase(),
                done: data.done,
                failed: data.failed,
                total: data.total,
              }))}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis dataKey="method" stroke="#6b7280" fontSize={11} />
                <YAxis stroke="#6b7280" fontSize={10} />
                <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
                <Bar dataKey="done" fill="#22c55e" name="Done" radius={[2, 2, 0, 0]} />
                <Bar dataKey="failed" fill="#dc2626" name="Failed" radius={[2, 2, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>

          {/* By created_by */}
          <div className="bg-[#16213e] rounded-lg p-4">
            <h3 className="text-sm font-semibold text-gray-300 mb-4">By Creator</h3>
            {stats.by_created_by && Object.keys(stats.by_created_by).length > 0 ? (
              <ResponsiveContainer width="100%" height={200}>
                <PieChart>
                  <Pie
                    data={Object.entries(stats.by_created_by).map(([creator, data]) => ({
                      name: creator.replace('_', ' '),
                      value: data.total,
                    }))}
                    cx="50%" cy="50%" outerRadius={70} dataKey="value"
                    label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                  >
                    {Object.keys(stats.by_created_by).map((_, i) => (
                      <Cell key={i} fill={PIE_COLORS[i % PIE_COLORS.length]} />
                    ))}
                  </Pie>
                  <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            ) : (
              <p className="text-gray-500 text-sm text-center py-8">No creator data yet</p>
            )}
          </div>
        </div>
      )}

      {/* Dispatch method details table */}
      {stats.by_dispatch_method && Object.keys(stats.by_dispatch_method).length > 0 && (
        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Dispatch Method Details</h3>
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Method</th>
                <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Total</th>
                <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Done</th>
                <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Failed</th>
                <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Avg Duration</th>
              </tr>
            </thead>
            <tbody>
              {Object.entries(stats.by_dispatch_method).map(([method, data]) => (
                <tr key={method} className="border-b border-gray-700/50">
                  <td className="px-4 py-2 text-white font-medium">{method.toUpperCase()}</td>
                  <td className="px-4 py-2 text-gray-300">{data.total}</td>
                  <td className="px-4 py-2 text-green-400">{data.done}</td>
                  <td className="px-4 py-2 text-red-400">{data.failed}</td>
                  <td className="px-4 py-2 text-gray-400">
                    {data.avg_duration ? `${Math.round(data.avg_duration / 60)}m` : '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
