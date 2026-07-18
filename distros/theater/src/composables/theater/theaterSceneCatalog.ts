import type { PokeChipDef, TheaterSkeleton } from './theaterLogic'

export type TheaterScenePresetId
  = | 'breakfast'
    | 'supermarket'
    | 'way_home'
    | 'bedtime'

export const THEATER_RUNTIME_SCENE_ID = 'theater:home' as const

export interface TheaterScenePreset {
  id: TheaterScenePresetId
  labelKey: string
  skeletonPath: string
  sceneBrief: string
  sceneSettingHint: string
  pokeEnabled: boolean
  pokeChips: PokeChipDef[]
  runtimeSceneId: typeof THEATER_RUNTIME_SCENE_ID
  order: number
}

export const THEATER_SCENE_STORAGE_KEY = 'oclive.theater.scene.v1'

export const DEFAULT_THEATER_SCENE_PRESET_ID: TheaterScenePresetId = 'breakfast'

const BREAKFAST_POKE_CHIPS: PokeChipDef[] = [
  {
    id: 'tea',
    emoji: '🍵',
    labelKey: 'theater.poke.tea',
    weight: 'high',
    dramaSeed: '把"喝下一碗苦中药"这件苦差事，变成两人拌嘴的笑料：一方嫌弃抗拒，另一方半哄半逼，制造嫌弃与无奈的反差。',
  },
  {
    id: 'late',
    emoji: '⏰',
    labelKey: 'theater.poke.late',
    weight: 'high',
    dramaSeed: '突然发现快要迟到，时间压力骤升，让两人手忙脚乱、语速变快、互相催促，节奏陡然紧张起来。',
  },
  {
    id: 'biteTongue',
    emoji: '👅',
    labelKey: 'theater.poke.biteTongue',
    weight: 'high',
    dramaSeed: '吃早饭时冷不丁咬到舌头，一个突发小意外打破平静：先是吃痛慌乱，再被对方关心查看，最后窘迫害羞，情绪起伏明显。',
  },
  {
    id: 'nickname',
    emoji: '😼',
    labelKey: 'theater.poke.nickname',
    weight: 'high',
    dramaSeed: '冷不防甩出一个出其不意的新称呼撩拨关系，让对方先愣神、再追问，引出害羞与微妙的暧昧拉扯。',
  },
]

const SUPERMARKET_POKE_CHIPS: PokeChipDef[] = [
  {
    id: 'buyMilk',
    emoji: '🥛',
    labelKey: 'theater.poke.buyMilk',
    weight: 'high',
    dramaSeed: '采购清单里突然要/争论买牛奶，融入超市动线：一方坚持要买、另一方嫌麻烦，或发现清单上漏写了牛奶。',
  },
  {
    id: 'forgotMilk',
    emoji: '🛒',
    labelKey: 'theater.poke.forgotMilk',
    weight: 'high',
    dramaSeed: '结账或出门才发现漏买牛奶，引发互相推责或补救：谁负责记清单、谁跑回去拿、还是改天再买。',
  },
  {
    id: 'milkSoldOut',
    emoji: '📦',
    labelKey: 'theater.poke.milkSoldOut',
    weight: 'high',
    dramaSeed: '目标货架空柜牛奶卖完了，换牌子、改计划或拌嘴：一方将就、另一方挑剔，或商量去别家买。',
  },
  {
    id: 'milkLottery',
    emoji: '🎁',
    labelKey: 'theater.poke.milkLottery',
    weight: 'high',
    dramaSeed: '超市活动/抽奖意外中奖一箱奶，惊喜或尴尬：一方兴奋、另一方嫌拎着沉，或担心保质期。',
  },
]

const WAY_HOME_POKE_CHIPS: PokeChipDef[] = [
  {
    id: 'strayCat',
    emoji: '🐱',
    labelKey: 'theater.poke.strayCat',
    weight: 'high',
    dramaSeed: '路边停步看小猫，一方想摸一方拦：担心卫生、怕被抓，或忍不住蹲下来逗猫。',
  },
  {
    id: 'hitPole',
    emoji: '🚧',
    labelKey: 'theater.poke.hitPole',
    weight: 'high',
    dramaSeed: '走路分心撞到电线杆，小事故引发关心或吐槽：一方笑对方不看路、另一方嘴硬说没事。',
  },
  {
    id: 'wrongWay',
    emoji: '🧭',
    labelKey: 'theater.poke.wrongWay',
    weight: 'high',
    dramaSeed: '导航或记忆出错走错方向绕路，拌嘴或默契纠正：谁带的路、要不要原路返回。',
  },
  {
    id: 'sprainedAnkle',
    emoji: '🦶',
    labelKey: 'theater.poke.sprainedAnkle',
    weight: 'high',
    dramaSeed: '路面不平崴到脚，搀扶或逞强：一方要背/扶、另一方嘴硬说能走。',
  },
]

