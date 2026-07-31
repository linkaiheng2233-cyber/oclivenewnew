import apiErrors from './fragments/apiErrors.zh'
import appBundle from './fragments/app.zh'
import chat from './fragments/chat.zh'
import chatStorageBundle from './fragments/chatStorage.zh'
import commonBundle from './fragments/common.zh'
import devTools from './fragments/devTools.zh'
import editorBundle from './fragments/editor.zh'
import emotionUi from './fragments/emotionUi.zh'
import kernelBundle from './fragments/kernel.zh'
import { modelManagerZh as modelManager } from './fragments/modelManager.zh'
import onboardingBundle from './fragments/onboarding.zh'
import pluginManagerBundle from './fragments/pluginManager.zh'
import roleRuntimeBundle from './fragments/roleRuntime.zh'
import settingsBundle from './fragments/settings.zh'
import { simplePluginManagerZh as simplePluginManager } from './fragments/simplePluginManager.zh'
import theaterBundle from './fragments/theater.zh'
import toolShellBundle from './fragments/toolShell.zh'
import virtualTime from './fragments/virtualTime.zh'

export default {
  apiErrors,
  app: appBundle.app,
  chat,
  ...chatStorageBundle,
  common: commonBundle.common,
  devTools,
  editor: editorBundle.editor,
  emotionUi,
  ...kernelBundle,
  pluginManager: pluginManagerBundle.pluginManager,
  pluginTerms: pluginManagerBundle.pluginTerms,
  relation: commonBundle.relation,
  roleRuntime: roleRuntimeBundle.roleRuntime,
  settings: settingsBundle.settings,
  hotkeys: settingsBundle.hotkeys,
  keybindings: settingsBundle.keybindings,
  simplePluginManager,
  modelManager,
  virtualTime,
  ...toolShellBundle,
  ...onboardingBundle,
  ...theaterBundle,
}
