'use client';

import { useRef, useEffect } from 'react';
import gsap from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';
import { TextPlugin } from 'gsap/TextPlugin';

gsap.registerPlugin(ScrollTrigger, TextPlugin);

interface TypewriterTerminalProps {
  lang: string;
}

const commands = [
  { prompt: '$', text: 'anvil new my-game', color: 'text-white' },
  { prompt: '$', text: 'cd my-game', color: 'text-white' },
  { prompt: '$', text: 'anvil run', color: 'text-[#00fbfb]' },
];

export function TypewriterTerminal({ lang }: TypewriterTerminalProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const hasPlayed = useRef(false);

  useEffect(() => {
    if (!containerRef.current) return;

    const lines = containerRef.current.querySelectorAll<HTMLElement>('.term-line');
    const textSpans = containerRef.current.querySelectorAll<HTMLElement>('.term-text');

    // Hide lines initially
    lines.forEach((line) => { line.style.opacity = '0'; });

    const tl = gsap.timeline({
      paused: true,
      scrollTrigger: {
        trigger: containerRef.current,
        start: 'top 80%',
        once: true,
        onEnter: () => {
          if (!hasPlayed.current) {
            hasPlayed.current = true;
            tl.play();
          }
        },
      },
    });

    commands.forEach((cmd, i) => {
      const line = lines[i];
      const textSpan = textSpans[i];
      if (!line || !textSpan) return;

      tl.set(line, { opacity: 1 })
        .set(textSpan, { text: '' })
        .to(textSpan, {
          text: { value: cmd.text },
          duration: cmd.text.length * 0.04,
          ease: 'none',
        })
        .to({}, { duration: 0.3 }); // pause between lines
    });

    return () => {
      tl.kill();
    };
  }, []);

  return (
    <div
      ref={containerRef}
      className="bg-black p-4 sm:p-6 rounded-lg border border-[#00fbfb]/20 font-[var(--font-mono)] text-xs sm:text-sm space-y-2"
    >
      {commands.map((cmd, i) => (
        <div key={i} className="term-line flex gap-3 sm:gap-4">
          <span className="text-[#00fbfb]/40">{cmd.prompt}</span>
          <span className={`term-text ${cmd.color}`}>{cmd.text}</span>
        </div>
      ))}
      <span className="cursor-blink" />
    </div>
  );
}
