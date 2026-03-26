'use client';

import { useRef, type ReactNode } from 'react';
import Link from 'next/link';
import gsap from 'gsap';

interface MagneticLinkProps {
  href: string;
  className?: string;
  children: ReactNode;
  strength?: number;
}

export function MagneticLink({ href, className, children, strength = 0.3 }: MagneticLinkProps) {
  const ref = useRef<HTMLAnchorElement>(null);

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!ref.current) return;
    const { left, top, width, height } = ref.current.getBoundingClientRect();
    const x = (e.clientX - left - width / 2) * strength;
    const y = (e.clientY - top - height / 2) * strength;
    gsap.to(ref.current, { x, y, duration: 0.3, ease: 'power2.out' });
  };

  const handleMouseLeave = () => {
    if (!ref.current) return;
    gsap.to(ref.current, { x: 0, y: 0, duration: 0.6, ease: 'elastic.out(1, 0.3)' });
  };

  return (
    <Link
      ref={ref}
      href={href}
      className={className}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={{ display: 'inline-block' }}
    >
      {children}
    </Link>
  );
}
