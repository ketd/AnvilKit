'use client';

import { useEffect, useRef, useId } from 'react';

interface MermaidProps {
  chart: string;
}

export function Mermaid({ chart }: MermaidProps) {
  const ref = useRef<HTMLDivElement>(null);
  const id = useId().replace(/:/g, '_');

  useEffect(() => {
    if (!ref.current) return;

    let cancelled = false;

    import('mermaid').then((m) => {
      if (cancelled) return;
      const mermaid = m.default;
      mermaid.initialize({
        startOnLoad: false,
        theme: 'neutral',
        fontFamily: 'ui-monospace, monospace',
        securityLevel: 'loose',
      });

      mermaid
        .render(`mermaid_${id}`, chart)
        .then(({ svg }) => {
          if (!cancelled && ref.current) {
            ref.current.innerHTML = svg;
          }
        })
        .catch(console.error);
    });

    return () => {
      cancelled = true;
    };
  }, [chart, id]);

  return (
    <div
      ref={ref}
      className="my-6 flex justify-center [&_svg]:max-w-full"
    />
  );
}
