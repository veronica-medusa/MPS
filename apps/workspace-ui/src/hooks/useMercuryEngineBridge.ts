import { useState, useCallback, useRef } from 'react';

export interface WorkspaceEngineClient {
  getStateJson(): string;
  dispatchCommandJson(cmdJson: string): string;
  importDocumentBytes?(fileName: string, bytes: Uint8Array): string;
  exportActiveTabBytes?(targetFormat: string): Uint8Array;
}

/**
 * Connects `MercuryWorkspaceShell.tsx` to `crates/wps-wasm` (`MercuryWasmEngine`).
 * Automatically uses the compiled WebAssembly instance when loaded, with a
 * synchronous in-memory fallback during hot-module reload.
 */
export function useMercuryEngineBridge<TState>(
  wasmEngineInstance: WorkspaceEngineClient | null,
  fallbackInitialState: TState
) {
  const engineRef = useRef<WorkspaceEngineClient | null>(wasmEngineInstance);
  const [state, setState] = useState<TState>(() => {
    if (engineRef.current) {
      try {
        return JSON.parse(engineRef.current.getStateJson()) as TState;
      } catch {
        return fallbackInitialState;
      }
    }
    return fallbackInitialState;
  });

  const dispatchCommand = useCallback((commandPayload: Record<string, unknown>) => {
    if (!engineRef.current) return;
    const updatedJson = engineRef.current.dispatchCommandJson(JSON.stringify(commandPayload));
    setState(JSON.parse(updatedJson) as TState);
  }, []);

  const importFile = useCallback((fileName: string, bytes: Uint8Array) => {
    if (engineRef.current?.importDocumentBytes) {
      const updatedJson = engineRef.current.importDocumentBytes(fileName, bytes);
      setState(JSON.parse(updatedJson) as TState);
    }
  }, []);

  const exportTab = useCallback((targetFormat: string): Uint8Array | null => {
    if (engineRef.current?.exportActiveTabBytes) {
      return engineRef.current.exportActiveTabBytes(targetFormat);
    }
    return null;
  }, []);

  return {
    state,
    dispatchCommand,
    importFile,
    exportTab,
  };
}
