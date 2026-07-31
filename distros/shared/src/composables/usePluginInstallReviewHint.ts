import type { InstallPluginFromMarketResponseDto } from '@oclive/shared/api'
import type { AppToastFn } from './useAppToast'
import { rt } from '@oclive/shared/i18n/runtimeT'

/** Info toast: plugin source is on disk — review before enabling high-risk permissions. */
export function showPluginInstallReviewHint(
  showToast: AppToastFn,
  result: Pick<InstallPluginFromMarketResponseDto, 'installedPluginId' | 'installPath'>,
): void {
  showToast(
    'info',
    rt('app.toast.pluginReviewSource', {
      id: result.installedPluginId,
      path: result.installPath,
    }),
  )
}
