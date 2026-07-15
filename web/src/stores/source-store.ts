import { create } from 'zustand'
import { persist } from 'zustand/middleware'

type SourceState = {
  selectedSourceId: string | null
  setSelectedSourceId: (sourceId: string | null) => void
  reset: () => void
}

export const useSourceStore = create<SourceState>()(
  persist(
    set => ({
      selectedSourceId: null,
      setSelectedSourceId: selectedSourceId => set({ selectedSourceId }),
      reset: () => set({ selectedSourceId: null })
    }),
    {
      name: 'jm-boom-source',
      partialize: state => ({ selectedSourceId: state.selectedSourceId })
    }
  )
)
