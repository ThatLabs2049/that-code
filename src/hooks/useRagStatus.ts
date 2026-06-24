import { useCallback, useEffect, useRef, useState } from "react";

import {
  cancelRagIndex,
  getRagStatus,
  indexWorkspaceRag,
  type IndexProgress,
  type RagStatus,
} from "../lib/rag";

export function useRagStatus(workspacePath: string | null, refreshKey = 0) {
  const [status, setStatus] = useState<RagStatus | null>(null);
  const [indexing, setIndexing] = useState(false);
  const [indexProgress, setIndexProgress] = useState<IndexProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const refreshGenerationRef = useRef(0);

  const refresh = useCallback(async () => {
    const generation = ++refreshGenerationRef.current;
    try {
      const next = await getRagStatus();
      if (generation !== refreshGenerationRef.current) return;
      setStatus(next);
      setError(null);
      if (!indexing) {
        setIndexProgress(null);
      }
    } catch (err) {
      if (generation !== refreshGenerationRef.current) return;
      setStatus(null);
      setError(err instanceof Error ? err.message : String(err));
    }
  }, [indexing]);

  useEffect(() => {
    void refresh();
  }, [refresh, workspacePath, refreshKey]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      const stop = await listen<IndexProgress>("rag-index-progress", (event) => {
        const { filesDone, filesTotal } = event.payload;
        setIndexProgress(event.payload);
        setIndexing(filesTotal === 0 || filesDone < filesTotal);
      });
      if (cancelled) {
        stop();
        return;
      }
      unlisten = stop;
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const reindex = useCallback(async () => {
    setIndexing(true);
    setIndexProgress(null);
    setError(null);
    try {
      await indexWorkspaceRag();
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIndexing(false);
      setIndexProgress(null);
    }
  }, [refresh]);

  const cancelIndex = useCallback(async () => {
    try {
      await cancelRagIndex();
    } catch {
      // Best-effort cancel
    } finally {
      setIndexing(false);
      setIndexProgress(null);
      void refresh();
    }
  }, [refresh]);

  return { status, indexing, indexProgress, error, refresh, reindex, cancelIndex };
}
