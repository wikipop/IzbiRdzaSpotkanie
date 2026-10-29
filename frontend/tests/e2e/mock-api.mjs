import { createServer } from 'node:http'

const baseEntries = () => ({
  'pn-09': [],
  'pn-10': [{ user_id: 'u_ola', nick: 'Ola', state: 'yes', updated_at: 1 }],
})
let entries = baseEntries()
const identities = new Map()

function json(response, status, body) {
  response.writeHead(status, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': 'http://127.0.0.1:3000',
    'Access-Control-Allow-Headers': 'Authorization, Content-Type',
    'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  })
  response.end(body === undefined ? undefined : JSON.stringify(body))
}

async function bodyOf(request) {
  const chunks = []
  for await (const chunk of request) chunks.push(chunk)
  return chunks.length ? JSON.parse(Buffer.concat(chunks).toString('utf8')) : undefined
}

function identityFor(request) {
  const token = request.headers.authorization?.replace(/^Bearer /, '')
  return token ? identities.get(token) : undefined
}

const server = createServer(async (request, response) => {
  const url = new URL(request.url ?? '/', 'http://127.0.0.1:8080')
  if (request.method === 'OPTIONS') return json(response, 204)
  if (request.method === 'POST' && url.pathname === '/__reset') {
    entries = baseEntries()
    identities.clear()
    return json(response, 204)
  }
  if (url.pathname === '/health') return json(response, 200, { status: 'ok' })
  if (request.method === 'POST' && url.pathname === '/events') {
    const body = await bodyOf(request)
    if (body?.name !== 'Wspólne izbienie' || JSON.stringify(body?.slots) !== JSON.stringify(['pn-09', 'sr-14'])) {
      return json(response, 400, { error: { code: 'invalid_body', message: 'unexpected e2e payload' } })
    }
    entries = baseEntries()
    return json(response, 201, { event_id: 'e_e2e' })
  }
  if (request.method === 'GET' && url.pathname === '/events/e_e2e') {
    return json(response, 200, { event_id: 'e_e2e', name: 'Wspólne izbienie', slots: ['pn-09', 'pn-10'] })
  }
  if (request.method === 'POST' && url.pathname === '/events/e_e2e/join') {
    const body = await bodyOf(request)
    const nick = String(body?.nick ?? 'Iza').trim()
    const userId = nick.toLocaleLowerCase('pl') === 'ola' ? 'u_ola' : 'u_iza'
    const token = `e2e-${userId}-token`
    identities.set(token, { userId, nick })
    return json(response, 200, { user_id: userId, token })
  }

  const slotMatch = /^\/slots\/(pn-(?:09|10))\/events\/e_e2e(\/me)?$/.exec(url.pathname)
  if (!slotMatch) return json(response, 404, { error: { code: 'event_not_found', message: 'missing' } })
  const slot = slotMatch[1]
  if (request.method === 'GET') return json(response, 200, { slot, entries: entries[slot] ?? [] })

  const identity = identityFor(request)
  if (!identity) return json(response, 401, { error: { code: 'missing_token', message: 'missing' } })
  entries[slot] = (entries[slot] ?? []).filter(entry => entry.user_id !== identity.userId)
  if (request.method === 'DELETE') return json(response, 204)
  if (request.method === 'PUT') {
    const body = await bodyOf(request)
    const entry = { user_id: identity.userId, nick: identity.nick, state: body.state, updated_at: 2 }
    entries[slot].push(entry)
    return json(response, 200, entry)
  }
  return json(response, 405, { error: { code: 'invalid_body', message: 'method' } })
})

server.listen(8080, '127.0.0.1')

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => server.close(() => process.exit(0)))
}
