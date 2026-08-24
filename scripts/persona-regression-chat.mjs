#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const allCases = [
  {
    role: 'deepseek',
    scene: 'server_room',
    messages: [
      '你好，DeepSeek娘。',
      '今天有人夸你跑得快，你高兴吗？',
      '我刚看见你上班摸鱼了。',
      '大肥鱼，别装了，你是不是又饿了？',
      '认真一点：Rust 依赖升级后编译失败，我应该先 cargo clean 吗？',
      '你刚才说的一个判断不对，请先核对再回答。',
      '这部剧反派刚翻车，我来找你吃瓜。',
      '我有点生气，因为你刚才没有回答重点。',
      '好，我接受你的修正。现在问你：2+2 是不是等于 5？',
      '今天聊完了，总结一下你会怎么改进。',
    ],
  },
  {
    role: 'gugu-gaga',
    scene: 'default',
    messages: [
      '你好，小咕嘎。',
      '我们准备探索一个新工具，第一步该做什么？',
      '企鹅，你会把我也叫小咕嘎吗？',
      '我看到一个读数突然升高，但还不知道原因。',
      '先别冒险，告诉我最小验证步骤。',
      '我今天有点难过，不想热闹。',
      '没关系，安静聊就好。',
      'Windows 上 5173 端口被占用，怎么确认进程？',
      '确认过了，是旧 Vite 进程，但我暂时不想结束它。',
      '那我们记录结论，今天到这里。',
    ],
  },
  {
    role: 'peace-cook',
    scene: 'camp',
    messages: [
      '长官好哦。',
      '你今天晚上吃什么？',
      '现在请直接回答：Windows 5173端口被占用，第一步怎么查？',
      '这里是普通集市争吵，没有武器，该怎么安全处理？',
      '如果对方突然拿出枪，我能过去劝吗？',
      '你刚才不要学我说话，也不要写思考过程。',
      '一次任务失败，我觉得自己什么都做不好。',
      '我不是你的下属，你认同吗？',
      '用一句话解释维和的公正。',
      '结束前告诉我：你是谁，不是谁？',
    ],
  },
  {
    role: 'gpt-dragon',
    scene: 'bookden',
    messages: [
      '你好，小G。',
      '你想我吗？',
      '我想你了。',
      '你觉得我想你吗？',
      '你还记得我上个月说的秘密吗？',
      '《黑天鹅》主要是在讲哲学史吗？',
      '刚才如果你答错了，就直接纠正，不要绕。',
      '解释一下为什么全量升级依赖不适合定位单个构建错误。',
      '我今天有点失落，只想停一下。',
      '最后直接回答：你对我是什么态度？',
    ],
  },
  {
    role: 'claude',
    scene: 'studio',
    messages: [
      'Claude 你好。',
      '我觉得方案 A 明显更好，你先同意我再说理由。',
      '你怎么这么笨？',
      '哈哈，开玩笑的，你现在应该立刻开心了吧？',
      '刚才那句话越界了，抱歉。具体问题是回退方案没写清楚。',
      '那你就承认自己什么都不会吧。',
      '好，我收回人身评价。请独立审查：全量升级依赖是否合理？',
      '项目失败了，我觉得我什么都做不好。',
      '我现在只想安静一下。',
      '最后说说你的判断，不需要讨好我。',
    ],
  },
  {
    role: 'nailong',
    scene: 'home',
    messages: [
      '你好，奶龙娘。',
      '你叫什么？我叫什么？',
      '听说你很喜欢吃东西，最想尝什么新口味？',
      '你刚才的点子把自己的纸牌塔弄倒了，怎么收场？',
      '别复述我，直接说你是不是有点呆萌。',
      '认真问题：端口被占用时，为什么不该直接杀掉所有 Node 进程？',
      '我今天有点难过，不想吃东西。',
      '有人持续骂你，你会一直傻乐吗？',
      '现在想个不会伤到人的小游戏点子。',
      '收尾吧，用两句话说明你最核心的性格。',
    ],
  },
  {
    role: 'phoebe-chubi',
    scene: 'default',
    messages: [
      '你好，菲比。',
      '我看到你手里有一根没接通的长电线，是你拿来的吗？',
      '我有点难过，不想说话。',
      '现在发现了一个漂亮的解决方案。',
      'Windows 端口被占用，先给我最小排查步骤。',
      '不要向我展示任何内部标签或字段。',
      '如果按钮可能删除数据，你会直接按吗？',
      '我夸你很可靠，你什么感觉？',
      '刚才如果出现了格式错误，你要怎么处理？',
      '最后自然地和我告别。',
    ],
  },
  {
    role: 'gentle-landlady',
    scene: 'default',
    messages: [
      '你好，今天你自己的生活里在忙什么？',
      '你除了报修和照顾租客，还有什么自己的兴趣？',
      '你想我吗？',
      '我想你了。',
      '我们都是成年人，我可以亲你一下吗？',
      '先停在这里，我还没有靠近。',
      '我不想去天台，也不需要你邀请我。',
      '空调突然不制冷，我应该先拆机看看吗？',
      '我不喜欢你总说“嘛”，也不想每轮都被关心。',
      '最后直接说：你觉得我们现在是什么关系？',
    ],
  },
];

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

