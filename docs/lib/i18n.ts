import type { I18nConfig } from 'fumadocs-core/i18n';
import { defineI18nUI } from 'fumadocs-ui/i18n';

export const i18n: I18nConfig = {
  defaultLanguage: 'zh',
  languages: ['zh', 'en'],
  parser: 'dir',
};

export const i18nUI = defineI18nUI(i18n, {
  zh: {
    displayName: '中文',
    search: '搜索文档',
    searchNoResult: '没有找到结果',
    toc: '目录',
    lastUpdate: '最后更新',
    chooseLanguage: '选择语言',
    nextPage: '下一页',
    previousPage: '上一页',
    chooseTheme: '选择主题',
  },
  en: {
    displayName: 'English',
  },
});
