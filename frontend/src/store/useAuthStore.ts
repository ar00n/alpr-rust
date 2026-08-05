import { create } from 'zustand'
import { queryClient } from '../main';

interface AuthState {
  token: string | null
  username: string | null
  isAuthenticated: boolean
  isAdmin: boolean
  setCredentials: (token: string, username?: string) => void
  logout: () => void
}

const parseJwtPayload = (token: string | null): Record<string, any> | null => {
  if (!token) return null
  try {
    const payloadBase64 = token.split('.')[1]
    if (!payloadBase64) return null

    const base64 = payloadBase64.replace(/-/g, '+').replace(/_/g, '/')
    return JSON.parse(atob(base64))
  } catch {
    return null
  }
}

/**
 * Helper to check if a JWT token is expired.
 */
const isTokenExpired = (token: string | null): boolean => {
  const payload = parseJwtPayload(token)
  if (!payload) return true

  if (!payload.exp) return true

  return Date.now() >= payload.exp * 1000
}

/**
 * Helper to extract `is_admin` claim from token.
 */
const checkIsAdmin = (token: string | null): boolean => {
  const payload = parseJwtPayload(token)
  return Boolean(payload?.is_admin)
}

/**
 * Utility to get valid initial auth state from localStorage
 */
const getInitialAuthState = () => {
  const token = localStorage.getItem('token')
  const username = localStorage.getItem('username')

  if (token && !isTokenExpired(token)) {
    return {
      token,
      username,
      isAuthenticated: true,
      isAdmin: checkIsAdmin(token),
    }
  }

  localStorage.removeItem('token')
  localStorage.removeItem('username')

  return { token: null, username: null, isAuthenticated: false, isAdmin: false }
}

export const useAuthStore = create<AuthState>((set) => ({
  ...getInitialAuthState(),

  setCredentials: (token, username) => {
    localStorage.setItem('token', token)
    if (username) localStorage.setItem('username', username)
    set({
      token,
      username: username ?? null,
      isAuthenticated: true,
      isAdmin: checkIsAdmin(token),
    })
  },

  logout: () => {
    localStorage.removeItem('token')
    localStorage.removeItem('username')
    set({ token: null, username: null, isAuthenticated: false, isAdmin: false })
  },
}))

useAuthStore.subscribe((state, prevState) => {
  // Update QueryClient default options dynamically
  queryClient.setDefaultOptions({
    queries: {
      enabled: state.isAuthenticated,
    },
  })

  // When user logs in, invalidate/refetch pending queries
  if (state.isAuthenticated && !prevState.isAuthenticated) {
    queryClient.invalidateQueries()
  }

  // When user logs out, clear cached sensitive data
  if (!state.isAuthenticated && prevState.isAuthenticated) {
    queryClient.clear()
  }
})