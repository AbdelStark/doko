import React, { createContext, useContext, useState, useEffect } from 'react'

const RoleContext = createContext(null)

export const ROLES = {
  ORACLE: 'oracle',
  ALICE: 'alice',
  BOB: 'bob',
  CHARLIE: 'charlie',
}

export const ROLE_CONFIG = {
  [ROLES.ORACLE]: {
    name: 'Oracle',
    description: 'Create markets and settle outcomes',
    color: 'bg-primary',
    wallet: __ORACLE_RPC_WALLET__,
    permissions: ['create_market', 'settle_market', 'view_all'],
  },
  [ROLES.ALICE]: {
    name: 'Alice',
    description: 'Player - Can bet on markets',
    color: 'bg-secondary',
    wallet: __ALICE_RPC_WALLET__,
    permissions: ['place_bet', 'claim_payout', 'view_markets'],
  },
  [ROLES.BOB]: {
    name: 'Bob',
    description: 'Player - Can bet on markets',
    color: 'bg-tertiary',
    wallet: __BOB_RPC_WALLET__,
    permissions: ['place_bet', 'claim_payout', 'view_markets'],
  },
  [ROLES.CHARLIE]: {
    name: 'Charlie',
    description: 'Player - Can bet on markets',
    color: 'bg-success',
    wallet: __CHARLIE_RPC_WALLET__,
    permissions: ['place_bet', 'claim_payout', 'view_markets'],
  },
}

export function RoleProvider({ children }) {
  const [currentRole, setCurrentRole] = useState(() => {
    const savedRole = localStorage.getItem('markstr_current_role')
    return savedRole && ROLES[savedRole.toUpperCase()] ? savedRole : ROLES.ORACLE
  })

  useEffect(() => {
    localStorage.setItem('markstr_current_role', currentRole)
  }, [currentRole])

  const switchRole = (newRole) => {
    if (ROLES[newRole.toUpperCase()]) {
      setCurrentRole(newRole)
    }
  }

  const getCurrentRoleConfig = () => {
    return ROLE_CONFIG[currentRole]
  }

  const hasPermission = (permission) => {
    const roleConfig = getCurrentRoleConfig()
    return roleConfig?.permissions?.includes(permission) || false
  }

  const isOracle = () => currentRole === ROLES.ORACLE
  const isPlayer = () => [ROLES.ALICE, ROLES.BOB, ROLES.CHARLIE].includes(currentRole)

  const value = {
    currentRole,
    setCurrentRole,
    switchRole,
    getCurrentRoleConfig,
    hasPermission,
    isOracle,
    isPlayer,
    ROLES,
    ROLE_CONFIG,
  }

  return (
    <RoleContext.Provider value={value}>
      {children}
    </RoleContext.Provider>
  )
}

export const useRole = () => {
  const context = useContext(RoleContext)
  if (!context) {
    throw new Error('useRole must be used within a RoleProvider')
  }
  return context
}