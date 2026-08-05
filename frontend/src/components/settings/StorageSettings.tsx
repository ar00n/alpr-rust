import { useState, useEffect } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { HardDrive, AlertCircle, CheckCircle2, Loader2, Clock } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { 
  useGetTrimSnapshots,
  useUpdateTrimSnapshots,
  getGetTrimSnapshotsQueryKey,
  useGetTrimHistory,
  useUpdateTrimHistory,
  getGetTrimHistoryQueryKey
} from '@/lib/rust_api/schema';

export default function StorageSettings() {
  const queryClient = useQueryClient();

  // --- Snapshot Storage Trim Settings ---
  const { data: trimData, isLoading: isLoadingTrim } = useGetTrimSnapshots();
  const updateTrimMutation = useUpdateTrimSnapshots();
  const [trimMb, setTrimMb] = useState<number | ''>('');
  const [trimError, setTrimError] = useState<string | null>(null);
  const [trimSuccess, setTrimSuccess] = useState<string | null>(null);

  useEffect(() => {
    if (trimData?.trim_snapshots_mb !== undefined) {
      setTrimMb(trimData.trim_snapshots_mb ?? '');
    }
  }, [trimData?.trim_snapshots_mb]);

  const handleSaveTrim = async (e: React.FormEvent) => {
    e.preventDefault();
    setTrimError(null);
    setTrimSuccess(null);

    const parsedValue = trimMb === '' ? null : Number(trimMb);
    if (parsedValue !== null && (isNaN(parsedValue) || parsedValue < 0)) {
      setTrimError("Please enter a valid positive storage limit.");
      return;
    }

    try {
      await updateTrimMutation.mutateAsync({
        data: { trim_snapshots_mb: parsedValue },
      });
      queryClient.invalidateQueries({ queryKey: getGetTrimSnapshotsQueryKey() });
      setTrimSuccess("Snapshot storage limit updated successfully!");
    } catch (error: any) {
      setTrimError(error?.response?.data || error?.message || "Failed to update storage limit.");
    }
  };

  // --- History Days Trim Settings ---
  const { data: historyData, isLoading: isLoadingHistory } = useGetTrimHistory();
  const updateHistoryMutation = useUpdateTrimHistory();
  const [historyDays, setHistoryDays] = useState<number | ''>('');
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [historySuccess, setHistorySuccess] = useState<string | null>(null);

  useEffect(() => {
    if (historyData?.trim_history_days !== undefined) {
      setHistoryDays(historyData.trim_history_days ?? '');
    }
  }, [historyData?.trim_history_days]);

  const handleSaveHistory = async (e: React.FormEvent) => {
    e.preventDefault();
    setHistoryError(null);
    setHistorySuccess(null);

    const parsedValue = historyDays === '' ? null : Number(historyDays);
    if (parsedValue !== null && (isNaN(parsedValue) || parsedValue < 0)) {
      setHistoryError("Please enter a valid number of days.");
      return;
    }

    try {
      await updateHistoryMutation.mutateAsync({
        data: { trim_history_days: parsedValue },
      });
      queryClient.invalidateQueries({ queryKey: getGetTrimHistoryQueryKey() });
      setHistorySuccess("History retention limit updated successfully!");
    } catch (error: any) {
      setHistoryError(error?.response?.data || error?.message || "Failed to update history limit.");
    }
  };

  return (
    <Card className="w-full min-w-full h-fit">
      <CardHeader>
        <CardTitle>Data Retention & Storage</CardTitle>
        <CardDescription>
          Configure how long plate reads are kept in the database and set disk usage limits for snapshots.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-8">
        
        {/* History Trim Section */}
        <form onSubmit={handleSaveHistory} className="space-y-4">
          <h3 className="text-sm font-semibold">Database History Retention</h3>
          {historyError && (
            <div className="flex items-start gap-2.5 rounded-md border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <p className="opacity-90">{historyError}</p>
            </div>
          )}
          {historySuccess && (
            <div className="flex items-start gap-2.5 rounded-md border border-emerald-500/50 bg-emerald-500/10 p-3 text-xs text-emerald-600 dark:text-emerald-400">
              <CheckCircle2 className="h-4 w-4 shrink-0 mt-0.5" />
              <p className="opacity-90">{historySuccess}</p>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="historyDays">Keep records for (Days)</Label>
            <div className="relative">
              <Clock className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                id="historyDays"
                type="number"
                min="0"
                placeholder="e.g. 30 (leave empty to keep forever)"
                className="pl-9"
                value={historyDays}
                disabled={isLoadingHistory || updateHistoryMutation.isPending}
                onChange={(e) => {
                  setHistoryError(null);
                  setHistorySuccess(null);
                  setHistoryDays(e.target.value === '' ? '' : Number(e.target.value));
                }}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              Older plate reads and their snapshots will be automatically deleted. Leave empty to disable.
            </p>
          </div>
          <Button type="submit" className="w-full" disabled={isLoadingHistory || updateHistoryMutation.isPending}>
            {updateHistoryMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Save Retention Limit"}
          </Button>
        </form>

        <hr className="border-border" />

        {/* Snapshot Trim Section */}
        <form onSubmit={handleSaveTrim} className="space-y-4">
          <h3 className="text-sm font-semibold">Snapshot Disk Limit</h3>
          {trimError && (
            <div className="flex items-start gap-2.5 rounded-md border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive">
              <AlertCircle className="h-4 w-4 shrink-0 mt-0.5" />
              <p className="opacity-90">{trimError}</p>
            </div>
          )}
          {trimSuccess && (
            <div className="flex items-start gap-2.5 rounded-md border border-emerald-500/50 bg-emerald-500/10 p-3 text-xs text-emerald-600 dark:text-emerald-400">
              <CheckCircle2 className="h-4 w-4 shrink-0 mt-0.5" />
              <p className="opacity-90">{trimSuccess}</p>
            </div>
          )}

          <div className="space-y-2">
            <Label htmlFor="trimMb">Trim Threshold (MB)</Label>
            <div className="relative">
              <HardDrive className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
              <Input
                id="trimMb"
                type="number"
                min="0"
                placeholder="e.g. 1000 (leave empty to disable)"
                className="pl-9"
                value={trimMb}
                disabled={isLoadingTrim || updateTrimMutation.isPending}
                onChange={(e) => {
                  setTrimError(null);
                  setTrimSuccess(null);
                  setTrimMb(e.target.value === '' ? '' : Number(e.target.value));
                }}
              />
            </div>
            <p className="text-xs text-muted-foreground">
              Specify size in megabytes (e.g. 1000 MB ≈ 1 GB). Oldest images drop if limit is reached.
            </p>
          </div>
          <Button type="submit" className="w-full" disabled={isLoadingTrim || updateTrimMutation.isPending}>
            {updateTrimMutation.isPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : "Save Storage Limit"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}