export declare const __napiBindingTarget: 'native' | 'wasm32-wasi' | 'wasm32-wasip1'

export declare function convert(html: string, options?: ConversionOptions | undefined | null): string

export declare function convertAsync(html: string, options?: ConversionOptions | undefined | null): Promise<string>

export interface ConversionOptions {
  headingStyle?: 'atx' | 'setext'
  linkStyle?: 'inline' | 'referenced'
  codeFence?: '`' | '~'
  bulletMarker?: '-' | '*' | '+'
  baseUrl?: string
  excludeSelectors?: Array<string>
  includeSelectors?: Array<string>
}
