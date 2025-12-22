import { useEffect, useRef, useState } from 'react'

export interface WebSocketMessage {
  type: 'scan_started' | 'scan_completed' | 'file_added' | 'cache_stats'
  agent?: string
  count?: number
  file_path?: string
  total?: number
  by_agent?: Record<string, number>
}

interface UseWebSocketOptions {
  url: string
  onMessage?: (message: WebSocketMessage) => void
  onOpen?: () => void
  onClose?: () => void
  onError?: (error: Event) => void
  reconnectInterval?: number
}

export function useWebSocket({
  url,
  onMessage,
  onOpen,
  onClose,
  onError,
  reconnectInterval = 3000,
}: UseWebSocketOptions) {
  const [isConnected, setIsConnected] = useState(false)
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)

  useEffect(() => {
    let isMounted = true

    const connect = () => {
      if (!isMounted) return

      try {
        const ws = new WebSocket(url)
        wsRef.current = ws

        ws.onopen = () => {
          console.log('WebSocket connected')
          setIsConnected(true)
          onOpen?.()
        }

        ws.onmessage = (event: MessageEvent) => {
          try {
            const message = JSON.parse(event.data) as WebSocketMessage
            console.log('WebSocket message:', message)
            setLastMessage(message)
            onMessage?.(message)
          } catch (err) {
            console.error('Failed to parse WebSocket message:', err)
          }
        }

        ws.onclose = () => {
          console.log('WebSocket disconnected')
          setIsConnected(false)
          onClose?.()

          if (isMounted) {
            console.log(`Reconnecting in ${reconnectInterval}ms...`)
            reconnectTimeoutRef.current = setTimeout(connect, reconnectInterval)
          }
        }

        ws.onerror = (error) => {
          console.error('WebSocket error:', error)
          onError?.(error)
        }
      } catch (err) {
        console.error('Failed to create WebSocket:', err)
        if (isMounted) {
          reconnectTimeoutRef.current = setTimeout(connect, reconnectInterval)
        }
      }
    }

    connect()

    return () => {
      isMounted = false
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }
      if (wsRef.current) {
        wsRef.current.close()
      }
    }
  }, [url, onMessage, onOpen, onClose, onError, reconnectInterval])

  return {
    isConnected,
    lastMessage,
  }
}
