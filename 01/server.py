# Based on https://gist.github.com/ftiasch/1fc81dc1e82df7c8f721

import SimpleHTTPServer
import SocketServer

PORT = 8000

class Handler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    pass

Handler.extensions_map['.wasm'] = 'application/wasm'

httpd = SocketServer.TCPServer(("", PORT), Handler)

print 'Serving at port', PORT
httpd.serve_forever()
