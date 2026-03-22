import defaultMdxComponents from 'fumadocs-ui/mdx';

export function getMDXComponents(components?: Record<string, unknown>) {
  return {
    ...defaultMdxComponents,
    ...components,
  };
}
