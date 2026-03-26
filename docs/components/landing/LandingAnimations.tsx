'use client';

import gsap from 'gsap';
import { useGSAP } from '@gsap/react';
import { ScrollTrigger } from 'gsap/ScrollTrigger';
import { SplitText } from 'gsap/SplitText';
import { TextPlugin } from 'gsap/TextPlugin';

gsap.registerPlugin(useGSAP, ScrollTrigger, SplitText, TextPlugin);

/* ── SVG hand-drawn underline paths (6 variants) ── */
const scribblePaths = [
  'M0,8 Q25,2 50,8 T100,8',
  'M0,6 C20,12 40,0 60,8 S100,4 100,8',
  'M0,8 Q30,0 50,10 T100,6',
  'M0,10 C15,2 35,14 55,4 S85,12 100,6',
  'M0,7 Q20,14 50,5 T100,9',
  'M0,9 C30,2 60,14 100,7',
];

export function LandingAnimations() {
  useGSAP(() => {
    const prefersReduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (prefersReduced) {
      gsap.set('[data-anim]', { opacity: 1, clearProps: 'all' });
      return;
    }

    const isMobile = window.matchMedia('(max-width: 768px)').matches;

    /* ════════════════════════════════════════
       EFFECT 1 — Hero Entrance Sequence
    ════════════════════════════════════════ */
    const heroTl = gsap.timeline({ defaults: { ease: 'power3.out' } });

    heroTl
      .to('[data-anim="hero-glow"]', {
        scale: 1, opacity: 0.6, duration: 1.2,
      }, 0)
      .to('[data-anim="hero-badge"]', {
        y: 0, opacity: 1, scale: 1, duration: 0.6,
      }, 0.3);

    // SplitText for title (simplified on mobile)
    const titleEl = document.querySelector('[data-anim="hero-title"]') as HTMLElement;
    if (titleEl) {
      titleEl.style.opacity = '1';
      if (isMobile) {
        // Simple fade-up on mobile — no SplitText DOM overhead
        heroTl.from(titleEl, { y: 40, opacity: 0, duration: 0.7 }, 0.4);
      } else {
        const split = SplitText.create(titleEl, {
          type: 'chars, lines',
          mask: 'lines',
        });
        heroTl.from(split.chars, {
          yPercent: 120,
          stagger: { each: 0.025, from: 'start' },
          duration: 0.8,
          ease: 'power4.out',
        }, 0.4);
      }
    }

    heroTl
      .to('[data-anim="hero-subtitle"]', {
        y: 0, opacity: 1, duration: 0.7,
      }, 0.9)
      .to('[data-anim="hero-buttons"]', {
        y: 0, opacity: 1, duration: 0.6,
      }, 1.1)
      .to('[data-anim="hero-ascii"]', {
        opacity: 1, duration: 0.4,
      }, 1.3);

    // ASCII typewriter effect
    const asciiEl = document.querySelector('[data-anim="hero-ascii"] pre') as HTMLElement;
    if (asciiEl) {
      const originalText = asciiEl.textContent || '';
      asciiEl.textContent = '';
      heroTl.to(asciiEl, {
        text: { value: originalText, speed: 2.5 },
        ease: 'none',
      }, 1.4);
    }

    // Mouse-follow glow (desktop only)
    if (!isMobile) {
      const heroGlow = document.querySelector('[data-anim="hero-glow"]') as HTMLElement;
      const heroSection = document.querySelector('header') as HTMLElement;
      if (heroGlow && heroSection) {
        heroSection.addEventListener('mousemove', (e) => {
          const rect = heroSection.getBoundingClientRect();
          gsap.to(heroGlow, {
            left: e.clientX - rect.left - heroGlow.offsetWidth / 2,
            top: e.clientY - rect.top - heroGlow.offsetHeight / 2,
            duration: 1.2,
            ease: 'power2.out',
          });
        });
      }
    }

    /* ════════════════════════════════════════
       NAV — scroll-responsive background
    ════════════════════════════════════════ */
    const nav = document.querySelector('nav');
    if (nav) {
      ScrollTrigger.create({
        start: 80,
        onUpdate: (self) => {
          if (self.scroll() > 80) {
            gsap.to(nav, { backgroundColor: 'rgba(14,14,15,0.98)', paddingTop: '8px', paddingBottom: '8px', duration: 0.3 });
          } else {
            gsap.to(nav, { backgroundColor: 'rgba(14,14,15,0.9)', paddingTop: '', paddingBottom: '', duration: 0.3 });
          }
        },
      });
    }

    /* ════════════════════════════════════════
       EFFECT 7 — DrawSVG-style Nav Underlines (desktop only)
    ════════════════════════════════════════ */
    if (!isMobile) {
    let scribbleIndex = 0;
    document.querySelectorAll('[data-anim="nav-link"]').forEach((link) => {
      const el = link as HTMLElement;
      el.style.position = 'relative';

      const svgNS = 'http://www.w3.org/2000/svg';
      const svg = document.createElementNS(svgNS, 'svg');
      svg.setAttribute('viewBox', '0 0 100 16');
      svg.setAttribute('preserveAspectRatio', 'none');
      Object.assign(svg.style, {
        position: 'absolute', bottom: '-4px', left: '0', width: '100%', height: '6px',
        overflow: 'visible', pointerEvents: 'none',
      });

      const path = document.createElementNS(svgNS, 'path');
      path.setAttribute('fill', 'none');
      path.setAttribute('stroke', '#00fbfb');
      path.setAttribute('stroke-width', '3');
      path.setAttribute('stroke-linecap', 'round');
      path.setAttribute('d', scribblePaths[0]);
      svg.appendChild(path);
      el.appendChild(svg);

      // Set initial hidden state
      const len = path.getTotalLength();
      gsap.set(path, { strokeDasharray: len, strokeDashoffset: len });

      el.addEventListener('mouseenter', () => {
        const d = scribblePaths[scribbleIndex % scribblePaths.length];
        scribbleIndex++;
        path.setAttribute('d', d);
        const newLen = path.getTotalLength();
        gsap.set(path, { strokeDasharray: newLen });
        gsap.fromTo(path,
          { strokeDashoffset: newLen },
          { strokeDashoffset: 0, duration: 0.4, ease: 'power2.inOut' },
        );
      });

      el.addEventListener('mouseleave', () => {
        const curLen = path.getTotalLength();
        gsap.to(path, { strokeDashoffset: -curLen, duration: 0.3, ease: 'power2.in' });
      });
    });
    } // end !isMobile nav underlines

    /* ════════════════════════════════════════
       Architecture Section — Scroll Entrance
    ════════════════════════════════════════ */
    gsap.from('[data-anim="arch-text"]', {
      x: -60, opacity: 0, duration: 0.8,
      scrollTrigger: { trigger: '[data-anim="arch-text"]', start: 'top 80%' },
    });
    gsap.from('[data-anim="arch-diagram"]', {
      x: 60, opacity: 0, duration: 0.8,
      scrollTrigger: { trigger: '[data-anim="arch-diagram"]', start: 'top 80%' },
    });

    // Architecture cards inside arch-text
    gsap.from('[data-anim="arch-text"] .forge-border', {
      y: 40, opacity: 0, scale: 0.95, duration: 0.6, stagger: 0.15,
      ease: 'back.out(1.4)',
      scrollTrigger: { trigger: '[data-anim="arch-text"]', start: 'top 75%' },
    });

    /* ════════════════════════════════════════
       Code Example Section
    ════════════════════════════════════════ */
    gsap.from('[data-anim="code-left"]', {
      x: -50, opacity: 0, duration: 0.7,
      scrollTrigger: { trigger: '[data-anim="code-left"]', start: 'top 80%' },
    });
    gsap.from('[data-anim="code-right"]', {
      y: 40, opacity: 0, duration: 0.8,
      scrollTrigger: { trigger: '[data-anim="code-right"]', start: 'top 80%' },
    });

    /* ════════════════════════════════════════
       Code Block — Line-by-line highlight (desktop only)
    ════════════════════════════════════════ */
    if (!isMobile) {
    const codeBlock = document.querySelector('[data-anim="code-lines"] code') as HTMLElement;
    if (codeBlock) {
      const text = codeBlock.textContent || '';
      const lines = text.split('\n');
      codeBlock.innerHTML = lines.map((line, i) =>
        `<div class="code-line" style="padding:1px 0;border-left:2px solid transparent;padding-left:8px;transition:border-color 0.3s,background 0.3s">${line || ' '}</div>`
      ).join('');

      const lineEls = codeBlock.querySelectorAll('.code-line');
      // Scroll-triggered sequential highlight
      gsap.to(lineEls, {
        borderLeftColor: '#00fbfb',
        backgroundColor: 'rgba(0,251,251,0.04)',
        stagger: { each: 0.12 },
        duration: 0.3,
        scrollTrigger: {
          trigger: '[data-anim="code-lines"]',
          start: 'top 70%',
        },
      });
      // Then fade back to subtle
      gsap.to(lineEls, {
        borderLeftColor: 'transparent',
        backgroundColor: 'transparent',
        stagger: { each: 0.12 },
        duration: 0.4,
        delay: 0.25,
        scrollTrigger: {
          trigger: '[data-anim="code-lines"]',
          start: 'top 70%',
        },
      });
    }

    } // end !isMobile code highlight

    /* ════════════════════════════════════════
       Module Cards — Hover glow border (desktop only)
    ════════════════════════════════════════ */
    if (!isMobile)
    document.querySelectorAll('[data-anim="module-card"]').forEach((card) => {
      const el = card as HTMLElement;
      el.addEventListener('mouseenter', () => {
        gsap.to(el, {
          boxShadow: '0 0 20px rgba(0,251,251,0.15), inset 0 0 20px rgba(0,251,251,0.05)',
          scale: 1.03,
          duration: 0.3,
          ease: 'power2.out',
        });
      });
      el.addEventListener('mouseleave', () => {
        gsap.to(el, {
          boxShadow: 'none',
          scale: 1,
          duration: 0.4,
          ease: 'power2.out',
        });
      });
    });

    /* ════════════════════════════════════════
       EFFECT 4 — Modules Grid entrance
       (3D cylinder is complex; using enhanced stagger instead for stability)
    ════════════════════════════════════════ */
    const moduleCards = document.querySelectorAll('[data-anim="module-card"]');
    if (moduleCards.length) {
      gsap.from(moduleCards, {
        y: 60, opacity: 0, scale: 0.85, rotation: 5,
        duration: 0.6,
        stagger: {
          each: 0.08,
          grid: [2, 4],
          from: 'center',
        },
        ease: 'back.out(1.7)',
        scrollTrigger: {
          trigger: '[data-anim="modules-grid"]',
          start: 'top 80%',
        },
      });
    }

    /* ════════════════════════════════════════
       EFFECT 5 — Bento Grid Clip-Path Reveal
       (simplified to opacity on mobile for GPU savings)
    ════════════════════════════════════════ */
    if (isMobile) {
      gsap.from('[data-anim="bento-large"], [data-anim="bento-medium"], [data-anim="bento-small"]', {
        y: 30, opacity: 0, duration: 0.6, stagger: 0.12,
        scrollTrigger: { trigger: '[data-anim="bento-large"]', start: 'top 85%' },
      });
    } else {
      gsap.fromTo('[data-anim="bento-large"]',
        { clipPath: 'circle(0% at 50% 50%)' },
        {
          clipPath: 'circle(100% at 50% 50%)', duration: 1.0, ease: 'power3.inOut',
          scrollTrigger: { trigger: '[data-anim="bento-large"]', start: 'top 85%' },
        },
      );

      gsap.fromTo('[data-anim="bento-medium"]',
        { clipPath: 'inset(0 100% 0 0)' },
        {
          clipPath: 'inset(0 0% 0 0)', duration: 0.8, ease: 'power3.inOut',
          scrollTrigger: { trigger: '[data-anim="bento-medium"]', start: 'top 85%' },
        },
      );

      document.querySelectorAll('[data-anim="bento-small"]').forEach((el, i) => {
        gsap.fromTo(el,
          { clipPath: i === 0 ? 'inset(100% 0 0 0)' : 'inset(0 0 100% 0)' },
          {
            clipPath: 'inset(0 0 0 0)', duration: 0.7, ease: 'power3.inOut',
            scrollTrigger: { trigger: el, start: 'top 90%' },
          },
        );
      });
    }

    /* ════════════════════════════════════════
       Game Showcase — Parallax + Entrance
    ════════════════════════════════════════ */
    document.querySelectorAll('[data-anim="game-card"]').forEach((card, i) => {
      gsap.from(card, {
        x: i === 0 ? -80 : 80,
        opacity: 0,
        duration: 0.8,
        ease: 'power3.out',
        scrollTrigger: { trigger: card, start: 'top 85%' },
      });

      // Parallax on inner image
      const img = card.querySelector('img');
      if (img) {
        gsap.to(img, {
          y: -30,
          scrollTrigger: {
            trigger: card,
            start: 'top bottom',
            end: 'bottom top',
            scrub: 1,
          },
        });
      }
    });

    /* ════════════════════════════════════════
       Quick Start — Steps Entrance
    ════════════════════════════════════════ */
    gsap.from('[data-anim="cli-block"]', {
      x: -50, opacity: 0, duration: 0.7,
      scrollTrigger: { trigger: '[data-anim="cli-block"]', start: 'top 80%' },
    });

    gsap.from('[data-anim="step"]', {
      y: 40, opacity: 0, duration: 0.5,
      stagger: 0.15,
      ease: 'back.out(1.4)',
      scrollTrigger: {
        trigger: '[data-anim="steps-panel"]',
        start: 'top 80%',
      },
    });

    /* ════════════════════════════════════════
       EFFECT 9 — Deps Zipper Stagger
    ════════════════════════════════════════ */
    gsap.from('[data-anim="dep-badge"]', {
      y: gsap.utils.wrap([-40, 40]),
      opacity: 0,
      scale: 0.8,
      stagger: 0.08,
      duration: 0.5,
      ease: 'back.out(1.7)',
      scrollTrigger: {
        trigger: '[data-anim="open-source"]',
        start: 'top 80%',
      },
    });

    // Open source heading
    gsap.from('[data-anim="open-source"] h2', {
      y: 30, opacity: 0, duration: 0.7,
      scrollTrigger: { trigger: '[data-anim="open-source"]', start: 'top 80%' },
    });

    /* ════════════════════════════════════════
       Footer Entrance
    ════════════════════════════════════════ */
    gsap.from('[data-anim="footer"]', {
      y: 20, opacity: 0, duration: 0.6,
      scrollTrigger: { trigger: '[data-anim="footer"]', start: 'top 95%' },
    });

  }, []);

  return null;
}
