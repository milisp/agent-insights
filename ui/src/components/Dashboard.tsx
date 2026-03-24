import { useState, useEffect, useCallback } from 'react'
import { HeatmapChart } from './HeatmapChart'
import { Toast, type ToastProps } from './Toast'
import { fetchAllHeatmaps } from '@/services/api'
import { useWebSocket, type WebSocketMessage } from '@/hooks/useWebSocket'
import type { AllHeatmapsResponse } from '@/types/insight'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

export function Dashboard() {
  const [heatmaps, setHeatmaps] = useState<AllHeatmapsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [toasts, setToasts] = useState<ToastProps[]>([])

  const loadHeatmaps = useCallback(async () => {
    try {
      const data = await fetchAllHeatmaps()
      setHeatmaps(data)
      setLoading(false)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data')
      setLoading(false)
    }
  }, [])

  const addToast = useCallback((toast: Omit<ToastProps, 'onClose'>) => {
    setToasts(prev => [...prev, { ...toast, onClose: () => {
      setToasts(prev => prev.filter((_, i) => i !== 0))
    }}])
  }, [])

  const handleWebSocketMessage = useCallback((message: WebSocketMessage) => {
    switch (message.type) {
      case 'file_added':
        addToast({
          message: `New ${message.agent} file detected`,
          type: 'info',
          duration: 3000,
        })
        loadHeatmaps()
        break
      case 'scan_completed':
        addToast({
          message: `${message.agent} scan completed: ${message.count} files`,
          type: 'success',
          duration: 2000,
        })
        loadHeatmaps()
        break
    }
  }, [addToast, loadHeatmaps])

  const { isConnected } = useWebSocket({
    url: 'ws://127.0.0.1:3001/ws',
    onMessage: handleWebSocketMessage,
    onOpen: () => console.log('WebSocket connected'),
    onClose: () => console.log('WebSocket disconnected'),
  })

  useEffect(() => {
    loadHeatmaps()
  }, [loadHeatmaps])

  if (loading) {
    return (
      <div className="container mx-auto p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold">Agent Insights</h1>
          <p className="text-muted-foreground">Loading agent activity data...</p>
        </div>
        <div className="grid gap-6">
          {[1, 2, 3].map((i) => (
            <Card key={i}>
              <CardHeader>
                <div className="h-6 w-32 bg-gray-200 dark:bg-gray-800 rounded animate-pulse" />
                <div className="h-4 w-48 bg-gray-200 dark:bg-gray-800 rounded animate-pulse mt-2" />
              </CardHeader>
              <CardContent>
                <div className="h-24 bg-gray-200 dark:bg-gray-800 rounded animate-pulse" />
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="container mx-auto p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold">Agent Insights</h1>
          <p className="text-muted-foreground">AI Agent Usage Analytics</p>
        </div>
        <Card>
          <CardHeader>
            <CardTitle>Error Loading Data</CardTitle>
            <CardDescription className="text-red-500">{error}</CardDescription>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">
              Make sure the backend server is running on http://127.0.0.1:3001
            </p>
            <button
              onClick={loadHeatmaps}
              className="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
            >
              Retry
            </button>
          </CardContent>
        </Card>
      </div>
    )
  }

  if (!heatmaps || Object.keys(heatmaps).length === 0) {
    return (
      <div className="container mx-auto p-6">
        <div className="mb-8">
          <h1 className="text-3xl font-bold">Agent Insights</h1>
          <p className="text-muted-foreground">AI Agent Usage Analytics</p>
        </div>
        <Card>
          <CardHeader>
            <CardTitle>No Data Found</CardTitle>
            <CardDescription>
              No agent activity data found. Make sure you have used Claude, Gemini, or Codex.
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    )
  }

  const totalFiles = Object.values(heatmaps).reduce((sum, h) => sum + h.total_files, 0)
  const totalSize = Object.values(heatmaps).reduce((sum, h) => sum + h.total_size, 0)
  const agentCount = Object.keys(heatmaps).length

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  const sortedAgents = Object.entries(heatmaps).sort(([, a], [, b]) => b.total_files - a.total_files)

  return (
    <div className="container mx-auto p-6">
      <div className="mb-8">
        <div className="flex items-center justify-between mb-2">
          <h1 className="text-3xl font-bold">Agent Insights</h1>
          <div className="flex items-center gap-2">
            <div className={`h-2 w-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-gray-400'}`} />
            <span className="text-sm text-muted-foreground">
              {isConnected ? 'Live' : 'Offline'}
            </span>
          </div>
        </div>
        <p className="text-muted-foreground">
          AI Agent Usage Analytics · {agentCount} agents · {totalFiles} files · {formatBytes(totalSize)}
        </p>
      </div>

      <div className="grid gap-6">
        {sortedAgents.map(([agentName, heatmapData]) => (
          <HeatmapChart key={agentName} data={heatmapData} />
        ))}
      </div>

      {toasts.map((toast, index) => (
        <Toast key={index} {...toast} />
      ))}
    </div>
  )
}
