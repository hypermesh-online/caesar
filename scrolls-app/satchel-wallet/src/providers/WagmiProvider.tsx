import React from 'react';
import { WagmiProvider } from 'wagmi';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { config } from '../utils/wallet';

const queryClient = new QueryClient();

interface AppWagmiProviderProps {
  children: React.ReactNode;
}

const AppWagmiProvider: React.FC<AppWagmiProviderProps> = ({ children }) => {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </WagmiProvider>
  );
};

export default AppWagmiProvider;