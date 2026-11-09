import { createRootRoute, HeadContent, Outlet, Scripts } from '@tanstack/react-router';
import stylesheet from '../../../../.tanstack/wfhub-tailwind.css?url';
import type { ReactNode } from 'react';

export const Route = createRootRoute({
  head: () => ({
    meta: [
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { name: 'description', content: 'Track live activities, browse the item catalogue, and check market orders with WFHub.' },
      { name: 'theme-color', content: '#06121a' },
    ],
    links: [
      { rel: 'icon', href: '/assets/icon.png' },
      { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
      { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossOrigin: 'anonymous' },
      { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap' },
      { rel: 'stylesheet', href: stylesheet },
    ],
  }),
  shellComponent: Shell,
  component: () => <Outlet />,
});

function Shell({ children }: { children: ReactNode }) {
  return <html lang="en" className="scroll-smooth [color-scheme:dark] motion-reduce:scroll-auto"><head><HeadContent /></head>
    <body className="flex min-h-screen flex-col bg-[#06121a] font-[Inter,system-ui,-apple-system,sans-serif] text-base leading-[1.6] text-[#eaf1f4] antialiased [&_a:focus-visible]:rounded-md [&_a:focus-visible]:outline-2 [&_a:focus-visible]:outline-offset-[3px] [&_a:focus-visible]:outline-[#f0aa42]">{children}<Scripts /></body></html>;
}
