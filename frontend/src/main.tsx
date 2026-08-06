import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { 
  QueryClient, 
  QueryClientProvider, 
  QueryCache, 
  MutationCache 
} from '@tanstack/react-query'
import { toast } from 'sonner'
import './App.css'
import App from './App.tsx'
import { Toaster } from '@/components/ui/sonner.tsx'
import { useAuthStore } from './store/useAuthStore.ts';

export const getErrorMessage = (error: any, fallbackMessage?: string): string => {
  if (fallbackMessage) return fallbackMessage

  const responseData = error?.response?.data || error?.data
  const rawError = responseData?.error || responseData?.message || error?.message

  if (typeof rawError === 'string') {
    return rawError
  }

  if (typeof rawError === 'object' && rawError !== null) {
    return rawError.message || rawError.detail || JSON.stringify(rawError)
  }

  return 'An error occurred'
}

export const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (error: any, query: any) => {
      if (query.meta?.suppressToast) return

      const errorMessage = getErrorMessage(error, query.meta?.errorMessage as string)
      toast.error(errorMessage)
    },
  }),

  mutationCache: new MutationCache({
    onError: (error: any, _variables, _context, mutation) => {
      if (mutation.meta?.suppressToast) return

      const errorMessage = getErrorMessage(error, mutation.meta?.errorMessage as string)
      
      toast.error(errorMessage)
    },
  }),

  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5, // Data remains fresh for 5 minutes
      gcTime: 1000 * 60 * 10, // Inactive data is garbage collected after 10 minutes
      refetchOnWindowFocus: false, // Prevent refetching every time the user focuses the window
      retry: 1, // Retry failed requests once before displaying an error
      enabled: useAuthStore.getState().isAuthenticated, // Only fetch data if the user is authenticated
    },
  },
})

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
      <Toaster />
    </QueryClientProvider>
  </StrictMode>,
)