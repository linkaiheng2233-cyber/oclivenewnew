import { defineStore } from 'pinia'

const STORAGE_KEY = 'oclive-engagement-v1'

interface RoleEngagement {
  turnCount: number
  immersiveHintDismissed: boolean
  identitySurpriseSeen: boolean
}

interface EngagementPersisted {
  byRole: Record<string, RoleEngagement>
}

function loadPersisted(): EngagementPersisted {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw)
      return { byRole: {} }
    const parsed = JSON.parse(raw) as EngagementPersisted
    return parsed?.byRole ? parsed : { byRole: {} }
  }
  catch {
    return { byRole: {} }
  }
}

function savePersisted(data: EngagementPersisted): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
}

function defaultRole(): RoleEngagement {
  return {
    turnCount: 0,
    immersiveHintDismissed: false,
    identitySurpriseSeen: false,
  }
}

export const useEngagementStore = defineStore('engagement', {
  state: () => ({
    byRole: loadPersisted().byRole,
  }),

  actions: {
    roleState(roleId: string): RoleEngagement {
      return this.byRole[roleId] ?? defaultRole()
    },

    persist(): void {
      savePersisted({ byRole: this.byRole })
    },

    recordSuccessfulTurn(roleId: string): number {
      const cur = this.roleState(roleId)
      const next = { ...cur, turnCount: cur.turnCount + 1 }
      this.byRole[roleId] = next
      this.persist()
      return next.turnCount
    },

    dismissImmersiveHint(roleId: string): void {
      const cur = this.roleState(roleId)
      this.byRole[roleId] = { ...cur, immersiveHintDismissed: true }
      this.persist()
    },

    markIdentitySurpriseSeen(roleId: string): void {
      const cur = this.roleState(roleId)
      this.byRole[roleId] = { ...cur, identitySurpriseSeen: true }
      this.persist()
    },
  },
})
