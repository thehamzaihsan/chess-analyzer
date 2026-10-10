'use client';

import { useEffect, useRef } from 'react';

export default function ChessAnalyzerApp() {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    let cleanup: (() => void) | undefined;
    let cancelled = false;

    import('../ui/app').then(({ mountApp }) => {
      if (cancelled) return;
      cleanup = mountApp(container);
    });

    return () => {
      cancelled = true;
      cleanup?.();
    };
  }, []);

  return <div id="app" ref={containerRef} />;
}
