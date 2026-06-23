/** common — zh. */
import sharedCommon from '../../shared/common.zh'
import sharedShortcuts from '../../shared/shortcuts.zh'

export default {
  common: {
    ...sharedCommon,
    importPackTitle: '导入角色包',
    importPackFileProgress: '文件进度 {current} / {total}',
    importPackCurrentFile: '当前文件：{name}',
    chatInputLabel: '输入消息',
    chatPlaceholder: '对 {name} 说点什么...',
    sceneTravel: {
      togetherAria: '邀请同行并选择目的地',
      togetherLabel: '检测到邀请同行，请选择目的地',
      postAria: '选择要切换的场景',
      postLabel: '检测到出行或前往意图，请选择目的地',
      pickPlaceholder: '请选择目的地',
      solo: '仅我过去',
      together: '同行前往',
      dismiss: '稍后再说',
    },
    sceneMode: {
      title: '前往「{label}」',
      desc: '仅切换你的叙事视角，或让角色与你同往？',
      solo: '仅我过去（角色留守）',
      together: '同行前往',
    },
    autonomousNotice:
      '系统：虚拟时间变化后，角色场景已从「{from}」切换为「{to}」（叙事视角未自动改变）。',
    shortcutHelp: sharedShortcuts,
    rolePack: {
      exportFilterName: 'OCPak 角色包',
      importFilterName: 'OCPak / ZIP',
      exported: '角色包已导出',
      importedOverwrite: '已覆盖并导入角色: {id}',
      imported: '已导入角色: {name}',
      barTitle:
        '安装 .ocpak / .zip 压缩包，或已解压的目录（与 roles/{id}/ 一致）',
      export: '导出角色包',
      importArchive: '导入压缩包',
      importFolder: '从文件夹导入',
      conflictTitle: '角色已存在',
      conflictBody:
        '本地已有角色 ID「{id}」（{name} v{version}）。导入将覆盖该角色目录，是否继续？',
      overwrite: '覆盖导入',
    },
  },
  relation: {
    defaultOptionName: '默认身份（{label}）',
    upgradeAcquaintance: '关系更近了一步：你们不再陌生。',
    upgradeFriend: '✨ 你们成为了朋友！',
    upgradeCloseFriend: '🎉 你们已经是好朋友了！',
    upgradePartner: '💖 关系阶段：伴侣',
    upgradeUnknown: '关系阶段更新为：{state}',
  },
}
