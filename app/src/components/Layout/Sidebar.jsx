import React from 'react'
import { Link, useLocation } from 'react-router-dom'

const navLinks = [
  { to: '/overview', label: 'Overview' },
  { to: '/wallet', label: 'Wallet' },
  { to: '/vaults', label: 'Vaults' },
  { to: '/transactions', label: 'Transactions' },
]

export default function Sidebar() {
  const location = useLocation()
  return (
    <aside className="bg-gray-100 shadow-brutal-sm p-4 flex flex-col gap-4">
      {navLinks.map(link => (
        <Link
          key={link.to}
          to={link.to}
          className={`font-grotesk ${
            location.pathname === link.to ? 'text-primary' : 'text-dark'
          }`}
        >
          {link.label}
        </Link>
      ))}
    </aside>
  )
} 