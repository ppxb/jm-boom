import { create } from 'zustand'

import {
  clearSession,
  getCurrentSession,
  login as loginRequest,
  type UserProfile
} from '@/lib/api/user'

type UserStore = {
  user: UserProfile | null
  isInitializing: boolean
  isLoggingIn: boolean
  initialize: () => Promise<void>
  login: (params: { username: string; password: string }) => Promise<UserProfile>
  logout: () => Promise<void>
}

export const useUserStore = create<UserStore>()(set => ({
  user: null,
  isInitializing: false,
  isLoggingIn: false,
  initialize: async () => {
    set({ isInitializing: true })

    try {
      const result = await getCurrentSession()
      set({
        user: result?.user ?? null,
        isInitializing: false
      })
    } catch (error) {
      set({ user: null, isInitializing: false })
      throw error
    }
  },
  login: async ({ username, password }) => {
    set({ isLoggingIn: true })

    try {
      const result = await loginRequest({ username, password })

      set({
        user: result.user,
        isLoggingIn: false
      })

      return result.user
    } catch (error) {
      set({ isLoggingIn: false })
      throw error
    }
  },
  logout: async () => {
    await clearSession()
    set({ user: null })
  }
}))
