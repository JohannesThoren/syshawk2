import type { Metadata } from "next";
import localFont from "next/font/local";
import "./globals.css";

// Self-hosted (not next/font/google) so the Docker build never depends on
// reaching fonts.googleapis.com/gstatic.com - one less thing that can fail
// behind a restrictive network or offline build environment.
const sans = localFont({
  variable: "--font-sans",
  src: [
    { path: "./fonts/manrope-400.ttf", weight: "400", style: "normal" },
    { path: "./fonts/manrope-500.ttf", weight: "500", style: "normal" },
    { path: "./fonts/manrope-600.ttf", weight: "600", style: "normal" },
    { path: "./fonts/manrope-700.ttf", weight: "700", style: "normal" },
    { path: "./fonts/manrope-800.ttf", weight: "800", style: "normal" },
  ],
});

const mono = localFont({
  variable: "--font-mono",
  src: [
    { path: "./fonts/ibm-plex-mono-400.ttf", weight: "400", style: "normal" },
    { path: "./fonts/ibm-plex-mono-500.ttf", weight: "500", style: "normal" },
    { path: "./fonts/ibm-plex-mono-600.ttf", weight: "600", style: "normal" },
  ],
});

export const metadata: Metadata = {
  title: "Shawk",
  description: "Server monitoring",
};

export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="en" className={`${sans.variable} ${mono.variable} h-full`}>
      <body className="min-h-full">{children}</body>
    </html>
  );
}
