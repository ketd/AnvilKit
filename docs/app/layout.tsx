import './global.css';
import { Inter, Space_Grotesk, Fira_Code } from 'next/font/google';
import type { ReactNode } from 'react';
import type { Metadata } from 'next';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });
const spaceGrotesk = Space_Grotesk({ subsets: ['latin'], variable: '--font-headline' });
const firaCode = Fira_Code({ subsets: ['latin'], variable: '--font-mono' });

export const metadata: Metadata = {
  title: {
    template: '%s | AnvilKit',
    default: 'AnvilKit',
  },
  description: 'AnvilKit - Modular Game Infrastructure',
  icons: {
    icon: [
      { url: '/icon.svg', type: 'image/svg+xml' },
      { url: '/favicon-32.png', sizes: '32x32', type: 'image/png' },
      { url: '/favicon-16.png', sizes: '16x16', type: 'image/png' },
    ],
    apple: '/apple-touch-icon.png',
  },
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="zh" suppressHydrationWarning>
      <body className={`${inter.variable} ${spaceGrotesk.variable} ${firaCode.variable} ${inter.className}`}>
        {children}
      </body>
    </html>
  );
}
