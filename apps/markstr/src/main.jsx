import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Toaster } from 'react-hot-toast'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <BrowserRouter>
      <App />
      <Toaster
        position="top-right"
        toastOptions={{
          duration: 4000,
          style: {
            border: '2px solid #000',
            borderRadius: '0',
            boxShadow: '4px 4px 0px rgba(0,0,0,1)',
            fontFamily: 'Space Grotesk, monospace',
            fontWeight: '500',
          },
          success: {
            style: {
              background: '#00FF88',
              color: '#000',
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
  </React.StrictMode>
)