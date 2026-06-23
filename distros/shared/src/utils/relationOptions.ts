import type { UserRelationDto } from '@oclive/shared/api'
import { i18n } from '@oclive/shared/i18n/index'
import { OCLIVE_DEFAULT_RELATION_SENTINEL } from '@oclive/shared/api'

export interface RelationOptionRow { id: string, name: string }

/**
 * Relation dropdown options: first row is default relation, aligned with backend `set_user_relation` sentinel.
 * Shared by top bar, runtime panel, etc., to avoid copy-paste.
 */
export function buildRelationDropdownOptions(
  userRelations: UserRelationDto[],
  defaultRelation: string,
): RelationOptionRow[] {
  const rows = userRelations.map(r => ({
    id: r.id,
    name: r.name,
  }))
  const defId = defaultRelation || 'friend'
  const defLabel = rows.find(r => r.id === defId)?.name ?? defId
  return [
    {
      id: OCLIVE_DEFAULT_RELATION_SENTINEL,
      name: String(
        i18n.global.t('relation.defaultOptionName', {
          label: defLabel,
        }),
      ),
    },
    ...rows,
  ]
}
