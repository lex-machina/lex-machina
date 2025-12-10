"use client";

import "./globals.css";

const RootLayout = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <html className="dark">
      <body>{children}</body>
    </html>
  );
};

export default RootLayout;
