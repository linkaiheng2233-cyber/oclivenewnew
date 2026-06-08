/** Simple plugin manager — en-US */
export const simplePluginManagerEn = {
  title: 'Installed plugins',
  panelAria: 'Plugin manager',
  installZip: 'Install plugin',
  installingZip: 'Installing…',
  browseMarket: 'Browse market',
  tabInstalled: 'Installed',
  tabMarket: 'Market',
  embeddedNavAria: 'Plugin panel views',
  close: 'Close',
  loading: 'Loading…',
  empty: 'No plugins installed. Use Install plugin for a local zip, or browse the market.',
  uninstall: 'Uninstall',
  toggleHint: 'Enable or disable this plugin',
  confirmUninstall: 'Uninstall plugin “{id}”? This removes files and clears related state.',
  uninstalled: 'Uninstalled {id}',
  installed: 'Installed {id}',
  enabled: 'Enabled {id}',
  slotSelector: {
    title: 'Choose UI placement',
    lead: 'Plugin “{id}” can appear in these areas. Select where to show it:',
    listAria: 'Available slots',
    cancel: 'Cancel',
    confirm: 'Confirm and enable',
    needOne: 'Select at least one placement',
  },
  slots: {
    chat_toolbar: 'Above chat input',
    sidebar: 'Sidebar extension',
    settings: {
      panel: 'Settings extension',
      plugins: 'Plugin settings area',
      advanced: 'Settings advanced area',
    },
    role: {
      detail: 'Role detail area',
    },
    chat: {
      header: 'Top of chat column',
    },
    overlay: {
      floating: 'Global overlay',
    },
    launcher: {
      palette: 'Launcher palette',
    },
    debug: {
      dock: 'Debug dock',
    },
  },
}
