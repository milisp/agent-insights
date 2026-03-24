import type { HeatmapData, AllHeatmapsResponse } from '@/types/insight'

const API_BASE_URL = 'http://127.0.0.1:3001/api'

export async function fetchAllHeatmaps(): Promise<AllHeatmapsResponse> {
  const response = await fetch(`${API_BASE_URL}/heatmaps`)
  if (!response.ok) {
    throw new Error('Failed to fetch heatmaps')
  }
  return response.json()
}

export async function fetchAgentHeatmap(agent: string): Promise<HeatmapData> {
  const response = await fetch(`${API_BASE_URL}/heatmap/${agent.toLowerCase()}`)
  if (!response.ok) {
    throw new Error(`Failed to fetch ${agent} heatmap`)
  }
  return response.json()
}
