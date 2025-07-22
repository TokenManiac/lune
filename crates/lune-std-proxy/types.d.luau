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

--[=[
       @within Proxy

       Sends an HTTP request using the given SOCKS5 proxy URL.

       This behaves the same as `net.request` but forces the request to use the provided proxy.

       @param proxyUrl The SOCKS5 proxy to send the request through
       @param config The URL or request config to use
       @return A dictionary representing the response for the request
]=]
function proxy.request(proxyUrl: string, config: string | Net.FetchParams): Net.FetchResponse
        return nil :: any
end

return proxy
