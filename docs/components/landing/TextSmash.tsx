'use client';

import { useRef, useCallback, type ReactNode } from 'react';
import gsap from 'gsap';
import { SplitText } from 'gsap/SplitText';

gsap.registerPlugin(SplitText);

interface TextSmashProps {
  children: ReactNode;
}

export function TextSmash({ children }: TextSmashProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const isSmashed = useRef(false);

  const handleClick = useCallback(() => {
    if (!containerRef.current || isSmashed.current) return;
    isSmashed.current = true;

    const textElements = containerRef.current.querySelectorAll('span, p, a');
    const allChars: Element[] = [];

    textElements.forEach((el) => {
      if ((el as HTMLElement).textContent?.trim()) {
        const split = SplitText.create(el as HTMLElement, { type: 'chars' });
        allChars.push(...(split.chars as Element[]));
      }
    });

    if (allChars.length === 0) {
      isSmashed.current = false;
      return;
    }

    const tl = gsap.timeline({
      onComplete: () => {
        // Restore after 3s
        gsap.to(containerRef.current, {
          opacity: 0, duration: 0.3, onComplete: () => {
            // Revert all splits
            allChars.forEach((c) => {
              gsap.set(c, { clearProps: 'all' });
            });
            textElements.forEach((el) => {
              const parent = el as HTMLElement;
              // SplitText.revert is called internally
            });
            if (containerRef.current) {
              gsap.set(containerRef.current, { opacity: 1 });
            }
            isSmashed.current = false;
          },
        });
      },
    });

    tl.to(allChars, {
      y: () => gsap.utils.random(200, 600),
      x: () => gsap.utils.random(-200, 200),
      rotation: () => gsap.utils.random(-180, 180),
      opacity: 0,
      duration: () => gsap.utils.random(1, 2.5),
      ease: 'power2.in',
      stagger: { each: 0.02, from: 'random' },
    });
  }, []);

  return (
    <div ref={containerRef} onClick={handleClick} className="cursor-pointer" title="Click me!">
      {children}
    </div>
  );
}
