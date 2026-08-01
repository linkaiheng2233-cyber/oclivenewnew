declare module 'vue3-sfc-loader' {
  export function loadModule(
    path: string,
    options: unknown,
  ): Promise<unknown>
}
