import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import {
  Search,
  Loader2,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  RotateCw,
  AlertCircle,
} from 'lucide-react';
import type { PlateRead } from '@/lib/rust_api/schema';
import { useGetSnapshot } from '@/lib/rust_api/schema';

function useSnapshot(snapshotImage: string) {
  const { data: image, isLoading } = useGetSnapshot(snapshotImage);
  return { image, isLoading };
}

function SnapshotImage({ snapshotImage }: { snapshotImage?: string | null }) {
  const { image, isLoading } = useSnapshot(snapshotImage ?? '');
  const [imageUrl, setImageUrl] = useState<string | null>(null);

  useEffect(() => {
    if (!image) {
      setImageUrl(null);
      return;
    }

    const url = URL.createObjectURL(image);
    setImageUrl(url);

    return () => {
      URL.revokeObjectURL(url);
    };
  }, [image]);

  const placeholderClasses =
    'h-16 w-24 sm:h-20 sm:w-36 bg-slate-100 rounded border flex items-center justify-center text-xs text-slate-400 shrink-0';

  if (!snapshotImage) {
    return <div className={placeholderClasses}>No Image</div>;
  }

  if (isLoading) {
    return (
      <div className={placeholderClasses}>
        <Loader2 className="h-4 w-4 animate-spin text-slate-400" />
      </div>
    );
  }

  if (!imageUrl) {
    return <div className={placeholderClasses}>No Image</div>;
  }

  return (
    <img
      src={imageUrl}
      alt="Car snapshot"
      className="h-16 w-24 sm:h-20 sm:w-36 object-cover rounded shadow-sm border shrink-0"
      onClick={(e) => {
        e.stopPropagation();
        window.open(imageUrl, '_blank');
      }}
      style={{ cursor: 'pointer' }}
    />
  );
}

interface HistoryProps {
  plates: PlateRead[];
  page: number;
  totalPages: number;
  total: number;
  perPage: number;
  search: string;
  onSearchChange: (newSearch: string) => void;
  onPageChange: (newPage: number) => void;
  isLoading?: boolean;
  isError?: boolean;
  onReload?: () => void;
}

