import { create } from 'zustand'

import {
  clearSession,
  getCurrentSession,
  login as loginRequest,
  type UserProfile
} from '@/lib/api/user'

type UserStore = {
  user: UserProfile | null
  endpoint: string | null
  isInitializing: boolean
  isLoggingIn: boolean
  initialize: () => Promise<void>
  login: (params: {
    username: string
    password: string
    endpoint?: string | null
    rememberLogin?: boolean
  }) => Promise<UserProfile>
  logout: () => Promise<void>
}

export const useUserStore = create<UserStore>()(set => ({
  user: null,
  endpoint: null,
  isInitializing: false,
  isLoggingIn: false,
  initialize: async () => {
    set({ isInitializing: true })

    try {
      const result = await getCurrentSession()
      set({
        user: result?.user ?? null,
        endpoint: result?.endpoint ?? null,
        isInitializing: false
      })
    } catch (error) {
      set({ user: null, endpoint: null, isInitializing: false })
      throw error
    }
  },
  login: async ({ username, password, endpoint = null, rememberLogin = false }) => {
    set({ isLoggingIn: true })

    try {
      const result = await loginRequest({ username, password, endpoint, rememberLogin })

      set({
        user: result.user,
        endpoint: result.endpoint,
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
    set({ user: null, endpoint: null })
  }
}))
