export function formatEndpoint(endpoint: string) {
  return endpoint.replace(/^https?:\/\//, '')
}
