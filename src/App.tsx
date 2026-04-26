import { useEffect, useState, useRef, useCallback } from 'react'
import { useSimulationStore } from './stores/simulationStore'
import { useWasm } from './hooks/useWasm'
import { Header } from './components/organisms/Header'
import { ControlPanel } from './components/organisms/ControlPanel'
import { MetricsPanel } from './components/organisms/MetricsPanel'
import { VisualizationPanel } from './components/organisms/VisualizationPanel'
import { SettingsModal } from './features/settings/SettingsModal'
import { OptimizerModal } from './features/optimizer/OptimizerModal'
import { MetricsSelectionModal } from './features/metrics'

function App() {
  const { wasmModule, loading: wasmLoading, error: wasmError } = useWasm()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [optimizerOpen, setOptimizerOpen] = useState(false)
  const [metricsOpen, setMetricsOpen] = useState(false)
  const [initializing, setInitializing] = useState(true)

  const runSimulation = useSimulationStore((state) => state.runSimulation)
  const loadInitialZoneData = useSimulationStore((state) => state.loadInitialZoneData)
  const config = useSimulationStore((state) => state.config)
  const zoneDataLoaded = useSimulationStore((state) => state.zoneDataLoaded)

  // Debounce ref for simulation
  const debounceRef = useRef<NodeJS.Timeout | null>(null)

  // Load zone data on startup
  useEffect(() => {
    if (wasmModule && !wasmLoading) {
      loadInitialZoneData().then(() => {
        setInitializing(false)
      })
    }
  }, [wasmModule, wasmLoading, loadInitialZoneData])

  // Debounced simulation runner
  const debouncedRunSimulation = useCallback(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current)
    }
    debounceRef.current = setTimeout(() => {
      runSimulation()
    }, 50) // 50ms debounce
  }, [runSimulation])

  // Run simulation when WASM is ready, zone data loaded, and config changes
  useEffect(() => {
    if (wasmModule && !wasmLoading && zoneDataLoaded && !initializing) {
      debouncedRunSimulation()
    }
  }, [wasmModule, wasmLoading, zoneDataLoaded, initializing, config, debouncedRunSimulation])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return
      }

      switch (e.key.toLowerCase()) {
        case 's':
          setSettingsOpen((prev) => !prev)
          break
        case 'o':
          setOptimizerOpen((prev) => !prev)
          break
        case 'm':
          setMetricsOpen((prev) => !prev)
          break
        case 'r':
          // Reset to defaults
          useSimulationStore.getState().resetToDefaults()
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [])

  if (wasmLoading || initializing) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">
            {wasmLoading ? 'Loading simulation engine...' : 'Loading zone data...'}
          </p>
        </div>
      </div>
    )
  }

  if (wasmError) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-100">
        <div className="text-center text-red-600">
          <p className="text-xl font-semibold mb-2">Failed to load simulation engine</p>
          <p className="text-sm">{wasmError}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-gray-100">
      <Header
        onSettingsClick={() => setSettingsOpen(true)}
        onOptimizerClick={() => setOptimizerOpen(true)}
        onResetClick={() => useSimulationStore.getState().resetToDefaults()}
      />

      <main className="container mx-auto px-4 py-6">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Column: Controls */}
          <div className="lg:col-span-1">
            <ControlPanel />
          </div>

          {/* Right Column: Metrics & Visualization */}
          <div className="lg:col-span-2 space-y-6">
            <MetricsPanel onOpenMetricsModal={() => setMetricsOpen(true)} />
            <VisualizationPanel />
          </div>
        </div>
      </main>

      {/* Modals */}
      <SettingsModal isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
      <OptimizerModal isOpen={optimizerOpen} onClose={() => setOptimizerOpen(false)} />
      <MetricsSelectionModal isOpen={metricsOpen} onClose={() => setMetricsOpen(false)} />
    </div>
  )
}

export default App
