local proxy = {}

--[=[
        @within Proxy

        Connects to a WebSocket using the given SOCKS5 proxy URL.

        The proxy URL should be of the form `socks5://[user:pass@]host:port`.

        Returns the same `WebSocket` type as `net.socket`.
]=]
function proxy.socket(proxyUrl: string, url: string): Net.WebSocket
        return nil :: any
end

return proxy