export default function History({
  plates,
  page,
  totalPages,
  total,
  perPage,
  search,
  onSearchChange,
  onPageChange,
  isLoading = false,
  isError = false,
  onReload,
}: HistoryProps) {
  const [searchInput, setSearchInput] = useState(search);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (searchInput !== search) {
        onSearchChange(searchInput);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [searchInput, search, onSearchChange]);

  const showingStart = total === 0 ? 0 : (page - 1) * perPage + 1;
  const showingEnd = Math.min(page * perPage, total);

  return (
    <Card>
      <CardHeader className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-6 space-y-0">
        <CardTitle>Scan History</CardTitle>
        <div className="relative w-full sm:w-64">
          <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search license plate..."
            className="pl-9"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Mobile View: Stacked List */}
        <div className="space-y-3 md:hidden">
          {isError ? (
            <div className="text-center p-6 border rounded-lg text-muted-foreground">
              <div className="flex flex-col items-center justify-center gap-2">
                <div className="flex items-center gap-2 text-destructive font-medium">
                  <AlertCircle className="h-5 w-5" />
                  <span>Failed to load history data.</span>
                </div>
                {onReload && (
                  <Button variant="outline" size="sm" onClick={() => onReload()} className="gap-2 mt-1">
                    <RotateCw className="h-4 w-4" />
                    Reload
                  </Button>
                )}
              </div>
            </div>
          ) : isLoading ? (
            <div className="text-center p-6 border rounded-lg text-muted-foreground flex items-center justify-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin text-primary" />
              Loading page data...
            </div>
          ) : plates.length > 0 ? (
            plates.map((p) => (
              <div key={p.id} className="p-3 border rounded-lg flex items-center gap-3 bg-card shadow-sm">
                <SnapshotImage snapshotImage={p.snapshot_image} />
                <div className="flex-1 min-w-0 space-y-1">
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-mono text-base font-bold truncate">{p.plate}</span>
                    <Badge variant={p.confidence >= 0.9 ? 'default' : 'secondary'} className="shrink-0 text-xs">
                      {(p.confidence * 100).toFixed(1)}%
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {new Date(p.timestamp).toLocaleString()}
                  </p>
                </div>
              </div>
            ))
          ) : (
            <div className="text-center p-6 border rounded-lg text-muted-foreground text-sm">
              No results found.
            </div>
          )}
        </div>

        {/* Desktop View: Table */}
        <div className="hidden md:block rounded-md border overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Snapshot</TableHead>
                <TableHead>Plate Number</TableHead>
                <TableHead>Confidence</TableHead>
                <TableHead>Timestamp</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isError ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-center h-32 text-muted-foreground">
                    <div className="flex flex-col items-center justify-center gap-2">
                      <div className="flex items-center gap-2 text-destructive font-medium">
                        <AlertCircle className="h-5 w-5" />
                        <span>Failed to load history data.</span>
                      </div>
                      {onReload && (
                        <Button variant="outline" size="sm" onClick={() => onReload()} className="gap-2 mt-1">
                          <RotateCw className="h-4 w-4" />
                          Reload
                        </Button>
                      )}
                    </div>
                  </TableCell>
                </TableRow>
              ) : isLoading ? (
                <TableRow>
                  <TableCell colSpan={4} className="text-center h-24 text-muted-foreground">
                    <div className="flex items-center justify-center gap-2">
                      <Loader2 className="h-4 w-4 animate-spin text-primary" />
                      Loading page data...
                    </div>
                  </TableCell>
                </TableRow>
              ) : plates.length > 0 ? (
                plates.map((p) => (
                  <TableRow key={p.id}>
                    <TableCell>
                      <SnapshotImage snapshotImage={p.snapshot_image} />
                    </TableCell>
                    <TableCell className="font-mono text-lg font-bold">{p.plate}</TableCell>
                    <TableCell>
                      <Badge variant={p.confidence >= 0.9 ? 'default' : 'secondary'}>
                        {(p.confidence * 100).toFixed(1)}%
                      </Badge>
                    </TableCell>
                    <TableCell className="text-muted-foreground">
                      {new Date(p.timestamp).toLocaleString()}
                    </TableCell>
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell colSpan={4} className="text-center h-24 text-muted-foreground">
                    No results found.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        {/* Pagination Controls */}
        <div className="flex flex-col sm:flex-row items-center justify-between gap-3 pt-2">
          <div className="text-xs sm:text-sm text-muted-foreground text-center sm:text-left">
            Showing {showingStart} to {showingEnd} of {total} entries
          </div>
          <div className="flex items-center space-x-1 sm:space-x-2">
            <Button
              variant="outline"
              size="sm"
              className="h-8 w-8 p-0 sm:h-9 sm:w-auto sm:px-3"
              onClick={() => onPageChange(1)}
              disabled={page <= 1 || isLoading || isError}
              title="First Page"
            >
              <ChevronsLeft className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-8 w-8 p-0 sm:h-9 sm:w-auto sm:px-3"
              onClick={() => onPageChange(page - 1)}
              disabled={page <= 1 || isLoading || isError}
              title="Previous Page"
            >
              <ChevronLeft className="h-4 w-4" />
            </Button>
            <span className="text-xs sm:text-sm font-medium px-1 sm:px-2 whitespace-nowrap">
              Page {page} of {totalPages || 1}
            </span>
            <Button
              variant="outline"
              size="sm"
              className="h-8 w-8 p-0 sm:h-9 sm:w-auto sm:px-3"
              onClick={() => onPageChange(page + 1)}
              disabled={page >= totalPages || isLoading || isError}
              title="Next Page"
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
            <Button
              variant="outline"
              size="sm"
              className="h-8 w-8 p-0 sm:h-9 sm:w-auto sm:px-3"
              onClick={() => onPageChange(totalPages)}
              disabled={page >= totalPages || isLoading || isError}
              title="Last Page"
            >
              <ChevronsRight className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}