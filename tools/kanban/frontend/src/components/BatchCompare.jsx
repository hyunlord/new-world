import { useState, useEffect } from 'react'
import { useSearchParams } from 'react-router-dom'
import { fetchBatchCompare } from '../utils/api'
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from 'recharts'

const BAR_COLORS = ['#3b82f6', '#22c55e', '#f59e0b', '#8b5cf6', '#06b6d4']

function to_minutes(seconds) {
  if (seconds === null || seconds === undefined) return null
  return Math.round(seconds / 60)
}

export default function BatchCompare() {
  const [searchParams] = useSearchParams()
  const [compareData, setCompareData] = useState(null)
  const [loading, setLoading] = useState(true)
  const ids = searchParams.get('ids')

  useEffect(() => {
    if (!ids) {
      setCompareData(null)
      setLoading(false)
      return
    }

    setLoading(true)
    fetchBatchCompare(ids)
      .then(setCompareData)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [ids])

  if (loading) return <div className="p-6 text-gray-500">Loading comparison...</div>

  const batches = Array.isArray(compareData?.batches) ? compareData.batches : []

  if (batches.length < 2) {
    return (
      <div className="p-6">
        <h2 className="text-2xl font-bold text-gray-100 mb-4">Batch Comparison</h2>
        <div className="bg-[#16213e] rounded-lg p-6 text-gray-400">Select 2-5 batches to compare</div>
      </div>
    )
  }

  const chartData = batches.map((batch, i) => ({
    key: batch.id ?? i,
    title: batch.title || `Batch ${batch.id ?? i + 1}`,
    total: batch.total ?? 0,
    done: batch.done ?? 0,
    failed: batch.failed ?? 0,
    success_rate: batch.success_rate ?? 0,
    dispatch_ratio: batch.dispatch_ratio ?? 0,
    avg_duration_minutes: to_minutes(batch.avg_duration_seconds) ?? 0,
    avg_duration_seconds: batch.avg_duration_seconds,
    max_duration_seconds: batch.max_duration_seconds,
    min_duration_seconds: batch.min_duration_seconds,
  }))

  const rows = [
    { label: 'Total Tickets', key: 'total' },
    { label: 'Done', key: 'done' },
    { label: 'Failed', key: 'failed' },
    { label: 'Success Rate', key: 'success_rate', format: value => `${value}%` },
    { label: 'Avg Duration', key: 'avg_duration_seconds', format: value => value === null || value === undefined ? '-' : `${to_minutes(value)}m` },
    { label: 'Max Duration', key: 'max_duration_seconds', format: value => value === null || value === undefined ? '-' : `${to_minutes(value)}m` },
    { label: 'Min Duration', key: 'min_duration_seconds', format: value => value === null || value === undefined ? '-' : `${to_minutes(value)}m` },
    { label: 'Dispatch %', key: 'dispatch_ratio', format: value => `${value}%` },
  ]

  return (
    <div className="p-6 space-y-6">
      <h2 className="text-2xl font-bold text-gray-100">Batch Comparison</h2>

      <div className="grid grid-cols-3 gap-6">
        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Success Rate</h3>
          <ResponsiveContainer width="100%" height={240}>
            <BarChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="title" stroke="#6b7280" fontSize={10} />
              <YAxis stroke="#6b7280" fontSize={10} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Legend />
              <Bar dataKey="success_rate" name="Success Rate (%)" fill={BAR_COLORS[0]} radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Avg Duration</h3>
          <ResponsiveContainer width="100%" height={240}>
            <BarChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="title" stroke="#6b7280" fontSize={10} />
              <YAxis stroke="#6b7280" fontSize={10} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Legend />
              <Bar dataKey="avg_duration_minutes" name="Avg Duration (min)" fill={BAR_COLORS[1]} radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="bg-[#16213e] rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-300 mb-4">Dispatch Ratio</h3>
          <ResponsiveContainer width="100%" height={240}>
            <BarChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis dataKey="title" stroke="#6b7280" fontSize={10} />
              <YAxis stroke="#6b7280" fontSize={10} />
              <Tooltip contentStyle={{ backgroundColor: '#1a1a2e', border: '1px solid #374151', borderRadius: '8px' }} />
              <Legend />
              <Bar dataKey="dispatch_ratio" name="Dispatch Ratio (%)" fill={BAR_COLORS[2]} radius={[2, 2, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="bg-[#16213e] rounded-lg p-4">
        <h3 className="text-sm font-semibold text-gray-300 mb-4">Comparison Details</h3>
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700">
              <th className="text-left text-gray-500 text-xs font-medium px-4 py-2">Metric</th>
              {chartData.map(batch => (
                <th key={batch.key} className="text-left text-gray-500 text-xs font-medium px-4 py-2">
                  {batch.title}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map(row => (
              <tr key={row.key} className="border-b border-gray-700/50">
                <td className="px-4 py-2 text-gray-300 font-medium">{row.label}</td>
                {batches.map((batch, i) => (
                  <td key={`${row.key}-${batch.id ?? i}`} className="px-4 py-2 text-gray-300">
                    {row.format ? row.format(batch[row.key] ?? 0) : batch[row.key] ?? 0}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
