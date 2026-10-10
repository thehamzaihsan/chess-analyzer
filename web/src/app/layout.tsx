import type { Metadata } from 'next';
import { Source_Serif_4, Inter } from 'next/font/google';
import './globals.css';

const spectral = Source_Serif_4({
  subsets: ['latin'],
  weight: ['400', '500', '600', '700'],
  variable: '--font-spectral',
  display: 'swap',
});

const publicSans = Inter({
  subsets: ['latin'],
  weight: ['400', '500', '600'],
  variable: '--font-public-sans',
  display: 'swap',
});

export const metadata: Metadata = {
  title: 'Chess history analyzer',
  description: 'A full, private analysis of your Chess.com history, computed entirely in your browser.',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${spectral.variable} ${publicSans.variable}`}>
      <body>{children}</body>
    </html>
  );
}
