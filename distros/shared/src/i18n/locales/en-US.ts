import apiErrors from './fragments/apiErrors.en'
import appBundle from './fragments/app.en'
import chat from './fragments/chat.en'
import chatStorageBundle from './fragments/chatStorage.en'
import commonBundle from './fragments/common.en'
import devTools from './fragments/devTools.en'
import editorBundle from './fragments/editor.en'
import emotionUi from './fragments/emotionUi.en'
import kernelBundle from './fragments/kernel.en'
import pluginManagerBundle from './fragments/pluginManager.en'
import roleRuntimeBundle from './fragments/roleRuntime.en'
import settingsBundle from './fragments/settings.en'
import { modelManagerEn as modelManager } from './fragments/modelManager.en'
import { simplePluginManagerEn as simplePluginManager } from './fragments/simplePluginManager.en'
import virtualTime from './fragments/virtualTime.en'
import toolShellBundle from './fragments/toolShell.en'
import onboardingBundle from './fragments/onboarding.en'
import theaterBundle from './fragments/theater.en'

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
