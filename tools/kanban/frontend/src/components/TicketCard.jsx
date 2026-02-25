const PRIORITY_COLORS = {
  critical: { bg: 'bg-red-500/20', text: 'text-red-400', dot: 'bg-red-500' },
  high: { bg: 'bg-orange-500/20', text: 'text-orange-400', dot: 'bg-orange-500' },
  medium: { bg: 'bg-gray-500/20', text: 'text-gray-400', dot: 'bg-gray-500' },
  low: { bg: 'bg-gray-600/20', text: 'text-gray-500', dot: 'bg-gray-600' },
}

const ACTOR_DOT = {
  claude_code: 'bg-blue-500',
  codex: 'bg-green-500',
  manual: 'bg-gray-400',
}

function timeAgo(dateStr) {
  if (!dateStr) return ''
  const diff = Date.now() - new Date(dateStr).getTime()
  const mins = Math.floor(diff / 60000)
  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  return `${days}d ago`
}

export default function TicketCard({ ticket, onClick, onDelete, failed }) {
  const pc = PRIORITY_COLORS[ticket.priority] || PRIORITY_COLORS.medium

  return (
    <div
      onClick={onClick}
      className={`group relative bg-[#1a1a2e] rounded-lg p-3 cursor-pointer hover:bg-[#1f1f3a] transition-colors ${
        failed ? 'border border-red-600/50' : 'border border-transparent'
      }`}
    >
      {onDelete && (
        <button
          onClick={e => { e.stopPropagation(); onDelete(ticket.id) }}
          className="absolute top-1.5 right-1.5 w-5 h-5 rounded flex items-center justify-center text-gray-600 hover:text-red-400 hover:bg-red-400/10 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          &times;
        </button>
      )}
      <div className="flex items-center gap-2 mb-1">
        <span
          className={`w-2 h-2 rounded-full flex-shrink-0 ${ACTOR_DOT[ticket.created_by] || ACTOR_DOT.manual}`}
          title={`Created by: ${ticket.created_by || 'manual'}`}
        />
        <p className="text-sm text-white font-medium truncate">{ticket.title}</p>
      </div>
      <div className="flex items-center gap-2 mt-2 flex-wrap">
        <span
          className={`inline-flex items-center gap-1 text-xs px-1.5 py-0.5 rounded ${pc.bg} ${pc.text}`}
        >
          <span className={`w-1.5 h-1.5 rounded-full ${pc.dot}`}></span>
          {ticket.priority}
        </span>
        {ticket.system && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-400">
            {ticket.system}
          </span>
        )}
      </div>
      <div className="flex items-center justify-between mt-2 text-xs text-gray-500">
        {ticket.assignee && (
          <span className="truncate max-w-[120px]">{ticket.assignee}</span>
        )}
        <span className="ml-auto">{timeAgo(ticket.created_at)}</span>
      </div>
      {ticket.error_message && failed && (
        <p className="text-xs text-red-400 mt-1 truncate">{ticket.error_message}</p>
      )}
      {ticket.batch_id && ticket.ticket_number != null && (
        <p className="text-xs text-gray-600 mt-1 truncate">
          Batch #{ticket.ticket_number}
        </p>
      )}
    </div>
  )
}
