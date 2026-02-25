import { useEffect, useRef } from 'react'

const LEVEL_COLORS = {
  info: 'text-blue-400 bg-blue-500/10',
  warn: 'text-yellow-400 bg-yellow-500/10',
  error: 'text-red-400 bg-red-500/10',
  debug: 'text-gray-400 bg-gray-500/10',
}

export default function LogViewer({ logs }) {
  const bottomRef = useRef(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [logs])

  if (!logs || logs.length === 0) {
    return <p className="text-gray-500 text-sm">No logs yet</p>
  }

  return (
    <div className="space-y-1 font-mono text-xs">
      {logs.map((log, i) => (
        <div key={i} className="flex items-start gap-2">
          <span className="text-gray-500 shrink-0">
            {new Date(log.timestamp).toLocaleTimeString()}
          </span>
          <span
            className={`px-1.5 py-0.5 rounded text-xs uppercase shrink-0 ${
              LEVEL_COLORS[log.level] || LEVEL_COLORS.info
            }`}
          >
            {log.level}
          </span>
          <span className="text-gray-300 break-all">{log.message}</span>
        </div>
      ))}
      <div ref={bottomRef} />
    </div>
  )
}
