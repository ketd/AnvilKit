'use client';

import { useEffect, useRef, useState } from 'react';

export function FpsCounter() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fpsRef = useRef(0);
  const [fps, setFps] = useState(60);
  const framesRef = useRef<number[]>([]);
  const [isMobile, setIsMobile] = useState(true); // hidden on SSR

  useEffect(() => {
    const mobile = window.matchMedia('(max-width: 768px)').matches || 'ontouchstart' in window;
    setIsMobile(mobile);
    if (mobile) return;

    let lastTime = performance.now();
    let frameCount = 0;
    let animId = 0;

    const canvas = canvasRef.current;
    const ctx = canvas?.getContext('2d');

    const loop = (now: number) => {
      frameCount++;
      const delta = now - lastTime;

      framesRef.current.push(delta);
      if (framesRef.current.length > 60) framesRef.current.shift();

      if (delta >= 500) {
        const currentFps = Math.round((frameCount / delta) * 1000);
        fpsRef.current = currentFps;
        setFps(currentFps);
        frameCount = 0;
        lastTime = now;
      }

      if (ctx && canvas) {
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        const frames = framesRef.current;
        const barW = canvas.width / 60;
        for (let i = 0; i < frames.length; i++) {
          const ms = frames[i];
          const h = Math.min(ms / 33, 1) * canvas.height;
          const hue = ms < 17 ? 180 : ms < 33 ? 60 : 0;
          ctx.fillStyle = `hsla(${hue}, 100%, 50%, 0.6)`;
          ctx.fillRect(i * barW, canvas.height - h, barW - 0.5, h);
        }
      }

      animId = requestAnimationFrame(loop);
    };

    animId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(animId);
  }, [isMobile]);

  // Don't render on mobile
  if (isMobile) return null;

  const color = fps >= 55 ? '#00fbfb' : fps >= 30 ? '#f0c040' : '#e74c3c';

  return (
    <div
      className="fixed top-16 right-3 z-40 flex flex-col items-end gap-0.5 opacity-40 hover:opacity-90 transition-opacity select-none"
      title="Frame Rate"
    >
      <div className="flex items-baseline gap-1.5 font-[var(--font-mono)] text-[10px] tracking-widest">
        <span style={{ color }} className="font-bold text-sm tabular-nums">{fps}</span>
        <span className="text-[#3a494a] uppercase">FPS</span>
      </div>
      <canvas
        ref={canvasRef}
        width={80}
        height={24}
        className="rounded-sm"
        style={{ background: 'rgba(14,14,15,0.7)', border: '1px solid rgba(0,251,251,0.1)' }}
      />
      <div className="font-[var(--font-mono)] text-[8px] text-[#3a494a] tracking-wider">
        {(1000 / Math.max(fps, 1)).toFixed(1)}ms
      </div>
    </div>
  );
}
