import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';

export const baseOptions: BaseLayoutProps = {
  nav: {
    title: (
      <>
        <img src="/icon.svg" alt="AnvilKit" width={24} height={24} />
        <span style={{ fontWeight: 700 }}>AnvilKit</span>
      </>
    ),
  },
  githubUrl: 'https://github.com/ketd/AnvilKit',
};