const BEDTIME_POKE_CHIPS: PokeChipDef[] = [
  {
    id: 'drinkMilk',
    emoji: '🥛',
    labelKey: 'theater.poke.drinkMilk',
    weight: 'high',
    dramaSeed: '睡前温奶或递杯，日常温存或嫌弃：一方劝喝牛奶助眠、另一方嫌腻或怕胖。',
  },
  {
    id: 'insomnia',
    emoji: '😴',
    labelKey: 'theater.poke.insomnia',
    weight: 'high',
    dramaSeed: '翻来覆去睡不着，互相打扰或安抚：翻身、聊天、还是起来喝水。',
  },
  {
    id: 'midnightSnack',
    emoji: '🍜',
    labelKey: 'theater.poke.midnightSnack',
    weight: 'high',
    dramaSeed: '饿了想吃宵夜，厨房偷吃或点外卖，怕胖、怕吵或互相馋。',
  },
  {
    id: 'thunderstorm',
    emoji: '⛈️',
    labelKey: 'theater.poke.thunderstorm',
    weight: 'high',
    dramaSeed: '窗外打雷下雨，怕雷、关窗或多一句软话：一方逞强、另一方借机靠近。',
  },
]

const CATALOG: TheaterScenePreset[] = [
  {
    id: 'breakfast',
    labelKey: 'theater.header.scene.breakfast',
    skeletonPath: '/theater/scenes/breakfast.skeleton.json',
    sceneBrief: '早餐 · 上学前：厨房餐桌、温粥、收拾书包、出门前的日常照应与拌嘴。',
    sceneSettingHint: '地点限于家中厨房/餐桌/玄关；时间早晨上学前；禁止脱离居家早饭场景或引入第三人。',
    pokeEnabled: true,
    pokeChips: BREAKFAST_POKE_CHIPS,
    runtimeSceneId: THEATER_RUNTIME_SCENE_ID,
    order: 0,
  },
  {
    id: 'supermarket',
    labelKey: 'theater.header.scene.supermarket',
    skeletonPath: '/theater/scenes/supermarket.skeleton.json',
    sceneBrief: '超市采购：推购物车、抢特价、试吃拌嘴、结账忘带东西的小插曲。',
    sceneSettingHint: '地点限于超市卖场/货架/试吃台/收银台；时间白天采购；禁止脱离超市或引入店员以外的第三人对话。',
    pokeEnabled: true,
    pokeChips: SUPERMARKET_POKE_CHIPS,
    runtimeSceneId: THEATER_RUNTIME_SCENE_ID,
    order: 1,
  },
  {
    id: 'way_home',
    labelKey: 'theater.header.scene.way_home',
    skeletonPath: '/theater/scenes/way_home.skeleton.json',
    sceneBrief: '回家路上：采购或放学后同行，路灯/公交、拌嘴谁拿重物、随口关心。',
    sceneSettingHint: '地点限于街道/路灯下/公交站附近；时间傍晚或采购归途；禁止跳转到室内长戏或引入第三人。',
    pokeEnabled: true,
    pokeChips: WAY_HOME_POKE_CHIPS,
    runtimeSceneId: THEATER_RUNTIME_SCENE_ID,
    order: 2,
  },
  {
    id: 'bedtime',
    labelKey: 'theater.header.scene.bedtime',
    skeletonPath: '/theater/scenes/bedtime.skeleton.json',
    sceneBrief: '洗澡睡觉：洗漱顺序、吹头发/抢浴室、睡前一句软话的收束。',
    sceneSettingHint: '地点限于家中浴室/卧室门口/睡前片刻；时间夜晚就寝前；禁止脱离居家就寝场景。',
    pokeEnabled: true,
    pokeChips: BEDTIME_POKE_CHIPS,
    runtimeSceneId: THEATER_RUNTIME_SCENE_ID,
    order: 3,
  },
]

const PRESET_BY_ID = new Map(CATALOG.map(p => [p.id, p]))

export function listTheaterScenePresets(): TheaterScenePreset[] {
  return [...CATALOG].sort((a, b) => a.order - b.order)
}

export function getTheaterScenePreset(id: TheaterScenePresetId): TheaterScenePreset {
  const preset = PRESET_BY_ID.get(id)
  if (!preset)
    throw new Error(`unknown theater scene preset: ${id}`)
  return preset
}

export function isTheaterScenePresetId(value: string): value is TheaterScenePresetId {
  return PRESET_BY_ID.has(value as TheaterScenePresetId)
}

export function getTheaterScenePresetId(): TheaterScenePresetId {
  try {
    const raw = localStorage.getItem(THEATER_SCENE_STORAGE_KEY)
    if (raw && isTheaterScenePresetId(raw))
      return raw
  }
  catch {
    /* private mode */
  }
  return DEFAULT_THEATER_SCENE_PRESET_ID
}

export function setTheaterScenePresetId(id: TheaterScenePresetId): void {
  try {
    localStorage.setItem(THEATER_SCENE_STORAGE_KEY, id)
  }
  catch {
    /* ignore */
  }
}

/** Poke chips declared for a scene preset (catalog SSOT). */
export function getPokeChipsForPreset(id: TheaterScenePresetId): PokeChipDef[] {
  return getTheaterScenePreset(id).pokeChips
}

/** Chips that are both declared in catalog and backed by skeleton forks. */
export function resolveActivePokeChips(
  preset: TheaterScenePreset,
  skeleton: TheaterSkeleton | null | undefined,
): PokeChipDef[] {
  if (!preset.pokeEnabled || !skeleton)
    return []
  return preset.pokeChips.filter(chip => (skeleton.forks[chip.id]?.length ?? 0) > 0)
}

/** Legacy breakfast path for dev caches that still fetch the old URL. */
export const LEGACY_BREAKFAST_SKELETON_URL = '/theater/breakfast.skeleton.json'
