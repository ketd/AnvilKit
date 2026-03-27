'use client';

import { useEffect, useRef, useId, useState, useCallback } from 'react';

interface MermaidProps {
  chart: string;
}

export function Mermaid({ chart }: MermaidProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const svgWrapRef = useRef<HTMLDivElement>(null);
  const id = useId().replace(/:/g, '_');

  const [scale, setScale] = useState(1);
  const [translate, setTranslate] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [fullscreen, setFullscreen] = useState(false);

  // Render mermaid SVG
  useEffect(() => {
    if (!svgWrapRef.current) return;
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
          if (!cancelled && svgWrapRef.current) {
            svgWrapRef.current.innerHTML = svg;
            // Remove fixed width/height so SVG scales with container
            const svgEl = svgWrapRef.current.querySelector('svg');
            if (svgEl) {
              svgEl.style.maxWidth = '100%';
              svgEl.style.height = 'auto';
            }
          }
        })
        .catch(console.error);
    });

    return () => { cancelled = true; };
  }, [chart, id]);

  // Mouse wheel zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    setScale((s) => Math.min(Math.max(s * delta, 0.3), 5));
  }, []);

  // Mouse drag pan
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    setDragging(true);
    setDragStart({ x: e.clientX - translate.x, y: e.clientY - translate.y });
  }, [translate]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!dragging) return;
    setTranslate({
      x: e.clientX - dragStart.x,
      y: e.clientY - dragStart.y,
    });
  }, [dragging, dragStart]);

  const handleMouseUp = useCallback(() => {
    setDragging(false);
  }, []);

  // Reset view
  const resetView = useCallback(() => {
    setScale(1);
    setTranslate({ x: 0, y: 0 });
  }, []);

  // Toggle fullscreen
  const toggleFullscreen = useCallback(() => {
    setFullscreen((f) => !f);
    resetView();
  }, [resetView]);

  // ESC to exit fullscreen
  useEffect(() => {
    if (!fullscreen) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setFullscreen(false);
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [fullscreen]);

  return (
    <div
      ref={containerRef}
      className={`
        group relative my-6 rounded-lg border border-fd-border bg-fd-card
        ${fullscreen
          ? 'fixed inset-0 z-50 m-0 rounded-none border-none'
          : 'max-h-[600px]'
        }
      `}
    >
      {/* Toolbar */}
      <div className={`
        absolute top-2 right-2 z-10 flex gap-1
        opacity-0 group-hover:opacity-100 transition-opacity
        ${fullscreen ? 'opacity-100' : ''}
      `}>
        <ToolBtn onClick={() => setScale((s) => Math.min(s * 1.3, 5))} title="Zoom in">+</ToolBtn>
        <ToolBtn onClick={() => setScale((s) => Math.max(s * 0.7, 0.3))} title="Zoom out">-</ToolBtn>
        <ToolBtn onClick={resetView} title="Reset view">1:1</ToolBtn>
        <ToolBtn onClick={toggleFullscreen} title={fullscreen ? 'Exit fullscreen' : 'Fullscreen'}>
          {fullscreen ? '✕' : '⛶'}
        </ToolBtn>
      </div>

      {/* Pan/Zoom viewport */}
      <div
        className={`overflow-hidden ${fullscreen ? 'h-full' : 'max-h-[600px]'} ${dragging ? 'cursor-grabbing' : 'cursor-grab'}`}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <div
          ref={svgWrapRef}
          className="flex justify-center items-center min-h-[200px] p-4 select-none"
          style={{
            transform: `translate(${translate.x}px, ${translate.y}px) scale(${scale})`,
            transformOrigin: 'center center',
            transition: dragging ? 'none' : 'transform 0.15s ease-out',
          }}
        />
      </div>

      {/* Hint */}
      {!fullscreen && (
        <div className="absolute bottom-1 left-2 text-xs text-fd-muted-foreground opacity-0 group-hover:opacity-60 transition-opacity">
          Scroll to zoom · Drag to pan · Click ⛶ for fullscreen
        </div>
      )}
    </div>
  );
}

function ToolBtn({ onClick, title, children }: {
  onClick: () => void;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      className="
        flex h-7 w-7 items-center justify-center rounded
        bg-fd-background/80 backdrop-blur text-fd-foreground text-xs font-mono
        border border-fd-border shadow-sm
        hover:bg-fd-accent hover:text-fd-accent-foreground
        transition-colors
      "
    >
      {children}
    </button>
  );
}
