import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Plus, Trash2, Loader2 } from 'lucide-react';
import { 
  useGetAllowList, 
  useAddAllowList, 
  getGetAllowListQueryKey,
  useDeleteAllowList
} from '@/lib/rust_api/schema';

export default function Allowlist() {
  const queryClient = useQueryClient();
  const { data: list, isLoading } = useGetAllowList();

  const [newPlate, setNewPlate] = useState('');
  const [newExpiry, setNewExpiry] = useState('');
  const [open, setOpen] = useState(false);

  const { mutate: addPlate, isPending: isAdding } = useAddAllowList({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getGetAllowListQueryKey() });
        setNewPlate('');
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

  const handleAdd = () => {
    if (!newPlate.trim()) return;

    addPlate({
      data: {
        plate: newPlate.trim().toUpperCase(),
        expiry_date: newExpiry ? new Date(newExpiry).toISOString() : null,
      },
    });
  };

  const handleDelete = (plate: string) => {
    deletePlate({ plate }); 
  };

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-6">
        <div>
          <CardTitle>Authorized Vehicles</CardTitle>
          <CardDescription>Manage plates that have automatic gate access.</CardDescription>
        </div>
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
                <Label htmlFor="plate">License Plate Number</Label>
                <Input 
                  id="plate" 
                  placeholder="e.g. AB12 CDE" 
                  value={newPlate} 
                  onChange={e => setNewPlate(e.target.value)} 
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
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Plate Number</TableHead>
              <TableHead>Expiry Date</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={3} className="text-center py-6 text-muted-foreground">
                  Loading authorized vehicles...
                </TableCell>
              </TableRow>
            ) : !list || list.length === 0 ? (
              <TableRow>
                <TableCell colSpan={3} className="text-center py-6 text-muted-foreground">
                  No authorized vehicles found.
                </TableCell>
              </TableRow>
            ) : (
              list.map((item) => (
                <TableRow key={item.plate}>
                  <TableCell className="font-mono font-bold">{item.plate}</TableCell>
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
      </CardContent>
    </Card>
  );
}