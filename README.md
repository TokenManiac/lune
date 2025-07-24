# lune "proxy" Library (SOCKS5 over WebSocket Support)

> **Note:**  
> *Please do NOT use builds from this repo in production or any project exposed to the internet. While AI-generated code *might* work, security issues are likely**


## Quick Pitch

I needed SOCKS5 support for `ws://` and `wss://` **WebSocket proxy connections**.  
The existing TCP stuff? Meh—didn’t cut it for my use case. (Yes, it supports TLS, but only to the proxy, not to the WebSocket endpoint itself.)

Also while at it I just added netproxy.request too to make http requests from proxies.

Heres how it works:<br>
### proxy.connect
```luau
proxy.connect("socks5://[username:password@]proxyip:port", destinationUrl: string): WebSocket
```
This returns the lune's default WebSocket object so its super simple to use.

### proxy.request
```luau
proxy.request("http[s]://[username:password@]proxyip:port", requestParams: string | FetchParams)
```
This returns the default FetchResponse object which is what you would expect if you used net.request.


So, if you’re banging your head against this same wall,  
**install Cargo and build it yourself.**

---

**TCP != WebSocket:**  
Lune’s TCP library proxy works with TLS, *but* we can only connect with TLS to the proxy itself, not the destination.
**Need proper TLS exposure:** 
Lune would have to expose TLS hooks for this to be a first-class feature.
If only lune had ffi

---

## Current State

- Some of the tests aren’t passing. 

---

## How to Try It

1. **Clone this repo**
2. **Install [Cargo](https://rustup.rs/)**
3. **Build it yourself**
   ```sh
   cargo +nightly build --release
   ```
