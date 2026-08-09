import { useState, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Plus, Trash2, Loader2, Upload, Download, Search, AlertCircle, RotateCw } from 'lucide-react';
import { 
  useGetAllowList, 
  useAddAllowList, 
  getGetAllowListQueryKey,
  useDeleteAllowList,
  useImportAllowListCsv,
  exportAllowListCsv
} from '@/lib/rust_api/schema';

type SearchField = 'all' | 'plate' | 'name' | 'metadata';

export default function Allowlist() {
  const queryClient = useQueryClient();
  const { data: list, isLoading, isError, refetch } = useGetAllowList();
  
  // File input ref for CSV import
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Search state
  const [searchQuery, setSearchQuery] = useState('');
  const [searchField, setSearchField] = useState<SearchField>('all');

  // Add form state
  const [open, setOpen] = useState(false);
  const [newPlate, setNewPlate] = useState('');
  const [newName, setNewName] = useState('');
  const [newMetadata, setNewMetadata] = useState('');
  const [newExpiry, setNewExpiry] = useState('');

  // Mutations
  const { mutate: addPlate, isPending: isAdding } = useAddAllowList({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getGetAllowListQueryKey() });
        setNewPlate('');
        setNewName('');
        setNewMetadata('');
        setNewExpiry('');
        setOpen(false);
      },
    },
  });

  const { mutate: deletePlate, isPending: isDeleting } = useDeleteAllowList({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getGetAllowListQueryKey() });
      },
    },
  });

  const { mutate: importCsv, isPending: isImporting } = useImportAllowListCsv({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getGetAllowListQueryKey() });
        if (fileInputRef.current) fileInputRef.current.value = '';
      },
    }
  });

  // Handlers
  const handleAdd = () => {
    if (!newPlate.trim()) return;

    addPlate({
      data: {
        plate: newPlate.trim().toUpperCase(),
        name: newName.trim() || null,
        metadata: newMetadata.trim() || null,
        expiry_date: newExpiry ? new Date(newExpiry).toISOString() : null,
      },
    });
  };

  const handleDelete = (plate: string) => {
    deletePlate({ plate }); 
  };

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      const csvText = event.target?.result as string;
      importCsv({ data: csvText });
    };
    reader.readAsText(file);
  };

  const handleExport = async () => {
    try {
      const data = await exportAllowListCsv();
      const blob = new Blob([data as unknown as string], { type: 'text/csv' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'allow_list.csv';
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error("Failed to export CSV", error);
    }
  };

  const filteredList = list?.filter((item) => {
    if (!searchQuery.trim()) return true;

    const q = searchQuery.toLowerCase();
    const matchPlate = item.plate.toLowerCase().includes(q);
    const matchName = item.name?.toLowerCase().includes(q) ?? false;
    const matchMeta = item.metadata?.toLowerCase().includes(q) ?? false;

    if (searchField === 'all') return matchPlate || matchName || matchMeta;
    if (searchField === 'plate') return matchPlate;
    if (searchField === 'name') return matchName;
    if (searchField === 'metadata') return matchMeta;
    
    return true;
  });

  return (
    <Card>
      <CardHeader className="flex flex-col md:flex-row md:items-center justify-between space-y-4 md:space-y-0 pb-6 gap-4">
        <div>
          <CardTitle>Authorized Vehicles</CardTitle>
          <CardDescription>Manage plates that have automatic gate access.</CardDescription>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {/* Hidden file input for import */}
          <input 
            type="file" 
            accept=".csv" 
            className="hidden" 
            ref={fileInputRef} 
            onChange={handleImport} 
          />
          <Button 
            variant="outline" 
            className="gap-2" 
            onClick={() => fileInputRef.current?.click()}
            disabled={isImporting}
          >
            {isImporting ? <Loader2 className="size-4 animate-spin" /> : <Upload size={16} />}
            Import CSV
          </Button>

          <Button variant="outline" className="gap-2" onClick={handleExport}>
            <Download size={16} />
            Export CSV
          </Button>

          <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger>
              <Button className="gap-2"><Plus size={16} /> Add Plate</Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Add New Authorized Plate</DialogTitle>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid gap-2">
                  <Label htmlFor="plate">License Plate Number *</Label>
                  <Input 
                    id="plate" 
                    placeholder="e.g. AB12 CDE" 
                    value={newPlate} 
                    onChange={e => setNewPlate(e.target.value)} 
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="name">Name / Owner (Optional)</Label>
                  <Input 
                    id="name" 
                    placeholder="e.g. John Doe" 
                    value={newName} 
                    onChange={e => setNewName(e.target.value)} 
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="metadata">Metadata / Pitch (Optional)</Label>
                  <Input 
                    id="metadata" 
                    placeholder="e.g. Pitch 42" 
                    value={newMetadata} 
                    onChange={e => setNewMetadata(e.target.value)} 
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="expiry">Expiry Date (Optional)</Label>
                  <Input 
                    id="expiry" 
                    type="date" 
                    value={newExpiry} 
                    onChange={e => setNewExpiry(e.target.value)} 
                  />
                </div>
                <Button onClick={handleAdd} disabled={isAdding || !newPlate.trim()} className="mt-2">
                  {isAdding ? <Loader2 className="animate-spin size-4" /> : 'Save to Database'}
                </Button>
              </div>
            </DialogContent>
          </Dialog>
        </div>
      </CardHeader>
      <CardContent>
        {/* Search Toolbar */}
        <div className="flex flex-col sm:flex-row items-center gap-2 mb-6">
          <div className="relative flex-1 w-full">
            <Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search..."
              className="pl-9 w-full"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
          <Select value={searchField} onValueChange={(v) => setSearchField(v as SearchField)}>
            <SelectTrigger className="w-full sm:w-[180px]">
              <SelectValue placeholder="Search in..." />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Fields</SelectItem>
              <SelectItem value="plate">Plate</SelectItem>
              <SelectItem value="name">Name</SelectItem>
              <SelectItem value="metadata">Metadata</SelectItem>
            </SelectContent>
          </Select>
        </div>

        {/* Data Table */}
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Plate Number</TableHead>
                <TableHead>Name</TableHead>
                <TableHead>Metadata</TableHead>
                <TableHead>Expiry Date</TableHead>
                <TableHead className="text-right">Actions</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {isLoading ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-center py-6 text-muted-foreground">
                    Loading authorized vehicles...
                  </TableCell>
                </TableRow>
              ) : isError ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-center py-6 text-muted-foreground">
                    <div className="flex items-center justify-center gap-2 text-destructive font-medium mb-2">
                      <AlertCircle className="h-5 w-5" />
                      <span>Failed to load history data.</span>
                    </div>
                    <Button variant="outline" size="sm" onClick={() => refetch()} className="gap-2 mt-1">
                      <RotateCw className="h-4 w-4" />
                      Reload
                    </Button>
                  </TableCell>
                </TableRow>
              ) : !filteredList || filteredList.length === 0 ? (
                <TableRow>
                  <TableCell colSpan={5} className="text-center py-6 text-muted-foreground">
                    No matching vehicles found.
                  </TableCell>
                </TableRow>
              ) : (
                filteredList.map((item) => (
                  <TableRow key={item.plate}>
                    <TableCell className="font-mono font-bold">{item.plate}</TableCell>
                    <TableCell>{item.name || <span className="text-muted-foreground">-</span>}</TableCell>
                    <TableCell>{item.metadata || <span className="text-muted-foreground">-</span>}</TableCell>
                    <TableCell>
                      {item.expiry_date ? new Date(item.expiry_date).toLocaleDateString() : 'Never'}
                    </TableCell>
                    <TableCell className="text-right">
                      <Button 
                        variant="ghost" 
                        size="icon" 
                        onClick={() => handleDelete(item.plate)}
                        disabled={isDeleting}
                      >
                        <Trash2 size={16} className={isDeleting ? "text-muted-foreground" : "text-destructive"} />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))
              )}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  );
}