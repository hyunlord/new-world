import { useState, useCallback } from 'react'
import { fetchTickets, createTicket, updateTicket, deleteTicket } from '../utils/api'

export function useTickets() {
  const [tickets, setTickets] = useState([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)

  const load = useCallback(async (params = {}) => {
    setLoading(true)
    setError(null)
    try {
      const data = await fetchTickets(params)
      setTickets(data.tickets)
      return data
    } catch (err) {
      setError(err.message)
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const create = useCallback(async (data) => {
    const ticket = await createTicket(data)
    setTickets(prev => [ticket, ...prev])
    return ticket
  }, [])

  const update = useCallback(async (id, data) => {
    const ticket = await updateTicket(id, data)
    setTickets(prev => prev.map(t => t.id === id ? { ...t, ...ticket } : t))
    return ticket
  }, [])

  const remove = useCallback(async (id) => {
    await deleteTicket(id)
    setTickets(prev => prev.filter(t => t.id !== id))
  }, [])

  return { tickets, loading, error, load, create, update, remove }
}
