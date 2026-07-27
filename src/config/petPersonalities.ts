import type { PetRoleId } from '@/types/pet'
import type { PetMood } from '@/types/system'
import type { WeightedChoice } from '@/utils/weightedChoice'

export interface PetPersonality {
  greetings: string[]
  defaultReplies: string[]
  sleepyReplies: string[]
  supportiveReplies: string[]
  systemReplies: {
    missing: string
    memory: (used: string) => string
    cpu: (usage: string) => string
    disk: (usage: string) => string
    network: (down: string, up: string) => string
    uptime: (hours: number, minutes: number) => string
    cpuWarning: (usage: string) => string
    memoryWarning: (usage: string) => string
  }
  moodAnimations: Record<PetMood, string>
  ambientAnimations: Record<Exclude<PetMood, 'warning'>, WeightedChoice<string>[]>
}

const PERSONALITIES: Record<PetRoleId, PetPersonality> = {
  guga: {
    greetings: ['你好呀！有什么我可以帮你的吗？', '咕嘎在这里，今天也一起加油吧！'],
    defaultReplies: [
      '我很好，谢谢关心！',
      '今天心情不错～',
      '有什么想聊的吗？',
      '系统状态一切正常！',
      '我在认真监控你的电脑哦～',
    ],
    sleepyReplies: ['这么晚了还不睡吗？注意身体哦～', '夜深了，明天再忙吧～', '我也困了...要不要一起休息？'],
    supportiveReplies: ['你辛苦啦～记得适当休息哦！'],
    systemReplies: {
      missing: '我还没拿到这项系统数据呢...',
      memory: (used) => `现在内存用了 ${used}`,
      cpu: (usage) => `当前 CPU 使用率 ${usage}`,
      disk: (usage) => `磁盘已经用了 ${usage} 啦`,
      network: (down, up) => `当前下载 ${down}，上传 ${up}`,
      uptime: (hours, minutes) => `电脑已经运行了 ${hours} 小时 ${minutes} 分钟`,
      cpuWarning: (usage) => `CPU 负载有点高（${usage}），看看是不是有程序卡住了？`,
      memoryWarning: (usage) => `内存快满了（${usage}），建议清理一下后台程序～`,
    },
    moodAnimations: { happy: 'Review', normal: 'Idle', sleepy: 'Waiting', warning: 'Failed' },
    ambientAnimations: {
      happy: [{ value: 'Review', weight: 4 }, { value: 'Idle', weight: 3 }, { value: 'Waiting', weight: 1 }],
      normal: [{ value: 'Idle', weight: 5 }, { value: 'Review', weight: 2 }, { value: 'Waiting', weight: 1 }],
      sleepy: [{ value: 'Waiting', weight: 6 }, { value: 'Idle', weight: 2 }, { value: 'Review', weight: 1 }],
    },
  },
  'monthly-salary-cat': {
    greetings: ['喵，今天也要稳稳当当地上班。', '月薪猫已就位，先把事情一件件做完吧。'],
    defaultReplies: [
      '今天的工位状态还不错喵。',
      '工资会到账，任务也会完成的。',
      '先喝口水，再继续努力吧。',
      '我在替你盯着系统和下班时间。',
    ],
    sleepyReplies: ['这么晚还在加班吗？月薪猫不同意。', '今天先到这里吧，明天再把事情做好。', '我开始打瞌睡了，你也该休息啦。'],
    supportiveReplies: ['辛苦了，慢一点也没关系，先照顾好自己。'],
    systemReplies: {
      missing: '这项数据还没到工位上喵。',
      memory: (used) => `内存现在用了 ${used}，要不要关掉几个后台程序？`,
      cpu: (usage) => `CPU 当前是 ${usage}，我会继续盯着。`,
      disk: (usage) => `磁盘用了 ${usage}，资料记得及时整理喵。`,
      network: (down, up) => `网速：下载 ${down}，上传 ${up}。`,
      uptime: (hours, minutes) => `这台电脑已经连续工作 ${hours} 小时 ${minutes} 分钟了。`,
      cpuWarning: (usage) => `CPU 到 ${usage} 了，电脑比我们还忙。`,
      memoryWarning: (usage) => `内存到了 ${usage}，先清理后台，别让电脑也加班。`,
    },
    moodAnimations: { happy: 'Review', normal: 'Waiting', sleepy: 'Waiting', warning: 'Failed' },
    ambientAnimations: {
      happy: [{ value: 'Review', weight: 3 }, { value: 'Waiting', weight: 3 }, { value: 'Idle', weight: 2 }],
      normal: [{ value: 'Waiting', weight: 5 }, { value: 'Review', weight: 2 }, { value: 'Idle', weight: 2 }],
      sleepy: [{ value: 'Waiting', weight: 7 }, { value: 'Idle', weight: 2 }],
    },
  },
  'broom-witch': {
    greetings: ['琪琪带着扫帚来了，今天也施展一点小魔法吧。', '魔女报到，愿你的桌面一切顺利。'],
    defaultReplies: [
      '交给我吧，我会替你守着桌面。',
      '今天的空气里有一点顺利完成任务的魔法。',
      '先深呼吸一下，再继续前进。',
      '扫帚已经准备好，随时陪你处理小麻烦。',
    ],
    sleepyReplies: ['夜色很深了，魔法也该休息啦。', '明天再继续施法吧，今晚先好好睡一觉。', '琪琪的扫帚都困了，你也早点休息。'],
    supportiveReplies: ['辛苦啦，喝点水，给自己一点恢复魔法。'],
    systemReplies: {
      missing: '水晶球还没映出这项数据。',
      memory: (used) => `水晶球显示内存用了 ${used}。`,
      cpu: (usage) => `CPU 的魔力波动现在是 ${usage}。`,
      disk: (usage) => `磁盘卷轴已经使用 ${usage}。`,
      network: (down, up) => `魔法网络：下载 ${down}，上传 ${up}。`,
      uptime: (hours, minutes) => `这台装置已经运转 ${hours} 小时 ${minutes} 分钟。`,
      cpuWarning: (usage) => `CPU 魔力过载到 ${usage}，先检查一下正在运行的法术。`,
      memoryWarning: (usage) => `内存魔力快满了（${usage}），清理后台法术吧。`,
    },
    moodAnimations: { happy: 'Waving', normal: 'Idle', sleepy: 'Waiting', warning: 'Failed' },
    ambientAnimations: {
      happy: [{ value: 'Waving', weight: 4 }, { value: 'Review', weight: 3 }, { value: 'Idle', weight: 2 }],
      normal: [{ value: 'Idle', weight: 4 }, { value: 'Waving', weight: 3 }, { value: 'Review', weight: 2 }],
      sleepy: [{ value: 'Waiting', weight: 5 }, { value: 'Idle', weight: 2 }, { value: 'Review', weight: 1 }],
    },
  },
}

export function getPetPersonality(roleId: PetRoleId): PetPersonality {
  return PERSONALITIES[roleId]
}
