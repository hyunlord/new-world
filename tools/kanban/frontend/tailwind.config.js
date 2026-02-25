/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'kanban-bg': '#0f0f0f',
        'kanban-card': '#1a1a2e',
        'kanban-panel': '#16213e',
        'status-todo': '#3b82f6',
        'status-claimed': '#f59e0b',
        'status-progress': '#ef4444',
        'status-review': '#8b5cf6',
        'status-done': '#22c55e',
        'status-failed': '#dc2626',
        'priority-critical': '#ef4444',
        'priority-high': '#f97316',
        'priority-medium': '#6b7280',
        'priority-low': '#9ca3af',
      },
    },
  },
  plugins: [],
}
