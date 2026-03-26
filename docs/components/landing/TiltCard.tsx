'use client';

import { useRef, useEffect, useState, type ReactNode } from 'react';
import gsap from 'gsap';

interface TiltCardProps {
  children: ReactNode;
  className?: string;
  strength?: number;
  glare?: boolean;
  'data-anim'?: string;
}

export function TiltCard({ children, className = '', strength = 10, glare = true, ...rest }: TiltCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const glareRef = useRef<HTMLDivElement>(null);
  const [isMobile, setIsMobile] = useState(false);

  useEffect(() => {
    setIsMobile(window.matchMedia('(max-width: 768px)').matches || 'ontouchstart' in window);
  }, []);

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isMobile) return;
    const card = cardRef.current;
    if (!card) return;
    const rect = card.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width - 0.5;
    const y = (e.clientY - rect.top) / rect.height - 0.5;

    gsap.to(card, { rotateY: x * strength, rotateX: -y * strength, duration: 0.4, ease: 'power2.out' });

    if (glare && glareRef.current) {
      gsap.to(glareRef.current, {
        opacity: 0.15,
        background: `radial-gradient(circle at ${(x + 0.5) * 100}% ${(y + 0.5) * 100}%, rgba(0,251,251,0.3), transparent 60%)`,
        duration: 0.3,
      });
    }
  };

  const handleMouseLeave = () => {
    if (isMobile) return;
    const card = cardRef.current;
    if (!card) return;
    gsap.to(card, { rotateY: 0, rotateX: 0, duration: 0.6, ease: 'elastic.out(1, 0.5)' });
    if (glare && glareRef.current) {
      gsap.to(glareRef.current, { opacity: 0, duration: 0.4 });
    }
  };

  return (
    <div
      ref={cardRef}
      className={className}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={isMobile ? { position: 'relative' } : { perspective: '800px', transformStyle: 'preserve-3d', position: 'relative' }}
      {...rest}
    >
      {children}
      {!isMobile && glare && (
        <div
          ref={glareRef}
          className="absolute inset-0 pointer-events-none rounded-[inherit] z-10"
          style={{ opacity: 0 }}
        />
      )}
    </div>
  );
}
