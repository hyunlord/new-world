import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { useState, useCallback, useEffect } from 'react'
import TopBar from './components/TopBar'
import KanbanBoard from './components/KanbanBoard'
import HistoryTable from './components/HistoryTable'
import StatsView from './components/StatsView'
import { BatchList, BatchDetail } from './components/BatchView'
import BatchCompare from './components/BatchCompare'
import AgentView from './components/AgentView'
import TicketDetail from './components/TicketDetail'
import { useWebSocket } from './hooks/useWebSocket'
import { fetchTickets, fetchStats, deleteTicket, clearAllTickets, dismissTicket } from './utils/api'

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
      // Browser notifications for done/failed
      if (typeof Notification !== 'undefined' && Notification.permission === 'granted') {
        const status = msg.data?.status
        if (status === 'done') {
          new Notification('Ticket Done', { body: msg.data.title })
        } else if (status === 'failed') {
          new Notification('Ticket Failed', { body: `${msg.data.title}: ${msg.data.error_message || 'Unknown error'}` })
        }
      }
    } else if (msg.type === 'log_added') {
      if (selectedTicket && selectedTicket.id === msg.ticket_id) {
        setSelectedTicket(prev => prev ? { ...prev, _logRefresh: Date.now() } : null)
      }
    } else if (msg.type === 'tickets_cleared') {
      setTickets([])
      loadStats()
    } else if (msg.type === 'batch_deleted' || msg.type === 'batch_bulk_deleted') {
      loadStats()
    }
  }, [selectedTicket, loadStats])

  const handleDeleteTicket = useCallback(async (id) => {
    try {
      await deleteTicket(id)
      setTickets(prev => prev.filter(t => t.id !== id))
      loadStats()
    } catch (err) {
      console.error('Failed to delete ticket:', err)
    }
  }, [loadStats])

  const handleDismissTicket = useCallback(async (id) => {
    try {
      await dismissTicket(id)
      setTickets(prev => prev.filter(t => t.id !== id))
      loadStats()
    } catch (err) {
      console.error('Failed to dismiss ticket:', err)
    }
  }, [loadStats])

  const handleClearAll = useCallback(async () => {
    try {
      await clearAllTickets()
      setTickets([])
      loadStats()
    } catch (err) {
      console.error('Failed to clear tickets:', err)
    }
  }, [loadStats])

  useWebSocket(handleWsMessage)

  return (
    <BrowserRouter>
      <div className="min-h-screen bg-kanban-bg">
        <TopBar stats={stats} onClearAll={handleClearAll} />
        <div className="relative">
          <Routes>
            <Route path="/board" element={
              <KanbanBoard tickets={tickets} onSelectTicket={setSelectedTicket} onDeleteTicket={handleDeleteTicket} onDismissTicket={handleDismissTicket} />
            } />
            <Route path="/history" element={
              <HistoryTable onSelectTicket={setSelectedTicket} />
            } />
            <Route path="/batches" element={<BatchList />} />
            <Route path="/batches/compare" element={<BatchCompare />} />
            <Route path="/batches/:batchId" element={<BatchDetail />} />
            <Route path="/agents" element={<AgentView />} />
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
