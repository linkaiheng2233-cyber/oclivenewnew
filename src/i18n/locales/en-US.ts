import apiErrors from './fragments/apiErrors.en'
import appBundle from './fragments/app.en'
import chat from './fragments/chat.en'
import commonBundle from './fragments/common.en'
import devTools from './fragments/devTools.en'
import editorBundle from './fragments/editor.en'
import emotionUi from './fragments/emotionUi.en'
import { expertConfigEn } from './fragments/expert.en'
import pluginManagerBundle from './fragments/pluginManager.en'
import roleRuntimeBundle from './fragments/roleRuntime.en'
import settingsBundle from './fragments/settings.en'
import { modelManagerEn as modelManager } from './fragments/modelManager.en'
import { simplePluginManagerEn as simplePluginManager } from './fragments/simplePluginManager.en'
import virtualTime from './fragments/virtualTime.en'

export default {
  apiErrors,
  app: appBundle.app,
  chat,
  common: commonBundle.common,
  devTools,
  editor: editorBundle.editor,
  emotionUi,
  ...expertConfigEn,
  pluginManager: pluginManagerBundle.pluginManager,
  pluginTerms: pluginManagerBundle.pluginTerms,
  relation: commonBundle.relation,
  roleRuntime: roleRuntimeBundle.roleRuntime,
  settings: settingsBundle.settings,
  hotkeys: settingsBundle.hotkeys,
  simplePluginManager,
  modelManager,
  virtualTime,
}
