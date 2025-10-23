/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_PLAID_CLIENT_ID: string
  readonly VITE_PLAID_SECRET: string
  readonly VITE_PLAID_ENVIRONMENT: 'sandbox' | 'development' | 'production'
  readonly VITE_ALCHEMY_API_KEY: string
  readonly VITE_INFURA_PROJECT_ID: string
  readonly VITE_ETHERSCAN_API_KEY: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}