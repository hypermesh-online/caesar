import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'
import AppWagmiProvider from './providers/WagmiProvider'
import { ToastProvider } from './providers/ToastProvider'
import { ErrorBoundary } from './components/ErrorStates'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <AppWagmiProvider>
        <ToastProvider>
          <App />
        </ToastProvider>
      </AppWagmiProvider>
    </ErrorBoundary>
  </React.StrictMode>,
)