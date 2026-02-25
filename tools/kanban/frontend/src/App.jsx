import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useState, useCallback, useEffect } from 'react'
import TopBar from './components/TopBar'
import KanbanBoard from './components/KanbanBoard'
import HistoryTable from './components/HistoryTable'
import StatsView from './components/StatsView'
import { BatchList, BatchDetail } from './components/BatchView'
import TicketDetail from './components/TicketDetail'
import { useWebSocket } from './hooks/useWebSocket'
import { fetchTickets, fetchStats } from './utils/api'

function App() {
  const [tickets, setTickets] = useState([])
  const [stats, setStats] = useState(null)
  const [selectedTicket, setSelectedTicket] = useState(null)

  const loadTickets = useCallback(async () => {
    try {
      const data = await fetchTickets({ limit: 200 })
      setTickets(data.tickets)
    } catch (err) {
      console.error('Failed to load tickets:', err)
    }
  }, [])

  const loadStats = useCallback(async () => {
    try {
      const data = await fetchStats()
      setStats(data)
    } catch (err) {
      console.error('Failed to load stats:', err)
    }
  }, [])

  useEffect(() => {
    loadTickets()
    loadStats()
  }, [loadTickets, loadStats])

  const handleWsMessage = useCallback((msg) => {
    if (msg.type === 'ticket_created') {
      setTickets(prev => {
        if (prev.some(t => t.id === msg.data.id)) return prev
        return [msg.data, ...prev]
      })
      loadStats()
    } else if (msg.type === 'ticket_updated') {
      setTickets(prev => prev.map(t => t.id === msg.ticket_id ? { ...t, ...msg.data } : t))
      loadStats()
    } else if (msg.type === 'log_added') {
      // Refresh selected ticket if it matches
      if (selectedTicket && selectedTicket.id === msg.ticket_id) {
        setSelectedTicket(prev => prev ? { ...prev, _logRefresh: Date.now() } : null)
      }
    }
  }, [selectedTicket, loadStats])

  useWebSocket(handleWsMessage)

  return (
    <BrowserRouter>
      <div className="min-h-screen bg-kanban-bg">
        <TopBar stats={stats} />
        <div className="relative">
          <Routes>
            <Route path="/board" element={
              <KanbanBoard tickets={tickets} onSelectTicket={setSelectedTicket} />
            } />
            <Route path="/history" element={
              <HistoryTable onSelectTicket={setSelectedTicket} />
            } />
            <Route path="/batches" element={<BatchList />} />
            <Route path="/batches/:batchId" element={<BatchDetail />} />
            <Route path="/stats" element={<StatsView />} />
            <Route path="*" element={<Navigate to="/board" replace />} />
          </Routes>
          {selectedTicket && (
            <TicketDetail
              ticket={selectedTicket}
              onClose={() => setSelectedTicket(null)}
            />
          )}
        </div>
      </div>
    </BrowserRouter>
  )
}

export default App
