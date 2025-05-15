import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  transpilePackages: ['shared', 'shared_types'],
};

export default nextConfig;
