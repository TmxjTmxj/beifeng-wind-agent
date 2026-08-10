import { useEffect, useState } from "react";
import { BenchmarkPayload, desktopApi } from "../services/desktopApi";
import { useAppState } from "../store/appState";

const emptyBenchmark: BenchmarkPayload = {
  latest_report: null,
  markdown: "",
  scores: {}
};

export function useBenchmark() {
  const { workspaceState } = useAppState();
  const [benchmark, setBenchmark] = useState<BenchmarkPayload>(emptyBenchmark);
  const [loading, setLoading] = useState(true);

  const refresh = async () => {
    setLoading(true);
    const payload = await desktopApi.readLatestBenchmarkReport();
    setBenchmark(payload);
    setLoading(false);
    return payload;
  };

  useEffect(() => {
    refresh();
  }, [workspaceState.current_workspace]);

  return { benchmark, loading, refresh };
}
