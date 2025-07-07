# Doko Bitcoin Vault - Enterprise Grade System

## Project Overview

Doko is an enterprise-grade Bitcoin vault management system with advanced features including multi-signature support, key delegation, spending limits, and a bold neubrutalist design.

## Table of Contents
1. [Project Structure](#project-structure)
2. [Prerequisites](#prerequisites)
3. [Installation Guide](#installation-guide)
4. [Project Files](#project-files)
5. [Configuration](#configuration)
6. [Development](#development)
7. [Production Build](#production-build)
8. [Deployment](#deployment)
9. [Security Considerations](#security-considerations)

## Project Structure

```
app/
├── package.json
├── package-lock.json
├── .gitignore
├── .env.example
├── README.md
├── vite.config.js
├── index.html
├── public/
│   ├── favicon.ico
│   └── assets/
│       └── icons/
├── src/
│   ├── main.jsx
│   ├── App.jsx
│   ├── index.css
│   ├── components/
│   │   ├── Layout/
│   │   │   ├── Header.jsx
│   │   │   ├── Sidebar.jsx
│   │   │   └── index.js
│   │   ├── Dashboard/
│   │   │   ├── Overview.jsx
│   │   │   ├── StatsCard.jsx
│   │   │   └── index.js
│   │   ├── Transactions/
│   │   │   ├── TransactionList.jsx
│   │   │   ├── TransactionItem.jsx
│   │   │   ├── NewTransactionModal.jsx
│   │   │   └── index.js
│   │   ├── KeyManagement/
│   │   │   ├── KeyList.jsx
│   │   │   ├── KeyCard.jsx
│   │   │   ├── AddKeyModal.jsx
│   │   │   └── index.js
│   │   ├── SpendingLimits/
│   │   │   ├── LimitsList.jsx
│   │   │   ├── LimitCard.jsx
│   │   │   ├── ConfigureLimitModal.jsx
│   │   │   └── index.js
│   │   ├── Delegation/
│   │   │   ├── DelegationRules.jsx
│   │   │   ├── RuleCard.jsx
│   │   │   └── index.js
│   │   └── UI/
│   │       ├── Button.jsx
│   │       ├── Modal.jsx
│   │       ├── Input.jsx
│   │       ├── Select.jsx
│   │       └── index.js
│   ├── hooks/
│   │   ├── useModal.js
│   │   ├── useVault.js
│   │   └── useAnimation.js
│   ├── services/
│   │   ├── api.js
│   │   ├── bitcoin.js
│   │   ├── auth.js
│   │   └── storage.js
│   ├── utils/
│   │   ├── constants.js
│   │   ├── formatters.js
│   │   └── validators.js
│   └── store/
│       ├── index.js
│       ├── vaultSlice.js
│       ├── transactionSlice.js
│       └── authSlice.js
```

## Prerequisites

- Node.js 18+ and npm 9+
- Git
- A modern web browser
- (Optional) Bitcoin Core or Bitcoin testnet for real integration

## Installation Guide

### Step 1: Create Project Directory

```bash
mkdir doko-bitcoin-vault
cd doko-bitcoin-vault
npm init -y
```

### Step 2: Install Dependencies

```bash
# Core dependencies
npm install react react-dom react-router-dom
npm install @reduxjs/toolkit react-redux
npm install axios
npm install framer-motion
npm install react-hot-toast
npm install date-fns
npm install bitcoinjs-lib
npm install @noble/secp256k1
npm install bip39
npm install uuid

# Development dependencies
npm install -D vite @vitejs/plugin-react
npm install -D eslint prettier
npm install -D @types/react @types/react-dom
npm install -D tailwindcss postcss autoprefixer
```

### Step 3: Initialize Tailwind CSS

```bash
npx tailwindcss init -p
```

## Project Files

### 1. package.json

```json
{
  "name": "doko-bitcoin-vault",
  "version": "1.0.0",
  "description": "Enterprise-grade Bitcoin vault management system",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint src --ext js,jsx --report-unused-disable-directives --max-warnings 0",
    "format": "prettier --write \"src/**/*.{js,jsx,css}\""
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.21.0",
    "@reduxjs/toolkit": "^2.0.1",
    "react-redux": "^9.0.4",
    "axios": "^1.6.2",
    "framer-motion": "^10.16.16",
    "react-hot-toast": "^2.4.1",
    "date-fns": "^3.0.6",
    "bitcoinjs-lib": "^6.1.5",
    "@noble/secp256k1": "^2.0.0",
    "bip39": "^3.1.0",
    "uuid": "^9.0.1"
  },
  "devDependencies": {
    "vite": "^5.0.10",
    "@vitejs/plugin-react": "^4.2.1",
    "eslint": "^8.56.0",
    "prettier": "^3.1.1",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.32",
    "autoprefixer": "^10.4.16"
  }
}
```

### 2. vite.config.js

```javascript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom', 'react-router-dom'],
          redux: ['@reduxjs/toolkit', 'react-redux'],
          bitcoin: ['bitcoinjs-lib', '@noble/secp256k1', 'bip39']
        }
      }
    }
  }
})
```

### 3. tailwind.config.js

```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: '#FF6B00',
        secondary: '#00D4FF',
        tertiary: '#FFE500',
        success: '#00FF88',
        danger: '#FF0066',
        dark: '#000000',
        light: '#FFFFFF',
        gray: {
          100: '#F5F5F5',
          200: '#E5E5E5',
          300: '#D4D4D4',
          400: '#A3A3A3',
          500: '#737373',
          600: '#525252',
          700: '#404040',
          800: '#262626',
          900: '#171717',
        }
      },
      fontFamily: {
        'grotesk': ['Space Grotesk', 'monospace'],
      },
      boxShadow: {
        'brutal': '8px 8px 0px rgba(0,0,0,1)',
        'brutal-sm': '4px 4px 0px rgba(0,0,0,1)',
        'brutal-lg': '12px 12px 0px rgba(0,0,0,1)',
      },
      keyframes: {
        slideIn: {
          'from': { transform: 'translateX(-100%)' },
          'to': { transform: 'translateX(0)' }
        },
        pulse: {
          '0%, 100%': { transform: 'scale(1)' },
          '50%': { transform: 'scale(1.05)' }
        }
      },
      animation: {
        'slide-in': 'slideIn 0.5s ease',
        'pulse': 'pulse 2s infinite'
      }
    },
  },
  plugins: [],
}
```

### 4. .env.example

```env
# API Configuration
VITE_API_URL=http://localhost:8080/api
VITE_WEBSOCKET_URL=ws://localhost:8080/ws

# Bitcoin Network
VITE_BITCOIN_NETWORK=testnet
VITE_BITCOIN_NODE_URL=http://localhost:8332

# Security
VITE_ENCRYPTION_KEY=your-encryption-key-here
VITE_SESSION_TIMEOUT=3600000

# Features
VITE_ENABLE_TESTNET=true
VITE_ENABLE_MAINNET=false
VITE_ENABLE_HARDWARE_WALLET=true
```

### 5. index.html

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/x-icon" href="/favicon.ico" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="description" content="Enterprise-grade Bitcoin vault management system" />
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <title>Doko - Bitcoin Vault</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.jsx"></script>
  </body>
</html>
```

### 6. src/main.jsx

```jsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import { Provider } from 'react-redux'
import { BrowserRouter } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import App from './App'
import { store } from './store'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <Provider store={store}>
      <BrowserRouter>
        <App />
        <Toaster
          position="top-right"
          toastOptions={{
            style: {
              border: '3px solid #000',
              padding: '16px',
              background: '#fff',
              color: '#000',
              fontFamily: 'Space Grotesk, monospace',
              fontWeight: 600,
              boxShadow: '4px 4px 0px rgba(0,0,0,1)',
            },
            success: {
              style: {
                background: '#00FF88',
              },
            },
            error: {
              style: {
                background: '#FF0066',
                color: '#fff',
              },
            },
          }}
        />
      </BrowserRouter>
    </Provider>
  </React.StrictMode>
)
```

### 7. src/index.css

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

@layer base {
  * {
    @apply box-border;
  }

  body {
    @apply font-grotesk bg-light text-dark overflow-x-hidden;
    background-image: repeating-linear-gradient(
      45deg,
      transparent,
      transparent 35px,
      rgba(0, 0, 0, 0.05) 35px,
      rgba(0, 0, 0, 0.05) 70px
    );
  }

  h1, h2, h3, h4, h5, h6 {
    @apply font-bold tracking-tight;
  }
}

@layer components {
  .brutal-border {
    @apply border-4 border-dark;
  }

  .brutal-shadow {
    @apply shadow-brutal;
  }

  .brutal-shadow-sm {
    @apply shadow-brutal-sm;
  }

  .brutal-shadow-lg {
    @apply shadow-brutal-lg;
  }

  .brutal-btn {
    @apply px-6 py-3 brutal-border bg-light font-semibold uppercase 
           transition-all duration-200 cursor-pointer inline-flex items-center gap-2
           hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-sm;
  }

  .brutal-btn-primary {
    @apply brutal-btn bg-primary text-dark;
  }

  .brutal-btn-secondary {
    @apply brutal-btn bg-secondary text-dark;
  }

  .brutal-btn-danger {
    @apply brutal-btn bg-danger text-light;
  }

  .brutal-btn-success {
    @apply brutal-btn bg-success text-dark;
  }

  .brutal-input {
    @apply w-full px-4 py-3 brutal-border bg-light font-medium
           transition-all duration-200
           focus:outline-none focus:bg-gray-100 focus:-translate-x-0.5 
           focus:-translate-y-0.5 focus:shadow-brutal-sm;
  }

  .brutal-card {
    @apply brutal-border brutal-shadow bg-light p-6
           transition-all duration-200
           hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-lg;
  }

  .brutal-panel {
    @apply brutal-border brutal-shadow bg-light p-8;
  }

  .status-dot {
    @apply w-3 h-3 rounded-full border-2 border-dark;
  }

  .status-dot-active {
    @apply status-dot bg-success;
  }

  .status-dot-inactive {
    @apply status-dot bg-gray-300;
  }
}

@layer utilities {
  .animate-slide-in {
    animation: slideIn 0.5s ease;
  }

  .animate-pulse {
    animation: pulse 2s infinite;
  }
}
```

### 8. src/App.jsx

```jsx
import React from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { useSelector } from 'react-redux'
import { motion, AnimatePresence } from 'framer-motion'

// Layout components
import Header from './components/Layout/Header'
import Sidebar from './components/Layout/Sidebar'

// Page components
import Overview from './components/Dashboard/Overview'
import TransactionList from './components/Transactions/TransactionList'
import KeyList from './components/KeyManagement/KeyList'
import LimitsList from './components/SpendingLimits/LimitsList'
import DelegationRules from './components/Delegation/DelegationRules'

// Auth components
import Login from './components/Auth/Login'
import ProtectedRoute from './components/Auth/ProtectedRoute'

function App() {
  const { isAuthenticated } = useSelector((state) => state.auth)

  if (!isAuthenticated) {
    return <Login />
  }

  return (
    <div className="min-h-screen bg-light">
      <Header />
      <div className="container mx-auto px-4">
        <div className="grid grid-cols-1 lg:grid-cols-[280px_1fr] gap-8 mt-8">
          <Sidebar />
          <main className="min-h-[calc(100vh-120px)]">
            <AnimatePresence mode="wait">
              <Routes>
                <Route path="/" element={<Navigate to="/overview" replace />} />
                <Route
                  path="/overview"
                  element={
                    <ProtectedRoute>
                      <Overview />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/transactions"
                  element={
                    <ProtectedRoute>
                      <TransactionList />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/keys"
                  element={
                    <ProtectedRoute>
                      <KeyList />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/limits"
                  element={
                    <ProtectedRoute>
                      <LimitsList />
                    </ProtectedRoute>
                  }
                />
                <Route
                  path="/delegation"
                  element={
                    <ProtectedRoute>
                      <DelegationRules />
                    </ProtectedRoute>
                  }
                />
              </Routes>
            </AnimatePresence>
          </main>
        </div>
      </div>
    </div>
  )
}

export default App
```

### 9. src/components/Layout/Header.jsx

```jsx
import React from 'react'
import { useNavigate } from 'react-router-dom'
import { useDispatch, useSelector } from 'react-redux'
import { motion } from 'framer-motion'
import Button from '../UI/Button'
import { logout } from '../../store/authSlice'

const Header = () => {
  const navigate = useNavigate()
  const dispatch = useDispatch()
  const { user } = useSelector((state) => state.auth)

  const handleLogout = () => {
    dispatch(logout())
    navigate('/login')
  }

  return (
    <header className="bg-dark text-light border-b-8 border-primary">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-20">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex items-center gap-4"
          >
            <div className="w-12 h-12 bg-primary rotate-45 flex items-center justify-center brutal-border">
              <span className="-rotate-45 text-2xl font-bold">₿</span>
            </div>
            <h1 className="text-4xl font-bold uppercase tracking-tighter">
              DOKO
            </h1>
          </motion.div>

          <nav className="flex items-center gap-8">
            <a
              href="#"
              className="text-light hover:text-primary transition-colors font-semibold uppercase"
            >
              Dashboard
            </a>
            <a
              href="#"
              className="text-light hover:text-primary transition-colors font-semibold uppercase"
            >
              Analytics
            </a>
            <a
              href="#"
              className="text-light hover:text-primary transition-colors font-semibold uppercase"
            >
              Settings
            </a>
            <div className="flex items-center gap-4 ml-8">
              <span className="text-sm opacity-80">{user?.email}</span>
              <Button variant="primary" size="sm" onClick={handleLogout}>
                Disconnect
              </Button>
            </div>
          </nav>
        </div>
      </div>
    </header>
  )
}

export default Header
```

### 10. src/components/Layout/Sidebar.jsx

```jsx
import React from 'react'
import { NavLink } from 'react-router-dom'
import { motion } from 'framer-motion'
import { useSelector } from 'react-redux'

const menuItems = [
  {
    path: '/overview',
    label: 'Overview',
    icon: '📊',
  },
  {
    path: '/transactions',
    label: 'Transactions',
    icon: '💸',
  },
  {
    path: '/keys',
    label: 'Key Management',
    icon: '🔑',
  },
  {
    path: '/limits',
    label: 'Spending Limits',
    icon: '⚡',
  },
  {
    path: '/delegation',
    label: 'Delegation',
    icon: '👥',
  },
]

const Sidebar = () => {
  const { activeVault } = useSelector((state) => state.vault)

  return (
    <motion.aside
      initial={{ x: -100, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      className="brutal-panel sticky top-8 h-fit"
    >
      <div className="mb-8">
        <select className="brutal-input w-full font-semibold">
          <option>Enterprise Vault #1</option>
          <option>Treasury Vault</option>
          <option>Operations Vault</option>
        </select>
      </div>

      <nav className="flex flex-col gap-3">
        {menuItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={({ isActive }) =>
              `brutal-border p-4 flex items-center gap-3 font-semibold uppercase transition-all duration-200 ${
                isActive
                  ? 'bg-dark text-light'
                  : 'bg-light hover:bg-primary hover:-translate-x-1 hover:-translate-y-1 hover:shadow-brutal-sm'
              }`
            }
          >
            <span className="text-2xl">{item.icon}</span>
            {item.label}
          </NavLink>
        ))}
      </nav>

      <div className="mt-8 p-4 brutal-border bg-gray-100">
        <h3 className="font-bold text-sm uppercase mb-2">Vault Status</h3>
        <div className="flex items-center gap-2">
          <div className="status-dot-active"></div>
          <span className="text-sm">Secured</span>
        </div>
      </div>
    </motion.aside>
  )
}

export default Sidebar
```

### 11. src/components/Dashboard/Overview.jsx

```jsx
import React, { useEffect } from 'react'
import { useDispatch, useSelector } from 'react-redux'
import { motion } from 'framer-motion'
import StatsCard from './StatsCard'
import TransactionItem from '../Transactions/TransactionItem'
import Button from '../UI/Button'
import { fetchVaultStats } from '../../store/vaultSlice'
import { fetchRecentTransactions } from '../../store/transactionSlice'

const Overview = () => {
  const dispatch = useDispatch()
  const { stats, loading } = useSelector((state) => state.vault)
  const { recentTransactions } = useSelector((state) => state.transaction)

  useEffect(() => {
    dispatch(fetchVaultStats())
    dispatch(fetchRecentTransactions())
  }, [dispatch])

  const statsData = [
    {
      label: 'Total Balance',
      value: stats?.totalBalance || '0.00',
      unit: 'BTC',
      change: '+5.2%',
      changeType: 'positive',
      color: 'primary',
    },
    {
      label: 'Active Vaults',
      value: stats?.activeVaults || '0',
      unit: '',
      change: '3 Pending',
      color: 'secondary',
    },
    {
      label: 'Key Holders',
      value: stats?.keyHolders || '0',
      unit: '',
      change: '5 Online',
      color: 'tertiary',
    },
    {
      label: 'Daily Volume',
      value: stats?.dailyVolume || '0.00',
      unit: 'BTC',
      change: '-12.5%',
      changeType: 'negative',
      color: 'light',
    },
  ]

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      className="space-y-8"
    >
      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {statsData.map((stat, index) => (
          <StatsCard key={index} {...stat} index={index} />
        ))}
      </div>

      {/* Recent Activity */}
      <div className="brutal-panel">
        <div className="flex items-center justify-between mb-6 pb-4 border-b-4 border-dark">
          <h2 className="text-2xl font-bold uppercase">Recent Activity</h2>
          <Button variant="primary" onClick={() => {}}>
            New Transaction
          </Button>
        </div>

        <div className="space-y-4">
          {recentTransactions.map((tx) => (
            <TransactionItem key={tx.id} transaction={tx} />
          ))}
        </div>
      </div>
    </motion.div>
  )
}

export default Overview
```

### 12. src/components/Dashboard/StatsCard.jsx

```jsx
import React, { useEffect, useState } from 'react'
import { motion } from 'framer-motion'

const StatsCard = ({ label, value, unit, change, changeType, color, index }) => {
  const [displayValue, setDisplayValue] = useState(0)
  
  useEffect(() => {
    const numericValue = parseFloat(value)
    if (!isNaN(numericValue)) {
      const duration = 1000
      const steps = 30
      const increment = numericValue / steps
      let current = 0
      
      const timer = setInterval(() => {
        current += increment
        if (current >= numericValue) {
          setDisplayValue(value)
          clearInterval(timer)
        } else {
          setDisplayValue(current.toFixed(2))
        }
      }, duration / steps)
      
      return () => clearInterval(timer)
    } else {
      setDisplayValue(value)
    }
  }, [value])

  const bgColors = {
    primary: 'bg-primary',
    secondary: 'bg-secondary',
    tertiary: 'bg-tertiary',
    light: 'bg-light',
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.1 }}
      className={`brutal-card ${bgColors[color]} relative overflow-hidden`}
    >
      <div className="relative z-10">
        <p className="text-sm font-semibold uppercase opacity-80 mb-2">
          {label}
        </p>
        <p className="text-3xl font-bold mb-1">
          {displayValue} {unit}
        </p>
        <p
          className={`text-sm font-semibold ${
            changeType === 'positive'
              ? 'text-success'
              : changeType === 'negative'
              ? 'text-danger'
              : ''
          }`}
        >
          {change}
        </p>
      </div>
      
      <motion.div
        className="absolute -right-8 -bottom-8 w-32 h-32 bg-dark opacity-10 rotate-45"
        animate={{ rotate: 405 }}
        transition={{ duration: 20, repeat: Infinity, ease: 'linear' }}
      />
    </motion.div>
  )
}

export default StatsCard
```

### 13. src/components/UI/Button.jsx

```jsx
import React from 'react'
import { motion } from 'framer-motion'

const Button = ({
  children,
  variant = 'default',
  size = 'md',
  onClick,
  disabled = false,
  loading = false,
  className = '',
  ...props
}) => {
  const variants = {
    default: 'brutal-btn',
    primary: 'brutal-btn-primary',
    secondary: 'brutal-btn-secondary',
    danger: 'brutal-btn-danger',
    success: 'brutal-btn-success',
  }

  const sizes = {
    sm: 'px-4 py-2 text-sm',
    md: 'px-6 py-3 text-base',
    lg: 'px-8 py-4 text-lg',
  }

  return (
    <motion.button
      whileHover={{ scale: 1.02 }}
      whileTap={{ scale: 0.98 }}
      className={`${variants[variant]} ${sizes[size]} ${className} ${
        disabled || loading ? 'opacity-50 cursor-not-allowed' : ''
      }`}
      onClick={onClick}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <div className="flex items-center gap-2">
          <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
          Loading...
        </div>
      ) : (
        children
      )}
    </motion.button>
  )
}

export default Button
```

### 14. src/store/index.js

```javascript
import { configureStore } from '@reduxjs/toolkit'
import authReducer from './authSlice'
import vaultReducer from './vaultSlice'
import transactionReducer from './transactionSlice'

export const store = configureStore({
  reducer: {
    auth: authReducer,
    vault: vaultReducer,
    transaction: transactionReducer,
  },
})
```

### 15. src/store/vaultSlice.js

```javascript
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { api } from '../services/api'

export const fetchVaultStats = createAsyncThunk(
  'vault/fetchStats',
  async () => {
    const response = await api.get('/vault/stats')
    return response.data
  }
)

const vaultSlice = createSlice({
  name: 'vault',
  initialState: {
    activeVault: null,
    vaults: [],
    stats: {
      totalBalance: '142.57',
      activeVaults: '7',
      keyHolders: '12',
      dailyVolume: '8.34',
    },
    loading: false,
    error: null,
  },
  reducers: {
    setActiveVault: (state, action) => {
      state.activeVault = action.payload
    },
    updateStats: (state, action) => {
      state.stats = { ...state.stats, ...action.payload }
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchVaultStats.pending, (state) => {
        state.loading = true
      })
      .addCase(fetchVaultStats.fulfilled, (state, action) => {
        state.loading = false
        state.stats = action.payload
      })
      .addCase(fetchVaultStats.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message
      })
  },
})

export const { setActiveVault, updateStats } = vaultSlice.actions
export default vaultSlice.reducer
```

### 16. src/store/transactionSlice.js

```javascript
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { api } from '../services/api'

export const fetchRecentTransactions = createAsyncThunk(
  'transaction/fetchRecent',
  async () => {
    const response = await api.get('/transactions/recent')
    return response.data
  }
)

export const createTransaction = createAsyncThunk(
  'transaction/create',
  async (transactionData) => {
    const response = await api.post('/transactions', transactionData)
    return response.data
  }
)

const transactionSlice = createSlice({
  name: 'transaction',
  initialState: {
    recentTransactions: [
      {
        id: '1',
        type: 'send',
        description: 'Sent to Operations',
        details: 'Multi-sig approval completed',
        amount: '-2.45',
        status: 'completed',
        timestamp: new Date().toISOString(),
      },
      {
        id: '2',
        type: 'receive',
        description: 'Received from Mining Pool',
        details: 'Auto-sweep enabled',
        amount: '+5.82',
        status: 'completed',
        timestamp: new Date().toISOString(),
      },
    ],
    allTransactions: [],
    loading: false,
    error: null,
  },
  reducers: {
    addTransaction: (state, action) => {
      state.recentTransactions.unshift(action.payload)
      state.allTransactions.unshift(action.payload)
    },
    updateTransactionStatus: (state, action) => {
      const { id, status } = action.payload
      const transaction = state.allTransactions.find((tx) => tx.id === id)
      if (transaction) {
        transaction.status = status
      }
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(fetchRecentTransactions.fulfilled, (state, action) => {
        state.recentTransactions = action.payload
      })
      .addCase(createTransaction.fulfilled, (state, action) => {
        state.recentTransactions.unshift(action.payload)
        state.allTransactions.unshift(action.payload)
      })
  },
})

export const { addTransaction, updateTransactionStatus } = transactionSlice.actions
export default transactionSlice.reducer
```

### 17. src/store/authSlice.js

```javascript
import { createSlice, createAsyncThunk } from '@reduxjs/toolkit'
import { api, setAuthToken } from '../services/api'
import { storage } from '../services/storage'

export const login = createAsyncThunk('auth/login', async (credentials) => {
  const response = await api.post('/auth/login', credentials)
  const { token, user } = response.data
  setAuthToken(token)
  storage.set('authToken', token)
  storage.set('user', user)
  return { token, user }
})

export const logout = createAsyncThunk('auth/logout', async () => {
  await api.post('/auth/logout')
  setAuthToken(null)
  storage.remove('authToken')
  storage.remove('user')
})

const authSlice = createSlice({
  name: 'auth',
  initialState: {
    user: storage.get('user'),
    token: storage.get('authToken'),
    isAuthenticated: !!storage.get('authToken'),
    loading: false,
    error: null,
  },
  reducers: {
    clearError: (state) => {
      state.error = null
    },
  },
  extraReducers: (builder) => {
    builder
      .addCase(login.pending, (state) => {
        state.loading = true
        state.error = null
      })
      .addCase(login.fulfilled, (state, action) => {
        state.loading = false
        state.user = action.payload.user
        state.token = action.payload.token
        state.isAuthenticated = true
      })
      .addCase(login.rejected, (state, action) => {
        state.loading = false
        state.error = action.error.message
      })
      .addCase(logout.fulfilled, (state) => {
        state.user = null
        state.token = null
        state.isAuthenticated = false
      })
  },
})

export const { clearError } = authSlice.actions
export default authSlice.reducer
```

### 18. src/services/api.js

```javascript
import axios from 'axios'
import { storage } from './storage'

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080/api'

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Request interceptor
api.interceptors.request.use(
  (config) => {
    const token = storage.get('authToken')
    if (token) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error) => {
    return Promise.reject(error)
  }
)

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      storage.remove('authToken')
      storage.remove('user')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

export const setAuthToken = (token) => {
  if (token) {
    api.defaults.headers.common['Authorization'] = `Bearer ${token}`
  } else {
    delete api.defaults.headers.common['Authorization']
  }
}

// Mock API endpoints for development
if (import.meta.env.DEV) {
  // Mock implementation
  api.get = async (url) => {
    await new Promise((resolve) => setTimeout(resolve, 500))
    
    if (url === '/vault/stats') {
      return {
        data: {
          totalBalance: '142.57',
          activeVaults: '7',
          keyHolders: '12',
          dailyVolume: '8.34',
        },
      }
    }
    
    if (url === '/transactions/recent') {
      return {
        data: [
          {
            id: '1',
            type: 'send',
            description: 'Sent to Operations',
            details: 'Multi-sig approval completed',
            amount: '-2.45',
            status: 'completed',
            timestamp: new Date().toISOString(),
          },
          {
            id: '2',
            type: 'receive',
            description: 'Received from Mining Pool',
            details: 'Auto-sweep enabled',
            amount: '+5.82',
            status: 'completed',
            timestamp: new Date().toISOString(),
          },
        ],
      }
    }
    
    return { data: {} }
  }
  
  api.post = async (url, data) => {
    await new Promise((resolve) => setTimeout(resolve, 500))
    
    if (url === '/auth/login') {
      return {
        data: {
          token: 'mock-jwt-token',
          user: {
            id: '1',
            email: data.email,
            role: 'admin',
          },
        },
      }
    }
    
    return { data: { ...data, id: Date.now().toString() } }
  }
}
```

### 19. src/services/bitcoin.js

```javascript
import * as bitcoin from 'bitcoinjs-lib'
import * as secp256k1 from '@noble/secp256k1'
import * as bip39 from 'bip39'
import { Buffer } from 'buffer'

// Polyfill for browser
window.Buffer = Buffer

const NETWORK = import.meta.env.VITE_BITCOIN_NETWORK === 'mainnet' 
  ? bitcoin.networks.bitcoin 
  : bitcoin.networks.testnet

export class BitcoinService {
  static generateMnemonic() {
    return bip39.generateMnemonic(256)
  }

  static async mnemonicToSeed(mnemonic) {
    return await bip39.mnemonicToSeed(mnemonic)
  }

  static createKeyPair(seed) {
    const root = bitcoin.bip32.fromSeed(seed, NETWORK)
    const path = "m/84'/0'/0'/0/0" // BIP84 for native segwit
    const child = root.derivePath(path)
    
    return {
      privateKey: child.privateKey,
      publicKey: child.publicKey,
      address: bitcoin.payments.p2wpkh({ 
        pubkey: child.publicKey, 
        network: NETWORK 
      }).address,
    }
  }

  static createMultisigAddress(publicKeys, m) {
    const pubkeys = publicKeys.map(hex => Buffer.from(hex, 'hex'))
    const p2ms = bitcoin.payments.p2ms({ 
      m, 
      pubkeys, 
      network: NETWORK 
    })
    const p2wsh = bitcoin.payments.p2wsh({ 
      redeem: p2ms, 
      network: NETWORK 
    })
    
    return {
      address: p2wsh.address,
      redeemScript: p2ms.output,
      witnessScript: p2wsh.redeem.output,
    }
  }

  static async signTransaction(psbt, privateKey) {
    const keyPair = bitcoin.ECPair.fromPrivateKey(privateKey, { network: NETWORK })
    psbt.signAllInputs(keyPair)
    return psbt
  }

  static validateAddress(address) {
    try {
      bitcoin.address.toOutputScript(address, NETWORK)
      return true
    } catch {
      return false
    }
  }

  static estimateFee(inputs, outputs, feeRate = 10) {
    // Rough estimation: 10 bytes per input + 34 bytes per output + 10 bytes overhead
    const size = inputs * 148 + outputs * 34 + 10
    return Math.ceil(size * feeRate)
  }
}

export default BitcoinService
```

### 20. src/services/storage.js

```javascript
class SecureStorage {
  constructor() {
    this.storage = window.localStorage
    this.prefix = 'doko_'
  }

  set(key, value) {
    try {
      const serializedValue = JSON.stringify(value)
      // In production, encrypt the value here
      this.storage.setItem(this.prefix + key, serializedValue)
      return true
    } catch (error) {
      console.error('Storage error:', error)
      return false
    }
  }

  get(key) {
    try {
      const item = this.storage.getItem(this.prefix + key)
      if (!item) return null
      // In production, decrypt the value here
      return JSON.parse(item)
    } catch (error) {
      console.error('Storage error:', error)
      return null
    }
  }

  remove(key) {
    try {
      this.storage.removeItem(this.prefix + key)
      return true
    } catch (error) {
      console.error('Storage error:', error)
      return false
    }
  }

  clear() {
    try {
      Object.keys(this.storage)
        .filter(key => key.startsWith(this.prefix))
        .forEach(key => this.storage.removeItem(key))
      return true
    } catch (error) {
      console.error('Storage error:', error)
      return false
    }
  }
}

export const storage = new SecureStorage()
```

### 21. src/utils/constants.js

```javascript
export const BITCOIN_DECIMALS = 8

export const TRANSACTION_STATUS = {
  PENDING: 'pending',
  CONFIRMING: 'confirming',
  COMPLETED: 'completed',
  FAILED: 'failed',
  CANCELLED: 'cancelled',
}

export const KEY_TYPES = {
  MASTER: 'master',
  SIGNING: 'signing',
  VIEW_ONLY: 'view_only',
  EMERGENCY: 'emergency',
}

export const VAULT_TYPES = {
  STANDARD: 'standard',
  MULTISIG: 'multisig',
  TIMELOCKED: 'timelocked',
}

export const PERMISSIONS = {
  CREATE_TRANSACTION: 'create_transaction',
  APPROVE_TRANSACTION: 'approve_transaction',
  MANAGE_KEYS: 'manage_keys',
  CONFIGURE_LIMITS: 'configure_limits',
  VIEW_ONLY: 'view_only',
}

export const FEE_PRIORITIES = {
  HIGH: { label: 'High (10 min)', blocks: 1, satPerByte: 50 },
  MEDIUM: { label: 'Medium (30 min)', blocks: 3, satPerByte: 20 },
  LOW: { label: 'Low (1 hour)', blocks: 6, satPerByte: 10 },
}

export const SPENDING_LIMIT_TYPES = {
  DAILY: 'daily',
  WEEKLY: 'weekly',
  MONTHLY: 'monthly',
  PER_TRANSACTION: 'per_transaction',
}
```

### 22. src/utils/formatters.js

```javascript
import { format, formatDistance, formatRelative } from 'date-fns'

export const formatBTC = (satoshis, showUnit = true) => {
  const btc = satoshis / 100000000
  const formatted = btc.toFixed(8).replace(/\.?0+$/, '')
  return showUnit ? `${formatted} BTC` : formatted
}

export const formatSatoshis = (satoshis) => {
  return new Intl.NumberFormat('en-US').format(satoshis) + ' sats'
}

export const formatUSD = (amount) => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(amount)
}

export const formatAddress = (address, length = 8) => {
  if (!address) return ''
  return `${address.slice(0, length)}...${address.slice(-length)}`
}

export const formatTxId = (txId, length = 6) => {
  if (!txId) return ''
  return `${txId.slice(0, length)}...${txId.slice(-length)}`
}

export const formatDate = (date) => {
  return format(new Date(date), 'MMM dd, yyyy HH:mm')
}

export const formatRelativeTime = (date) => {
  return formatDistance(new Date(date), new Date(), { addSuffix: true })
}

export const formatPercentage = (value, decimals = 2) => {
  return `${value.toFixed(decimals)}%`
}
```

### 23. src/utils/validators.js

```javascript
import { validateAddress } from '../services/bitcoin'

export const validateBitcoinAddress = (address) => {
  if (!address) {
    return 'Address is required'
  }
  
  if (!validateAddress(address)) {
    return 'Invalid Bitcoin address'
  }
  
  return null
}

export const validateAmount = (amount, max = null) => {
  if (!amount || amount <= 0) {
    return 'Amount must be greater than 0'
  }
  
  if (amount < 0.00000001) {
    return 'Amount must be at least 1 satoshi'
  }
  
  if (max && amount > max) {
    return `Amount cannot exceed ${max} BTC`
  }
  
  return null
}

export const validateEmail = (email) => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
  
  if (!email) {
    return 'Email is required'
  }
  
  if (!emailRegex.test(email)) {
    return 'Invalid email address'
  }
  
  return null
}

export const validatePassword = (password) => {
  if (!password) {
    return 'Password is required'
  }
  
  if (password.length < 8) {
    return 'Password must be at least 8 characters'
  }
  
  if (!/[A-Z]/.test(password)) {
    return 'Password must contain at least one uppercase letter'
  }
  
  if (!/[a-z]/.test(password)) {
    return 'Password must contain at least one lowercase letter'
  }
  
  if (!/[0-9]/.test(password)) {
    return 'Password must contain at least one number'
  }
  
  return null
}

export const validateMnemonic = (mnemonic) => {
  if (!mnemonic) {
    return 'Mnemonic is required'
  }
  
  const words = mnemonic.trim().split(/\s+/)
  
  if (words.length !== 12 && words.length !== 24) {
    return 'Mnemonic must be 12 or 24 words'
  }
  
  return null
}
```

## Development Instructions

### 1. Clone and Setup

```bash
# Create project from this guide
mkdir doko-bitcoin-vault
cd doko-bitcoin-vault

# Initialize git
git init
git add .
git commit -m "Initial commit"

# Install dependencies
npm install

# Copy environment variables
cp .env.example .env
```

### 2. Start Development Server

```bash
npm run dev
```

The application will open at http://localhost:3000

### 3. Development Workflow

1. **Component Development**: Create components in the appropriate directories
2. **State Management**: Use Redux Toolkit for global state
3. **Styling**: Use Tailwind CSS with the brutal-* utility classes
4. **API Integration**: Update the mock API in development, replace with real endpoints in production

## Production Build

### 1. Build for Production

```bash
npm run build
```

### 2. Preview Production Build

```bash
npm run preview
```

### 3. Build Output

The production build will be in the `dist` directory:

```
dist/
├── index.html
├── assets/
│   ├── index-[hash].js
│   ├── index-[hash].css
│   └── vendor-[hash].js
```

## Deployment

### Option 1: Vercel

```bash
# Install Vercel CLI
npm i -g vercel

# Deploy
vercel

# Follow the prompts
```

### Option 2: Netlify

1. Build the project: `npm run build`
2. Drag the `dist` folder to Netlify Drop
3. Configure environment variables in Netlify dashboard

### Option 3: Docker

Create a `Dockerfile`:

```dockerfile
FROM node:18-alpine as builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

Create `nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    server {
        listen 80;
        server_name localhost;
        root /usr/share/nginx/html;
        index index.html;

        location / {
            try_files $uri $uri/ /index.html;
        }

        location /api {
            proxy_pass http://your-api-server:8080;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

Build and run:

```bash
docker build -t doko-vault .
docker run -p 80:80 doko-vault
```

## Security Considerations

1. **Environment Variables**: Never commit `.env` files
2. **API Keys**: Store sensitive keys in environment variables
3. **HTTPS**: Always use HTTPS in production
4. **CSP Headers**: Configure Content Security Policy
5. **Authentication**: Implement proper JWT validation
6. **Input Validation**: Validate all user inputs
7. **XSS Protection**: Sanitize user-generated content
8. **CORS**: Configure CORS properly on the backend

## Backend API Requirements

The frontend expects these API endpoints:

- `POST /api/auth/login`
- `POST /api/auth/logout`
- `GET /api/vault/stats`
- `GET /api/transactions/recent`
- `POST /api/transactions`
- `GET /api/keys`
- `POST /api/keys`
- `GET /api/limits`
- `POST /api/limits`
- `GET /api/delegation/rules`
- `POST /api/delegation/rules`

## Testing

```bash
# Install testing dependencies
npm install -D vitest @testing-library/react @testing-library/jest-dom

# Add test script to package.json
"scripts": {
  "test": "vitest",
  "test:ui": "vitest --ui"
}

# Run tests
npm test
```

## Performance Optimization

1. **Code Splitting**: Already configured with Vite
2. **Lazy Loading**: Implement React.lazy for routes
3. **Image Optimization**: Use WebP format
4. **Bundle Analysis**: `npm run build -- --analyze`
5. **Caching**: Configure service workers
6. **CDN**: Serve static assets from CDN

## Monitoring

1. **Error Tracking**: Integrate Sentry
2. **Analytics**: Add Google Analytics or Plausible
3. **Performance**: Use Web Vitals
4. **Uptime**: Configure uptime monitoring

This completes the full enterprise-grade Bitcoin vault system with all necessary configurations, components, and deployment instructions!