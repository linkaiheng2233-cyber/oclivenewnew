/** Cloud LLM vendor presets (OpenAI-compatible APIs). */
export type CloudLlmVendorId
  = | 'deepseek'
    | 'openai'
    | 'moonshot'
    | 'zhipu'
    | 'siliconflow'
    | 'dashscope'
    | 'custom'

export interface CloudLlmVendorPreset {
  id: CloudLlmVendorId
  /** i18n key under modelManager.vendors.* */
  labelKey: string
  baseUrl: string
  models: string[]
  /** Default OpenAI-compatible; use oclive_jsonrpc only for OCLIVE sidecar. */
  apiStyle: 'openai' | 'oclive_jsonrpc'
}

export const CLOUD_LLM_VENDORS: CloudLlmVendorPreset[] = [
  {
    id: 'deepseek',
    labelKey: 'modelManager.vendors.deepseek',
    baseUrl: 'https://api.deepseek.com',
    models: ['deepseek-chat', 'deepseek-reasoner'],
    apiStyle: 'openai',
  },
  {
    id: 'openai',
    labelKey: 'modelManager.vendors.openai',
    baseUrl: 'https://api.openai.com',
    models: ['gpt-4o-mini', 'gpt-4o', 'gpt-4.1-mini'],
    apiStyle: 'openai',
  },
  {
    id: 'moonshot',
    labelKey: 'modelManager.vendors.moonshot',
    baseUrl: 'https://api.moonshot.cn',
    models: ['moonshot-v1-8k', 'moonshot-v1-32k'],
    apiStyle: 'openai',
  },
  {
    id: 'zhipu',
    labelKey: 'modelManager.vendors.zhipu',
    baseUrl: 'https://open.bigmodel.cn/api/paas',
    models: ['glm-4-flash', 'glm-4-plus'],
    apiStyle: 'openai',
  },
  {
    id: 'siliconflow',
    labelKey: 'modelManager.vendors.siliconflow',
    baseUrl: 'https://api.siliconflow.cn',
    models: ['deepseek-ai/DeepSeek-V3', 'Qwen/Qwen2.5-7B-Instruct'],
    apiStyle: 'openai',
  },
  {
    id: 'dashscope',
    labelKey: 'modelManager.vendors.dashscope',
    baseUrl: 'https://dashscope.aliyuncs.com/compatible-mode',
    models: ['qwen-plus', 'qwen-turbo'],
    apiStyle: 'openai',
  },
  {
    id: 'custom',
    labelKey: 'modelManager.vendors.custom',
    baseUrl: '',
    models: [],
    apiStyle: 'openai',
  },
]

export function findCloudVendor(id: string): CloudLlmVendorPreset | undefined {
  return CLOUD_LLM_VENDORS.find(v => v.id === id)
}
