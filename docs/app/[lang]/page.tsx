import Link from 'next/link';

const t = (lang: string, zh: string, en: string) => lang === 'zh' ? zh : en;

const features = [
  { icon: '🌳', title: 'ECS', titleZh: 'ECS 架构', desc: 'Bevy ECS at the core. Parallel schedules, archetypes, change detection.', descZh: '基于 Bevy ECS，并行调度、原型存储、变更检测，开箱即用。', accent: false },
  { icon: '🎨', title: 'Rendering', titleZh: '渲染', desc: 'PBR, HDR bloom, SSAO, CSM shadows — all via wgpu.', descZh: 'PBR、HDR 泛光、SSAO、级联阴影——全部基于 wgpu。', accent: false },
  { icon: '⚛', title: 'Physics', titleZh: '物理', desc: 'Built-in AABB collisions + optional Rapier3D integration.', descZh: '内置 AABB 碰撞检测，可选集成 Rapier3D。', accent: false },
  { icon: '🔊', title: 'Audio', titleZh: '音频', desc: '3D spatial audio powered by Rodio. WAV, Vorbis, MP3.', descZh: '基于 Rodio 的 3D 空间音频，支持 WAV/Vorbis/MP3。', accent: false },
  { icon: '📦', title: 'Assets', titleZh: '资产', desc: 'glTF loading, hot-reload, procedural mesh generation.', descZh: 'glTF 加载、热重载、程序化网格生成。', accent: true },
  { icon: '🎮', title: 'Input', titleZh: '输入', desc: 'Keyboard, mouse, gamepad — one unified API.', descZh: '键盘、鼠标、手柄——统一的输入抽象。', accent: true },
  { icon: '🖥', title: 'UI', titleZh: 'UI', desc: 'Flexbox layout engine with text rendering and z-order.', descZh: 'Flexbox 布局引擎，支持文字渲染和层级排序。', accent: true },
  { icon: '🔧', title: 'Dev Tools', titleZh: '开发工具', desc: 'Frame profiler, debug console, wireframe renderer.', descZh: '帧分析器、调试控制台、线框渲染器。', accent: true },
];

const deps = ['bevy_ecs', 'wgpu', 'winit', 'glam', 'rodio', 'rapier'];

