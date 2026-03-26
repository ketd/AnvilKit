'use client';

import { useEffect, useRef, useState } from 'react';
import gsap from 'gsap';

export function GameCursor() {
  const cursorRef = useRef<HTMLDivElement>(null);
  const trailRef = useRef<HTMLCanvasElement>(null);
  const posRef = useRef({ x: -100, y: -100 });
  const particles = useRef<Array<{ x: number; y: number; life: number; vx: number; vy: number }>>([]);
  const rafRef = useRef(0);
  const [isTouch, setIsTouch] = useState(true); // default hidden on SSR

  useEffect(() => {
    const touch = 'ontouchstart' in window || navigator.maxTouchPoints > 0;
    setIsTouch(touch);
    if (touch) return;

    const cursor = cursorRef.current;
    const canvas = trailRef.current;
    if (!cursor || !canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    document.documentElement.style.cursor = 'none';
    cursor.style.display = 'block';
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    let prevX = 0, prevY = 0;

    const onMouseMove = (e: MouseEvent) => {
      posRef.current = { x: e.clientX, y: e.clientY };
      gsap.to(cursor, { x: e.clientX, y: e.clientY, duration: 0.15, ease: 'power2.out' });

      const speed = Math.sqrt((e.clientX - prevX) ** 2 + (e.clientY - prevY) ** 2);
      if (speed > 3) {
        const count = Math.min(Math.floor(speed / 8), 4);
        for (let i = 0; i < count; i++) {
          particles.current.push({
            x: e.clientX + (Math.random() - 0.5) * 6,
            y: e.clientY + (Math.random() - 0.5) * 6,
            life: 1,
            vx: (Math.random() - 0.5) * 1.5,
            vy: (Math.random() - 0.5) * 1.5,
          });
        }
      }
      prevX = e.clientX;
      prevY = e.clientY;
    };

    const onMouseDown = () => {
      gsap.to(cursor, { scale: 0.7, duration: 0.1 });
      for (let i = 0; i < 12; i++) {
        const angle = (i / 12) * Math.PI * 2;
        particles.current.push({
          x: posRef.current.x, y: posRef.current.y,
          life: 1,
          vx: Math.cos(angle) * 3, vy: Math.sin(angle) * 3,
        });
      }
    };

    const onMouseUp = () => {
      gsap.to(cursor, { scale: 1, duration: 0.2, ease: 'elastic.out(1, 0.4)' });
    };

    const onResize = () => { canvas.width = window.innerWidth; canvas.height = window.innerHeight; };

    const onOverInteractive = () => gsap.to(cursor, { scale: 1.8, borderColor: '#f94bf5', duration: 0.2 });
    const onLeaveInteractive = () => gsap.to(cursor, { scale: 1, borderColor: '#00fbfb', duration: 0.2 });

    const drawParticles = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      particles.current = particles.current.filter((p) => {
        p.x += p.vx; p.y += p.vy; p.life -= 0.025;
        if (p.life <= 0) return false;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.life * 2, 0, Math.PI * 2);
        ctx.fillStyle = `rgba(0, 251, 251, ${p.life * 0.5})`;
        ctx.fill();
        return true;
      });
      rafRef.current = requestAnimationFrame(drawParticles);
    };

    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mousedown', onMouseDown);
    window.addEventListener('mouseup', onMouseUp);
    window.addEventListener('resize', onResize);

    const interactives = document.querySelectorAll('a, button, [role="button"]');
    interactives.forEach((el) => {
      el.addEventListener('mouseenter', onOverInteractive);
      el.addEventListener('mouseleave', onLeaveInteractive);
    });

    rafRef.current = requestAnimationFrame(drawParticles);

    return () => {
      document.documentElement.style.cursor = '';
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mousedown', onMouseDown);
      window.removeEventListener('mouseup', onMouseUp);
      window.removeEventListener('resize', onResize);
      interactives.forEach((el) => {
        el.removeEventListener('mouseenter', onOverInteractive);
        el.removeEventListener('mouseleave', onLeaveInteractive);
      });
      cancelAnimationFrame(rafRef.current);
    };
  }, [isTouch]);

  // Render nothing on touch devices
  if (isTouch) return null;

  return (
    <>
      <canvas ref={trailRef} className="fixed inset-0 z-[9998] pointer-events-none" />
      <div
        ref={cursorRef}
        className="fixed z-[9999] pointer-events-none hidden"
        style={{
          width: 28, height: 28, marginLeft: -14, marginTop: -14,
          borderRadius: '50%',
          border: '1.5px solid #00fbfb',
          boxShadow: '0 0 12px rgba(0,251,251,0.3), inset 0 0 8px rgba(0,251,251,0.1)',
        }}
      >
        <div style={{ position: 'absolute', top: '50%', left: '50%', width: 3, height: 3, marginLeft: -1.5, marginTop: -1.5, borderRadius: '50%', background: '#00fbfb' }} />
        <div style={{ position: 'absolute', top: -6, left: '50%', width: 1, height: 5, marginLeft: -0.5, background: 'rgba(0,251,251,0.5)' }} />
        <div style={{ position: 'absolute', bottom: -6, left: '50%', width: 1, height: 5, marginLeft: -0.5, background: 'rgba(0,251,251,0.5)' }} />
        <div style={{ position: 'absolute', left: -6, top: '50%', width: 5, height: 1, marginTop: -0.5, background: 'rgba(0,251,251,0.5)' }} />
        <div style={{ position: 'absolute', right: -6, top: '50%', width: 5, height: 1, marginTop: -0.5, background: 'rgba(0,251,251,0.5)' }} />
      </div>
    </>
  );
}
