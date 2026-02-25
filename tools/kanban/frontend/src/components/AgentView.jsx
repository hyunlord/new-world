import { useState, useEffect } from 'react'
import { fetchAgentStats } from '../utils/api'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts'

function get_rate_color(rate) {
  if (rate >= 90) return { dot: 'bg-green-400', text: 'text-green-400' }
  if (rate >= 70) return { dot: 'bg-blue-400', text: 'text-blue-400' }
  if (rate >= 50) return { dot: 'bg-yellow-400', text: 'text-yellow-400' }
  return { dot: 'bg-red-400', text: 'text-red-400' }
}

function format_minutes(seconds) {
  if (seconds === null || seconds === undefined) return '-'
  return `${(Math.round((seconds / 60) * 10) / 10).toFixed(1)}m`
}

export default function AgentView() {
  const [agents, setAgents] = useState([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchAgentStats()
      .then(data => setAgents(Array.isArray(data) ? data : data?.agents ?? []))
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])

  if (loading) {
    return <div className="min-h-full bg-[#0a0a1a] p-6 text-gray-500">Loading agent stats...</div>
  }

  const chartData = agents.map(agent => ({
    name: agent.agent,
    success_rate: agent.success_rate ?? 0,
    avg_duration_minutes: agent.avg_duration_seconds ? Math.round((agent.avg_duration_seconds / 60) * 10) / 10 : 0,
  }))

  return (
    <div className="min-h-full bg-[#0a0a1a] p-6 space-y-6 text-gray-300">
      <h2 className="text-xl font-semibold text-white">Agent Performance</h2>

      <div className="bg-[#16213e] rounded-lg p-4">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700">
              <th className="text-left text-gray-400 font-medium px-4 py-2">Agent</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">Total</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">Done</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">Failed</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">In Progress</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">Success Rate</th>
              <th className="text-left text-gray-400 font-medium px-4 py-2">Avg Duration</th>
            </tr>
          </thead>
          <tbody>
            {agents.map(agent => {
              const rate = agent.success_rate ?? 0
              const colors = get_rate_color(rate)
              return (
                <tr key={agent.agent} className="border-b border-gray-700 hover:bg-[#1f1f3a]">
                  <td className="px-4 py-2 text-white font-medium">
                    <div className="flex items-center gap-2">
                      <span className={`inline-block h-2.5 w-2.5 rounded-full ${colors.dot}`} />
                      <span>{agent.agent}</span>
                    </div>
                  </td>
                  <td className="px-4 py-2">{agent.total ?? 0}</td>
                  <td className="px-4 py-2 text-green-400">{agent.done ?? 0}</td>
                  <td className="px-4 py-2 text-red-400">{agent.failed ?? 0}</td>
                  <td className="px-4 py-2">{agent.in_progress ?? 0}</td>
                  <td className={`px-4 py-2 font-medium ${colors.text}`}>{rate}%</td>
                  <td className="px-4 py-2 text-gray-400">{format_minutes(agent.avg_duration_seconds)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>

      <div className="grid grid-cols-2 gap-6">
        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Success Rate by Agent</h3>
          <ResponsiveContainer width="100%" height={260}>
            <BarChart data={chartData} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis type="number" domain={[0, 100]} stroke="#6b7280" fontSize={10} />
              <YAxis type="category" dataKey="name" stroke="#6b7280" fontSize={11} width={100} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Bar dataKey="success_rate" fill="#22c55e" radius={[0, 2, 2, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Avg Duration by Agent</h3>
          <ResponsiveContainer width="100%" height={260}>
            <BarChart data={chartData} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis type="number" stroke="#6b7280" fontSize={10} />
              <YAxis type="category" dataKey="name" stroke="#6b7280" fontSize={11} width={100} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Bar dataKey="avg_duration_minutes" fill="#3b82f6" radius={[0, 2, 2, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  )
}
