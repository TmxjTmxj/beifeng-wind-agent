import { useEffect, useMemo, useState } from "react";
import { desktopApi, MemoryPayload } from "../services/desktopApi";
import { useAppState } from "../store/appState";

const emptyMemory: MemoryPayload = {
  turbine_profiles: [],
  fault_history: [],
  maintenance_history: [],
  report_history: [],
  timeline: []
};

export function useMemory() {
  const { workspaceState } = useAppState();
  const [memory, setMemory] = useState<MemoryPayload>(emptyMemory);
  const [selectedTurbine, setSelectedTurbine] = useState<string>("all");
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    setLoading(true);
    const payload = await desktopApi.readMemoryPayload();
    setMemory(payload);
    setLoading(false);
    return payload;
  };

  useEffect(() => {
    refresh();
  }, [workspaceState.current_workspace]);

  const turbineIds = useMemo(() => {
    const ids = new Set<string>();
    memory.timeline.forEach((item) => {
      if (item.turbine_id) ids.add(item.turbine_id);
    });
    return Array.from(ids).sort();
  }, [memory.timeline]);

  const timeline = useMemo(() => {
    if (selectedTurbine === "all") return memory.timeline;
    return memory.timeline.filter((item) => item.turbine_id === selectedTurbine);
  }, [memory.timeline, selectedTurbine]);

  return { memory, timeline, turbineIds, selectedTurbine, setSelectedTurbine, loading, refresh };
}
