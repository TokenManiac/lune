local proxy = require("@lune/proxy")
assert(type(proxy.socket) == "function")
assert(type(proxy.request) == "function")
