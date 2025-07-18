import React from 'react'

export default function Button({ children, onClick, type = 'button' }) {
  return (
    <button
      type={type}
      onClick={onClick}
      className="bg-primary text-light px-4 py-2 shadow-brutal hover:translate-x-1 hover:translate-y-1 transition-transform font-grotesk"
    >
      {children}
    </button>
  )
} 