import { useEffect, useState, useMemo, useCallback, useRef } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Badge } from '@/components/ui/badge';
import { Activity, History as HistoryIcon, Settings as SettingsIcon, Car, LogOut, Zap } from 'lucide-react';
import { useQueryClient } from '@tanstack/react-query';

import {
  useGetHistoryHandler,
  getGetHistoryHandlerQueryKey,
  type PlateRead,
  type PaginatedHistoryResponse,
} from '@/lib/rust_api/schema';

// Import divided components
import LiveView from './components/LiveView';
import History from './components/History';
import Allowlist from './components/Allowlist';
import Settings from './components/Settings';
import { LoginModal } from './components/LoginModal';
import { useAuthStore } from './store/useAuthStore';
import { Button } from '@/components/ui/button';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { RtspSetupModal } from './components/RtspSetupModal';
import CustomActions from './components/CustomActions';

export default function App() {
  const { isAdmin, logout } = useAuthStore();
  const queryClient = useQueryClient();
  const [status, setStatus] = useState<'Connected' | 'Disconnected' | 'Error' | 'Reconnecting'>('Disconnected');

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const perPage = 10;

  const handleSearchChange = useCallback((newSearch: string) => {
    setSearch(newSearch);
    setPage(1);
  }, []);

  const queryParams = useMemo(
    () => ({
      page,
      per_page: perPage,
      plate: search ? search : undefined,
    }),
    [page, perPage, search]
  );

  const historyQueryKey = useMemo(() => {
    return typeof getGetHistoryHandlerQueryKey === 'function'
      ? getGetHistoryHandlerQueryKey(queryParams)
      : ['history', queryParams];
  }, [queryParams]);

  const historyQueryKeyRef = useRef(historyQueryKey);
  useEffect(() => {
    historyQueryKeyRef.current = historyQueryKey;
  }, [historyQueryKey]);

  const {
    data: historyResponse,
    isLoading,
    isError,
    refetch: refetchHistory,
  } = useGetHistoryHandler(queryParams);

  const plates = historyResponse?.items ?? [];
  const totalPages = historyResponse?.total_pages ?? 1;
  const total = historyResponse?.total ?? 0;

  useEffect(() => {
    let ws: WebSocket | null = null;
    let timeoutId: ReturnType<typeof setTimeout> | null = null;
    let isUnmounted = false;
    let retryCount = 0;

    const connect = () => {
      if (isUnmounted) return;

      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${wsProtocol}//${window.location.hostname}:${window.location.port}/api/ws`;
      ws = new WebSocket(wsUrl);

      ws.onopen = () => {
        if (isUnmounted) return;
        setStatus('Connected');
        retryCount = 0; // Reset retry counter on successful connection
      };

      ws.onclose = () => {
        if (isUnmounted) return;
        setStatus('Disconnected');
        scheduleReconnect();
      };

      ws.onerror = () => {
        if (isUnmounted) return;
        setStatus('Error');
      };

      ws.onmessage = (event) => {
        if (isUnmounted) return;
        try {
          const newRead: PlateRead = JSON.parse(event.data);
          queryClient.setQueryData<PaginatedHistoryResponse>(historyQueryKeyRef.current, (oldData) => {
            if (!oldData) return oldData;
            return {
              ...oldData,
              items: [newRead, ...oldData.items.slice(0, oldData.per_page - 1)],
              total: oldData.total + 1,
              total_pages: Math.ceil((oldData.total + 1) / oldData.per_page),
            };
          });
        } catch (err) {
          console.error('Failed to parse incoming WebSocket message:', err);
        }
      };
    };

    const scheduleReconnect = () => {
      // Exponential backoff: 1s, 2s, 4s, 8s, up to 10s max delay
      const delay = Math.min(1000 * Math.pow(2, retryCount), 10000);
      retryCount++;

      setStatus('Reconnecting');
      timeoutId = setTimeout(() => {
        connect();
      }, delay);
    };

    connect();

    return () => {
      isUnmounted = true;
      if (timeoutId) clearTimeout(timeoutId);
      if (ws) {
        ws.onopen = null;
        ws.onclose = null;
        ws.onerror = null;
        ws.onmessage = null;
        ws.close();
      }
    };
  }, [queryClient]);

  return (
    <div className="container mx-auto px-4 py-4 sm:px-6 sm:py-8 space-y-4 sm:space-y-8 max-w-7xl">
      <LoginModal />
      <RtspSetupModal />

      {/* Responsive Header */}
      <header className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 sm:pb-6 border-b">
        {/* Title & Icon */}
        <div className="flex items-center gap-3 sm:gap-4">
          <div className="p-2 sm:p-2.5 bg-primary/10 rounded-lg text-primary shrink-0">
            <Car className="w-5 h-5 sm:w-6 sm:h-6" />
          </div>
          <div>
            <h1 className="text-xl sm:text-2xl md:text-3xl font-bold tracking-tight">ANPR Dashboard</h1>
            <p className="text-xs sm:text-sm text-muted-foreground">Live Traffic & Access Control</p>
          </div>
        </div>

        {/* Status & Actions */}
        <div className="flex items-center justify-between sm:justify-end gap-3 sm:gap-4 border-t sm:border-0 pt-3 sm:pt-0">
          <div className="flex items-center gap-2">
            <span className="text-xs sm:text-sm font-medium text-muted-foreground">System Status</span>
            <Badge variant={status === 'Connected' ? 'default' : status === 'Reconnecting' ? 'outline' : 'destructive'} className="text-xs">
              {status === 'Connected' ? 'Live' : status}
            </Badge>
          </div>

          {/* Logout Confirmation Dialog */}
          <AlertDialog>
            <AlertDialogTrigger>
              <Button
                variant="ghost"
                size="sm"
                className="text-muted-foreground hover:text-foreground gap-1.5 text-xs sm:text-sm shrink-0"
              >
                <LogOut className="w-4 h-4" />
                <span className="hidden sm:inline">Logout</span>
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Are you sure you want to log out?</AlertDialogTitle>
                <AlertDialogDescription>
                  You will need to sign in again to access the dashboard and live monitoring.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Cancel</AlertDialogCancel>
                <AlertDialogAction 
                  onClick={logout}
                  className="bg-destructive text-white hover:bg-destructive/90"
                >
                  Log out
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        </div>
      </header>

      {/* Responsive Scrollable / Adaptive Tabs */}
      <Tabs defaultValue="live" className="space-y-4 sm:space-y-6">
        <div className="w-full overflow-x-auto pb-1 sm:pb-0 scrollbar-none">
          <TabsList className={`w-full min-w-[360px] sm:min-w-0 grid ${isAdmin ? 'grid-cols-5' : 'grid-cols-3'} h-auto p-1`}>
            <TabsTrigger value="live" className="gap-1.5 sm:gap-2 py-2 px-2 sm:px-3 text-xs sm:text-sm">
              <Activity className="w-3.5 h-3.5 sm:w-4 sm:h-4 shrink-0" />
              <span className="truncate">Live View</span>
            </TabsTrigger>
            <TabsTrigger value="history" className="gap-1.5 sm:gap-2 py-2 px-2 sm:px-3 text-xs sm:text-sm">
              <HistoryIcon className="w-3.5 h-3.5 sm:w-4 sm:h-4 shrink-0" />
              <span className="truncate">History</span>
            </TabsTrigger>
            <TabsTrigger value="allowlist" className="gap-1.5 sm:gap-2 py-2 px-2 sm:px-3 text-xs sm:text-sm">
              <Car className="w-3.5 h-3.5 sm:w-4 sm:h-4 shrink-0" />
              <span className="truncate">Allowlist</span>
            </TabsTrigger>
            {isAdmin && (
              <TabsTrigger value="actions" className="gap-1.5 sm:gap-2 py-2 px-2 sm:px-3 text-xs sm:text-sm">
                <Zap className="w-3.5 h-3.5 sm:w-4 sm:h-4 shrink-0" />
                <span className="truncate">Actions</span>
              </TabsTrigger>
            )}
            {isAdmin && (
              <TabsTrigger value="settings" className="gap-1.5 sm:gap-2 py-2 px-2 sm:px-3 text-xs sm:text-sm">
                <SettingsIcon className="w-3.5 h-3.5 sm:w-4 sm:h-4 shrink-0" />
                <span className="truncate">Settings</span>
              </TabsTrigger>
            )}
          </TabsList>
        </div>

        <TabsContent value="live">
          <LiveView
            recentPlates={plates.slice(0, 5)}
            isLoading={isLoading}
            isError={isError}
            onReload={refetchHistory}
          />
        </TabsContent>
        <TabsContent value="history">
          <History
            plates={plates}
            page={page}
            totalPages={totalPages}
            total={total}
            perPage={perPage}
            search={search}
            onSearchChange={handleSearchChange}
            onPageChange={setPage}
            isLoading={isLoading}
            isError={isError}
            onReload={refetchHistory}
          />
        </TabsContent>
        <TabsContent value="allowlist">
          <Allowlist />
        </TabsContent>
        {isAdmin && (
          <TabsContent value="actions">
            <CustomActions />
          </TabsContent>
        )}
        {isAdmin && (
        <TabsContent value="settings">
          <Settings />
        </TabsContent>
        )}
      </Tabs>
    </div>
  );
}