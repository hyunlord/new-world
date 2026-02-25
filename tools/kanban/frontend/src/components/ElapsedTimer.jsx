import { useState, useEffect } from 'react'

export default function ElapsedTimer({ startedAt }) {
  const [elapsed, setElapsed] = useState(0)

  useEffect(() => {
    if (!startedAt) return
    const start = new Date(startedAt).getTime()
    const update = () => setElapsed(Math.max(0, Math.floor((Date.now() - start) / 1000)))
    update()
    const interval = setInterval(update, 1000)
    return () => clearInterval(interval)
  }, [startedAt])

  const minutes = Math.floor(elapsed / 60)
  const seconds = elapsed % 60
  const display = `${minutes}m ${String(seconds).padStart(2, '0')}s`

  let colorClass = 'text-gray-400'
  if (minutes >= 20) colorClass = 'text-red-400 animate-pulse'
  else if (minutes >= 10) colorClass = 'text-orange-400'
  else if (minutes >= 5) colorClass = 'text-yellow-400'

  return <span className={`text-xs font-mono ${colorClass}`}>{display}</span>
}
