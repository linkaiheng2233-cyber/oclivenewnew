import { describe, expect, it } from 'vitest'
import { redactDiagnosticText } from './diagnosticText'

const credential = 'cmV2aWV3LXVzZXI6c3ludGhldGljLXBhc3N3b3Jk'

describe('diagnostic credential redaction', () => {
  it.each([
    [`Authorization: Basic ${credential}`, 'Authorization: [redacted]'],
    [`authorization=Basic\t${credential}`, 'authorization=[redacted]'],
    [`Proxy-Authorization: Basic ${credential}`, 'Proxy-Authorization: [redacted]'],
    [`Authorization: Bearer ${credential}; status=503`, 'Authorization: [redacted]; status=503'],
    [`{"Authorization":"Basic ${credential}","code":"LLM_ERROR"}`, '{"Authorization":"[redacted]","code":"LLM_ERROR"}'],
    [`{'authorization': 'Basic ${credential}'}`, '{\'authorization\': \'[redacted]\'}'],
    ['Authorization: Digest username="synthetic", nonce="private", response="private"\r\nERROR timeout', 'Authorization: [redacted]\r\nERROR timeout'],
    ['{"authorization":"Digest username=\\"synthetic\\", response=\\"private\\"","code":"IO_ERROR"}', '{"authorization":"[redacted]","code":"IO_ERROR"}'],
  ])('redacts complete authentication in %s', (input, expected) => {
    const result = redactDiagnosticText(input)
    expect(result).toBe(expected)
    expect(redactDiagnosticText(result)).toBe(result)
  })

  it('keeps neighboring error context and does not consume the next line after an empty header', () => {
    expect(redactDiagnosticText(`ERROR request\r\nAuthorization: Basic ${credential}\r\nHTTP 503 retry later`))
      .toBe('ERROR request\r\nAuthorization: [redacted]\r\nHTTP 503 retry later')
    expect(redactDiagnosticText('Authorization: \r\nERROR timeout')).toBe('Authorization: \r\nERROR timeout')
  })

  it('preserves existing key, bearer and URL-userinfo redaction without mutating plain diagnostics', () => {
    const result = redactDiagnosticText('api_key=private-key; token=private-token; Bearer private-bearer; https://user:password@example.com/log\nIO_ERROR: locked')
    expect(result).not.toMatch(/private-key|private-token|private-bearer|password/)
    expect(result).toContain('https://[redacted]@example.com/log\nIO_ERROR: locked')
    expect(redactDiagnosticText('IO_ERROR: file locked')).toBe('IO_ERROR: file locked')
    expect(redactDiagnosticText(result)).toBe(result)
  })
})
