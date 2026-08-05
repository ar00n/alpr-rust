import { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { 
  useGetUsers, useCreateUser, useDeleteUser, useChangePassword, getGetUsersQueryKey 
} from '@/lib/rust_api/schema';

export default function UserManagement() {
  const queryClient = useQueryClient();
  const { data: usersData, isLoading: isLoadingUsers } = useGetUsers();
  const createUserMutation = useCreateUser();
  const deleteUserMutation = useDeleteUser();
  const changePasswordMutation = useChangePassword();

  const [newUser, setNewUser] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [newRole, setNewRole] = useState('User');

  const handleAddUser = () => {
    if (!newUser || !newPassword) return;

    createUserMutation.mutate(
      { 
        data: { username: newUser, password: newPassword, is_admin: newRole === 'Admin' } 
      },
      {
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: getGetUsersQueryKey() });
          setNewUser('');
          setNewPassword('');
          setNewRole('User');
        },
      }
    );
  };

  const handleDeleteUser = (id: number) => {
    if (window.confirm("Are you sure you want to delete this user?")) {
      deleteUserMutation.mutate(
        { id },
        { onSuccess: () => queryClient.invalidateQueries({ queryKey: getGetUsersQueryKey() }) }
      );
    }
  };

  const handleChangePassword = (id: number) => {
    const updatedPassword = window.prompt("Enter the new password:");
    if (!updatedPassword) return;

    changePasswordMutation.mutate(
      { id, data: { new_password: updatedPassword } },
      {
        onSuccess: () => alert("Password updated successfully."),
        onError: (err) => alert(`Failed to update password: ${err}`)
      }
    );
  };

  return (
    <Card className="w-full h-fit">
      <CardHeader>
        <CardTitle>User Access</CardTitle>
        <CardDescription>Manage dashboard login accounts.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        
        {/* Add New User Form */}
        <div className="space-y-4 border rounded-lg p-4 bg-slate-50">
          <h4 className="text-sm font-semibold">Add New User</h4>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            <div className="space-y-2">
              <Label>Username</Label>
              <Input value={newUser} onChange={e => setNewUser(e.target.value)} placeholder="e.g. jdoe" />
            </div>
            <div className="space-y-2">
              <Label>Password</Label>
              <Input type="password" value={newPassword} onChange={e => setNewPassword(e.target.value)} placeholder="***" />
            </div>
            <div className="space-y-2">
              <Label>Role</Label>
              <Select value={newRole} onValueChange={(val) => setNewRole(val ?? '')}>
                <SelectTrigger>
                  <SelectValue placeholder="Role" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="Admin">Admin</SelectItem>
                  <SelectItem value="User">User</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <Button 
            onClick={handleAddUser} 
            disabled={createUserMutation.isPending || !newUser || !newPassword}
            className="w-full"
          >
            {createUserMutation.isPending ? 'Creating...' : 'Create User'}
          </Button>
        </div>

        {/* Active Users List */}
        <div>
          <h4 className="text-sm font-semibold mb-3">Active Users</h4>
          <div className="space-y-2">
            {isLoadingUsers ? (
              <p className="text-sm text-muted-foreground">Loading users...</p>
            ) : (
              usersData?.map((u: any) => (
                <div key={u.id} className="flex justify-between items-center p-3 border rounded bg-white">
                  <div className="flex items-center gap-3">
                    <span className="font-medium">{u.username}</span>
                    <span className={`text-xs px-2 py-1 rounded ${u.is_admin ? 'bg-blue-100 text-blue-800' : 'bg-slate-100'}`}>
                      {u.is_admin ? 'Admin' : 'User'}
                    </span>
                  </div>
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline" onClick={() => handleChangePassword(u.id)}>
                      Change Pwd
                    </Button>
                    <Button size="sm" variant="destructive" onClick={() => handleDeleteUser(u.id)}>
                      Delete
                    </Button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}