/** common — en. */
import sharedCommon from '../../shared/common.en'
import sharedShortcuts from '../../shared/shortcuts.en'

export default {
  common: {
    ...sharedCommon,
    importPackTitle: 'Importing role pack',
    importPackFileProgress: 'File progress {current} / {total}',
    importPackCurrentFile: 'Current file: {name}',
    chatInputLabel: 'Message',
    chatPlaceholder: 'Say something to {name}…',
    sceneTravel: {
      togetherAria: 'Together travel — pick destination',
      togetherLabel: 'Together travel detected — pick a destination',
      postAria: 'Pick scene to switch',
      postLabel: 'Travel intent detected — pick a destination',
      pickPlaceholder: 'Choose destination',
      solo: 'Go alone',
      together: 'Go together',
      dismiss: 'Not now',
    },
    sceneMode: {
      title: 'Go to “{label}”',
      desc: 'Switch only your narrative view, or go together?',
      solo: 'I go alone (character stays)',
      together: 'Go together',
    },
    autonomousNotice:
      'System: after virtual time changed, the character’s scene moved from “{from}” to “{to}” (your narrative view did not auto-follow).',
    shortcutHelp: sharedShortcuts,
    rolePack: {
      exportFilterName: 'OCPak role pack',
      importFilterName: 'OCPak / ZIP',
      exported: 'Role pack exported',
      importedOverwrite: 'Imported (overwrite): {id}',
      imported: 'Imported role: {name}',
      barTitle:
        'Install .ocpak / .zip or an extracted folder (same layout as roles/{id}/)',
      export: 'Export pack',
      importArchive: 'Import archive',
      importFolder: 'Import folder',
      conflictTitle: 'Role already exists',
      conflictBody:
        'Local role ID “{id}” already exists ({name} v{version}). Import will overwrite that folder. Continue?',
      overwrite: 'Overwrite import',
    },
  },
  relation: {
    defaultOptionName: 'Default identity ({label})',
    upgradeAcquaintance: 'You grew closer — no longer strangers.',
    upgradeFriend: '✨ You became friends!',
    upgradeCloseFriend: '🎉 You’re close friends now!',
    upgradePartner: '💖 Relationship stage: partner',
    upgradeUnknown: 'Relationship stage updated: {state}',
  },
}
