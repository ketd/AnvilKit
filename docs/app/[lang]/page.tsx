import Link from 'next/link';

export default async function HomePage({
  params,
}: {
  params: Promise<{ lang: string }>;
}) {
  const { lang } = await params;
  const isZh = lang === 'zh';

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-6 text-center px-4">
      <h1 className="text-5xl font-bold">AnvilKit</h1>
      <p className="text-fd-muted-foreground text-lg max-w-xl">
        {isZh
          ? '模块化游戏基础设施框架 — 为 2D/3D 游戏开发提供可组合的核心工具'
          : 'Modular game infrastructure framework — composable core tools for 2D/3D game development'}
      </p>
      <div className="flex gap-4">
        <Link
          href={`/${lang}/docs`}
          className="rounded-lg bg-fd-primary px-6 py-3 text-fd-primary-foreground font-medium hover:opacity-90 transition-opacity"
        >
          {isZh ? '开始阅读' : 'Get Started'}
        </Link>
        <a
          href="https://github.com/ketd/AnvilKit"
          className="rounded-lg border border-fd-border px-6 py-3 font-medium hover:bg-fd-accent transition-colors"
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub
        </a>
      </div>
    </main>
  );
}
