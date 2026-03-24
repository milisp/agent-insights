import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import type { HeatmapData } from '@/types/insight'

interface CellData {
  date: string
  count: number
  size: number
  x: number
  y: number
}

interface HeatmapChartProps {
  data: HeatmapData
}

export function HeatmapChart({ data }: HeatmapChartProps) {
  const [showDetails, setShowDetails] = useState(true)

  const generateYearGrid = (): CellData[] => {
    const cells: CellData[] = []
    const today = new Date()
    const oneYearAgo = new Date(today)
    oneYearAgo.setFullYear(today.getFullYear() - 1)

    const activityMap = new Map<string, { count: number; size: number }>()
    data.data.forEach(({ date, count, size }) => {
      activityMap.set(date, { count, size })
    })

    let currentDate = new Date(oneYearAgo)
    let week = 0
    let day = currentDate.getDay()

    while (currentDate <= today) {
      const dateStr = currentDate.toISOString().split('T')[0]
      const activity = activityMap.get(dateStr) || { count: 0, size: 0 }

      cells.push({
        date: dateStr,
        count: activity.count,
        size: activity.size,
        x: week,
        y: day,
      })

      currentDate.setDate(currentDate.getDate() + 1)
      day = (day + 1) % 7
      if (day === 0) {
        week++
      }
    }

    return cells
  }

  const getColor = (count: number): string => {
    if (count === 0) return 'bg-gray-100 dark:bg-gray-800'

    const maxCount = data.max_count
    const intensity = count / maxCount

    if (intensity > 0.75) return 'bg-emerald-600'
    if (intensity > 0.5) return 'bg-emerald-500'
    if (intensity > 0.25) return 'bg-emerald-400'
    return 'bg-emerald-300'
  }

  const getMonthLabels = (cells: CellData[]): { label: string; x: number }[] => {
    const labels: { label: string; x: number }[] = []
    const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
    let lastMonth = -1

    cells.forEach((cell) => {
      const date = new Date(cell.date)
      const month = date.getMonth()
      const dayOfMonth = date.getDate()

      if (month !== lastMonth && (dayOfMonth <= 7 || cell.x === 0)) {
        labels.push({
          label: months[month],
          x: cell.x,
        })
        lastMonth = month
      }
    })

    return labels
  }

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
  }

  const formatNumber = (num: number): string => {
    return new Intl.NumberFormat('en', {
      notation: 'compact',
      maximumFractionDigits: 1,
    }).format(num)
  }

  const cells = generateYearGrid()
  const monthLabels = getMonthLabels(cells)
  const weeks = Math.max(...cells.map((c) => c.x)) + 1
  const cellSize = 12
  const cellGap = 3

  const activeDays = data.data.length

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <span>{data.agent}</span>
        </CardTitle>
        <CardDescription>
          {data.total_files} files · {activeDays} days active · {formatBytes(data.total_size)}
        </CardDescription>
        <div className="flex items-center gap-2 pt-2">
          <Switch checked={showDetails} onCheckedChange={setShowDetails} />
          <span className="text-sm text-muted-foreground">
            Show tool calls & usage
          </span>
        </div>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <div className="inline-block min-w-full">
            {/* Month labels */}
            <div className="relative mb-2" style={{ height: '20px', marginLeft: '30px' }}>
              {monthLabels.map((label) => (
                <div
                  key={label.label + label.x}
                  className="absolute text-xs text-muted-foreground"
                  style={{ left: `${label.x * (cellSize + cellGap)}px` }}
                >
                  {label.label}
                </div>
              ))}
            </div>

            {/* Heatmap grid */}
            <div className="flex">
              {/* Day labels */}
              <div className="flex flex-col justify-around pr-2" style={{ width: '30px' }}>
                <div className="text-xs text-muted-foreground">Mon</div>
                <div className="text-xs text-muted-foreground">Wed</div>
                <div className="text-xs text-muted-foreground">Fri</div>
              </div>

              {/* Grid */}
              <div
                className="relative"
                style={{
                  width: `${weeks * (cellSize + cellGap)}px`,
                  height: `${7 * (cellSize + cellGap)}px`,
                }}
              >
                {cells.map((cell) => (
                  <div
                    key={cell.date}
                    className={`absolute rounded-sm ${getColor(cell.count)} hover:ring-2 hover:ring-primary cursor-pointer transition-all`}
                    style={{
                      left: `${cell.x * (cellSize + cellGap)}px`,
                      top: `${cell.y * (cellSize + cellGap)}px`,
                      width: `${cellSize}px`,
                      height: `${cellSize}px`,
                    }}
                    title={`${cell.date}: ${cell.count} file${cell.count !== 1 ? 's' : ''} (${formatBytes(cell.size)})`}
                  />
                ))}
              </div>
            </div>

            {/* Legend */}
            <div className="flex items-center justify-end mt-4 space-x-2 text-xs text-muted-foreground">
              <span>Less</span>
              <div className="flex space-x-1">
                <div className={`w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-800`} />
                <div className={`w-3 h-3 rounded-sm bg-emerald-300`} />
                <div className={`w-3 h-3 rounded-sm bg-emerald-400`} />
                <div className={`w-3 h-3 rounded-sm bg-emerald-500`} />
                <div className={`w-3 h-3 rounded-sm bg-emerald-600`} />
              </div>
              <span>More</span>
            </div>
          </div>
        </div>

        {/* Statistics */}
        {showDetails && (
          <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Tool Calls */}
            {data.tool_calls && data.tool_calls.length > 0 && (
              <div>
                <h4 className="text-sm font-semibold mb-2">Top Tool Calls</h4>
                <div className="space-y-1">
                  {data.tool_calls.slice(0, 5).map((tool) => (
                    <div key={tool.tool_name} className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">{tool.tool_name}</span>
                      <span className="font-mono font-medium">{tool.count.toLocaleString()}</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Token Stats */}
            {data.token_stats && (
              <div>
                <h4 className="text-sm font-semibold mb-2">Token Usage</h4>
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Input Tokens</span>
                    <span className="font-mono font-medium">{formatNumber(data.token_stats.input_tokens)}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Output Tokens</span>
                    <span className="font-mono font-medium">{formatNumber(data.token_stats.output_tokens)}</span>
                  </div>
                  {data.token_stats.reasoning_tokens !== undefined && data.token_stats.reasoning_tokens > 0 && (
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">{data.agent === 'Codex' ? "Reasoning Tokens" : "Thoughts"}</span>
                      <span className="font-mono font-medium">{formatNumber(data.token_stats.reasoning_tokens)}</span>
                    </div>
                  )}
                  {data.agent === 'Claude' && (
                    <div className="flex items-center justify-between text-sm">
                      <span className="text-muted-foreground">Cache Created</span>
                      <span className="font-mono font-medium">{formatNumber(data.token_stats.cache_creation_tokens)}</span>
                    </div>
                  )}
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-muted-foreground">Cache Read</span>
                    <span className="font-mono font-medium">{formatNumber(data.token_stats.cache_read_tokens)}</span>
                  </div>
                  <div className="flex items-center justify-between text-sm pt-1 border-t">
                    <span className="font-semibold">Total</span>
                    <span className="font-mono font-semibold">{formatNumber(data.token_stats.total_tokens)}</span>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