export default async function HomePage({ params }: { params: Promise<{ lang: string }> }) {
  const { lang } = await params;

  return (
    <div className="landing-bg text-[#e5e2e3] selection:bg-[#00fbfb]/30 selection:text-white">

      {/* ═══ Nav ═══ */}
      <nav className="fixed top-0 z-50 w-full px-4 sm:px-8 py-3 sm:py-4 flex justify-between items-center bg-[#0e0e0f]/90 backdrop-blur-md border-b border-white/5">
        <div className="flex items-center gap-4 sm:gap-8">
          <Link href={`/${lang}`} className="flex items-center gap-2">
            <img alt="AnvilKit" className="h-8 sm:h-10 w-auto" src="/icon.svg" />
            <span className="font-[var(--font-headline)] font-bold text-base sm:text-lg tracking-tight hidden sm:inline">AnvilKit</span>
          </Link>
          <div className="hidden md:flex gap-6">
            {[
              { href: `/${lang}/docs`, label: 'Docs', active: true },
              { href: `/${lang}/docs/getting-started`, label: 'Quick Start', active: false },
              { href: `/${lang}/docs/games/craft`, label: 'Games', active: false },
            ].map((link) => (
              <Link key={link.label} href={link.href}
                className={`font-[var(--font-headline)] tracking-tight font-bold uppercase text-[0.6875rem] transition-colors ${
                  link.active ? 'text-[#00fbfb] border-b border-[#00fbfb] pb-1' : 'text-slate-400 hover:text-[#00fbfb]'
                }`}>
                {link.label}
              </Link>
            ))}
            <a href="https://github.com/ketd/AnvilKit" target="_blank" rel="noopener noreferrer"
              className="font-[var(--font-headline)] tracking-tight font-bold uppercase text-[0.6875rem] text-slate-400 hover:text-[#00fbfb] transition-colors">
              GitHub
            </a>
          </div>
        </div>
        <Link href={`/${lang}/docs/getting-started`}
          className="bg-[#00fbfb] hover:bg-white text-black font-[var(--font-headline)] font-bold uppercase tracking-widest px-4 sm:px-5 py-2 rounded-sm text-[0.625rem] sm:text-[0.6875rem] transition-all">
          {t(lang, '开始使用', 'GET STARTED')}
        </Link>
      </nav>

      {/* ═══ Hero ═══ */}
      <header className="relative min-h-[100svh] flex flex-col justify-center items-center px-4 sm:px-6 pt-20 overflow-hidden">
        <div className="absolute inset-0 z-0 hero-glow opacity-60" />
        <div className="absolute inset-0 z-0 opacity-[0.05]">
          <img alt="" className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] sm:w-[800px] grayscale opacity-50" src="/icon.svg" />
        </div>
        <div className="z-10 text-center max-w-5xl px-2">
          <div className="inline-flex items-center gap-2 px-3 py-1 bg-[#2a2a2b] rounded-full border border-[#00fbfb]/20 mb-6 sm:mb-8">
            <span className="w-2 h-2 rounded-full bg-[#00fbfb] animate-pulse" />
            <span className="text-[0.625rem] sm:text-[0.6875rem] font-bold tracking-[0.15em] sm:tracking-[0.2em] uppercase text-[#00fbfb]">v0.2.0</span>
          </div>
          <h1 className="font-[var(--font-headline)] text-4xl sm:text-5xl md:text-8xl font-black tracking-tighter uppercase mb-4 sm:mb-6 leading-[0.9]">
            {t(lang, '用 Rust 写游戏', 'Build Games')} <br />
            <span className="kinetic-gradient italic">{t(lang, '模块化，高性能。', 'in pure Rust.')}</span>
          </h1>
          <p className="text-base sm:text-xl md:text-2xl text-[#a8b8b9] max-w-2xl mx-auto mb-8 sm:mb-10 font-light leading-relaxed">
            {t(lang,
              'AnvilKit 是一个模块化的 Rust 游戏引擎。把渲染、物理、ECS 这些拆成独立 crate，用多少拿多少，不用的不编译。',
              "A modular Rust game engine. Rendering, physics, ECS — each is an independent crate. Take what you need, skip what you don't."
            )}
          </p>
          <div className="flex flex-col sm:flex-row gap-3 sm:gap-4 justify-center">
            <Link href={`/${lang}/docs/getting-started`}
              className="thermal-gradient text-black font-[var(--font-headline)] font-bold uppercase tracking-widest px-8 sm:px-10 py-3 sm:py-4 rounded-sm hover:scale-[1.02] active:scale-95 transition-all shadow-[0_0_30px_rgba(0,251,251,0.2)] text-sm">
              {t(lang, '5 分钟上手', 'Get Started')}
            </Link>
            <Link href={`/${lang}/docs`}
              className="bg-[#2a2a2b] ghost-border text-white font-[var(--font-headline)] font-bold uppercase tracking-widest px-8 sm:px-10 py-3 sm:py-4 rounded-sm hover:bg-[#3a3a3b] transition-all text-sm">
              {t(lang, '看文档', 'View Docs')}
            </Link>
          </div>
        </div>
        <div className="mt-12 sm:mt-20 font-[var(--font-mono)] text-[9px] sm:text-xs text-[#00fbfb]/40 select-none hidden md:block">
          <pre className="tracking-widest">{`[core] ← [render] ← [app]
  |         |          |
[ecs] ← [physics] ← [input]`}</pre>
        </div>
      </header>

      {/* ═══ 模块化架构 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 bg-[#0e0e0f] border-y border-white/5">
        <div className="max-w-7xl mx-auto grid md:grid-cols-2 gap-10 sm:gap-16 items-center">
          <div>
            <div className="text-[#f94bf5] text-xs font-bold tracking-[0.3em] uppercase mb-2">{t(lang, '核心设计', 'Architecture')}</div>
            <h2 className="font-[var(--font-headline)] text-3xl sm:text-4xl font-black tracking-tighter uppercase mb-6 sm:mb-8 border-l-4 border-[#00fbfb] pl-4 sm:pl-6">
              {t(lang, '不是黑盒，是工具箱', "Not a black box — it's a toolbox")}
            </h2>
            <p className="text-base sm:text-lg text-[#a8b8b9] mb-6 leading-relaxed font-light">
              {t(lang,
                '每个功能都是独立的 crate。想用完整引擎？一行 use anvilkit::prelude::*。只想用渲染？单独引 anvilkit-render。不会拖进来你不需要的东西。',
                "Every feature is a standalone crate. Want the full engine? One line: use anvilkit::prelude::*. Just need rendering? Pull in anvilkit-render alone. Nothing you don't need comes along for the ride."
              )}
            </p>
            <div className="space-y-3 sm:space-y-4">
              <div className="flex items-start gap-3 sm:gap-4 p-4 sm:p-5 bg-[#201f20] rounded-lg border border-white/5 forge-border">
                <span className="text-[#00fbfb] text-xl sm:text-2xl">&#x2699;</span>
                <div>
                  <h4 className="font-[var(--font-headline)] font-bold text-sm uppercase">{t(lang, '独立 Crate', 'Independent Crates')}</h4>
                  <p className="text-xs sm:text-sm text-[#a8b8b9]">{t(lang, '渲染、物理、资产各自独立，按需组合。', 'Render, physics, assets — each stands alone. Compose as needed.')}</p>
                </div>
              </div>
              <div className="flex items-start gap-3 sm:gap-4 p-4 sm:p-5 bg-[#201f20] rounded-lg border border-white/5 forge-border">
                <span className="text-[#f94bf5] text-xl sm:text-2xl">&#x26A1;</span>
                <div>
                  <h4 className="font-[var(--font-headline)] font-bold text-sm uppercase">{t(lang, '编译期零开销', 'Zero Overhead')}</h4>
                  <p className="text-xs sm:text-sm text-[#a8b8b9]">{t(lang, 'Rust 的零成本抽象 + 内存安全，不牺牲性能。', "Rust's zero-cost abstractions + memory safety. No runtime penalty.")}</p>
                </div>
              </div>
            </div>
          </div>
          {/* 架构图 */}
          <div className="relative group">
            <div className="absolute -inset-1 bg-gradient-to-r from-[#00fbfb] to-[#f94bf5] rounded-lg blur opacity-10 group-hover:opacity-20 transition duration-1000" />
            <div className="relative bg-[#131314] p-4 sm:p-8 rounded-lg border border-[#00fbfb]/20 shadow-2xl">
              <div className="flex justify-between items-center mb-4 sm:mb-6">
                <div className="flex gap-2">
                  <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-[#f94bf5]/40" />
                  <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-[#00fbfb]/40" />
                  <div className="w-2.5 h-2.5 sm:w-3 sm:h-3 rounded-full bg-white/20" />
                </div>
                <span className="font-[var(--font-mono)] text-[0.5rem] sm:text-[0.6rem] text-[#3a494a] uppercase tracking-widest">crate_graph</span>
              </div>
              <pre className="font-[var(--font-mono)] text-[0.55rem] sm:text-sm text-[#00fbfb]/70 leading-relaxed overflow-x-auto p-3 sm:p-4 bg-black/40 border border-white/5 whitespace-pre">
{`┌─────────────────────────────┐
│        anvilkit (facade)    │
└──────────────┬──────────────┘
               ▼
┌──────────────┴──────────────┐
│      anvilkit-ecs (bevy)    │
└──────┬──────────────┬───────┘
       ▼              ▼
┌──────┴──────┐ ┌─────┴──────┐
│anvilkit-    │ │anvilkit-   │
│render (wgpu)│ │physics     │
└──────┬──────┘ └─────┬──────┘
       ▼              ▼
┌──────┴──────┐ ┌─────┴──────┐
│anvilkit-    │ │anvilkit-   │
│assets (gltf)│ │input       │
└─────────────┘ └────────────┘`}
              </pre>
            </div>
          </div>
        </div>
      </section>

      {/* ═══ 代码示例 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 bg-[#131314]">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row gap-8 sm:gap-12">
          <div className="md:w-1/3">
            <div className="text-[#00fbfb] text-xs font-bold tracking-[0.3em] uppercase mb-2">{t(lang, '写代码', 'Code')}</div>
            <h2 className="font-[var(--font-headline)] text-3xl sm:text-4xl font-black tracking-tighter uppercase mb-4 sm:mb-6">
              {t(lang, '上手就三行', 'Three lines to start')}
            </h2>
            <p className="text-[#a8b8b9] mb-6 sm:mb-8 leading-relaxed font-light text-sm sm:text-base">
              {t(lang,
                'use 一下，创建 App，注册系统，run。不需要配置文件，不需要代码生成。Rust 编译器就是你的类型检查器。',
                "use the prelude, create an App, register your systems, run. No config files, no codegen. The Rust compiler is your type checker."
              )}
            </p>
            <div className="p-3 sm:p-4 bg-black rounded border-l-2 border-[#00fbfb]">
              <span className="font-[var(--font-mono)] text-[0.6rem] text-[#00fbfb]/50 block mb-1 sm:mb-2">Terminal</span>
              <code className="text-[#00fbfb] font-[var(--font-mono)] text-xs sm:text-sm">$ cargo run --release</code>
            </div>
          </div>
          <div className="md:w-2/3 bg-[#0e0e0f] rounded-lg overflow-hidden border border-[#00fbfb]/20">
            <div className="bg-[#2a2a2b] px-4 sm:px-6 py-2 flex justify-between items-center border-b border-white/5">
              <div className="flex items-center gap-2">
                <span className="text-[#f94bf5] text-sm">{'</>'}</span>
                <span className="font-[var(--font-mono)] text-[0.6rem] text-[#a8b8b9] uppercase tracking-widest">src/main.rs</span>
              </div>
            </div>
            <pre className="p-4 sm:p-8 text-xs sm:text-sm font-[var(--font-mono)] overflow-x-auto text-slate-300">
              <code>{`use anvilkit::prelude::*;

fn main() {
    App::new()
        .add_plugins(RenderPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, game_logic)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        CameraComponent::default(),
        Transform::from_xyz(0.0, 2.0, -5.0),
    ));
}`}</code>
            </pre>
          </div>
        </div>
      </section>

      {/* ═══ 功能一览 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 bg-[#1c1b1c]">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-10 sm:mb-16">
            <div className="text-[#f94bf5] text-xs font-bold tracking-[0.3em] uppercase mb-2">{t(lang, '内置功能', 'Built-in')}</div>
            <h2 className="font-[var(--font-headline)] text-2xl sm:text-3xl font-black tracking-tighter uppercase">
              {t(lang, '开箱即用的 8 大模块', '8 Modules, Ready to Go')}
            </h2>
          </div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-px bg-[#00fbfb]/10 rounded-lg overflow-hidden border border-[#00fbfb]/10">
            {features.map((f) => (
              <div key={f.icon} className="bg-[#131314] p-4 sm:p-8 flex flex-col items-center text-center group hover:bg-[#1c1b1c] transition-colors">
                <span className={`text-2xl sm:text-4xl mb-3 sm:mb-4 ${f.accent ? 'opacity-80' : ''}`}>{f.icon}</span>
                <h3 className="font-[var(--font-headline)] font-bold text-[0.65rem] sm:text-[0.75rem] uppercase mb-1 sm:mb-2 tracking-widest">
                  {t(lang, f.titleZh, f.title)}
                </h3>
                <p className="text-[0.6rem] sm:text-[0.7rem] text-[#a8b8b9] leading-relaxed">
                  {t(lang, f.descZh, f.desc)}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ═══ 技术能力 Bento Grid ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-6 max-w-7xl mx-auto">
        <h2 className="font-[var(--font-headline)] font-bold text-3xl sm:text-5xl mb-10 sm:mb-16 tracking-tighter uppercase italic">
          Technical_<span className="text-[#00fbfb]">{t(lang, '能力', 'Capabilities')}</span>
        </h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 md:grid-rows-2 gap-3 sm:gap-4">
          {/* 大卡片 — ECS */}
          <div className="sm:col-span-2 md:row-span-2 glass-panel p-6 sm:p-10 border border-[#484849]/20 relative group overflow-hidden">
            <div className="absolute top-0 right-0 p-3 sm:p-4 font-[var(--font-mono)] text-[9px] sm:text-[10px] opacity-20">REF_ID: #4492-X</div>
            <h4 className="font-[var(--font-headline)] font-bold text-2xl sm:text-3xl mb-3 sm:mb-4 uppercase tracking-tighter">
              {t(lang, '多线程', 'Multi-Threaded')} <br />
              {t(lang, 'ECS 管线', 'ECS Pipeline')}
            </h4>
            <p className="text-[#a8b8b9] mb-6 sm:mb-8 max-w-sm text-sm">
              {t(lang,
                '系统默认并行跑。调度器自动把工作分到所有 CPU 核心，不用你操心锁和竞争条件。',
                'Systems run in parallel by default. The scheduler spreads work across all CPU cores — no locks, no race conditions to worry about.'
              )}
            </p>
            <img alt="ECS" className="w-full h-32 sm:h-48 object-cover opacity-60 group-hover:opacity-100 transition-opacity rounded" src="/images/ecs-hero.jpg" />
            <div className="mt-4 sm:mt-8 flex gap-3 sm:gap-4 flex-wrap">
              <div className="bg-[#00fbfb]/10 text-[#00fbfb] px-2 sm:px-3 py-1 font-[var(--font-mono)] text-[9px] sm:text-[10px] tracking-widest border border-[#00fbfb]/20">LOCK_FREE</div>
              <div className="bg-[#00fbfb]/10 text-[#00fbfb] px-2 sm:px-3 py-1 font-[var(--font-mono)] text-[9px] sm:text-[10px] tracking-widest border border-[#00fbfb]/20">AUTO_THREADED</div>
            </div>
          </div>
          {/* 中卡片 — WGPU */}
          <div className="sm:col-span-2 glass-panel p-6 sm:p-10 border border-[#484849]/20">
            <div className="flex items-center gap-4 sm:gap-6">
              <div className="flex-1">
                <h4 className="font-[var(--font-headline)] font-bold text-xl sm:text-2xl mb-2 uppercase tracking-tighter">
                  {t(lang, 'wgpu 渲染', 'wgpu Rendering')}
                </h4>
                <p className="text-[#a8b8b9] text-xs sm:text-sm">
                  {t(lang,
                    '一套代码跑 Vulkan、Metal、DX12。不用写平台特定代码，每个平台都是原生性能。',
                    'One codebase targets Vulkan, Metal, and DX12. No platform-specific code — native performance everywhere.'
                  )}
                </p>
              </div>
              <div className="w-16 h-16 sm:w-24 sm:h-24 bg-[#262627] border border-[#484849] flex items-center justify-center rotate-45 shrink-0">
                <span className="text-[#f94bf5] text-2xl sm:text-4xl -rotate-45">🎨</span>
              </div>
            </div>
          </div>
          {/* 小卡片 — Zero Cost */}
          <div className="glass-panel p-4 sm:p-6 border border-[#484849]/20 flex flex-col justify-between">
            <span className="text-[#f94bf5] text-xl sm:text-2xl mb-3 sm:mb-4">⚡</span>
            <div>
              <h5 className="font-[var(--font-headline)] font-bold uppercase text-xs sm:text-sm mb-1">{t(lang, '零成本抽象', 'Zero Cost')}</h5>
              <p className="text-[10px] sm:text-[11px] text-[#a8b8b9]">{t(lang, '抽象在编译时全部内联，运行时零开销。', 'Abstractions inline at compile time. Zero runtime overhead.')}</p>
            </div>
          </div>
          {/* 小卡片 — Memory Safe */}
          <div className="glass-panel p-4 sm:p-6 border border-[#484849]/20 flex flex-col justify-between">
            <span className="text-[#00fbfb] text-xl sm:text-2xl mb-3 sm:mb-4">🛡</span>
            <div>
              <h5 className="font-[var(--font-headline)] font-bold uppercase text-xs sm:text-sm mb-1">{t(lang, '内存安全', 'Memory Safe')}</h5>
              <p className="text-[10px] sm:text-[11px] text-[#a8b8b9]">{t(lang, '没有段错误，没有内存泄漏。编译器帮你兜底。', 'No segfaults, no leaks. The compiler has your back.')}</p>
            </div>
          </div>
        </div>
      </section>

      {/* ═══ 游戏展示 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 bg-[#131314]">
        <div className="max-w-7xl mx-auto">
          <div className="mb-10 sm:mb-16">
            <div className="text-[#00fbfb] text-xs font-bold tracking-[0.3em] uppercase mb-2">{t(lang, '实战项目', 'Showcase')}</div>
            <h2 className="font-[var(--font-headline)] text-2xl sm:text-3xl font-black tracking-tighter uppercase">
              {t(lang, '用 AnvilKit 做的游戏', 'Built with AnvilKit')}
            </h2>
          </div>
          <div className="grid md:grid-cols-2 gap-6 sm:gap-12">
            {/* Craft */}
            <div className="group relative overflow-hidden rounded-lg bg-black border border-white/5 forge-border">
              <div className="aspect-video w-full overflow-hidden">
                <img alt="Craft" className="w-full h-full object-cover grayscale opacity-60 group-hover:grayscale-0 group-hover:opacity-100 transition-all duration-700 scale-105 group-hover:scale-100" src="/images/craft-hero.jpg" />
              </div>
              <div className="absolute inset-0 bg-gradient-to-t from-black via-black/20 to-transparent" />
              <div className="absolute bottom-0 left-0 p-4 sm:p-8 w-full">
                <span className="bg-[#00fbfb] text-black text-[0.55rem] sm:text-[0.6rem] font-bold px-2 py-0.5 sm:py-1 uppercase tracking-tighter mb-3 sm:mb-4 inline-block">CRAFT</span>
                <h3 className="font-[var(--font-headline)] text-lg sm:text-2xl font-bold uppercase text-white mb-1 sm:mb-2">
                  {t(lang, 'Craft — 体素沙盒', 'Craft — Voxel Sandbox')}
                </h3>
                <p className="text-xs sm:text-sm text-[#a8b8b9] mb-4 sm:mb-6 font-light">
                  {t(lang, '程序化地形生成、方块建造破坏、昼夜循环、后处理滤镜。', 'Procedural terrain, block building, day-night cycle, post-processing filters.')}
                </p>
                <div className="bg-black/80 backdrop-blur p-2.5 sm:p-4 rounded border border-[#00fbfb]/20">
                  <code className="font-[var(--font-mono)] text-[0.6rem] sm:text-xs text-[#00fbfb]">cargo run -p craft</code>
                </div>
              </div>
            </div>
            {/* Billiards */}
            <div className="group relative overflow-hidden rounded-lg bg-black border border-white/5 forge-border">
              <div className="aspect-video w-full overflow-hidden">
                <img alt="Billiards" className="w-full h-full object-cover grayscale opacity-60 group-hover:grayscale-0 group-hover:opacity-100 transition-all duration-700 scale-105 group-hover:scale-100" src="/images/billiards-hero.jpg" />
              </div>
              <div className="absolute inset-0 bg-gradient-to-t from-black via-black/20 to-transparent" />
              <div className="absolute bottom-0 left-0 p-4 sm:p-8 w-full">
                <span className="bg-[#f94bf5] text-white text-[0.55rem] sm:text-[0.6rem] font-bold px-2 py-0.5 sm:py-1 uppercase tracking-tighter mb-3 sm:mb-4 inline-block">BILLIARDS</span>
                <h3 className="font-[var(--font-headline)] text-lg sm:text-2xl font-bold uppercase text-white mb-1 sm:mb-2">
                  {t(lang, 'Billiards — 台球模拟', 'Billiards — Pool Sim')}
                </h3>
                <p className="text-xs sm:text-sm text-[#a8b8b9] mb-4 sm:mb-6 font-light">
                  {t(lang, '完整 PBR 渲染、自定义碰撞物理、瞄准射击、计分系统。', 'Full PBR rendering, custom collision physics, aiming mechanics, scoring system.')}
                </p>
                <div className="bg-black/80 backdrop-blur p-2.5 sm:p-4 rounded border border-[#f94bf5]/20">
                  <code className="font-[var(--font-mono)] text-[0.6rem] sm:text-xs text-[#f94bf5]">cargo run -p billiards</code>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ═══ 快速开始 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 bg-[#0e0e0f] border-t border-white/5">
        <div className="max-w-7xl mx-auto grid md:grid-cols-2 gap-10 sm:gap-16">
          <div className="space-y-8 sm:space-y-12">
            <div>
              <div className="text-[#f94bf5] text-xs font-bold tracking-[0.3em] uppercase mb-2">{t(lang, '命令行', 'CLI')}</div>
              <h2 className="font-[var(--font-headline)] text-3xl sm:text-4xl font-black tracking-tighter uppercase mb-4 sm:mb-6">
                Anvil CLI
              </h2>
              <p className="text-[#a8b8b9] leading-relaxed mb-4 sm:mb-6 font-light text-sm sm:text-base">
                {t(lang,
                  '一个命令创建项目，一个命令跑起来。自动处理项目结构和依赖，你只管写游戏逻辑。',
                  'One command to create a project, one command to run it. Handles project structure and dependencies so you can focus on game logic.'
                )}
              </p>
              <div className="bg-black p-4 sm:p-6 rounded-lg border border-[#00fbfb]/20 font-[var(--font-mono)] text-xs sm:text-sm space-y-2">
                <div className="flex gap-3 sm:gap-4"><span className="text-[#00fbfb]/40">$</span><span className="text-white">anvil new my-game</span></div>
                <div className="flex gap-3 sm:gap-4"><span className="text-[#00fbfb]/40">$</span><span className="text-white">cd my-game</span></div>
                <div className="flex gap-3 sm:gap-4"><span className="text-[#00fbfb]/40">$</span><span className="text-[#00fbfb]">anvil run</span></div>
              </div>
            </div>
            <div>
              <h3 className="font-[var(--font-headline)] text-xl sm:text-2xl font-bold uppercase mb-4 sm:mb-6 flex items-center gap-2">
                <span className="text-[#00fbfb]">&#x2713;</span>
                {t(lang, '环境要求', 'Prerequisites')}
              </h3>
              <ul className="space-y-2 sm:space-y-3 text-xs sm:text-sm text-[#a8b8b9] font-light">
                <li className="flex items-center gap-3"><span className="w-1.5 h-1.5 rounded-full bg-[#00fbfb]" />Rust (latest stable)</li>
                <li className="flex items-center gap-3"><span className="w-1.5 h-1.5 rounded-full bg-[#00fbfb]" />Git</li>
                <li className="flex items-center gap-3"><span className="w-1.5 h-1.5 rounded-full bg-[#00fbfb]" />{t(lang, 'C 编译器 (构建原生依赖)', 'C compiler (native deps)')}</li>
              </ul>
            </div>
          </div>
          {/* 步骤 */}
          <div className="bg-[#201f20] p-6 sm:p-10 rounded-lg border border-white/5 relative forge-border">
            <div className="absolute top-0 right-0 p-3 sm:p-4 font-[var(--font-mono)] text-[9px] sm:text-[10px] text-[#00fbfb]/40">v0.2.0</div>
            <h2 className="font-[var(--font-headline)] text-xl sm:text-2xl font-black uppercase mb-8 sm:mb-12 text-center">
              {t(lang, '三步跑起来', '3 Steps to Launch')}
            </h2>
            <div className="space-y-8 sm:space-y-12">
              {[
                { num: '01', title: t(lang, '装 CLI', 'Install'), desc: t(lang, '一行装好脚手架工具。', 'One command to get the scaffolding tool.'), code: 'cargo install anvil-cli' },
                { num: '02', title: t(lang, '建项目', 'Create'), desc: t(lang, '生成项目模板。', 'Generate a project from template.'), code: 'anvil new my_game' },
                { num: '03', title: t(lang, '跑起来', 'Run'), desc: t(lang, '编译并启动。', 'Compile and launch.'), code: 'cargo run' },
              ].map((step) => (
                <div key={step.num} className="flex gap-4 sm:gap-6">
                  <div className="shrink-0 w-10 h-10 sm:w-12 sm:h-12 bg-black border border-[#00fbfb]/30 text-[#00fbfb] flex items-center justify-center font-bold text-lg sm:text-xl">{step.num}</div>
                  <div className="min-w-0">
                    <h4 className="font-bold text-xs sm:text-sm uppercase mb-1 text-white">{step.title}</h4>
                    <p className="text-[0.6rem] sm:text-[0.7rem] text-[#a8b8b9] mb-2 sm:mb-3">{step.desc}</p>
                    <code className="bg-black px-3 sm:px-4 py-1.5 sm:py-2 rounded text-[0.6rem] sm:text-[0.7rem] text-[#00fbfb] block border border-[#00fbfb]/10 font-[var(--font-mono)] overflow-x-auto">{step.code}</code>
                  </div>
                </div>
              ))}
            </div>
            <div className="mt-10 sm:mt-16 flex flex-col items-center">
              <Link href={`/${lang}/docs/getting-started`}
                className="bg-gradient-to-r from-[#00fbfb] to-[#f94bf5] text-black font-black px-8 sm:px-10 py-4 sm:py-5 rounded-sm uppercase tracking-[0.15em] sm:tracking-[0.2em] hover:scale-105 transition-all text-[0.65rem] sm:text-xs">
                {t(lang, '开始写游戏', 'START BUILDING')}
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* ═══ 开源 ═══ */}
      <section className="py-16 sm:py-24 px-4 sm:px-8 border-t border-white/5">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="font-[var(--font-headline)] text-2xl sm:text-3xl font-black tracking-tighter uppercase mb-4 sm:mb-6">
            {t(lang, '完全开源', 'Fully Open Source')}
          </h2>
          <p className="text-[#a8b8b9] mb-8 sm:mb-10 leading-relaxed font-light text-sm sm:text-base">
            {t(lang,
              'MIT / Apache 2.0 双许可。站在这些优秀项目的肩膀上：',
              'Dual-licensed MIT / Apache 2.0. Standing on the shoulders of:'
            )}
          </p>
          <div className="flex flex-wrap justify-center gap-2 sm:gap-3">
            {deps.map((dep) => (
              <span key={dep} className="px-3 sm:px-4 py-1 sm:py-1.5 bg-black rounded text-[0.6rem] sm:text-[0.65rem] font-[var(--font-mono)] text-[#00fbfb]/70 border border-[#00fbfb]/20">{dep}</span>
            ))}
          </div>
        </div>
      </section>

      {/* ═══ Footer ═══ */}
      <footer className="bg-black flex flex-col sm:flex-row justify-between items-center w-full px-6 sm:px-12 py-8 sm:py-12 border-t border-white/5 gap-6">
        <div className="flex flex-col gap-3 items-center sm:items-start">
          <div className="flex items-center gap-2">
            <img alt="AnvilKit" className="h-6 sm:h-8 w-auto" src="/icon.svg" />
            <span className="font-[var(--font-headline)] font-bold text-sm sm:text-base">AnvilKit</span>
          </div>
          <p className="font-[var(--font-headline)] text-[0.55rem] sm:text-[0.65rem] tracking-[0.15em] sm:tracking-[0.2em] uppercase text-slate-500">
            &copy; 2025 AnvilKit
          </p>
        </div>
        <div className="flex gap-6 sm:gap-10">
          <Link href={`/${lang}/docs`} className="font-[var(--font-headline)] text-[0.625rem] sm:text-[0.6875rem] tracking-widest uppercase text-slate-500 hover:text-[#00fbfb] transition-colors">
            {t(lang, '文档', 'DOCS')}
          </Link>
          <a href="https://github.com/ketd/AnvilKit" target="_blank" rel="noopener noreferrer"
            className="font-[var(--font-headline)] text-[0.625rem] sm:text-[0.6875rem] tracking-widest uppercase text-slate-500 hover:text-[#00fbfb] transition-colors">
            GITHUB
          </a>
        </div>
      </footer>
    </div>
  );
}
