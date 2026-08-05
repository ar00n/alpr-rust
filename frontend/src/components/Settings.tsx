import { useAuthStore } from "../store/useAuthStore";
import EngineSettings from "./settings/EngineSettings";
import RtspSettings from "./settings/RtspSettings";
import StorageSettings from "./settings/StorageSettings";
import UserManagement from "./settings/UserManagement";

export default function Settings() {
  const { isAdmin } = useAuthStore();

  if (!isAdmin) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-lg text-muted-foreground">You do not have permission to access this page.</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-6 w-full min-w-full">
      {/* Left Column Stack */}
      <div className="space-y-6">
        <RtspSettings />
        <StorageSettings />
        <EngineSettings />
      </div>

      {/* Right Column Stack */}
      <div className="space-y-6">
        <UserManagement />
      </div>
    </div>
  );
}