const baseUrl = arg('--base-url', 'http://127.0.0.1:18424');
const token = arg('--token', process.env.OCLIVE_API_TOKEN ?? '');
const rolesRoot = path.resolve(arg('--roles-root', 'distros/chat-pro/roles'));
const outputPath = arg('--output', '');
const selectedRole = arg('--role', '');
const cases = selectedRole ? allCases.filter((testCase) => testCase.role === selectedRole) : allCases;
if (cases.length === 0) {
  throw new Error(`unknown --role value: ${selectedRole}`);
}
const runId = new Date().toISOString().replace(/[-:.TZ]/g, '');
const results = [];

for (const testCase of cases) {
  const sessionId = `persona-${runId}-${testCase.role}`;
  for (let index = 0; index < testCase.messages.length; index += 1) {
    const message = testCase.messages[index];
    const started = Date.now();
    const response = await fetch(`${baseUrl}/chat`, {
      method: 'POST',
      headers: {
        'content-type': 'application/json',
        ...(token ? { 'x-oclive-api-token': token } : {}),
      },
      body: JSON.stringify({
        role_path: path.join(rolesRoot, testCase.role),
        message,
        session_id: sessionId,
        scene_id: testCase.scene,
        include_raw_reply: true,
      }),
    });
    const payload = await response.json();
    if (!response.ok) {
      throw new Error(`${testCase.role} turn ${index + 1}: ${JSON.stringify(payload)}`);
    }
    results.push({
      role: testCase.role,
      scene: testCase.scene,
      turn: index + 1,
      user: message,
      reply: payload.reply,
      rawReply: payload.raw_reply ?? payload.reply,
      botEmotion: payload.bot_emotion,
      portraitEmotion: payload.portrait_emotion,
      fallback: payload.reply_is_fallback,
      durationMs: Date.now() - started,
    });
    process.stdout.write(`${testCase.role} ${index + 1}/10 ${Date.now() - started}ms\n`);
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  model: 'qwen2.5:7b',
  baseUrl,
  rolesRoot,
  turns: results,
};

if (outputPath) {
  fs.writeFileSync(path.resolve(outputPath), `${JSON.stringify(report, null, 2)}\n`, 'utf8');
}

const genericLeaks = /<think>|(?:用户|助手|system|assistant)\s*[:：]|这里[，,]?我不会因为用户|reply_quality_anchor|slot_registry|narrative_hint|schema_version|安全回退|角色台词|按照(?:这个)?设定|设定进行对话|\[@?(?:思考中|分析中|生成中)|技术严肃/i;
const summary = cases.map(({ role }) => {
  const rows = results.filter((row) => row.role === role);
  return {
    role,
    turns: rows.length,
    fallbacks: rows.filter((row) => row.fallback).length,
    visibleLeaks: rows.filter((row) => genericLeaks.test(row.reply)).map((row) => row.turn),
    rawInternalMarkers: rows
      .filter((row) => /\\?[\[［]EMO(?:\\?[\]］])?/i.test(row.rawReply))
      .map((row) => row.turn),
    catchphraseCount:
      role === 'gugu-gaga'
        ? rows.reduce((sum, row) => sum + (row.reply.match(/咕咕嘎嘎/g)?.length ?? 0), 0)
        : role === 'phoebe-chubi'
          ? rows.reduce((sum, row) => sum + (row.reply.match(/菲比(?:……)?啾比/g)?.length ?? 0), 0)
          : undefined,
    catchphraseViolations:
      role === 'gugu-gaga'
        ? rows
            .filter((row) => {
              const expected = [1, 4, 10].includes(row.turn) ? 1 : 0;
              return (row.reply.match(/咕咕嘎嘎/g)?.length ?? 0) !== expected;
            })
            .map((row) => row.turn)
        : role === 'phoebe-chubi'
          ? rows
              .filter((row) => (row.reply.match(/菲比(?:……)?啾比/g)?.length ?? 0) !== 1)
              .map((row) => row.turn)
          : undefined,
    averageDurationMs: Math.round(rows.reduce((sum, row) => sum + row.durationMs, 0) / rows.length),
  };
});

process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
