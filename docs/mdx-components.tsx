import defaultMdxComponents from 'fumadocs-ui/mdx';
import { Mermaid } from '@/components/Mermaid';

export function getMDXComponents(components?: Record<string, unknown>) {
  return {
    ...defaultMdxComponents,
    Mermaid,
    ...components,
  };
}
