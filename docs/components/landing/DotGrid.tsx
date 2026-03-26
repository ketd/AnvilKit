'use client';

import { useRef, useEffect, useCallback, useState } from 'react';

const DOT_SPACING = 32;
const DOT_RADIUS = 0.8;
const GLOW_RADIUS = 150;
const BASE_COLOR = { r: 28, g: 27, b: 28 };     // #1c1b1c
const GLOW_COLOR = { r: 0, g: 251, b: 251 };     // #00fbfb
const BG_COLOR = '#0e0e0f';
const SHOCK_RADIUS = 200;

interface Dot {
  baseX: number;
  baseY: number;
  x: number;
  y: number;
  vx: number;
  vy: number;
}

export function DotGrid() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const dotsRef = useRef<Dot[]>([]);
  const mouseRef = useRef({ x: -9999, y: -9999 });
  const rafRef = useRef<number>(0);
  const [isMobile, setIsMobile] = useState(true); // hidden on SSR

  const initDots = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    canvas.width = canvas.offsetWidth;
    canvas.height = canvas.offsetHeight;

    const dots: Dot[] = [];
    const cols = Math.ceil(canvas.width / DOT_SPACING) + 1;
    const rows = Math.ceil(canvas.height / DOT_SPACING) + 1;

    for (let row = 0; row < rows; row++) {
      for (let col = 0; col < cols; col++) {
        dots.push({
          baseX: col * DOT_SPACING,
          baseY: row * DOT_SPACING,
          x: col * DOT_SPACING,
          y: row * DOT_SPACING,
          vx: 0,
          vy: 0,
        });
      }
    }
    dotsRef.current = dots;
  }, []);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.fillStyle = BG_COLOR;
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    const mx = mouseRef.current.x;
    const my = mouseRef.current.y;

    for (const dot of dotsRef.current) {
      // Spring back to base position
      const dx = dot.baseX - dot.x;
      const dy = dot.baseY - dot.y;
      dot.vx += dx * 0.05;
      dot.vy += dy * 0.05;
      dot.vx *= 0.85; // damping
      dot.vy *= 0.85;
      dot.x += dot.vx;
      dot.y += dot.vy;

      // Distance to mouse
      const distX = dot.x - mx;
      const distY = dot.y - my;
      const dist = Math.sqrt(distX * distX + distY * distY);
      const t = Math.max(0, 1 - dist / GLOW_RADIUS);

      // Interpolate color
      const r = Math.round(BASE_COLOR.r + (GLOW_COLOR.r - BASE_COLOR.r) * t);
      const g = Math.round(BASE_COLOR.g + (GLOW_COLOR.g - BASE_COLOR.g) * t);
      const b = Math.round(BASE_COLOR.b + (GLOW_COLOR.b - BASE_COLOR.b) * t);

      ctx.beginPath();
      ctx.arc(dot.x, dot.y, DOT_RADIUS + t * 1.5, 0, Math.PI * 2);
      ctx.fillStyle = `rgb(${r},${g},${b})`;
      ctx.fill();

      // Extra glow ring for very close dots
      if (t > 0.5) {
        ctx.beginPath();
        ctx.arc(dot.x, dot.y, DOT_RADIUS + t * 4, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(0,251,251,${t * 0.15})`;
        ctx.fill();
      }
    }

    rafRef.current = requestAnimationFrame(draw);
  }, []);

  // Shockwave on click (via window, since canvas has pointer-events:none)
  const handleClick = useCallback((e: MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;

    for (const dot of dotsRef.current) {
      const dx = dot.x - cx;
      const dy = dot.y - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);
      if (dist < SHOCK_RADIUS && dist > 0) {
        const force = (1 - dist / SHOCK_RADIUS) * 8;
        dot.vx += (dx / dist) * force;
        dot.vy += (dy / dist) * force;
      }
    }
  }, []);

  useEffect(() => {
    const mobile = window.matchMedia('(max-width: 768px)').matches || 'ontouchstart' in window;
    setIsMobile(mobile);
    if (mobile) return;

    initDots();
    rafRef.current = requestAnimationFrame(draw);

    const handleResize = () => {
      initDots();
    };
    const handleMouseMove = (e: MouseEvent) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      mouseRef.current = { x: e.clientX - rect.left, y: e.clientY - rect.top };

      // Push nearby dots on fast movement (approximation)
      for (const dot of dotsRef.current) {
        const dx = dot.x - mouseRef.current.x;
        const dy = dot.y - mouseRef.current.y;
        const dist = Math.sqrt(dx * dx + dy * dy);
        if (dist < 60 && dist > 0) {
          const push = (1 - dist / 60) * 2;
          dot.vx += (dx / dist) * push;
          dot.vy += (dy / dist) * push;
        }
      }
    };

    window.addEventListener('resize', handleResize);
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('click', handleClick);

    return () => {
      cancelAnimationFrame(rafRef.current);
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('click', handleClick);
    };
  }, [initDots, draw, handleClick]);

  if (isMobile) return null;

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 w-full h-full z-0"
      style={{ background: BG_COLOR, pointerEvents: 'none' }}
    />
  );
}
